//! In-process, per-league cache of the game-data providers so the sync MCP
//! lookup tools (affix search, prices, legal actions, simulations) can
//! answer without a worker round-trip. Loading delegates to
//! [`crate::league::load_league_data`], which already file-caches the CoE
//! and poe.ninja downloads; this layer only avoids re-parsing on every tool
//! call and refreshes after the economy cache TTL.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::RwLock;

use craftpath_core::prelude::{ItemInfoProvider, MarketPriceProvider};

use crate::config::Config;
use crate::league::load_league_data;

pub struct LeagueData {
    pub items: ItemInfoProvider,
    pub market: MarketPriceProvider,
    pub loaded_at: Instant,
}

/// Cheap-to-clone handle on the shared cache.
#[derive(Clone)]
pub struct ProviderCache {
    config: Config,
    inner: Arc<RwLock<HashMap<String, Arc<LeagueData>>>>,
}

impl ProviderCache {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fetch (or reuse) the providers for a league; `None` uses the
    /// configured default league. Concurrent first loads of the same league
    /// may duplicate work once; the result converges in the cache.
    pub async fn get(&self, league: Option<&str>) -> Result<Arc<LeagueData>> {
        let league = league
            .filter(|l| !l.trim().is_empty())
            .unwrap_or(&self.config.default_league)
            .to_string();
        let ttl = Duration::from_secs(self.config.economy_cache_ttl_secs);

        if let Some(data) = self.inner.read().await.get(&league) {
            if data.loaded_at.elapsed() < ttl {
                return Ok(data.clone());
            }
        }

        let config = self.config.clone();
        let league_arg = league.clone();
        let (items, market) =
            tokio::task::spawn_blocking(move || load_league_data(&config, &league_arg)).await??;

        let data = Arc::new(LeagueData {
            items,
            market,
            loaded_at: Instant::now(),
        });
        self.inner.write().await.insert(league, data.clone());
        Ok(data)
    }
}
