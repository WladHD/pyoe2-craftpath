//! REST + WebSocket API node. Pure I/O: enqueues jobs, reads status/results,
//! never computes.

pub mod proto_or_json;
mod ws;

use anyhow::{Context, Result};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use prost::Message as _;

use craftpath_core::api::item::ItemSnapshot;
use craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
use craftpath_proto::v1;

use crate::config::Config;
use crate::jobs::{JobsClient, is_terminal};
use proto_or_json::{ApiError, ProtoOrJson, WantsProto, respond};

#[derive(Clone)]
pub struct AppState {
    pub jobs: JobsClient,
    /// Dedicated client for pub/sub subscriptions (the managed connection
    /// cannot subscribe).
    pub redis: redis::Client,
    pub config: Config,
}

pub async fn serve(config: Config) -> Result<()> {
    let jobs = JobsClient::connect(&config.redis_url, config.job_ttl_secs).await?;
    jobs.ensure_group().await?;
    let redis = redis::Client::open(config.redis_url.as_str())?;

    let state = AppState {
        jobs,
        redis,
        config: config.clone(),
    };

    let app = router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("could not bind '{}'", config.bind_addr))?;
    tracing::info!("REST API listening on {}", config.bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/jobs", post(submit_job))
        .route("/api/v1/jobs/{id}", get(job_status))
        .route("/api/v1/jobs/{id}", delete(cancel_job))
        .route("/api/v1/jobs/{id}/result", get(job_result))
        .route("/api/v1/jobs/{id}/ws", get(ws::job_events_ws))
        .route("/api/v1/presets", get(list_presets))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

/// Minimal health router for worker pods.
pub async fn serve_health_only(addr: &str, jobs: JobsClient) -> Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(worker_readyz))
        .with_state(jobs);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("could not bind '{addr}'"))?;
    tracing::info!("worker health endpoints on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> Response {
    match state.jobs.ping().await {
        Ok(()) => (StatusCode::OK, "ready").into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, format!("redis: {e:#}")).into_response(),
    }
}

async fn worker_readyz(State(jobs): State<JobsClient>) -> Response {
    match jobs.ping().await {
        Ok(()) => (StatusCode::OK, "ready").into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, format!("redis: {e:#}")).into_response(),
    }
}

/// Reject structurally invalid requests before they hit the queue.
fn validate_request(request: &v1::SubmitJobRequest, config: &Config) -> Result<(), ApiError> {
    let invalid = |message: String| {
        Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            &message,
        ))
    };

    let Some(start) = request.start.as_ref() else {
        return invalid("missing 'start' item".into());
    };
    let Some(target) = request.target.as_ref() else {
        return invalid("missing 'target' item".into());
    };
    if let Err(e) = ItemSnapshot::try_from(start) {
        return invalid(format!("invalid 'start' item: {e}"));
    }
    if let Err(e) = ItemSnapshot::try_from(target) {
        return invalid(format!("invalid 'target' item: {e}"));
    }

    for p in &request.path_analyzers {
        let proto = v1::StatisticAnalyzerPathPreset::try_from(*p)
            .unwrap_or(v1::StatisticAnalyzerPathPreset::Unspecified);
        if StatisticAnalyzerPathPreset::try_from(proto).is_err() {
            return invalid(format!("unknown path analyzer preset value {p}"));
        }
    }
    for p in &request.group_analyzers {
        let proto = v1::StatisticAnalyzerCurrencyGroupPreset::try_from(*p)
            .unwrap_or(v1::StatisticAnalyzerCurrencyGroupPreset::Unspecified);
        if StatisticAnalyzerCurrencyGroupPreset::try_from(proto).is_err() {
            return invalid(format!("unknown group analyzer preset value {p}"));
        }
    }

    if let Some(limits) = request.limits.as_ref()
        && limits.max_ram_in_bytes > config.max_ram_limit_bytes
    {
        return invalid(format!(
            "max_ram_in_bytes {} exceeds the server limit of {}",
            limits.max_ram_in_bytes, config.max_ram_limit_bytes
        ));
    }

    Ok(())
}

async fn submit_job(
    State(state): State<AppState>,
    wants_proto: WantsProto,
    ProtoOrJson(request): ProtoOrJson<v1::SubmitJobRequest>,
) -> Result<Response, ApiError> {
    validate_request(&request, &state.config)?;

    let status = state
        .jobs
        .submit(&request)
        .await
        .map_err(ApiError::internal)?;

    let response = v1::SubmitJobResponse {
        job_id: status.job_id.clone(),
        status: Some(status),
    };
    Ok(respond(wants_proto, StatusCode::ACCEPTED, &response))
}

async fn job_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    wants_proto: WantsProto,
) -> Result<Response, ApiError> {
    let status = state
        .jobs
        .status(&id)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found(&id))?;
    Ok(respond(wants_proto, StatusCode::OK, &status))
}

async fn job_result(
    State(state): State<AppState>,
    Path(id): Path<String>,
    wants_proto: WantsProto,
) -> Result<Response, ApiError> {
    let status = state
        .jobs
        .status(&id)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found(&id))?;

    let state_enum = v1::JobState::try_from(status.state).unwrap_or(v1::JobState::Unspecified);

    match state_enum {
        v1::JobState::Succeeded => {
            let bytes = state
                .jobs
                .result_bytes(&id)
                .await
                .map_err(ApiError::internal)?
                .ok_or_else(|| ApiError::not_found(&id))?;
            let result = v1::JobResult::decode(bytes.as_slice())
                .map_err(|e| ApiError::internal(format!("corrupt stored result: {e}")))?;
            Ok(respond(wants_proto, StatusCode::OK, &result))
        }
        s if is_terminal(s) => Err(ApiError::new(
            StatusCode::CONFLICT,
            "JOB_NOT_SUCCESSFUL",
            &format!("job is {s:?}, no result available"),
        )),
        _ => Err(ApiError::new(
            StatusCode::CONFLICT,
            "JOB_NOT_FINISHED",
            "job has not finished yet; poll status or subscribe to the WebSocket",
        )),
    }
}

async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    wants_proto: WantsProto,
) -> Result<Response, ApiError> {
    let status = state
        .jobs
        .cancel(&id)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found(&id))?;
    Ok(respond(wants_proto, StatusCode::ACCEPTED, &status))
}

async fn list_presets(wants_proto: WantsProto) -> Response {
    let matrix_builders = [MatrixBuilderPreset::HappyPathMatrixBuilder]
        .iter()
        .map(|p| {
            let i = p.get_instance();
            v1::PresetInfo {
                name: i.0.get_name().to_string(),
                description: i.0.get_description().to_string(),
            }
        })
        .collect();

    let path_analyzers = [
        StatisticAnalyzerPathPreset::UniquePathChance,
        StatisticAnalyzerPathPreset::UniquePathEfficiency,
        StatisticAnalyzerPathPreset::UniquePathCost,
        StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy,
    ]
    .iter()
    .map(|p| {
        let i = p.get_instance();
        v1::PresetInfo {
            name: i.0.get_name().to_string(),
            description: i.0.get_description().to_string(),
        }
    })
    .collect();

    let group_analyzers = [
        StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance,
        StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy,
    ]
    .iter()
    .map(|p| {
        let i = p.get_instance();
        v1::PresetInfo {
            name: i.0.get_name().to_string(),
            description: i.0.get_description().to_string(),
        }
    })
    .collect();

    let list = v1::PresetList {
        matrix_builders,
        path_analyzers,
        group_analyzers,
    };
    respond(wants_proto, StatusCode::OK, &list)
}
