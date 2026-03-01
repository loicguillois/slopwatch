use std::collections::HashMap;
use std::time::SystemTime;

use super::PackageMetadata;
use crate::parser::Ecosystem;

#[derive(serde::Deserialize)]
struct PyPIResponse {
    info: PyPIInfo,
    #[serde(default)]
    releases: HashMap<String, Vec<PyPIRelease>>,
}

#[derive(serde::Deserialize)]
struct PyPIInfo {
    #[serde(default)]
    name: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    author_email: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    home_page: Option<String>,
    #[serde(default)]
    project_urls: Option<HashMap<String, String>>,
    #[serde(default)]
    classifiers: Vec<String>,
    #[serde(default)]
    requires_dist: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct PyPIRelease {
    #[serde(default)]
    upload_time_iso_8601: Option<String>,
    #[serde(default)]
    upload_time: Option<String>,
}

pub fn fetch(client: &reqwest::blocking::Client, name: &str) -> PackageMetadata {
    let url = format!("https://pypi.org/pypi/{}/json", name);
    let resp = match client.get(&url).send() {
        Ok(r) if r.status().is_success() => r,
        _ => return PackageMetadata::not_found(name, Ecosystem::PyPI),
    };

    let data: PyPIResponse = match resp.json() {
        Ok(d) => d,
        Err(_) => return PackageMetadata::not_found(name, Ecosystem::PyPI),
    };

    let info = &data.info;

    // Count versions with files
    let versions_count = data
        .releases
        .values()
        .filter(|files| !files.is_empty())
        .count() as u64;

    // Find earliest upload date
    let mut earliest: Option<String> = None;
    for files in data.releases.values() {
        for file in files {
            let upload = file
                .upload_time_iso_8601
                .as_deref()
                .or(file.upload_time.as_deref());
            if let Some(u) = upload {
                if earliest.as_deref().is_none_or(|e| u < e) {
                    earliest = Some(u.to_string());
                }
            }
        }
    }

    let age_days = earliest.as_deref().and_then(chrono_parse_age).unwrap_or(0);

    let description = info.description.as_deref().unwrap_or("");
    let summary = info.summary.as_deref().unwrap_or("");
    let description_length = description.len() + summary.len();

    let has_author = info.author.as_ref().is_some_and(|a| !a.trim().is_empty());
    let has_author_email = info
        .author_email
        .as_ref()
        .is_some_and(|e| !e.trim().is_empty());
    let has_license = info.license.as_ref().is_some_and(|l| !l.trim().is_empty());

    // Check for source repo in project_urls
    let has_source_repo = info
        .project_urls
        .as_ref()
        .map(|urls| {
            urls.keys().any(|k| {
                let k = k.to_lowercase();
                k.contains("source")
                    || k.contains("repository")
                    || k.contains("github")
                    || k.contains("code")
                    || k.contains("homepage")
            })
        })
        .unwrap_or(false);

    let deps_count = info
        .requires_dist
        .as_ref()
        .map(|d| d.len() as u64)
        .unwrap_or(0);

    PackageMetadata {
        name: info.name.clone(),
        ecosystem: Ecosystem::PyPI,
        exists: true,
        downloads_week: 0, // PyPI doesn't expose downloads in the main API
        age_days,
        versions_count,
        description_length,
        has_source_repo,
        has_license,
        has_author,
        has_author_email,
        classifiers_count: info.classifiers.len() as u64,
        deps_count,
    }
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
    let pkg_days_since_epoch = ((year - 1970) * 365 + month * 30 + day) as u64;

    now_days_since_epoch.checked_sub(pkg_days_since_epoch)
}
