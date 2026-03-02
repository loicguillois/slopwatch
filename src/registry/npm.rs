use std::collections::HashMap;
use std::time::SystemTime;

use super::PackageMetadata;
use crate::parser::Ecosystem;

/// Batch fetch download counts for multiple npm packages.
/// Returns a map of package name -> weekly downloads.
///
/// Note: npm's bulk API doesn't support scoped packages (@org/name),
/// so those are fetched individually.
pub fn batch_fetch_downloads(
    client: &reqwest::blocking::Client,
    names: &[&str],
) -> HashMap<String, u64> {
    let mut results = HashMap::new();

    // Separate scoped packages (not supported in bulk API) from regular packages
    let (scoped, regular): (Vec<_>, Vec<_>) = names.iter().partition(|n| n.starts_with('@'));

    // Batch fetch regular packages (npm API allows max ~128 per request)
    for chunk in regular.chunks(100) {
        let packages = chunk.join(",");

        let url = format!(
            "https://api.npmjs.org/downloads/point/last-week/{}",
            packages
        );

        if let Ok(resp) = client.get(&url).send() {
            if let Ok(data) = resp.json::<HashMap<String, serde_json::Value>>() {
                for (name, value) in data {
                    if let Some(downloads) = value.get("downloads").and_then(|d| d.as_u64()) {
                        results.insert(name, downloads);
                    }
                }
            }
        }
    }

    // Fetch scoped packages individually
    for name in scoped {
        let downloads = fetch_downloads(client, name);
        results.insert(name.to_string(), downloads);
    }

    results
}

#[derive(serde::Deserialize)]
struct NpmPackage {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    time: HashMap<String, String>,
    #[serde(default, rename = "dist-tags")]
    dist_tags: HashMap<String, String>,
    #[serde(default)]
    versions: HashMap<String, serde_json::Value>,
    #[serde(default)]
    maintainers: Vec<serde_json::Value>,
    #[serde(default)]
    repository: Option<serde_json::Value>,
    #[serde(default)]
    license: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct NpmDownloads {
    #[serde(default)]
    downloads: u64,
}

pub fn fetch(
    client: &reqwest::blocking::Client,
    name: &str,
    prefetched_downloads: Option<&HashMap<String, u64>>,
) -> PackageMetadata {
    let url = format!("https://registry.npmjs.org/{}", name);
    let resp = match client.get(&url).send() {
        Ok(r) if r.status().is_success() => r,
        _ => return PackageMetadata::not_found(name, Ecosystem::Npm),
    };

    let pkg: NpmPackage = match resp.json() {
        Ok(p) => p,
        Err(_) => return PackageMetadata::not_found(name, Ecosystem::Npm),
    };

    let description = pkg.description.unwrap_or_default();
    let versions_count = pkg.versions.len() as u64;

    // Age from created time
    let age_days = pkg
        .time
        .get("created")
        .and_then(|c| chrono_parse_age(c))
        .unwrap_or(0);

    // Check for repository
    let has_source_repo = pkg.repository.is_some()
        && !matches!(&pkg.repository, Some(serde_json::Value::String(s)) if s.is_empty());

    // License
    let has_license = pkg.license.is_some()
        && !matches!(&pkg.license, Some(serde_json::Value::String(s)) if s.is_empty());

    let latest_ver = pkg.dist_tags.get("latest").cloned().unwrap_or_default();

    // Dependencies count from latest version
    let deps_count = pkg
        .versions
        .get(&latest_ver)
        .and_then(|v| v.get("dependencies"))
        .and_then(|d| d.as_object())
        .map(|d| d.len() as u64)
        .unwrap_or(0);

    // Use prefetched downloads if available, otherwise fetch individually
    let downloads_week = prefetched_downloads
        .and_then(|m| m.get(name).copied())
        .unwrap_or_else(|| fetch_downloads(client, name));

    PackageMetadata {
        name: pkg.name,
        ecosystem: Ecosystem::Npm,
        exists: true,
        downloads_week,
        age_days,
        versions_count,
        description_length: description.len(),
        has_source_repo,
        has_license,
        has_author: !pkg.maintainers.is_empty(),
        has_author_email: false, // npm doesn't expose this reliably
        classifiers_count: 0,    // npm doesn't have classifiers
        deps_count,
    }
}

fn fetch_downloads(client: &reqwest::blocking::Client, name: &str) -> u64 {
    let url = format!("https://api.npmjs.org/downloads/point/last-week/{}", name);
    client
        .get(&url)
        .send()
        .ok()
        .and_then(|r| r.json::<NpmDownloads>().ok())
        .map(|d| d.downloads)
        .unwrap_or(0)
}

fn chrono_parse_age(date_str: &str) -> Option<u64> {
    let date_str = date_str.trim();
    if date_str.len() < 10 {
        return None;
    }
    let year: i64 = date_str[..4].parse().ok()?;
    let month: i64 = date_str[5..7].parse().ok()?;
    let day: i64 = date_str[8..10].parse().ok()?;

    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let now_days_since_epoch = now_secs / 86400;
    // Convert to same approximate scale: epoch is 1970-01-01
    let pkg_days_since_epoch = ((year - 1970) * 365 + month * 30 + day) as u64;

    now_days_since_epoch.checked_sub(pkg_days_since_epoch)
}
