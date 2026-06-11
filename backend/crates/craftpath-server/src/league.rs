use anyhow::{Context, Result};

use craftpath_core::api::provider::item_info::ItemInfoProvider;
use craftpath_core::api::provider::market_prices::MarketPriceProvider;
use craftpath_core::api::types::THashMap;
use craftpath_core::external_api::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
use craftpath_core::external_api::fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order;
use craftpath_core::external_api::pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider;

use crate::config::Config;

/// Fetch (or serve from the file cache) the CraftOfExile item data and the
/// poe.ninja economy data for a league, then parse them into providers.
///
/// Blocking (uses the blocking reqwest fetcher from craftpath-core) — call
/// from `spawn_blocking` in async contexts. Clients never upload this data;
/// it is resolved server-side per league.
pub fn load_league_data(
    config: &Config,
    league: &str,
) -> Result<(ItemInfoProvider, MarketPriceProvider)> {
    let league_dir = format!(
        "{}/{}",
        config.cache_dir,
        league.replace(['/', '\\', ' '], "_")
    );
    std::fs::create_dir_all(&league_dir)
        .with_context(|| format!("could not create cache dir '{league_dir}'"))?;

    // CoE data is league independent; cache it once at the top level.
    let item_provider_hm = THashMap::from_iter(vec![(
        format!("{}/coe2.json", config.cache_dir),
        "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
    )]);

    let economy_provider_hm = THashMap::from_iter(
        ["Abyss", "Currency", "Essences", "Ritual"].map(|kind| {
            (
                format!("{}/pn_{}.json", league_dir, kind.to_lowercase()),
                format!(
                    "https://poe.ninja/poe2/api/economy/exchange/current/overview?league={league}&type={kind}"
                ),
            )
        }),
    );

    let item_cached_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
        item_provider_hm,
        config.coe_cache_ttl_secs,
    )?;
    let economy_cached_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
        economy_provider_hm,
        config.economy_cache_ttl_secs,
    )?;

    let item_provider = CraftOfExileItemInfoProvider::parse_from_json(
        item_cached_jsons
            .first()
            .context("missing CoE data download")?,
    )?;
    let market_info = PoeNinjaMarketPriceProvider::parse_from_json_list(&economy_cached_jsons)?;

    Ok((item_provider, market_info))
}
