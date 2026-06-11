//! Redis-backed job queue shared by the REST, worker and MCP modes.
//!
//! Data layout (prefix `cp:`):
//! - `cp:jobs:stream`      stream + consumer group `workers` (dispatch)
//! - `cp:queue:waiting`    zset, score = enqueue ms (O(log n) queue position)
//! - `cp:job:{id}`         hash: state/stage/progress/timestamps/heartbeat
//! - `cp:job:{id}:req`     SubmitJobRequest proto bytes
//! - `cp:job:{id}:result`  JobResult proto bytes
//! - `cp:job:{id}:cancel`  cooperative cancellation flag
//! - pub/sub `cp:events:{id}`  JSON-encoded JobStatus on every change

use anyhow::{Context, Result};
use prost::Message;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

use craftpath_proto::v1;

pub const STREAM_KEY: &str = "cp:jobs:stream";
pub const GROUP: &str = "workers";
pub const WAITING_ZSET: &str = "cp:queue:waiting";

pub fn job_key(id: &str) -> String {
    format!("cp:job:{id}")
}
pub fn req_key(id: &str) -> String {
    format!("cp:job:{id}:req")
}
pub fn result_key(id: &str) -> String {
    format!("cp:job:{id}:result")
}
pub fn cancel_key(id: &str) -> String {
    format!("cp:job:{id}:cancel")
}
pub fn events_channel(id: &str) -> String {
    format!("cp:events:{id}")
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn state_to_str(state: v1::JobState) -> &'static str {
    match state {
        v1::JobState::Queued => "QUEUED",
        v1::JobState::Running => "RUNNING",
        v1::JobState::Succeeded => "SUCCEEDED",
        v1::JobState::Failed => "FAILED",
        v1::JobState::Cancelled => "CANCELLED",
        v1::JobState::Expired => "EXPIRED",
        v1::JobState::Unspecified => "UNSPECIFIED",
    }
}

fn state_from_str(s: &str) -> v1::JobState {
    match s {
        "QUEUED" => v1::JobState::Queued,
        "RUNNING" => v1::JobState::Running,
        "SUCCEEDED" => v1::JobState::Succeeded,
        "FAILED" => v1::JobState::Failed,
        "CANCELLED" => v1::JobState::Cancelled,
        "EXPIRED" => v1::JobState::Expired,
        _ => v1::JobState::Unspecified,
    }
}

pub fn is_terminal(state: v1::JobState) -> bool {
    matches!(
        state,
        v1::JobState::Succeeded
            | v1::JobState::Failed
            | v1::JobState::Cancelled
            | v1::JobState::Expired
    )
}

#[derive(Clone)]
pub struct JobsClient {
    conn: ConnectionManager,
    job_ttl_secs: u64,
}

impl JobsClient {
    pub async fn connect(redis_url: &str, job_ttl_secs: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .with_context(|| format!("invalid REDIS_URL '{redis_url}'"))?;
        let conn = ConnectionManager::new(client)
            .await
            .context("could not connect to redis")?;
        Ok(Self { conn, job_ttl_secs })
    }

    pub async fn ping(&self) -> Result<()> {
        let mut conn = self.conn.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .context("redis PING failed")?;
        Ok(())
    }

    /// Create the consumer group if it does not exist yet.
    pub async fn ensure_group(&self) -> Result<()> {
        let mut conn = self.conn.clone();
        let res: redis::RedisResult<String> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(STREAM_KEY)
            .arg(GROUP)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("BUSYGROUP") => Ok(()),
            Err(e) => Err(e).context("XGROUP CREATE failed"),
        }
    }

    // ------------------------------------------------------------------
    // Producer side (REST / MCP)
    // ------------------------------------------------------------------

    pub async fn submit(&self, request: &v1::SubmitJobRequest) -> Result<v1::JobStatus> {
        let mut conn = self.conn.clone();
        let job_id = uuid::Uuid::now_v7().to_string();
        let created_at = now_rfc3339();
        let enqueue_ms = chrono::Utc::now().timestamp_millis();

        let req_bytes = request.encode_to_vec();

        redis::pipe()
            .atomic()
            .hset_multiple(
                job_key(&job_id),
                &[
                    ("state", state_to_str(v1::JobState::Queued)),
                    ("stage", "queued"),
                    ("progress_percent", "0"),
                    ("created_at", created_at.as_str()),
                ],
            )
            .ignore()
            .set(req_key(&job_id), req_bytes)
            .ignore()
            .zadd(WAITING_ZSET, &job_id, enqueue_ms)
            .ignore()
            .xadd(STREAM_KEY, "*", &[("job_id", job_id.as_str())])
            .ignore()
            .exec_async(&mut conn)
            .await
            .context("failed to enqueue job")?;

        let status = self
            .status(&job_id)
            .await?
            .context("job vanished right after enqueue")?;
        self.publish_status(&status).await.ok();
        Ok(status)
    }

    pub async fn status(&self, job_id: &str) -> Result<Option<v1::JobStatus>> {
        let mut conn = self.conn.clone();
        let fields: std::collections::HashMap<String, String> =
            conn.hgetall(job_key(job_id)).await?;
        if fields.is_empty() {
            return Ok(None);
        }

        let state = state_from_str(fields.get("state").map(String::as_str).unwrap_or(""));

        let queue_position = if state == v1::JobState::Queued {
            let rank: Option<i64> = conn.zrank(WAITING_ZSET, job_id).await?;
            rank.map(|r| (r + 1) as u32)
        } else {
            None
        };

        let progress = Some(v1::JobProgress {
            phase: fields.get("stage").cloned().unwrap_or_default(),
            percent: fields
                .get("progress_percent")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            routes_found: fields.get("routes_found").and_then(|v| v.parse().ok()),
            ram_used_bytes: fields.get("ram_used_bytes").and_then(|v| v.parse().ok()),
        });

        let error = fields.get("error_code").map(|code| v1::Error {
            code: code.clone(),
            message: fields.get("error_message").cloned().unwrap_or_default(),
            details: Default::default(),
        });

        Ok(Some(v1::JobStatus {
            job_id: job_id.to_string(),
            state: state as i32,
            queue_position,
            progress,
            error,
            created_at: fields.get("created_at").cloned().unwrap_or_default(),
            started_at: fields.get("started_at").cloned(),
            finished_at: fields.get("finished_at").cloned(),
        }))
    }

    /// Result bytes (proto `JobResult`) if the job succeeded.
    pub async fn result_bytes(&self, job_id: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.conn.clone();
        let bytes: Option<Vec<u8>> = conn.get(result_key(job_id)).await?;
        Ok(bytes)
    }

    /// Request cooperative cancellation. Queued jobs are cancelled
    /// immediately; running jobs observe the flag at the next progress tick.
    pub async fn cancel(&self, job_id: &str) -> Result<Option<v1::JobStatus>> {
        let mut conn = self.conn.clone();
        let Some(status) = self.status(job_id).await? else {
            return Ok(None);
        };

        let state = v1::JobState::try_from(status.state).unwrap_or(v1::JobState::Unspecified);
        if is_terminal(state) {
            return Ok(Some(status));
        }

        let _: () = conn.set(cancel_key(job_id), "1").await?;

        if state == v1::JobState::Queued {
            // flip directly; the worker skips cancelled jobs when claiming
            let finished = now_rfc3339();
            redis::pipe()
                .atomic()
                .hset(job_key(job_id), "state", state_to_str(v1::JobState::Cancelled))
                .ignore()
                .hset(job_key(job_id), "finished_at", finished.as_str())
                .ignore()
                .zrem(WAITING_ZSET, job_id)
                .ignore()
                .exec_async(&mut conn)
                .await?;
            self.expire_job_keys(job_id).await?;
        }

        let status = self.status(job_id).await?;
        if let Some(s) = &status {
            self.publish_status(s).await.ok();
        }
        Ok(status)
    }

    // ------------------------------------------------------------------
    // Worker side
    // ------------------------------------------------------------------

    /// Poll up to `wait_ms` for the next queued job. Returns
    /// `(stream_entry_id, job_id, delivery_count)`.
    ///
    /// Deliberately avoids `XREADGROUP BLOCK`: blocking commands head-of-line
    /// block (and desync) the shared multiplexed connection. Polling every
    /// 200ms is plenty for jobs that run seconds to minutes.
    pub async fn claim_next(
        &self,
        consumer: &str,
        wait_ms: u64,
    ) -> Result<Option<(String, String, u64)>> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(wait_ms);

        loop {
            let mut conn = self.conn.clone();

            // First, reclaim entries stuck pending on dead consumers (> 60s idle).
            let reclaim: redis::Value = redis::cmd("XAUTOCLAIM")
                .arg(STREAM_KEY)
                .arg(GROUP)
                .arg(consumer)
                .arg(60_000)
                .arg("0-0")
                .arg("COUNT")
                .arg(1)
                .query_async(&mut conn)
                .await
                .context("XAUTOCLAIM failed")?;

            if let Some(entry) = parse_xautoclaim_reply(&reclaim) {
                let delivery = self.delivery_count(&entry.0).await.unwrap_or(1);
                return Ok(Some((entry.0, entry.1, delivery)));
            }

            // Then check for fresh work (non-blocking).
            let reply: redis::Value = redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(GROUP)
                .arg(consumer)
                .arg("COUNT")
                .arg(1)
                .arg("STREAMS")
                .arg(STREAM_KEY)
                .arg(">")
                .query_async(&mut conn)
                .await
                .context("XREADGROUP failed")?;

            if let Some((entry_id, job_id)) = parse_xreadgroup_reply(&reply) {
                return Ok(Some((entry_id, job_id, 1)));
            }

            if tokio::time::Instant::now() >= deadline {
                return Ok(None);
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    async fn delivery_count(&self, entry_id: &str) -> Result<u64> {
        let mut conn = self.conn.clone();
        let reply: redis::Value = redis::cmd("XPENDING")
            .arg(STREAM_KEY)
            .arg(GROUP)
            .arg(entry_id)
            .arg(entry_id)
            .arg(1)
            .query_async(&mut conn)
            .await?;

        // [[id, consumer, idle, delivery-count]]
        if let redis::Value::Array(items) = reply
            && let Some(redis::Value::Array(fields)) = items.first()
            && let Some(redis::Value::Int(count)) = fields.get(3)
        {
            return Ok(*count as u64);
        }
        Ok(1)
    }

    pub async fn ack(&self, entry_id: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        redis::pipe()
            .cmd("XACK")
            .arg(STREAM_KEY)
            .arg(GROUP)
            .arg(entry_id)
            .ignore()
            .cmd("XDEL")
            .arg(STREAM_KEY)
            .arg(entry_id)
            .ignore()
            .exec_async(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn load_request(&self, job_id: &str) -> Result<Option<v1::SubmitJobRequest>> {
        let mut conn = self.conn.clone();
        let bytes: Option<Vec<u8>> = conn.get(req_key(job_id)).await?;
        match bytes {
            None => Ok(None),
            Some(b) => Ok(Some(
                v1::SubmitJobRequest::decode(b.as_slice()).context("corrupt job request")?,
            )),
        }
    }

    pub async fn mark_running(&self, job_id: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let started = now_rfc3339();
        redis::pipe()
            .atomic()
            .hset_multiple(
                job_key(job_id),
                &[
                    ("state", state_to_str(v1::JobState::Running)),
                    ("stage", "starting"),
                    ("started_at", started.as_str()),
                    ("heartbeat_at", started.as_str()),
                ],
            )
            .ignore()
            .zrem(WAITING_ZSET, job_id)
            .ignore()
            .exec_async(&mut conn)
            .await?;
        self.publish_current_status(job_id).await
    }

    pub async fn update_progress(
        &self,
        job_id: &str,
        stage: &str,
        percent: u32,
        routes_found: u64,
    ) -> Result<()> {
        let mut conn = self.conn.clone();
        let now = now_rfc3339();
        let _: () = conn
            .hset_multiple(
                job_key(job_id),
                &[
                    ("stage", stage),
                    ("progress_percent", &percent.to_string()),
                    ("routes_found", &routes_found.to_string()),
                    ("heartbeat_at", now.as_str()),
                ],
            )
            .await?;
        self.publish_current_status(job_id).await
    }

    pub async fn is_cancel_requested(&self, job_id: &str) -> Result<bool> {
        let mut conn = self.conn.clone();
        let flag: Option<String> = conn.get(cancel_key(job_id)).await?;
        Ok(flag.is_some())
    }

    pub async fn mark_succeeded(&self, job_id: &str, result: &v1::JobResult) -> Result<()> {
        let mut conn = self.conn.clone();
        let finished = now_rfc3339();
        redis::pipe()
            .atomic()
            .hset_multiple(
                job_key(job_id),
                &[
                    ("state", state_to_str(v1::JobState::Succeeded)),
                    ("stage", "done"),
                    ("progress_percent", "100"),
                    ("finished_at", finished.as_str()),
                ],
            )
            .ignore()
            .set(result_key(job_id), result.encode_to_vec())
            .ignore()
            .exec_async(&mut conn)
            .await?;
        self.expire_job_keys(job_id).await?;
        self.publish_current_status(job_id).await
    }

    pub async fn mark_failed(&self, job_id: &str, code: &str, message: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let finished = now_rfc3339();
        let state = if code == "CANCELLED" {
            v1::JobState::Cancelled
        } else {
            v1::JobState::Failed
        };
        let _: () = conn
            .hset_multiple(
                job_key(job_id),
                &[
                    ("state", state_to_str(state)),
                    ("finished_at", finished.as_str()),
                    ("error_code", code),
                    ("error_message", message),
                ],
            )
            .await?;
        self.expire_job_keys(job_id).await?;
        self.publish_current_status(job_id).await
    }

    async fn expire_job_keys(&self, job_id: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let ttl = self.job_ttl_secs as i64;
        redis::pipe()
            .expire(job_key(job_id), ttl)
            .ignore()
            .expire(req_key(job_id), ttl)
            .ignore()
            .expire(result_key(job_id), ttl)
            .ignore()
            .expire(cancel_key(job_id), ttl)
            .ignore()
            .exec_async(&mut conn)
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Events
    // ------------------------------------------------------------------

    async fn publish_current_status(&self, job_id: &str) -> Result<()> {
        if let Some(status) = self.status(job_id).await? {
            self.publish_status(&status).await?;
        }
        Ok(())
    }

    pub async fn publish_status(&self, status: &v1::JobStatus) -> Result<()> {
        let mut conn = self.conn.clone();
        let payload = serde_json::to_string(status)?;
        let _: () = redis::cmd("PUBLISH")
            .arg(events_channel(&status.job_id))
            .arg(payload)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

fn parse_stream_entry(entry: &redis::Value) -> Option<(String, String)> {
    // entry = [id, [field, value, ...]]
    let redis::Value::Array(parts) = entry else {
        return None;
    };
    let id = match parts.first()? {
        redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
        redis::Value::SimpleString(s) => s.clone(),
        _ => return None,
    };
    let redis::Value::Array(fields) = parts.get(1)? else {
        return None;
    };
    let mut job_id = None;
    let mut iter = fields.iter();
    while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
        if let (redis::Value::BulkString(k), redis::Value::BulkString(v)) = (k, v)
            && k.as_slice() == b"job_id"
        {
            job_id = Some(String::from_utf8_lossy(v).to_string());
        }
    }
    Some((id, job_id?))
}

fn parse_xautoclaim_reply(reply: &redis::Value) -> Option<(String, String)> {
    // [next-cursor, [entries...], [deleted...]]
    let redis::Value::Array(parts) = reply else {
        return None;
    };
    let redis::Value::Array(entries) = parts.get(1)? else {
        return None;
    };
    entries.iter().find_map(parse_stream_entry)
}

fn parse_xreadgroup_reply(reply: &redis::Value) -> Option<(String, String)> {
    // [[stream, [entries...]]] or Nil on timeout
    let redis::Value::Array(streams) = reply else {
        return None;
    };
    let redis::Value::Array(stream) = streams.first()? else {
        return None;
    };
    let redis::Value::Array(entries) = stream.get(1)? else {
        return None;
    };
    entries.iter().find_map(parse_stream_entry)
}
