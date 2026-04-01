//! Email generator CLI
//!
//! A high-performance email generator using Markov chains and Bloom filters.

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use mailgen::wordlist::{get_default_domains, get_default_names, load_domains, load_names};
use mailgen::{EmailGenerator, EmailPattern, GeneratorConfig};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "mailgen")]
#[command(author = "Akin <akinpasha82@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "High-performance email generator using Markov chains and Bloom filters", long_about = None)]
struct Args {
    /// Number of emails to generate
    #[arg(short, long, default_value_t = 1000)]
    count: usize,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to names wordlist file
    #[arg(short, long)]
    names: Option<PathBuf>,

    /// Path to domains file
    #[arg(short, long)]
    domains: Option<PathBuf>,

    /// Minimum username length
    #[arg(long, default_value_t = 5)]
    min_length: usize,

    /// Maximum username length
    #[arg(long, default_value_t = 30)]
    max_length: usize,

    /// Bloom filter capacity (estimated max emails)
    #[arg(long, default_value_t = 1_000_000)]
    capacity: usize,

    /// Bloom filter false positive rate
    #[arg(long, default_value_t = 0.01)]
    fpr: f64,

    /// Fast mode (100% wordlist, no Markov generation)
    #[arg(long, default_value_t = false)]
    fast: bool,

    /// Wordlist name percentage (0-100, default: auto)
    /// If not specified, calculated from remaining percentage
    #[arg(long)]
    wordlist_percent: Option<u8>,

    /// Cached name percentage (0-100, default: auto)
    /// If not specified, calculated from remaining percentage
    #[arg(long)]
    cache_percent: Option<u8>,

    /// Markov generation percentage (0-100, default: 30)
    /// If not specified, uses 0 in fast mode, 30 otherwise
    #[arg(long)]
    markov_percent: Option<u8>,

    /// Show statistics after generation
    #[arg(long, default_value_t = false)]
    stats: bool,

    /// Quiet mode (no output except errors)
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    // Load names
    let names = if let Some(names_path) = &args.names {
        if !args.quiet {
            eprintln!("Loading names from {}...", names_path.display());
        }
        load_names(names_path).unwrap_or_else(|e| {
            eprintln!("Warning: Could not load names: {}. Using defaults.", e);
            get_default_names()
        })
    } else {
        get_default_names()
    };

    // Load domains
    let domains = if let Some(domains_path) = &args.domains {
        if !args.quiet {
            eprintln!("Loading domains from {}...", domains_path.display());
        }
        load_domains(domains_path).unwrap_or_else(|e| {
            eprintln!("Warning: Could not load domains: {}. Using defaults.", e);
            get_default_domains()
        })
    } else {
        get_default_domains()
    };

    if !args.quiet {
        eprintln!("Loaded {} names and {} domains", names.len(), domains.len());
    }

    // Calculate ratios with auto-fill
    let (wordlist, cache, markov) = if args.fast {
        (50, 50, 0)
    } else {
        let markov = args.markov_percent.unwrap_or(30); // Default 30% Markov
        let wordlist_given = args.wordlist_percent;
        let cache_given = args.cache_percent;

        match (wordlist_given, cache_given) {
            (Some(w), Some(c)) => (w, c, markov),
            (Some(w), None) => {
                // Wordlist specified, split remaining between cache and markov
                let remaining = 100 - w;
                let markov_actual = markov.min(remaining);
                (w, remaining - markov_actual, markov_actual)
            }
            (None, Some(c)) => {
                // Cache specified, split remaining between wordlist and markov
                let remaining = 100 - c;
                let markov_actual = markov.min(remaining);
                (remaining - markov_actual, c, markov_actual)
            }
            (None, None) => {
                // Neither specified, use default split with given markov (30%)
                let remaining = 100 - markov;
                (remaining - (remaining / 2), remaining / 2, markov)
            }
        }
    };

    // Create generator with custom capacity
    let mut generator = EmailGenerator::with_names_and_domains(names, domains)
        .with_capacity(args.capacity, args.fpr)
        .with_fast_mode(args.fast)
        .with_name_source_ratios(wordlist, cache, markov)
        .with_config(GeneratorConfig {
            min_username_length: args.min_length,
            max_username_length: args.max_length,
            patterns: vec![
                (EmailPattern::FirstLast, 35),
                (EmailPattern::FirstLastNoSep, 25),
                (EmailPattern::FirstInitialLast, 20),
                (EmailPattern::FirstUnderscoreLast, 10),
                (EmailPattern::FirstNumber, 10),
            ],
            add_numbers: true,
            max_number: 999,
        });

    if !args.quiet {
        if args.fast {
            eprintln!("Fast mode enabled (100% wordlist/cached names)");
        } else if args.wordlist_percent.is_some()
            || args.cache_percent.is_some()
            || args.markov_percent.is_some()
        {
            eprintln!(
                "Name sources: {}% wordlist, {}% cached, {}% Markov",
                wordlist, cache, markov
            );
            if args.wordlist_percent.is_none()
                || args.cache_percent.is_none()
                || args.markov_percent.is_none()
            {
                eprintln!("(auto-calculated from specified values)");
            }
        }
        eprintln!(
            "Parallel generation enabled ({} threads)",
            rayon::current_num_threads()
        );
    }

    // Generate emails
    let start_time = std::time::Instant::now();

    if let Some(output_path) = &args.output {
        // Generate to file with optional TUI progress bar
        let count = if args.quiet {
            // No progress bar in quiet mode for maximum performance
            generator
                .generate_to_file_async(args.count, output_path)
                .await?
        } else {
            let pb = ProgressBar::new(args.count as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {per_sec} ETA: {eta}")
                .unwrap()
                .progress_chars("=>-"));

            eprintln!(
                "Generating {} emails to {}...",
                args.count,
                output_path.display()
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let count = generator
                .generate_to_file_async_with_progress(
                    args.count,
                    output_path,
                    Some(Arc::new(pb.clone())),
                )
                .await?;
            pb.finish_with_message("✅ Complete!");
            count
        };

        let elapsed = start_time.elapsed();

        if args.stats {
            print_stats(&generator, count, elapsed);
        }
    } else {
        // Generate to stdout with optional TUI progress bar
        let emails = if args.quiet {
            // No progress bar in quiet mode for maximum performance
            generator.generate_many_parallel(args.count)
        } else {
            let pb = ProgressBar::new(args.count as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {per_sec} ETA: {eta}")
                .unwrap()
                .progress_chars("=>-"));

            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let emails = generator
                .generate_many_parallel_with_progress(args.count, Some(Arc::new(pb.clone())));
            pb.finish_with_message("✅ Complete!");
            emails
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for email in &emails {
            writeln!(handle, "{}", email)?;
        }

        let elapsed = start_time.elapsed();

        if args.stats {
            print_stats(&generator, emails.len(), elapsed);
        }
    }

    Ok(())
}

fn print_stats(generator: &EmailGenerator, count: usize, elapsed: std::time::Duration) {
    eprintln!("\n=== Generation Statistics ===");
    eprintln!("Emails generated: {}", count);
    eprintln!("Time elapsed: {:.2?}", elapsed);
    eprintln!(
        "Throughput: {:.0} emails/sec",
        count as f64 / elapsed.as_secs_f64()
    );
    eprintln!("Memory usage: {:.2} MB", generator.memory_usage_mb());
    eprintln!(
        "Bloom filter FPR: {:.2}%",
        generator.false_positive_rate() * 100.0
    );
}
