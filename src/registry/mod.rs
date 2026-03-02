pub mod npm;
pub mod pypi;

use std::collections::HashMap;

use crate::parser::{Dependency, Ecosystem};

/// Metadata fetched from a package registry.
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub ecosystem: Ecosystem,
    pub exists: bool,
    pub downloads_week: u64,
    pub age_days: u64,
    pub versions_count: u64,
    pub description_length: usize,
    pub has_source_repo: bool,
    pub has_license: bool,
    pub has_author: bool,
    pub has_author_email: bool,
    pub classifiers_count: u64,
    pub deps_count: u64,
}

impl PackageMetadata {
    /// Returns metadata for a package that doesn't exist on the registry.
    pub fn not_found(name: &str, ecosystem: Ecosystem) -> Self {
        Self {
            name: name.to_string(),
            ecosystem,
            exists: false,
            downloads_week: 0,
            age_days: 0,
            versions_count: 0,
            description_length: 0,
            has_source_repo: false,
            has_license: false,
            has_author: false,
            has_author_email: false,
            classifiers_count: 0,
            deps_count: 0,
        }
    }
}

/// Prefetch download counts for npm packages using batch API.
pub fn prefetch_npm_downloads(
    client: &reqwest::blocking::Client,
    deps: &[Dependency],
) -> HashMap<String, u64> {
    let npm_names: Vec<&str> = deps
        .iter()
        .filter(|d| d.ecosystem == Ecosystem::Npm)
        .map(|d| d.name.as_str())
        .collect();

    if npm_names.is_empty() {
        return HashMap::new();
    }

    npm::batch_fetch_downloads(client, &npm_names)
}

/// Fetch metadata for a package from the appropriate registry.
pub fn fetch_metadata(
    client: &reqwest::blocking::Client,
    name: &str,
    ecosystem: Ecosystem,
    npm_downloads: Option<&HashMap<String, u64>>,
) -> PackageMetadata {
    match ecosystem {
        Ecosystem::Npm => npm::fetch(client, name, npm_downloads),
        Ecosystem::PyPI => pypi::fetch(client, name),
    }
}
