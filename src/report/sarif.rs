use crate::scorer::{ScanResult, Severity};

pub fn render(results: &[ScanResult]) -> String {
    let rules: Vec<serde_json::Value> = vec![
        rule(
            "SLOP001",
            "critical-trust-score",
            "Package has critically low trust score, likely slopsquatting",
        ),
        rule(
            "SLOP002",
            "high-risk-package",
            "Package has high risk indicators",
        ),
        rule(
            "SLOP003",
            "medium-risk-package",
            "Package has moderate risk indicators",
        ),
        rule(
            "SLOP004",
            "package-not-found",
            "Package does not exist on the registry",
        ),
    ];

    let sarif_results: Vec<serde_json::Value> = results
        .iter()
        .filter(|r| r.severity != Severity::Ok)
        .map(|r| {
            let (rule_id, level) = match r.severity {
                Severity::Critical => ("SLOP001", "error"),
                Severity::High => ("SLOP002", "warning"),
                Severity::Medium => ("SLOP003", "note"),
                Severity::NotFound => ("SLOP004", "error"),
                Severity::Ok => unreachable!(),
            };

            let message = if r.signals.is_empty() {
                format!(
                    "{} ({}): trust score {:.0}/100",
                    r.name, r.ecosystem, r.trust_score
                )
            } else {
                format!(
                    "{} ({}): trust score {:.0}/100 — {}",
                    r.name,
                    r.ecosystem,
                    r.trust_score,
                    r.signals.join("; ")
                )
            };

            // Determine the dependency file based on ecosystem
            let dep_file = match r.ecosystem.as_str() {
                "npm" => "package.json",
                "pypi" => "requirements.txt",
                _ => "dependencies",
            };

            serde_json::json!({
                "ruleId": rule_id,
                "level": level,
                "message": { "text": message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": dep_file,
                            "uriBaseId": "%SRCROOT%"
                        },
                        "region": {
                            "startLine": 1
                        }
                    }
                }],
                "properties": {
                    "package": r.name,
                    "ecosystem": r.ecosystem,
                    "trustScore": r.trust_score,
                    "severity": format!("{}", r.severity),
                }
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "slopwatch",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/slopwatch/slopwatch",
                    "rules": rules,
                }
            },
            "results": sarif_results,
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}

fn rule(id: &str, name: &str, description: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "shortDescription": { "text": description },
    })
}
