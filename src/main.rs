mod parser;
mod registry;
mod report;
mod scorer;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "slopwatch",
    about = "Detect slopsquatting attacks — AI-hallucinated packages registered by attackers",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Scan a project directory or dependency file
    Scan {
        /// Path to a project directory or dependency file
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Specific dependency file to scan
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output format: terminal, json, sarif
        #[arg(long, default_value = "terminal")]
        format: String,

        /// Output file path (prints to stdout if not set)
        #[arg(short, long)]
        output: Option<String>,

        /// Custom trust score threshold for CRITICAL (default: 22)
        #[arg(long, default_value = "22")]
        threshold: f64,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan {
            path,
            file,
            format,
            output,
            threshold: _,
        } => {
            // Parse dependencies
            let deps = if let Some(file_path) = &file {
                match parser::parse_file(file_path) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let dir = if path.is_dir() {
                    &path
                } else {
                    path.parent().unwrap_or(&path)
                };
                parser::detect_and_parse(dir)
            };

            if deps.is_empty() {
                eprintln!("No dependencies found. Provide a directory with package.json, requirements.txt, or pyproject.toml.");
                std::process::exit(1);
            }

            eprintln!(
                "Found {} dependencies. Checking registries...\n",
                deps.len()
            );

            // Build HTTP client
            let client = reqwest::blocking::Client::builder()
                .user_agent("slopwatch/0.1")
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client");

            // Fetch metadata and score each dependency
            let mut results = Vec::new();
            for (i, dep) in deps.iter().enumerate() {
                eprint!(
                    "\r\x1B[K  [{}/{}] Checking {}...",
                    i + 1,
                    deps.len(),
                    dep.name
                );

                let meta = registry::fetch_metadata(&client, &dep.name, dep.ecosystem);
                let result = scorer::score(&meta);
                results.push(result);

                // Rate limiting
                thread::sleep(Duration::from_millis(200));
            }
            eprintln!("\r{}\r", " ".repeat(80)); // Clear progress line

            // Output results
            report::output(&results, &format, output.as_deref());

            // Exit code: 1 if any critical/not-found
            let has_critical = results.iter().any(|r| {
                matches!(
                    r.severity,
                    scorer::Severity::Critical | scorer::Severity::NotFound
                )
            });
            if has_critical {
                std::process::exit(1);
            }
        }
    }
}
