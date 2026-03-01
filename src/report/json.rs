use crate::scorer::ScanResult;

pub fn render(results: &[ScanResult]) -> String {
    serde_json::to_string_pretty(results).unwrap_or_else(|_| "[]".to_string())
}
