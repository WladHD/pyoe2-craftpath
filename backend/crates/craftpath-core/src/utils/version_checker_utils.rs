use std::{thread::sleep, time::Duration};

use anyhow::Result;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, USER_AGENT},
};
use semver::Version;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    prerelease: bool,
}

pub fn check_new_version(repo: &str) -> Result<bool> {
    let current_version = env!("CARGO_PKG_VERSION");
    let pkg_name = env!("CARGO_PKG_NAME");
    let current = Version::parse(current_version)?;

    // GitHub endpoint for releases
    let url = format!("https://api.github.com/repos/{repo}/releases");

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        format!("{}/{}", pkg_name, current_version).parse()?,
    );

    let client = Client::builder().default_headers(headers).build()?;
    let resp = client.get(&url).send()?.error_for_status()?;

    let releases: Vec<GithubRelease> = resp.json()?;

    let mut parsed_releases: Vec<(Version, bool)> = releases
        .into_iter()
        .filter_map(|r| {
            Version::parse(r.tag_name.trim_start_matches('v'))
                .ok()
                .map(|v| (v, r.prerelease))
        })
        .collect();

    parsed_releases.sort_by(|a, b| b.0.cmp(&a.0));

    let latest_stable = parsed_releases.iter().find(|(_, pre)| !*pre);
    let latest_pre = parsed_releases.iter().find(|(_, pre)| *pre);

    let mut stable_update_available = false;

    if let Some((v, _)) = latest_stable {
        if v > &current {
            stable_update_available = true;
            tracing::warn!("🔔 New stable release available: {v} (current: {current})");
        }
    }

    if let Some((v, _)) = latest_pre {
        if v > &current {
            // do not sleep on dev release
            // updates_available = true;
            tracing::info!("⚙️ New DEV/prerelease build available: {v} (current: {current})");
        }
    }

    let normal_repo = format!("https://github.com/{}", repo);

    match stable_update_available {
        true => {
            tracing::info!("Check out {normal_repo}/releases to download the new version.",);
            sleep(Duration::from_millis(500_u64));
        }
        false => tracing::info!(
            "You are up to date running {pkg_name} with version {current}!\nYou can always check out {normal_repo} if you encounter issues or have ideas.",
        ),
    }

    Ok(stable_update_available)
}
