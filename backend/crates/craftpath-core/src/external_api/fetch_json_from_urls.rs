use std::{
    fs,
    io::Write,
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use tracing::instrument;

use crate::api::types::THashMap;

#[instrument(skip_all)]
pub fn retrieve_contents_from_urls_with_cache_unstable_order(
    cache_url_map: THashMap<String, String>,
    max_cache_duration_in_sec: u64,
) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let max_age = Duration::from_secs(max_cache_duration_in_sec);

    for (cache_path, url) in cache_url_map {
        let path = Path::new(&cache_path);

        let should_download = if !path.exists() {
            true
        } else {
            let metadata = fs::metadata(path)?;
            let modified = metadata.modified()?;
            let age = SystemTime::now().duration_since(modified)?;
            age > max_age
        };

        let data = if should_download {
            tracing::info!("Downloading fresh data for {}...", cache_path);
            let response = reqwest::blocking::get(&url)?.text()?;

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = fs::File::create(&path)?;
            file.write_all(response.as_bytes())?;
            response
        } else {
            tracing::info!("Loading cached contents from '{}'", path.display());
            fs::read_to_string(&path)?
        };

        results.push(data);
    }

    Ok(results)
}
