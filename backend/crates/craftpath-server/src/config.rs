use anyhow::{Context, Result};

/// Environment-driven configuration shared by all server modes.
#[derive(Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub bind_addr: String,
    /// Health/readiness listener for worker pods (k8s probes).
    pub worker_health_addr: String,
    /// TTL applied to job hash/request/result keys once a job is terminal.
    pub job_ttl_secs: u64,
    /// Wall-clock timeout applied to jobs that do not set their own.
    pub job_timeout_secs: u64,
    pub default_league: String,
    pub cache_dir: String,
    pub coe_cache_ttl_secs: u64,
    pub economy_cache_ttl_secs: u64,
    /// Upper bound for per-job max_ram_in_bytes requested by clients.
    pub max_ram_limit_bytes: u64,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(key) {
        Ok(v) => v.parse::<T>().with_context(|| format!("invalid {key}")),
        Err(_) => Ok(default),
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            redis_url: env_or("REDIS_URL", "redis://127.0.0.1:6379"),
            bind_addr: env_or("BIND_ADDR", "0.0.0.0:8080"),
            worker_health_addr: env_or("WORKER_HEALTH_ADDR", "0.0.0.0:8081"),
            job_ttl_secs: env_parse("JOB_TTL_SECS", 24 * 60 * 60)?,
            job_timeout_secs: env_parse("JOB_TIMEOUT_SECS", 60 * 60)?,
            default_league: env_or("POE2_LEAGUE_DEFAULT", "Standard"),
            cache_dir: env_or("CACHE_DIR", "./cache"),
            coe_cache_ttl_secs: env_parse("COE_CACHE_TTL_SECS", 24 * 60 * 60)?,
            economy_cache_ttl_secs: env_parse("ECONOMY_CACHE_TTL_SECS", 60 * 60)?,
            max_ram_limit_bytes: env_parse("MAX_RAM_LIMIT_BYTES", 4_000_000_000)?,
        })
    }
}
