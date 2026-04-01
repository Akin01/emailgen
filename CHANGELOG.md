# Changelog

All notable changes to this project will be documented in this file.

## [0.1.5] - 2026-04-01

### Fixed
- **Binary distribution consistency**: Aligned all supported install paths on the `mailgen` executable name
  - Shell installer now downloads and installs `mailgen`
  - PowerShell installer now downloads and verifies `mailgen.exe`
  - Helper scripts and CLI usage docs now reference `mailgen`

### Changed
- Version bump: 0.1.4 → 0.1.5 across all registries
  - Cargo: mailgen 0.1.5
  - NPM: @akin01/mailgen 0.1.5
  - PyPI: mailgen-rs 0.1.5

## [0.1.4] - 2026-04-01

### Fixed
- **NPM launcher**: Fixed the global `mailgen` shim to execute the installed `mailgen` binary
  - The wrapper now resolves `mailgen` and keeps a fallback to the legacy `emailgen` name
  - Global installs via `vp i -g @akin01/mailgen` now run correctly after installation

### Changed
- Version bump: 0.1.3 → 0.1.4 across all registries
  - Cargo: mailgen 0.1.4
  - NPM: @akin01/mailgen 0.1.4
  - PyPI: mailgen-rs 0.1.4

## [0.1.3] - 2026-04-01

### Fixed
- **Release workflow**: Removed `--locked` flag issue by updating Cargo.lock with all target dependencies
- **NPM install script**: Fixed to download `mailgen` binaries instead of `emailgen`
  - Install script now correctly downloads mailgen binaries for all platforms

### Changed
- Version bump: 0.1.2 → 0.1.3 across all registries
  - Cargo: mailgen 0.1.3
  - NPM: @akin01/mailgen 0.1.3
  - PyPI: mailgen-rs 0.1.3

### CI/CD
- Cargo.lock updated with all cross-compilation target dependencies
- Release workflow uses --locked flag for reproducible builds
- All 17 tests passing (9 unit + 8 doc tests)

## [0.1.2] - 2026-04-01

### Fixed
- **NPM install script**: Fixed to download `mailgen` binaries instead of `emailgen`
  - Updated asset names in scripts/install.js
  - Binary name: emailgen → mailgen
  - Asset names updated for all platforms

## [0.1.1] - 2026-04-01

### Fixed
- **Release workflow**: Fixed binary packaging in GitHub Actions release workflow
  - Changed artifact names from `emailgen` to `mailgen` across all platforms
  - Linux: `mailgen-linux-x86_64.tar.gz`
  - Windows: `mailgen-windows-x86_64.zip`
  - macOS x86_64: `mailgen-macos-x86_64.tar.gz`
  - macOS aarch64: `mailgen-macos-aarch64.tar.gz`

### Changed
- Package rename from `emailgen` to `mailgen` across all registries
  - Cargo: `mailgen`
  - NPM: `@akin01/mailgen`
  - PyPI: `mailgen-rs`
  - Binary: `mailgen`

### CI/CD
- Release workflow now correctly packages `mailgen` binary
- All 17 tests passing (9 unit + 8 doc tests)
- Rust formatting checks passing

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
