pub mod json;
pub mod sarif;
pub mod terminal;

use crate::scorer::ScanResult;

pub fn output(results: &[ScanResult], format: &str, output_path: Option<&str>) {
    let content = match format {
        "json" => json::render(results),
        "sarif" => sarif::render(results),
        _ => {
            terminal::render(results);
            return;
        }
    };

    if let Some(path) = output_path {
        if let Err(e) = std::fs::write(path, &content) {
            eprintln!("Error writing to {}: {}", path, e);
        } else {
            eprintln!("Report written to {}", path);
        }
    } else {
        println!("{}", content);
    }
}
