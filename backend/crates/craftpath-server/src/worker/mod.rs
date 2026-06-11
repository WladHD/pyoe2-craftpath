//! Worker mode: consumes jobs from the Redis stream one at a time (rayon
//! already saturates the pod's CPUs; scale out with more pods).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use anyhow::{Context, Result};

use craftpath_core::api::calculator::{Calculator, GroupRoute};
use craftpath_core::api::errors::CraftPathError;
use craftpath_core::api::item::ItemSnapshot;
use craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
use craftpath_core::api::session::{CalculationConfig, CraftSession};
use craftpath_core::progress::ProgressSink;
use craftpath_proto::convert::{group_route_to_proto, item_route_to_proto};
use craftpath_proto::v1;

use crate::config::Config;
use crate::jobs::JobsClient;
use crate::league::load_league_data;

/// Progress state shared between the rayon-driven calculation (writes
/// atomics; never does I/O) and the async sampler task (flushes to Redis).
pub struct SharedProgress {
    message: Mutex<String>,
    /// "<phase>" plus the percent attributed when the phase was entered.
    phase: Mutex<(String, u32)>,
    routes_found: AtomicU64,
    cancelled: AtomicBool,
}

impl SharedProgress {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            message: Mutex::new(String::new()),
            phase: Mutex::new(("starting".to_string(), 0)),
            routes_found: AtomicU64::new(0),
            cancelled: AtomicBool::new(false),
        })
    }

    pub fn set_phase(&self, phase: &str, percent: u32) {
        *self.phase.lock().unwrap() = (phase.to_string(), percent);
    }

    pub fn snapshot(&self) -> (String, u32, u64) {
        let (phase, percent) = self.phase.lock().unwrap().clone();
        let message = self.message.lock().unwrap().clone();
        let stage = if message.is_empty() {
            phase
        } else {
            format!("{phase}: {message}")
        };
        (stage, percent, self.routes_found.load(Ordering::Relaxed))
    }

    pub fn request_cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl ProgressSink for SharedProgress {
    fn report(&self, message: &str, current: u64, _total: Option<u64>) {
        *self.message.lock().unwrap() = message.to_string();
        self.routes_found.store(current, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

pub async fn run(config: Config) -> Result<()> {
    let jobs = JobsClient::connect(&config.redis_url, config.job_ttl_secs).await?;
    jobs.ensure_group().await?;

    // health/readiness listener for k8s probes
    let health_jobs = jobs.clone();
    let health_addr = config.worker_health_addr.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::rest::serve_health_only(&health_addr, health_jobs).await {
            tracing::error!("worker health listener failed: {e:#}");
        }
    });

    let consumer = format!(
        "worker-{}-{}",
        std::env::var("HOSTNAME").unwrap_or_else(|_| "local".to_string()),
        uuid::Uuid::now_v7()
    );
    tracing::info!("Worker '{consumer}' waiting for jobs ...");

    loop {
        let claimed = match jobs.claim_next(&consumer, 5_000).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("claim failed (redis hiccup?): {e:#}");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let Some((entry_id, job_id, delivery_count)) = claimed else {
            continue;
        };

        if delivery_count > 2 {
            tracing::warn!("job {job_id} was redelivered {delivery_count} times, failing it");
            jobs.mark_failed(&job_id, "INTERNAL", "worker died repeatedly while processing this job")
                .await
                .ok();
            jobs.ack(&entry_id).await.ok();
            continue;
        }

        if let Err(e) = process_job(&config, &jobs, &job_id).await {
            tracing::error!("job {job_id} failed: {e:#}");
            jobs.mark_failed(&job_id, "INTERNAL", &format!("{e:#}")).await.ok();
        }

        jobs.ack(&entry_id).await.ok();
    }
}

async fn process_job(config: &Config, jobs: &JobsClient, job_id: &str) -> Result<()> {
    // cancelled while queued (or expired request)?
    if jobs.is_cancel_requested(job_id).await? {
        tracing::info!("job {job_id} was cancelled while queued, skipping");
        return Ok(());
    }

    let Some(request) = jobs.load_request(job_id).await? else {
        jobs.mark_failed(job_id, "JOB_NOT_FOUND", "job request payload expired or missing")
            .await?;
        return Ok(());
    };

    jobs.mark_running(job_id).await?;
    tracing::info!("processing job {job_id}");

    let progress = SharedProgress::new();

    // sampler: flush progress + heartbeat to redis every 500ms
    let sampler = {
        let jobs = jobs.clone();
        let job_id = job_id.to_string();
        let progress = Arc::clone(&progress);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let snap = progress.snapshot();
                // heartbeat must refresh even without visible progress
                if let Err(e) = jobs
                    .update_progress(&job_id, &snap.0, snap.1, snap.2)
                    .await
                {
                    tracing::warn!("progress flush failed: {e:#}");
                }
            }
        })
    };

    // cancellation poller: flag is observed by the hot loops via the sink
    let canceller = {
        let jobs = jobs.clone();
        let job_id = job_id.to_string();
        let progress = Arc::clone(&progress);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                match jobs.is_cancel_requested(&job_id).await {
                    Ok(true) => {
                        progress.request_cancel();
                        break;
                    }
                    Ok(false) => {}
                    Err(e) => tracing::warn!("cancel poll failed: {e:#}"),
                }
            }
        })
    };

    let timeout_secs = request
        .limits
        .as_ref()
        .and_then(|l| l.timeout_seconds)
        .map(u64::from)
        .unwrap_or(config.job_timeout_secs);

    let compute = {
        let config = config.clone();
        let request = request.clone();
        let progress = Arc::clone(&progress);
        tokio::task::spawn_blocking(move || run_job(&config, &request, progress.as_ref()))
    };

    let timed_out;
    let outcome = tokio::select! {
        res = compute => {
            timed_out = false;
            res.context("calculation task panicked")?
        }
    _ = tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)) => {
            timed_out = true;
            progress.request_cancel();
            // the hot loops observe the flag within their next check window;
            // nothing else to await here - mark the job and move on.
            Err(CraftPathError::Cancelled().into())
        }
    };

    sampler.abort();
    canceller.abort();

    match outcome {
        Ok(result) => jobs.mark_succeeded(job_id, &result).await?,
        Err(e) => {
            let (code, message) = if timed_out {
                (
                    "TIMEOUT".to_string(),
                    format!("job exceeded its wall-clock timeout of {timeout_secs}s"),
                )
            } else {
                (error_code(&e), format!("{e:#}"))
            };
            jobs.mark_failed(job_id, &code, &message).await?;
        }
    }

    Ok(())
}

fn error_code(err: &anyhow::Error) -> String {
    if let Some(convert) = err.downcast_ref::<craftpath_proto::ConvertError>() {
        let _ = convert;
        return "INVALID_REQUEST".to_string();
    }
    match err.downcast_ref::<CraftPathError>() {
        Some(CraftPathError::ItemMatrixCouldNotReachTarget()) => "TARGET_UNREACHABLE",
        Some(
            CraftPathError::ItemUnreachable(..)
            | CraftPathError::ItemUnreachableMinLevelConstraint(..),
        ) => "AFFIX_UNREACHABLE",
        Some(CraftPathError::RamLimitReached(..)) => "RAM_LIMIT_REACHED",
        Some(CraftPathError::EssenceIntermediaryStepRequired(..)) => {
            "ESSENCE_INTERMEDIARY_REQUIRED"
        }
        Some(CraftPathError::Cancelled()) => "CANCELLED",
        Some(_) => "PROVIDER_DATA_ERROR",
        None => "INTERNAL",
    }
    .to_string()
}

/// The blocking calculation pipeline. Mirrors the CLI flow:
/// league data -> matrix -> path analyzers -> group analyzers -> render.
pub fn run_job(
    config: &Config,
    request: &v1::SubmitJobRequest,
    progress: &SharedProgress,
) -> Result<v1::JobResult> {
    // -------- validate / convert the request --------
    let start = ItemSnapshot::try_from(
        request
            .start
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing 'start' item"))?,
    )?;
    let target = ItemSnapshot::try_from(
        request
            .target
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing 'target' item"))?,
    )?;

    let matrix_builder_proto =
        v1::MatrixBuilderPreset::try_from(request.matrix_builder)
            .unwrap_or(v1::MatrixBuilderPreset::Unspecified);
    let matrix_builder = if matrix_builder_proto == v1::MatrixBuilderPreset::Unspecified {
        MatrixBuilderPreset::HappyPathMatrixBuilder
    } else {
        MatrixBuilderPreset::try_from(matrix_builder_proto)?
    };

    let mut path_presets: Vec<StatisticAnalyzerPathPreset> = Vec::new();
    for p in &request.path_analyzers {
        let proto = v1::StatisticAnalyzerPathPreset::try_from(*p)
            .unwrap_or(v1::StatisticAnalyzerPathPreset::Unspecified);
        path_presets.push(StatisticAnalyzerPathPreset::try_from(proto)?);
    }
    let mut group_presets: Vec<StatisticAnalyzerCurrencyGroupPreset> = Vec::new();
    for p in &request.group_analyzers {
        let proto = v1::StatisticAnalyzerCurrencyGroupPreset::try_from(*p)
            .unwrap_or(v1::StatisticAnalyzerCurrencyGroupPreset::Unspecified);
        group_presets.push(StatisticAnalyzerCurrencyGroupPreset::try_from(proto)?);
    }
    if path_presets.is_empty() && group_presets.is_empty() {
        path_presets.push(StatisticAnalyzerPathPreset::UniquePathChance);
    }

    let limits = request.limits.clone().unwrap_or(v1::Limits {
        max_routes: 5,
        max_ram_in_bytes: 1_000_000_000,
        timeout_seconds: None,
    });
    let max_routes = if limits.max_routes == 0 { 5 } else { limits.max_routes };
    let max_ram = limits
        .max_ram_in_bytes
        .clamp(1, config.max_ram_limit_bytes);

    let options = request.result_options.clone().unwrap_or(v1::ResultOptions {
        include_pretty_strings: true,
        include_route_snapshots: false,
        top_n_pretty: None,
    });
    let top_n_pretty = options.top_n_pretty.unwrap_or(max_routes) as usize;

    let league = if request.league.is_empty() {
        config.default_league.clone()
    } else {
        request.league.clone()
    };

    // -------- league data --------
    progress.set_phase("fetching_league_data", 1);
    let (item_provider, market_info) = load_league_data(config, &league)?;

    let session = CraftSession::new(&item_provider, &market_info)
        .with_config(
            CalculationConfig::builder()
                .max_routes(max_routes)
                .max_ram(max_ram)
                .league(league.clone())
                .build(),
        )
        .with_progress(progress);

    // -------- matrix --------
    progress.set_phase("building_matrix", 5);
    let builder = matrix_builder.get_instance();
    let calculator = session.build_matrix_with(start, target, builder.0.as_ref())?;

    let mut result = v1::JobResult {
        matrix_size: calculator.matrix.len() as u64,
        path_results: vec![],
        group_results: vec![],
        pretty_text: String::new(),
    };
    let mut pretty_text = String::new();

    // -------- group analyzers (first, like the CLI, so route rendering can
    // reference the groups) --------
    let analyzer_span = 55u32;
    let total_analyzers = (path_presets.len() + group_presets.len()).max(1) as u32;
    let mut analyzer_index = 0u32;

    let mut groups_by_preset: Vec<(StatisticAnalyzerCurrencyGroupPreset, Vec<GroupRoute>)> =
        Vec::new();

    for preset in &group_presets {
        let instance = preset.get_instance();
        analyzer_index += 1;
        progress.set_phase(
            &format!("analyzing:{}", instance.0.get_name()),
            40 + analyzer_span * analyzer_index / total_analyzers,
        );

        let groups = session.analyze_groups(&calculator, instance.0.as_ref())?;

        let mut proto_groups: Vec<v1::GroupRoute> = groups
            .iter()
            .map(|g| group_route_to_proto(g, Some(&item_provider)))
            .collect();

        if options.include_pretty_strings {
            for (i, group) in groups.iter().take(top_n_pretty).enumerate() {
                let pretty = session.render_group(group, instance.0.as_ref());
                pretty_text.push_str(&pretty);
                pretty_text.push('\n');
                proto_groups[i].pretty = Some(pretty);
            }
        }

        result.group_results.push(v1::GroupAnalyzerResult {
            preset: v1::StatisticAnalyzerCurrencyGroupPreset::from(preset) as i32,
            groups: proto_groups,
            lower_is_better: instance.0.lower_is_better(),
            unit_type: instance.0.get_unit_type().to_string(),
        });

        groups_by_preset.push((preset.clone(), groups));
    }

    // routes can be annotated with group info if any group analysis ran
    let groups_for_pretty: Option<&Vec<GroupRoute>> =
        groups_by_preset.first().map(|(_, groups)| groups);

    // -------- path analyzers --------
    for preset in &path_presets {
        let instance = preset.get_instance();
        analyzer_index += 1;
        progress.set_phase(
            &format!("analyzing:{}", instance.0.get_name()),
            40 + analyzer_span * analyzer_index / total_analyzers,
        );

        let routes = session.analyze_paths(&calculator, instance.0.as_ref())?;

        let mut proto_routes: Vec<v1::ItemRoute> = routes
            .iter()
            .map(|r| {
                let mut proto = item_route_to_proto(r, Some(&item_provider));
                if options.include_route_snapshots {
                    for node in proto.route.iter_mut() {
                        if let Some(matrix_node) = calculator.matrix.get(&node.item_matrix_id) {
                            node.resolved_item =
                                Some(v1::ItemSnapshot::from(&matrix_node.item.snapshot));
                        }
                    }
                }
                proto
            })
            .collect();

        if options.include_pretty_strings {
            pretty_text.push_str(&format!(
                "\n===== Results for '{}' =====\n",
                instance.0.get_name()
            ));
            for (i, route) in routes.iter().take(top_n_pretty).enumerate() {
                let pretty =
                    session.render_route(&calculator, route, instance.0.as_ref(), groups_for_pretty);
                pretty_text.push_str(&pretty);
                pretty_text.push('\n');
                proto_routes[i].pretty = Some(pretty);
            }
        }

        result.path_results.push(v1::PathAnalyzerResult {
            preset: v1::StatisticAnalyzerPathPreset::from(preset) as i32,
            routes: proto_routes,
            lower_is_better: instance.0.lower_is_better(),
            unit_type: instance.0.get_unit_type().to_string(),
        });
    }

    progress.set_phase("finalizing", 98);
    if options.include_pretty_strings {
        result.pretty_text = pretty_text;
    }

    Ok(result)
}
