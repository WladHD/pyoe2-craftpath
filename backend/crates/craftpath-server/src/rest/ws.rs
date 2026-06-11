//! WebSocket live mode: `GET /api/v1/jobs/{id}/ws[?encoding=proto]`.
//!
//! On connect the server pushes the current `JobEvent{status}` snapshot, then
//! forwards every status change published on `cp:events:{id}`. While the job
//! is queued a 2s ticker re-checks the queue position (other jobs finishing
//! do not publish on this job's channel). The connection closes after a
//! terminal event.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::Response;
use futures_util::StreamExt;
use prost::Message as _;

use craftpath_proto::v1;

use super::AppState;
use crate::jobs::{events_channel, is_terminal};

#[derive(serde::Deserialize)]
pub struct WsParams {
    #[serde(default)]
    encoding: Option<String>,
}

pub async fn job_events_ws(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let proto = params.encoding.as_deref() == Some("proto");
    upgrade.on_upgrade(move |socket| async move {
        if let Err(e) = stream_job_events(state, socket, id, proto).await {
            tracing::debug!("websocket closed with error: {e:#}");
        }
    })
}

fn encode_event(status: v1::JobStatus, proto: bool) -> anyhow::Result<Message> {
    let event = v1::JobEvent {
        job_id: status.job_id.clone(),
        event: Some(v1::job_event::Event::Status(status)),
    };
    Ok(if proto {
        Message::Binary(event.encode_to_vec().into())
    } else {
        Message::Text(serde_json::to_string(&event)?.into())
    })
}

async fn stream_job_events(
    state: AppState,
    mut socket: WebSocket,
    job_id: String,
    proto: bool,
) -> anyhow::Result<()> {
    // snapshot first - also validates the job id
    let Some(status) = state.jobs.status(&job_id).await? else {
        socket
            .send(Message::Text(
                serde_json::to_string(&v1::Error {
                    code: "JOB_NOT_FOUND".into(),
                    message: format!("no job with id '{job_id}'"),
                    details: Default::default(),
                })?
                .into(),
            ))
            .await
            .ok();
        return Ok(());
    };

    let mut last_state = status.state;
    let mut last_position = status.queue_position;
    let terminal =
        is_terminal(v1::JobState::try_from(status.state).unwrap_or(v1::JobState::Unspecified));
    socket.send(encode_event(status, proto)?).await?;
    if terminal {
        return Ok(());
    }

    // dedicated pub/sub connection for this socket
    let mut pubsub = state.redis.get_async_pubsub().await?;
    pubsub.subscribe(events_channel(&job_id)).await?;
    let mut events = pubsub.on_message();

    let mut position_tick = tokio::time::interval(std::time::Duration::from_secs(2));

    loop {
        tokio::select! {
            maybe_msg = events.next() => {
                let Some(msg) = maybe_msg else { break };
                let payload: String = msg.get_payload()?;
                let Ok(status) = serde_json::from_str::<v1::JobStatus>(&payload) else {
                    continue;
                };
                last_state = status.state;
                last_position = status.queue_position;
                let done = is_terminal(
                    v1::JobState::try_from(status.state).unwrap_or(v1::JobState::Unspecified),
                );
                socket.send(encode_event(status, proto)?).await?;
                if done {
                    break;
                }
            }
            _ = position_tick.tick() => {
                // queue-position changes are not published on this channel
                let queued = v1::JobState::try_from(last_state)
                    .map(|s| s == v1::JobState::Queued)
                    .unwrap_or(false);
                if !queued {
                    continue;
                }
                if let Some(status) = state.jobs.status(&job_id).await? {
                    let changed = status.queue_position != last_position
                        || status.state != last_state;
                    last_state = status.state;
                    last_position = status.queue_position;
                    let done = is_terminal(
                        v1::JobState::try_from(status.state)
                            .unwrap_or(v1::JobState::Unspecified),
                    );
                    if changed {
                        socket.send(encode_event(status, proto)?).await?;
                    }
                    if done {
                        break;
                    }
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    None | Some(Err(_)) => break,          // client went away
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {}                       // ignore client chatter / pings handled by axum
                }
            }
        }
    }

    socket.send(Message::Close(None)).await.ok();
    Ok(())
}
