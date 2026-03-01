use colored::Colorize;

use crate::scorer::{ScanResult, Severity};

pub fn render(results: &[ScanResult]) {
    if results.is_empty() {
        println!("No dependencies found.");
        return;
    }

    println!(
        "\n{}",
        "slopwatch — slopsquatting detector".bold().underline()
    );
    println!(
        "{}\n",
        format!("Scanned {} dependencies", results.len()).dimmed()
    );

    // Sort: critical first, then high, medium, ok
    let mut sorted: Vec<&ScanResult> = results.iter().collect();
    sorted.sort_by(|a, b| severity_order(&a.severity).cmp(&severity_order(&b.severity)));

    for result in &sorted {
        let severity_str = format_severity(&result.severity);
        let score_str = if result.exists {
            format!("{:.0}/100", result.trust_score)
        } else {
            "N/A".to_string()
        };

        println!(
            "  {} {:<42} score: {:<8} [{}]",
            severity_str,
            result.name,
            score_str,
            result.ecosystem.dimmed()
        );

        for signal in &result.signals {
            println!("    {} {}", "|".dimmed(), signal.dimmed());
        }
    }

    // Summary
    let critical = results
        .iter()
        .filter(|r| r.severity == Severity::Critical)
        .count();
    let high = results
        .iter()
        .filter(|r| r.severity == Severity::High)
        .count();
    let medium = results
        .iter()
        .filter(|r| r.severity == Severity::Medium)
        .count();
    let not_found = results
        .iter()
        .filter(|r| r.severity == Severity::NotFound)
        .count();
    let ok = results
        .iter()
        .filter(|r| r.severity == Severity::Ok)
        .count();

    println!("\n{}", "Summary".bold());
    if critical > 0 {
        println!(
            "  {} {}",
            format!("{} CRITICAL", critical).red().bold(),
            "— likely slopsquatting".red()
        );
    }
    if high > 0 {
        println!(
            "  {} {}",
            format!("{} HIGH", high).yellow().bold(),
            "— suspicious metadata".yellow()
        );
    }
    if medium > 0 {
        println!(
            "  {} {}",
            format!("{} MEDIUM", medium).blue().bold(),
            "— worth investigating".blue()
        );
    }
    if not_found > 0 {
        println!(
            "  {} {}",
            format!("{} NOT FOUND", not_found).red().bold(),
            "— package does not exist on registry".red()
        );
    }
    if ok > 0 {
        println!(
            "  {} {}",
            format!("{} OK", ok).green().bold(),
            "— trusted".green()
        );
    }

    // Exit hint
    if critical > 0 || not_found > 0 {
        println!(
            "\n{}",
            "Action required: review flagged packages before deployment."
                .red()
                .bold()
        );
    }
}

fn format_severity(severity: &Severity) -> String {
    match severity {
        Severity::Critical => "CRITICAL ".red().bold().to_string(),
        Severity::High => "HIGH     ".yellow().bold().to_string(),
        Severity::Medium => "MEDIUM   ".blue().bold().to_string(),
        Severity::Ok => "OK       ".green().to_string(),
        Severity::NotFound => "NOT FOUND".red().bold().to_string(),
    }
}

fn severity_order(s: &Severity) -> u8 {
    match s {
        Severity::NotFound => 0,
        Severity::Critical => 1,
        Severity::High => 2,
        Severity::Medium => 3,
        Severity::Ok => 4,
    }
}
