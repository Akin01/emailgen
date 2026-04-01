# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-04-01

### Added
- High-performance email generation engine using Markov chains and Bloom filters
- PowerShell installation script for Windows (`install.ps1`)
- Bash installation script for Linux/macOS (`install.sh`)
- NPM package: `@akin01/mailgen`
- PyPI package: `mailgen-rs`
- Cargo package: `mailgen`
- Multi-threaded generation using Rayon
- Asynchronous file I/O using Tokio
- TUI progress bar for command-line feedback
- Bloom filter for O(1) uniqueness checking
- Multiple email patterns (first.last, firstlast, flast#, first_last, first#)
- Configurable name source ratios (wordlist/cache/Markov)
- Fast mode for bulk generation (250K+ emails/sec)
- Custom wordlist and domain support

### Performance
- Fast Mode: 250K+ emails/sec
- 1M emails in ~4 seconds (default config)
- Large wordlist startup: <1 second
- Memory efficient: ~1.14 MB for 1M unique emails
- 100% unique emails guaranteed via Bloom filter

### Installation
- Windows: `powershell -ExecutionPolicy Bypass -Command "iwr -useb https://raw.githubusercontent.com/akin01/emailgen/main/install.ps1 | iex"`
- Linux/macOS: `curl -fsSL https://raw.githubusercontent.com/akin01/emailgen/main/install.sh | sudo bash`
- NPM: `npm install -g @akin01/mailgen`
- PyPI: `pip install mailgen-rs`
- Cargo: `cargo install mailgen`

### CI/CD
- GitHub Actions for automated testing
- Separate workflows for GitHub Releases, NPM, and PyPI
- Node.js 22 for NPM publishing
- Rust formatting checks in CI
