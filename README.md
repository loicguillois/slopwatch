# slopwatch

[![CI](https://github.com/loicguillois/slopwatch/actions/workflows/ci.yml/badge.svg)](https://github.com/loicguillois/slopwatch/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Detect **slopsquatting** attacks — AI-hallucinated packages registered by attackers in your dependencies.

## The problem

Large Language Models regularly hallucinate package names that don't exist. Attackers register these names on npm and PyPI, turning every `pip install` or `npm install` from AI-generated code into a potential supply-chain attack.

A [Trend Micro study](https://www.trendmicro.com/vinfo/us/security/news/cybercrime-and-digital-threats/slopsquatting-when-ai-agents-hallucinate-malicious-packages) found that **21 out of 126** AI-hallucinated package names had been registered on PyPI — some containing malicious code.

**slopwatch** scans your dependency files and scores each package with a trust model trained on real slopsquatting data. No dictionary of known-bad names: it uses statistical metadata analysis to detect suspicious packages regardless of whether they've been seen before.

## How it works

Each package is scored from 0 to 100 using weighted metadata signals:

| Signal | Weight | Rationale |
|---|---|---|
| Weekly downloads (log) | 8 | Hallucinated packages have near-zero downloads |
| Package age (log) | 8 | Most are registered days before detection |
| Version count (log) | 6 | Typically only 1 version published |
| Description length (log) | 4 | Often empty or minimal |
| Source repository linked | 12 | Legitimate projects link to GitHub/GitLab |
| License declared | 4 | Omitted in most squatting packages |
| Author information | 3 | Often missing |
| Author email | 3 | Often missing |
| Classifiers | 0.5/each | Rich metadata = legitimate package |
| Dependencies declared | 0.5/each | Squatting packages rarely declare deps |

Severity thresholds:
- **CRITICAL** (score < 22): Very likely slopsquatting or malicious
- **HIGH** (score < 40): Suspicious, investigate manually
- **MEDIUM** (score < 60): Low metadata quality, worth checking
- **OK** (score >= 60): Appears legitimate

This model was validated at **100% F1** on a dataset of 81 packages (21 confirmed slopsquatting + 60 legitimate including hard negatives with low download counts).

## Installation

### From source

```bash
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/loicguillois/slopwatch/releases):

```bash
# Linux x86_64
curl -L https://github.com/loicguillois/slopwatch/releases/latest/download/slopwatch-linux-x86_64 -o slopwatch
chmod +x slopwatch
```

## Usage

### Scan a project directory

```bash
slopwatch scan .
```

slopwatch auto-detects `package.json`, `requirements.txt`, and `pyproject.toml`.

### Scan a specific file

```bash
slopwatch scan --file requirements.txt
```

### Output formats

```bash
# Colored terminal output (default)
slopwatch scan .

# JSON for programmatic use
slopwatch scan . --format json

# SARIF for GitHub/GitLab Security tab
slopwatch scan . --format sarif --output report.sarif
```

### Example output

```
slopwatch — slopsquatting detector

  CRITICAL  wavesocket          (PyPI)   Score: 9.2/100
            ⚠ Very low downloads: 17/week
            ⚠ Very recent package: 8 days old
            ⚠ Only 1 version published
            ⚠ No description
            ⚠ No source repository linked
            ⚠ No license
            ⚠ No author information

  OK        requests            (PyPI)   Score: 82.5/100

─────────────────────────────────────────
Summary: 2 packages scanned
  CRITICAL: 1  |  OK: 1
```

## GitHub Action

Add slopwatch to your CI pipeline:

```yaml
name: Security
on: [push, pull_request]

jobs:
  slopwatch:
    runs-on: ubuntu-latest
    permissions:
      security-events: write
    steps:
      - uses: actions/checkout@v4

      - uses: loicguillois/slopwatch@v1
        with:
          path: "."
          format: "sarif"
          threshold: "22"
          fail-on-critical: "true"
```

### Action inputs

| Input | Default | Description |
|---|---|---|
| `path` | `.` | Project directory to scan |
| `format` | `sarif` | Output format: `terminal`, `json`, `sarif` |
| `threshold` | `22` | Trust score threshold for CRITICAL severity |
| `fail-on-critical` | `true` | Fail the workflow if CRITICAL packages are found |
| `version` | `latest` | slopwatch version to use |

### Action outputs

| Output | Description |
|---|---|
| `results` | JSON scan results |
| `critical-count` | Number of CRITICAL packages found |

When using `sarif` format, results are automatically uploaded to the **Security** tab of your GitHub repository.

## Supported ecosystems

| Ecosystem | Files detected |
|---|---|
| npm | `package.json` |
| PyPI | `requirements.txt`, `pyproject.toml` |

## Contributing

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Lint
cargo clippy -- -D warnings
```

## Author

Loïc Guillois — [GitHub](https://github.com/loicguillois)

## License

MIT — see [LICENSE](LICENSE)
