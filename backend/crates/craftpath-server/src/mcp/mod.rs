//! MCP mode: exposes the job API as Model Context Protocol tools so LLM
//! clients can submit and inspect crafting-path calculations. Like the REST
//! node this is just another Redis producer — no computation happens here.

use anyhow::{Context, Result};
use prost::Message as _;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};

use craftpath_proto::v1;

use crate::config::Config;
use crate::jobs::JobsClient;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubmitCalculationParams {
    /// Start item as canonical JSON of craftpath.v1.ItemSnapshot, e.g.
    /// {"itemLevel":81,"rarity":"ITEM_RARITY_NORMAL","baseId":20,"affixes":[]}.
    pub start_item: serde_json::Value,
    /// Target item in the same shape; affix tier bounds
    /// (AFFIX_TIER_LEVEL_BOUNDS_MINIMUM/EXACT) encode the constraints.
    pub target_item: serde_json::Value,
    /// PoE2 league name (defaults to the server's configured league).
    pub league: Option<String>,
    /// Number of best routes to return (default 5).
    pub max_routes: Option<u32>,
    /// RAM budget for the calculation in bytes (default 1 GB).
    pub max_ram_bytes: Option<u64>,
    /// Also compute currency-group statistics (slower; default false).
    #[serde(default)]
    pub include_groups: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct JobIdParams {
    pub job_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetResultParams {
    pub job_id: String,
    /// How many routes to include (default 5).
    pub top_n: Option<u32>,
    /// "pretty" (default; human-readable text, best for LLMs) or "json".
    pub format: Option<String>,
}

#[derive(Clone)]
pub struct CraftPathMcp {
    jobs: JobsClient,
    config: Config,
    tool_router: ToolRouter<CraftPathMcp>,
}

fn internal(err: impl std::fmt::Display) -> McpError {
    McpError::internal_error(err.to_string(), None)
}

fn invalid(err: impl std::fmt::Display) -> McpError {
    McpError::invalid_params(err.to_string(), None)
}

#[tool_router]
impl CraftPathMcp {
    pub fn new(jobs: JobsClient, config: Config) -> Self {
        Self {
            jobs,
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Submit a crafting-path calculation job. Returns the job id; poll get_job_status until SUCCEEDED, then call get_job_result. Calculations can take minutes."
    )]
    async fn submit_calculation(
        &self,
        Parameters(params): Parameters<SubmitCalculationParams>,
    ) -> Result<CallToolResult, McpError> {
        let start: v1::ItemSnapshot = serde_json::from_value(params.start_item)
            .map_err(|e| invalid(format!("start_item: {e}")))?;
        let target: v1::ItemSnapshot = serde_json::from_value(params.target_item)
            .map_err(|e| invalid(format!("target_item: {e}")))?;

        let request = v1::SubmitJobRequest {
            league: params
                .league
                .unwrap_or_else(|| self.config.default_league.clone()),
            start: Some(start),
            target: Some(target),
            matrix_builder: v1::MatrixBuilderPreset::HappyPath as i32,
            path_analyzers: vec![
                v1::StatisticAnalyzerPathPreset::UniquePathChance as i32,
                v1::StatisticAnalyzerPathPreset::UniquePathEfficiency as i32,
                v1::StatisticAnalyzerPathPreset::UniquePathCost as i32,
            ],
            group_analyzers: if params.include_groups {
                vec![v1::StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance as i32]
            } else {
                vec![]
            },
            limits: Some(v1::Limits {
                max_routes: params.max_routes.unwrap_or(5),
                max_ram_in_bytes: params.max_ram_bytes.unwrap_or(1_000_000_000),
                timeout_seconds: None,
            }),
            result_options: Some(v1::ResultOptions {
                include_pretty_strings: true,
                include_route_snapshots: false,
                top_n_pretty: params.max_routes,
            }),
        };

        let status = self.jobs.submit(&request).await.map_err(internal)?;
        let summary = serde_json::json!({
            "job_id": status.job_id,
            "state": "QUEUED",
            "queue_position": status.queue_position,
        });
        Ok(CallToolResult::success(vec![Content::text(
            summary.to_string(),
        )]))
    }

    #[tool(
        description = "Get the status of a calculation job: state (QUEUED/RUNNING/SUCCEEDED/FAILED/CANCELLED), queue position, progress and error info."
    )]
    async fn get_job_status(
        &self,
        Parameters(params): Parameters<JobIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let status = self
            .jobs
            .status(&params.job_id)
            .await
            .map_err(internal)?
            .ok_or_else(|| invalid(format!("no job with id '{}'", params.job_id)))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&status).map_err(internal)?,
        )]))
    }

    #[tool(
        description = "Fetch the result of a SUCCEEDED job. format='pretty' (default) returns human-readable route descriptions; format='json' returns the structured result."
    )]
    async fn get_job_result(
        &self,
        Parameters(params): Parameters<GetResultParams>,
    ) -> Result<CallToolResult, McpError> {
        let status = self
            .jobs
            .status(&params.job_id)
            .await
            .map_err(internal)?
            .ok_or_else(|| invalid(format!("no job with id '{}'", params.job_id)))?;

        let state = v1::JobState::try_from(status.state).unwrap_or(v1::JobState::Unspecified);
        if state != v1::JobState::Succeeded {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "job is {state:?}, no result yet — poll get_job_status"
            ))]));
        }

        let bytes = self
            .jobs
            .result_bytes(&params.job_id)
            .await
            .map_err(internal)?
            .ok_or_else(|| internal("result expired"))?;
        let mut result = v1::JobResult::decode(bytes.as_slice()).map_err(internal)?;

        let top_n = params.top_n.unwrap_or(5) as usize;
        for path_result in result.path_results.iter_mut() {
            path_result.routes.truncate(top_n);
        }
        for group_result in result.group_results.iter_mut() {
            group_result.groups.truncate(top_n);
        }

        let text = match params.format.as_deref() {
            Some("json") => serde_json::to_string(&result).map_err(internal)?,
            _ => {
                if result.pretty_text.is_empty() {
                    serde_json::to_string(&result).map_err(internal)?
                } else {
                    result.pretty_text
                }
            }
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Cancel a queued or running calculation job.")]
    async fn cancel_job(
        &self,
        Parameters(params): Parameters<JobIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let status = self
            .jobs
            .cancel(&params.job_id)
            .await
            .map_err(internal)?
            .ok_or_else(|| invalid(format!("no job with id '{}'", params.job_id)))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&status).map_err(internal)?,
        )]))
    }

    #[tool(
        description = "List the available matrix-builder and statistic-analyzer presets with descriptions."
    )]
    fn list_presets(&self) -> Result<CallToolResult, McpError> {
        use craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
        use craftpath_core::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
        use craftpath_core::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;

        let mut out = String::from("Matrix builders:\n");
        for p in [MatrixBuilderPreset::HappyPathMatrixBuilder] {
            let i = p.get_instance();
            out.push_str(&format!(
                "- {}: {}\n",
                i.0.get_name(),
                i.0.get_description()
            ));
        }
        out.push_str("\nPath analyzers:\n");
        for p in [
            StatisticAnalyzerPathPreset::UniquePathChance,
            StatisticAnalyzerPathPreset::UniquePathEfficiency,
            StatisticAnalyzerPathPreset::UniquePathCost,
            StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy,
        ] {
            let i = p.get_instance();
            out.push_str(&format!(
                "- {}: {}\n",
                i.0.get_name(),
                i.0.get_description()
            ));
        }
        out.push_str("\nGroup analyzers:\n");
        for p in [
            StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance,
            StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy,
        ] {
            let i = p.get_instance();
            out.push_str(&format!(
                "- {}: {}\n",
                i.0.get_name(),
                i.0.get_description()
            ));
        }
        Ok(CallToolResult::success(vec![Content::text(out)]))
    }
}

#[tool_handler]
impl ServerHandler for CraftPathMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "pyoe2-craftpath: calculates optimal Path of Exile 2 crafting paths \
                 between a start and a target item. submit_calculation enqueues a job \
                 (can run minutes); poll get_job_status; fetch get_job_result when \
                 SUCCEEDED."
                    .to_string(),
            )
    }
}

pub async fn serve(config: Config, transport: &str) -> Result<()> {
    let jobs = JobsClient::connect(&config.redis_url, config.job_ttl_secs).await?;
    jobs.ensure_group().await?;

    match transport {
        "stdio" => {
            let service = CraftPathMcp::new(jobs, config)
                .serve(stdio())
                .await
                .context("mcp stdio serve failed")?;
            service.waiting().await?;
            Ok(())
        }
        "http" => {
            let bind_addr = config.bind_addr.clone();
            let mcp = CraftPathMcp::new(jobs.clone(), config);

            let ct = tokio_util::sync::CancellationToken::new();
            let service = StreamableHttpService::new(
                move || Ok(mcp.clone()),
                LocalSessionManager::default().into(),
                StreamableHttpServerConfig::default()
                    // service sits behind the k8s ingress/service; host
                    // filtering is handled there
                    .disable_allowed_hosts()
                    .with_cancellation_token(ct.child_token()),
            );

            let router = axum::Router::new()
                .nest_service("/mcp", service)
                .route("/healthz", axum::routing::get(|| async { "ok" }))
                .route(
                    "/readyz",
                    axum::routing::get(move || {
                        let jobs = jobs.clone();
                        async move {
                            match jobs.ping().await {
                                Ok(()) => (axum::http::StatusCode::OK, "ready".to_string()),
                                Err(e) => (
                                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                                    format!("redis: {e:#}"),
                                ),
                            }
                        }
                    }),
                );

            let listener = tokio::net::TcpListener::bind(&bind_addr)
                .await
                .with_context(|| format!("could not bind '{bind_addr}'"))?;
            tracing::info!("MCP (streamable HTTP) listening on {bind_addr}/mcp");
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = tokio::signal::ctrl_c().await;
                    ct.cancel();
                })
                .await?;
            Ok(())
        }
        other => anyhow::bail!("unknown mcp transport '{other}' (use \"http\" or \"stdio\")"),
    }
}
