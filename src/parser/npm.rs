use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::{Dependency, Ecosystem};

/// Represents an npm dependency specification.
/// npm supports multiple formats for specifying dependencies.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum DependencySpec {
    /// Simple version string: "^4.18.0", "1.2.3", ">=1.0.0"
    Version(String),
    /// Complex specification (git, local path, etc.)
    /// e.g., { "version": "1.0", "path": "../my-lib" }
    Complex(serde_json::Map<String, serde_json::Value>),
}

impl DependencySpec {
    fn version(&self) -> Option<String> {
        match self {
            DependencySpec::Version(v) => Some(v.clone()),
            DependencySpec::Complex(map) => map
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: HashMap<String, DependencySpec>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: HashMap<String, DependencySpec>,
}

pub fn parse_package_json(path: &Path) -> Result<Vec<Dependency>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let pkg: PackageJson = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let deps = pkg
        .dependencies
        .iter()
        .chain(pkg.dev_dependencies.iter())
        .map(|(name, spec)| Dependency {
            name: name.clone(),
            version: spec.version(),
            ecosystem: Ecosystem::Npm,
        })
        .collect();

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_package_json() {
        let dir = std::env::temp_dir().join("slopwatch_test_npm");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("package.json");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"{{
                "dependencies": {{
                    "express": "^4.18.0",
                    "react": "18.2.0"
                }},
                "devDependencies": {{
                    "jest": "^29.0.0"
                }}
            }}"#
        )
        .unwrap();

        let deps = parse_package_json(&path).unwrap();
        assert_eq!(deps.len(), 3);

        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"express"));
        assert!(names.contains(&"react"));
        assert!(names.contains(&"jest"));

        for dep in &deps {
            assert_eq!(dep.ecosystem, Ecosystem::Npm);
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_empty_package_json() {
        let dir = std::env::temp_dir().join("slopwatch_test_npm_empty");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("package.json");
        std::fs::write(&path, "{}").unwrap();

        let deps = parse_package_json(&path).unwrap();
        assert_eq!(deps.len(), 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_complex_dependencies() {
        let dir = std::env::temp_dir().join("slopwatch_test_npm_complex");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("package.json");
        std::fs::write(
            &path,
            r#"{
                "dependencies": {
                    "normal": "^1.0.0",
                    "local-lib": { "version": "2.0.0", "path": "../local-lib" },
                    "git-dep": { "git": "https://github.com/user/repo" }
                }
            }"#,
        )
        .unwrap();

        let deps = parse_package_json(&path).unwrap();
        assert_eq!(deps.len(), 3);

        let normal = deps.iter().find(|d| d.name == "normal").unwrap();
        assert_eq!(normal.version, Some("^1.0.0".to_string()));

        let local = deps.iter().find(|d| d.name == "local-lib").unwrap();
        assert_eq!(local.version, Some("2.0.0".to_string()));

        let git = deps.iter().find(|d| d.name == "git-dep").unwrap();
        assert_eq!(git.version, None); // No version field in git dep

        std::fs::remove_dir_all(&dir).ok();
    }
}
