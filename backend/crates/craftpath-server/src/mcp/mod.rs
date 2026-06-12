//! MCP mode: exposes the job API as Model Context Protocol tools so LLM
//! clients can submit and inspect crafting-path calculations. Like the REST
//! node this is just another Redis producer - no computation happens here.

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

use futures_util::StreamExt;

use craftpath_core::api::types::{AffixId, BaseItemId, ItemLevel, THashMap};
use craftpath_core::features::{craftspec, inspect};
use craftpath_core::prelude::{CraftCurrencyEnum, ItemSnapshot};
use craftpath_proto::v1;

use crate::config::Config;
use crate::jobs::{events_channel, is_terminal, JobsClient};
use crate::providers::ProviderCache;
use crate::{meta, pob};

pub mod lookup;

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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchAffixesParams {
    /// Case-insensitive substring of the affix description, e.g. "physical"
    /// or "attack speed". Empty lists everything (use limit!).
    pub query: Option<String>,
    /// Restrict to (and attach tier tables of) this base item id.
    pub base_id: Option<u16>,
    /// prefix | suffix | socket
    pub location: Option<String>,
    /// base | essence | desecrated
    pub affix_class: Option<String>,
    pub league: Option<String>,
    /// Max results (default 25, cap 100).
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetBaseItemsParams {
    /// Case-insensitive substring of the base group name, e.g. "bow".
    pub query: Option<String>,
    pub league: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LeagueParams {
    pub league: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ParseItemParams {
    /// Craft of Exile emulator export JSON (in-game clipboard text is not
    /// supported yet).
    pub item_json: String,
    pub league: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ItemParams {
    /// Item as canonical JSON of craftpath.v1.ItemSnapshot.
    pub item: serde_json::Value,
    pub league: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SimulateActionParams {
    /// Item as canonical JSON of craftpath.v1.ItemSnapshot.
    pub item: serde_json::Value,
    /// Currency name, e.g. "Exalted Orb", "Greater Chaos Orb", "desecrate"
    /// or an essence name.
    pub currency: String,
    pub league: Option<String>,
    /// Max outcomes to return (default 25, cap 100).
    pub top_n: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ParseCraftSpecParams {
    /// EPPSSA craft spec: letters E (essence) / P (prefix) / S (suffix) /
    /// A (abyss/desecrated), optional tier digits ("P1" = tier 1 or
    /// better, "P2x" = exactly tier 2), optional "[name]" or "[#id]"
    /// bindings, optional "!" for fractured. Example: "P[phys]1 P S1 S A".
    pub spec: String,
    /// Base item id (resolve via get_base_items).
    pub base_id: u16,
    /// Target item level (default 81).
    pub item_level: Option<u8>,
    /// Pin slots by 0-based index to affix ids, e.g. {"0": 1234}.
    pub bindings: Option<std::collections::HashMap<String, u16>>,
    pub league: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetMetaItemsParams {
    /// Item class or slot, e.g. "bow", "ring".
    pub item_class: Option<String>,
    /// Character class, e.g. "Amazon".
    pub char_class: Option<String>,
    /// "leveling" or "endgame".
    pub level_bracket: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AwaitJobParams {
    pub job_id: String,
    /// How long to wait for a state change (default 30s, cap 120s).
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ImportPobBuildParams {
    /// Path of Building (PoE2) share code.
    pub code: String,
}

#[derive(Clone)]
pub struct CraftPathMcp {
    jobs: JobsClient,
    config: Config,
    providers: ProviderCache,
    redis: redis::Client,
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
    pub fn new(jobs: JobsClient, config: Config) -> Result<Self> {
        let redis = redis::Client::open(config.redis_url.as_str())
            .with_context(|| format!("invalid REDIS_URL '{}'", config.redis_url))?;
        Ok(Self {
            jobs,
            providers: ProviderCache::new(config.clone()),
            redis,
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Parse an `ItemParams`-style item value into a domain snapshot.
    async fn parse_item_value(
        &self,
        item: serde_json::Value,
        league: Option<&str>,
    ) -> Result<(ItemSnapshot, std::sync::Arc<crate::providers::LeagueData>), McpError> {
        let data = self.providers.get(league).await.map_err(internal)?;
        let proto: v1::ItemSnapshot =
            serde_json::from_value(item).map_err(|e| invalid(format!("item: {e}")))?;
        let snapshot =
            ItemSnapshot::try_from(&proto).map_err(|e| invalid(format!("item: {e}")))?;
        Ok((snapshot, data))
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
                "job is {state:?}, no result yet - poll get_job_status"
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

    #[tool(
        description = "Search affixes by description substring with optional base/location/class filters. Resolves human names ('attack speed') to affix ids; pass base_id to also get the tier table (weights, min item levels) on that base."
    )]
    async fn search_affixes(
        &self,
        Parameters(params): Parameters<SearchAffixesParams>,
    ) -> Result<CallToolResult, McpError> {
        let data = self
            .providers
            .get(params.league.as_deref())
            .await
            .map_err(internal)?;

        let query = params.query.unwrap_or_default().trim().to_lowercase();
        let location = params
            .location
            .as_deref()
            .map(lookup::parse_location)
            .transpose()
            .map_err(invalid)?;
        let class = params
            .affix_class
            .as_deref()
            .map(lookup::parse_affix_class)
            .transpose()
            .map_err(invalid)?;
        let base_pool = params
            .base_id
            .map(|id| {
                data.items
                    .lookup_base_item_mods(&BaseItemId::from(id))
                    .map_err(invalid)
            })
            .transpose()?;
        let limit = params.limit.unwrap_or(25).min(100);

        let mut results: Vec<serde_json::Value> = Vec::new();
        let mut total = 0usize;
        let mut defs: Vec<_> = data.items.cache_affix_def.iter().collect();
        defs.sort_by(|a, b| a.1.description_template.cmp(&b.1.description_template));

        for (affix_id, def) in defs {
            if let Some(loc) = &location {
                if def.affix_location != *loc {
                    continue;
                }
            }
            if let Some(cls) = &class {
                if def.affix_class != *cls {
                    continue;
                }
            }
            if !query.is_empty() && !def.description_template.to_lowercase().contains(&query) {
                continue;
            }
            let tiers = match &base_pool {
                None => None,
                Some(pool) => match pool.get(affix_id) {
                    // base filter active and affix not on this base: skip
                    None => continue,
                    Some(tier_map) => {
                        let mut tiers: Vec<serde_json::Value> = tier_map
                            .iter()
                            .map(|(tier, meta)| {
                                serde_json::json!({
                                    "tier": tier.get_raw_value(),
                                    "weight": meta.weight.get_raw_value(),
                                    "min_item_level": meta.min_item_level.get_raw_value(),
                                })
                            })
                            .collect();
                        tiers.sort_by_key(|t| t["tier"].as_u64());
                        Some(tiers)
                    }
                },
            };
            total += 1;
            if results.len() < limit {
                results.push(serde_json::json!({
                    "affix_id": affix_id.get_raw_value(),
                    "description": def.description_template,
                    "location": def.affix_location,
                    "class": def.affix_class,
                    "exclusive_groups": def.exlusive_groups,
                    "tiers": tiers,
                }));
            }
        }

        let out = serde_json::json!({ "total_matches": total, "results": results });
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "List base item groups (name, max affixes, max sockets, rarity ceiling) and the base item ids in each, optionally filtered by name ('bow')."
    )]
    async fn get_base_items(
        &self,
        Parameters(params): Parameters<GetBaseItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let data = self
            .providers
            .get(params.league.as_deref())
            .await
            .map_err(internal)?;
        let query = params.query.unwrap_or_default().trim().to_lowercase();

        let mut groups: Vec<serde_json::Value> = Vec::new();
        let mut group_defs: Vec<_> = data.items.base_group_definition.iter().collect();
        group_defs.sort_by_key(|(id, _)| *id.get_raw_value());

        for (group_id, def) in group_defs {
            if !query.is_empty() && !def.name_base_group.to_lowercase().contains(&query) {
                continue;
            }
            let mut base_ids: Vec<u16> = data
                .items
                .cache_base_group_table
                .iter()
                .filter(|(_, gid)| *gid == group_id)
                .map(|(bid, _)| *bid.get_raw_value())
                .collect();
            base_ids.sort_unstable();
            groups.push(serde_json::json!({
                "base_group_id": group_id.get_raw_value(),
                "name": def.name_base_group,
                "max_affix": def.max_affix,
                "max_sockets": def.max_sockets,
                "max_rarity": if def.is_rare { "Rare" } else { "Magic" },
                "base_item_ids": base_ids,
            }));
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({ "groups": groups }).to_string(),
        )]))
    }

    #[tool(
        description = "Current currency prices in divines plus the divine->exalted and divine->chaos exchange rates for a league."
    )]
    async fn get_currency_prices(
        &self,
        Parameters(params): Parameters<LeagueParams>,
    ) -> Result<CallToolResult, McpError> {
        let data = self
            .providers
            .get(params.league.as_deref())
            .await
            .map_err(internal)?;

        let mut prices: Vec<(String, f64)> = data
            .market
            .cache_market_prices
            .iter()
            .map(|(name, price)| (name.get_raw_value().clone(), price.get_divine_value()))
            .collect();
        prices.sort_by(|a, b| a.0.cmp(&b.0));
        let prices: serde_json::Map<String, serde_json::Value> = prices
            .into_iter()
            .map(|(k, v)| (k, serde_json::json!(v)))
            .collect();

        let out = serde_json::json!({
            "divine_to_exalted": data.market.cache_exchange_rate_div_to_exalted,
            "divine_to_chaos": data.market.cache_exchange_rate_div_to_chaos,
            "prices_in_divines": prices,
        });
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "Parse a Craft of Exile emulator item export (JSON) into a craftpath.v1.ItemSnapshot usable with the other tools, plus a human-readable rendering."
    )]
    async fn parse_item(
        &self,
        Parameters(params): Parameters<ParseItemParams>,
    ) -> Result<CallToolResult, McpError> {
        use craftpath_core::external_api::coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport;

        let data = self
            .providers
            .get(params.league.as_deref())
            .await
            .map_err(internal)?;
        let snapshot = CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
            &params.item_json,
            &data.items,
        )
        .map_err(|e| invalid(format!("item_json: {e:#}")))?;

        let proto = v1::ItemSnapshot::from(&snapshot);
        let out = serde_json::json!({
            "item": serde_json::to_value(&proto).map_err(internal)?,
            "pretty": snapshot.to_pretty_string(&data.items, true),
        });
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "Which currencies can legally be applied to this item right now, each with a risk class (safe / destructive reroll / removal risk / permanent / irreversible), a reason and its market price. Answers 'what can I do next?' and 'which steps are dangerous?'."
    )]
    async fn get_legal_actions(
        &self,
        Parameters(params): Parameters<ItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let (snapshot, data) = self
            .parse_item_value(params.item, params.league.as_deref())
            .await?;

        let actions = inspect::legal_actions(&snapshot, &data.items).map_err(invalid)?;
        let actions: Vec<serde_json::Value> = actions
            .into_iter()
            .map(|a| {
                // the essence entry is a placeholder id, it has no single price
                let price = match &a.currency {
                    CraftCurrencyEnum::Essence(_) => None,
                    c => Some(
                        data.market
                            .try_lookup_currency_in_divines_default_if_fail(c, &data.items)
                            .get_divine_value(),
                    ),
                };
                serde_json::json!({
                    "currency": a.currency_name,
                    "risk": a.risk,
                    "risk_description": a.risk_description,
                    "reason": a.reason,
                    "price_divines": price,
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({ "actions": actions }).to_string(),
        )]))
    }

    #[tool(
        description = "One-step outcome distribution of applying a single currency to an item: which mods can roll (or be removed) and with what chance. Supports the additive orbs, desecration, concrete essences and the orb of annulment. Answers 'if I exalt-slam this, what are the odds?' and 'if I desecrate, what can I get?'."
    )]
    async fn simulate_action(
        &self,
        Parameters(params): Parameters<SimulateActionParams>,
    ) -> Result<CallToolResult, McpError> {
        let (snapshot, data) = self
            .parse_item_value(params.item, params.league.as_deref())
            .await?;

        let currency =
            lookup::resolve_currency(&params.currency, &snapshot, &data.items).map_err(invalid)?;
        let mut sim =
            inspect::simulate_action(&snapshot, &currency, &data.items).map_err(invalid)?;

        let top_n = params.top_n.unwrap_or(25).min(100);
        let total = sim.outcomes.len();
        sim.outcomes.truncate(top_n);

        let out = serde_json::json!({
            "simulation": sim,
            "total_outcomes": total,
            "price_divines": data.market
                .try_lookup_currency_in_divines_default_if_fail(&currency, &data.items)
                .get_divine_value(),
        });
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "Parse an EPPSSA craft spec (E essence / P prefix / S suffix / A abyss-desecrated, tier digits, [name] bindings, ! fractured) against a base item. Returns the slot candidate pools, the concrete-target fan-out, and - when every slot is pinned - an exact target item for submit_calculation. Pin slots via bindings until the fan-out is 1."
    )]
    async fn parse_craft_spec(
        &self,
        Parameters(params): Parameters<ParseCraftSpecParams>,
    ) -> Result<CallToolResult, McpError> {
        let data = self
            .providers
            .get(params.league.as_deref())
            .await
            .map_err(internal)?;

        let mut bindings: THashMap<u8, AffixId> = THashMap::default();
        for (key, value) in params.bindings.unwrap_or_default() {
            let index: u8 = key
                .parse()
                .map_err(|_| invalid(format!("bindings key '{key}' is not a slot index")))?;
            bindings.insert(index, AffixId::from(value));
        }

        let template = craftspec::parse_craft_spec(
            &params.spec,
            BaseItemId::from(params.base_id),
            ItemLevel::from(params.item_level.unwrap_or(81)),
            &bindings,
            &data.items,
        )
        .map_err(invalid)?;

        let exact_target = template
            .exact_target
            .as_ref()
            .map(|t| serde_json::to_value(v1::ItemSnapshot::from(t)))
            .transpose()
            .map_err(internal)?;

        let slots: Vec<serde_json::Value> = template
            .slots
            .iter()
            .map(|s| {
                let candidates: Vec<serde_json::Value> = s
                    .candidates
                    .iter()
                    .take(25)
                    .map(|id| {
                        let description = data
                            .items
                            .lookup_affix_definition(id)
                            .map(|d| d.description_template.clone())
                            .unwrap_or_default();
                        serde_json::json!({
                            "affix_id": id.get_raw_value(),
                            "description": description,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "index": s.index,
                    "letter": s.letter,
                    "location": s.location,
                    "class": s.class,
                    "tier": s.tier,
                    "fractured": s.fractured,
                    "pinned": s.pinned.as_ref().map(|p| p.get_raw_value()),
                    "candidate_count": s.candidates.len(),
                    "candidates": candidates,
                })
            })
            .collect();

        let out = serde_json::json!({
            "base_id": template.base_id.get_raw_value(),
            "item_level": template.item_level.get_raw_value(),
            "rarity": template.rarity,
            "slots": slots,
            "estimated_concrete_targets": template.estimated_concrete_targets.to_string(),
            "exact_target": exact_target,
        });
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "Currently-good item archetypes per item class / character class / level bracket ('what is a good bow right now?'). Data source is a curated static catalog (data_freshness discloses it); affix names feed search_affixes, craft_spec feeds parse_craft_spec."
    )]
    fn get_meta_items(
        &self,
        Parameters(params): Parameters<GetMetaItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let catalog = meta::load_catalog().map_err(internal)?;
        let archetypes = meta::filter_archetypes(
            &catalog,
            params.item_class.as_deref(),
            params.char_class.as_deref(),
            params.level_bracket.as_deref(),
        );
        let out = serde_json::json!({
            "data_freshness": {
                "source": catalog.source,
                "updated": catalog.updated,
                "game_patch": catalog.game_patch,
                "disclaimer": catalog.disclaimer,
            },
            "archetypes": archetypes,
        });
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "Wait (long-poll) until a job changes state or the timeout elapses, then return its status. Avoids polling get_job_status in a loop; progress includes the live calculation phase."
    )]
    async fn await_job(
        &self,
        Parameters(params): Parameters<AwaitJobParams>,
    ) -> Result<CallToolResult, McpError> {
        let timeout = params.timeout_seconds.unwrap_or(30).clamp(1, 120);

        let status = self
            .jobs
            .status(&params.job_id)
            .await
            .map_err(internal)?
            .ok_or_else(|| invalid(format!("no job with id '{}'", params.job_id)))?;
        let state = v1::JobState::try_from(status.state).unwrap_or(v1::JobState::Unspecified);
        if is_terminal(state) {
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&status).map_err(internal)?,
            )]));
        }

        let mut pubsub = self.redis.get_async_pubsub().await.map_err(internal)?;
        pubsub
            .subscribe(events_channel(&params.job_id))
            .await
            .map_err(internal)?;
        let mut events = pubsub.on_message();

        let deadline = tokio::time::sleep(std::time::Duration::from_secs(timeout));
        tokio::pin!(deadline);

        let (status, timed_out) = loop {
            tokio::select! {
                maybe_msg = events.next() => {
                    let Some(msg) = maybe_msg else {
                        break (None, false);
                    };
                    let payload: String = msg.get_payload().map_err(internal)?;
                    if let Ok(update) = serde_json::from_str::<v1::JobStatus>(&payload) {
                        break (Some(update), false);
                    }
                }
                _ = &mut deadline => break (None, true),
            }
        };

        // re-read for the freshest queue position / progress
        let status = match status {
            Some(s) => s,
            None => self
                .jobs
                .status(&params.job_id)
                .await
                .map_err(internal)?
                .ok_or_else(|| invalid(format!("job '{}' disappeared", params.job_id)))?,
        };

        let mut out = serde_json::to_value(&status).map_err(internal)?;
        if let Some(obj) = out.as_object_mut() {
            obj.insert("timed_out".into(), serde_json::json!(timed_out));
        }
        Ok(CallToolResult::success(vec![Content::text(
            out.to_string(),
        )]))
    }

    #[tool(
        description = "Decode a Path of Building (PoE2) share code: character class, ascendancy, level and the equipped item texts. Use it to make answers build-aware ('what should I craft for my build?')."
    )]
    fn import_pob_build(
        &self,
        Parameters(params): Parameters<ImportPobBuildParams>,
    ) -> Result<CallToolResult, McpError> {
        let build = pob::parse_pob_build(&params.code).map_err(invalid)?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&build).map_err(internal)?,
        )]))
    }
}

#[tool_handler]
impl ServerHandler for CraftPathMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "pyoe2-craftpath: Path of Exile 2 crafting assistant. Heavy route \
                 calculations: submit_calculation enqueues a job (can run minutes); \
                 await_job or get_job_status to follow it; get_job_result when \
                 SUCCEEDED. Instant lookups: search_affixes / get_base_items resolve \
                 names to ids, get_currency_prices for rates, get_meta_items for \
                 currently-good archetypes, get_legal_actions for what can be applied \
                 (with danger warnings), simulate_action for one-step odds, \
                 parse_craft_spec for EPPSSA target specs, parse_item for CoE \
                 exports, import_pob_build for Path of Building share codes."
                    .to_string(),
            )
    }
}

pub async fn serve(config: Config, transport: &str) -> Result<()> {
    let jobs = JobsClient::connect(&config.redis_url, config.job_ttl_secs).await?;
    jobs.ensure_group().await?;

    match transport {
        "stdio" => {
            let service = CraftPathMcp::new(jobs, config)?
                .serve(stdio())
                .await
                .context("mcp stdio serve failed")?;
            service.waiting().await?;
            Ok(())
        }
        "http" => {
            let bind_addr = config.bind_addr.clone();
            let mcp = CraftPathMcp::new(jobs.clone(), config)?;

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
