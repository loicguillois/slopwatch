use std::path::Path;

use super::{Dependency, Ecosystem};

pub fn parse_requirements_txt(path: &Path) -> Result<Vec<Dependency>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut deps = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }

        // Split on version specifiers: ==, >=, <=, ~=, !=, >, <
        let (name, version) = if let Some(pos) = line.find(['=', '>', '<', '~', '!']) {
            let name = line[..pos].trim();
            let version = line[pos..].trim_start_matches(['=', '>', '<', '~', '!']);
            (name.to_string(), Some(version.to_string()))
        } else {
            // Handle extras like package[extra]
            let name = if let Some(pos) = line.find('[') {
                &line[..pos]
            } else {
                line
            };
            (name.trim().to_string(), None)
        };

        if !name.is_empty() {
            deps.push(Dependency {
                name,
                version,
                ecosystem: Ecosystem::PyPI,
            });
        }
    }

    Ok(deps)
}

pub fn parse_pyproject_toml(path: &Path) -> Result<Vec<Dependency>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc: toml::Value = content
        .parse()
        .map_err(|e: toml::de::Error| e.to_string())?;

    let mut deps = Vec::new();

    // [project.dependencies]
    if let Some(project_deps) = doc
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for dep in project_deps {
            if let Some(s) = dep.as_str() {
                if let Some(d) = parse_pep508(s) {
                    deps.push(d);
                }
            }
        }
    }

    // [project.optional-dependencies]
    if let Some(optional) = doc
        .get("project")
        .and_then(|p| p.get("optional-dependencies"))
        .and_then(|d| d.as_table())
    {
        for (_group, group_deps) in optional {
            if let Some(arr) = group_deps.as_array() {
                for dep in arr {
                    if let Some(s) = dep.as_str() {
                        if let Some(d) = parse_pep508(s) {
                            deps.push(d);
                        }
                    }
                }
            }
        }
    }

    Ok(deps)
}

fn parse_pep508(spec: &str) -> Option<Dependency> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    // Find where the name ends (at version specifier, extra bracket, or semicolon)
    let name_end = spec
        .find(['>', '<', '=', '~', '!', '[', ';'])
        .unwrap_or(spec.len());

    let name = spec[..name_end].trim().to_string();
    if name.is_empty() {
        return None;
    }

    let rest = spec[name_end..].trim();
    let version = if rest.is_empty() || rest.starts_with('[') || rest.starts_with(';') {
        None
    } else {
        let ver = rest.trim_start_matches(['>', '<', '=', '~', '!']);
        // Stop at semicolon (environment markers)
        let ver = if let Some(pos) = ver.find(';') {
            &ver[..pos]
        } else {
            ver
        };
        Some(ver.trim().to_string())
    };

    Some(Dependency {
        name,
        version,
        ecosystem: Ecosystem::PyPI,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_requirements_txt() {
        let dir = std::env::temp_dir().join("slopwatch_test_pypi");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("requirements.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "# A comment\nrequests==2.31.0\nfastapi>=0.104.0\nnumpy\n-r other.txt\n"
        )
        .unwrap();

        let deps = parse_requirements_txt(&path).unwrap();
        assert_eq!(deps.len(), 3);

        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"requests"));
        assert!(names.contains(&"fastapi"));
        assert!(names.contains(&"numpy"));

        assert_eq!(deps[2].version, None); // numpy has no version

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_pyproject_toml() {
        let dir = std::env::temp_dir().join("slopwatch_test_pyproject");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pyproject.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"[project]
name = "myapp"
dependencies = [
    "fastapi>=0.104.0",
    "uvicorn",
]

[project.optional-dependencies]
dev = ["pytest>=7.0"]
"#
        )
        .unwrap();

        let deps = parse_pyproject_toml(&path).unwrap();
        assert_eq!(deps.len(), 3);

        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"fastapi"));
        assert!(names.contains(&"uvicorn"));
        assert!(names.contains(&"pytest"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
