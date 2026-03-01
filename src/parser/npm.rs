use std::collections::HashMap;
use std::path::Path;

use super::{Dependency, Ecosystem};

#[derive(serde::Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: HashMap<String, serde_json::Value>,
}

pub fn parse_package_json(path: &Path) -> Result<Vec<Dependency>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let pkg: PackageJson = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut deps = Vec::new();

    for (name, version) in pkg.dependencies.iter().chain(pkg.dev_dependencies.iter()) {
        let ver = match version {
            serde_json::Value::String(s) => Some(s.clone()),
            _ => None,
        };
        deps.push(Dependency {
            name: name.clone(),
            version: ver,
            ecosystem: Ecosystem::Npm,
        });
    }

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
}
