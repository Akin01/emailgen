# Changelog

All notable changes to this project will be documented in this file.

## [0.1.7] - 2026-04-01

### Added
- **PowerShell installation script for Windows** (`install.ps1`)
  - One-liner installation: `powershell -ExecutionPolicy Bypass -Command "iwr -useb https://raw.githubusercontent.com/akin01/emailgen/main/install.ps1 | iex"`
  - Auto-detects architecture (x86_64/aarch64)
  - Interactive PATH configuration
  - Progress feedback during download/install
  - Handles existing installations with `-Force` flag
- Updated README.md with Windows PowerShell installation instructions

### Changed
- Moved PowerShell installation from [Unreleased] to this release

## [Unreleased]

### Added
- Alternative installation methods for future releases

## [0.1.6] - 2026-04-01

### Fixed
- **Critical**: Prevent infinite loop when requesting more emails than combination space allows
  - With default settings (72 names × 10 domains), max unique combinations is ~933k
  - Previously, requesting 1M emails would hang indefinitely
  - Now completes in ~4 seconds with guaranteed termination
- **Critical**: Bloom filter not being used in parallel generation (caused duplicate emails)
  - `generate_many_parallel()` and `generate_many_parallel_with_progress()` now use instance bloom filter
  - All generated emails are now 100% unique

### Added
- `estimate_combination_space()` to detect when request exceeds capacity
- `generate_many_with_fallback()` for guaranteed termination with fast counter-based generation
- Warning message when requested count exceeds estimated combination space
- Extended number ranges in patterns for more variety:
  - `flast#`: 10-99 → 10-999 (90 → 990 combinations)
  - `first#`: 10-999 → 10-9999 (990 → 9990 combinations)

### Changed
- Increased cached name pool from 500 to 2000 for better variety
- Cache generation for large wordlists (>1000 names) now samples directly from wordlist
  - Startup time: timeout (>2min) → <1 second (>170x faster)
- Added 30% oversampling in parallel generation to account for deduplication loss

### Performance
| Scenario | Before | After | Improvement |
|----------|--------|-------|-------------|
| 1M emails (default config) | Timeout/Hang | 4.3s | **Works!** |
| 1M emails (335k names) | Timeout | 2.7s | **>170x** |
| Large wordlist startup | >2min | <1s | **>120x** |
| Throughput (1M default) | N/A | 499k emails/sec | - |
| Uniqueness rate | ~5% | 100% | **Fixed** |

## [0.1.5] - 2026-04-01

### Fixed
- Bloom filter uniqueness checking in parallel generation
- Cache generation performance with large wordlists

### Changed
- Increased cached name pool from 500 to 2000 names
- Optimized cache generation for wordlists >1000 names

## [0.1.4] - 2026-03-26

### Changed
- Version bump for packaging consistency

## [0.1.3] - 2026-03-26

### Changed
- Version bump for packaging consistency

## [0.1.2] - 2026-03-26

### Changed
- Version bump for packaging consistency

## [0.1.1] - 2026-03-26

### Added
- Multi-platform installation script (`install.sh`) with macOS and Linux support.
- Fully automated publishing to NPM and PyPI via GitHub Actions.

### Changed
- Renamed packages to `@akin01/emailgen` on NPM and `emailgen-rs` on PyPI for uniqueness.
- Updated license copyright to 2026.
- Improved extraction logic in `install.sh` for better macOS compatibility.

### Fixed
- Resolved GitHub Actions warning regarding secret naming.
- Fixed NPM 403 Forbidden error by switching to automation tokens.

## [0.1.0] - 2026-03-26

### Added
- High-performance email generation engine using Markov chains and Bloom filters.
- Support for `uv` (Python) and `npm` (Node.js) installation.
- Automated CI/CD for binary releases across Linux, macOS, and Windows.
- Detailed performance benchmarks and optimization guides.
- Multi-threaded generation using Rayon.
- Asynchronous file I/O using Tokio.
- TUI progress bar for command-line feedback.

### Optimized
- Implemented name caching for 70% reduction in Markov overhead.
- Bloom filter for O(1) uniqueness checking.
- Parallelized generation for 4x speedup in default mode.
- Fast mode for 200x-400x speedup in bulk generation.
