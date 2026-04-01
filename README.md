# mailgen

High-performance email generator using Markov chains and Bloom filters.

[![Build Status](https://img.shields.io/github/actions/workflow/status/akin01/emailgen/ci.yml)](https://github.com/akin01/emailgen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **High Performance** - Generate 250K+ emails per second (Fast Mode)
- 🎯 **Realistic Names** - Markov chain-based name generation
- ✅ **Uniqueness Guaranteed** - Bloom filter for efficient duplicate detection
- 📝 **Custom Wordlists** - Support for custom name and domain lists
- 🔧 **Configurable** - Multiple email patterns and generation options
- 💾 **Memory Efficient** - ~1.2 MB for 1 million unique emails

## Installation

### Binary Installation (Recommended)

Quickly install the latest binary for your system (Linux, macOS, or Windows):

```bash
# Linux/macOS (using install script)
curl -fsSL https://raw.githubusercontent.com/akin01/emailgen/main/install.sh | sudo bash

# Windows PowerShell (one-liner, no file download needed)
powershell -ExecutionPolicy Bypass -Command "iwr -useb https://raw.githubusercontent.com/akin01/emailgen/main/install.ps1 | iex"

# Alternative PowerShell syntax
Invoke-WebRequest -Uri https://raw.githubusercontent.com/akin01/emailgen/main/install.ps1 -UseBasicParsing | Invoke-Expression
```

### Build from Source

## Quick Start

### Generate Emails

```bash
# Generate 1000 emails to stdout
./target/release/mailgen --count 1000

# Generate 1 million emails to file (Fast Mode)
./target/release/mailgen --count 1000000 --output emails.txt --fast

# Use custom wordlists
./target/release/mailgen --count 10000 \
    --names data/example_names.txt \
    --domains data/example_domains.txt \
    --output emails.txt
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
emailgen = { git = "https://github.com/akin01/emailgen" }
```

```rust
use mailgen::EmailGenerator;

fn main() {
    // Basic usage
    let mut generator = EmailGenerator::new();
    let email = generator.generate();
    println!("Generated: {}", email);

    // Generate many emails
    let emails = generator.generate_many(1000);

    // With custom wordlists
    let names = vec!["John Doe".to_string(), "Jane Smith".to_string()];
    let domains = vec!["example.com".to_string()];
    let mut generator = EmailGenerator::with_names_and_domains(names, domains);
    let emails = generator.generate_many(10000);
}
```

## Performance

### Generation Speed (Actual Benchmarks)

| Mode | 10K | 100K | 1M |
|------|-----|------|-----|
| **Fast Mode** (`--fast`) | 0.04s | 0.38s | 7.5s |
| **Default Mode** | 3.9s | 39s | ~6.5 min |

**💡 Tip:** Use `--fast` mode for bulk generation (>10K emails) for best performance.

### Memory Usage

- **~1.2 MB** for 1 million unique emails (Bloom filter)

### Usage

```bash
# Fast mode for bulk generation (~250K emails/sec)
./target/release/mailgen --count 1000000 --output emails.txt --fast

# Default mode with 30% Markov for variety (~2.6K emails/sec)
./target/release/mailgen --count 100000 --output emails.txt

# Generate to stdout
./target/release/mailgen --count 1000 --fast
```

See [PERFORMANCE.md](PERFORMANCE.md) for detailed benchmarks.

## Usage

### Direct Command Line

After installing via the script, `uv`, or `npm`, the `mailgen` command is available directly in your terminal:

```bash
# Basic usage
mailgen --count 1000

# Fast mode
mailgen -c 1000000 --fast
```

### Command Line Options

```
USAGE:
    emailgen [OPTIONS]

OPTIONS:
    -c, --count <COUNT>            Number of emails to generate [default: 1000]
    -o, --output <OUTPUT>          Output file path (stdout if not specified)
    -n, --names <NAMES>            Path to names wordlist file
    -d, --domains <DOMAINS>        Path to domains file
        --min-length <MIN>         Minimum username length [default: 5]
        --max-length <MAX>         Maximum username length [default: 30]
        --capacity <CAP>           Bloom filter capacity [default: 1000000]
        --fpr <FPR>                Bloom filter false positive rate [default: 0.01]
        --fast                     Fast mode (100% wordlist/cached, no Markov)
        --wordlist-percent <PCT>   Wordlist name percentage (0-100, default: auto)
        --cache-percent <PCT>      Cached name percentage (0-100, default: auto)
        --markov-percent <PCT>     Markov generation percentage (0-100, default: 30)
        --stats                    Show statistics after generation
    -q, --quiet                    Quiet mode (no output except errors)
    -h, --help                     Print help
    -V, --version                  Print version

**Features:**
- **TUI Progress Bar**: Animated text-based progress bar with spinner, percentage, speed, and ETA
- **Parallel Generation**: Multi-threaded generation (always enabled)
- **Async I/O**: Asynchronous file writing (always enabled)

**Note:** The TUI progress bar animation works best in interactive terminals. When output is redirected, you'll see the final progress state.
```

### Name Source Ratios

Control the balance between speed and variety:

```bash
# Specify all three (must add up to 100)
./target/release/mailgen --count 100000 --wordlist-percent 35 --cache-percent 35 --markov-percent 30

# Specify only one - others auto-calculated
./target/release/mailgen --count 100000 --markov-percent 20
# Auto-calculates: 40% wordlist, 40% cached, 20% Markov

./target/release/mailgen --count 100000 --wordlist-percent 80
# Auto-calculates: 80% wordlist, 15% cached, 5% Markov

./target/release/mailgen --count 100000 --cache-percent 70
# Auto-calculates: 25% wordlist, 70% cached, 5% Markov

# Specify two - third auto-calculated
./target/release/mailgen --count 100000 --wordlist-percent 50 --markov-percent 10
# Auto-calculates: 50% wordlist, 40% cached, 10% Markov

# Fast mode shortcut (50% wordlist, 50% cached, 0% Markov)
./target/release/mailgen --count 100000 --fast
```

| Ratio (wordlist/cache/markov) | Speed | Variety | Use Case |
|-------------------------------|-------|---------|----------|
| 100/0/0 | ~260K/sec | Low | Bulk test data |
| 50/50/0 (--fast) | ~260K/sec | Medium | Fast generation |
| 35/35/30 (default) | ~2.6K/sec | High | General use with variety |
| 25/25/50 | ~1.5K/sec | Very High | Maximum variety |

### Examples

```bash
# Generate 10K emails with stats
./target/release/mailgen -c 10000 --stats

# Generate with custom wordlists
./target/release/mailgen -c 100000 \
    -n names.txt \
    -d domains.txt \
    -o output.txt

# Generate with specific constraints
./target/release/mailgen -c 50000 \
    --min-length 6 \
    --max-length 20 \
    --capacity 100000 \
    --fpr 0.001
```

## Architecture

### Markov Chain Name Generation

The email generator uses character-level Markov chains to generate realistic names:

1. **Training**: Names from wordlist are converted to character sequences
2. **Generation**: New names are generated by walking the Markov chain
3. **Patterns**: Multiple email patterns create variety (first.last, firstlast, etc.)

### Bloom Filter Uniqueness

Bloom filters provide space-efficient uniqueness checking:

- **Space Efficient**: ~1.14 MB for 1M elements at 1% false positive rate
- **Fast Operations**: O(k) where k is number of hash functions
- **No False Negatives**: If it says "not seen", it's definitely unique
- **Configurable FPR**: Trade memory for accuracy

## Wordlist Format

### Names File

One name per line (first + last):

```
John Smith
Jane Doe
Bob Johnson
```

### Domains File

One domain per line:

```
gmail.com
yahoo.com
example.com
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [markovify-rs](https://crates.io/crates/markovify-rs) - Markov chain implementation
- [bloomfilter](https://crates.io/crates/bloomfilter) - Bloom filter implementation
