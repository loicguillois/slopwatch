pub mod npm;
pub mod pypi;

use std::path::Path;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    #[allow(dead_code)]
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ecosystem {
    Npm,
    PyPI,
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Npm => write!(f, "npm"),
            Ecosystem::PyPI => write!(f, "pypi"),
        }
    }
}

/// Auto-detect dependency files in a directory and parse them.
pub fn detect_and_parse(dir: &Path) -> Vec<Dependency> {
    let mut deps = Vec::new();

    // npm
    let package_json = dir.join("package.json");
    if package_json.exists() {
        match npm::parse_package_json(&package_json) {
            Ok(d) => deps.extend(d),
            Err(e) => eprintln!("Warning: failed to parse {}: {}", package_json.display(), e),
        }
    }

    // PyPI
    let requirements = dir.join("requirements.txt");
    if requirements.exists() {
        match pypi::parse_requirements_txt(&requirements) {
            Ok(d) => deps.extend(d),
            Err(e) => eprintln!("Warning: failed to parse {}: {}", requirements.display(), e),
        }
    }

    let pyproject = dir.join("pyproject.toml");
    if pyproject.exists() {
        match pypi::parse_pyproject_toml(&pyproject) {
            Ok(d) => deps.extend(d),
            Err(e) => eprintln!("Warning: failed to parse {}: {}", pyproject.display(), e),
        }
    }

    deps
}

/// Parse a specific file based on its name.
pub fn parse_file(path: &Path) -> Result<Vec<Dependency>, String> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file path".to_string())?;

    match filename {
        "package.json" => npm::parse_package_json(path),
        "requirements.txt" => pypi::parse_requirements_txt(path),
        "pyproject.toml" => pypi::parse_pyproject_toml(path),
        _ => Err(format!("Unsupported file: {filename}")),
    }
}
