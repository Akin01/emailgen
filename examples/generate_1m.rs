//! Example: Generate 1 million unique email addresses
//!
//! This example demonstrates how to generate a large number of unique
//! email addresses efficiently using Bloom filters for uniqueness checking.
//!
//! Run with: cargo run --example generate_1m

use mailgen::EmailGenerator;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

fn main() {
    println!("=== Email Generator: 1 Million Emails ===\n");

    // Configuration
    let target_count = 1_000_000;
    let output_file = "target/million_emails.txt";

    // Ensure target directory exists
    std::fs::create_dir_all("target").ok();

    println!("Configuration:");
    println!("  Target count: {} emails", target_count);
    println!("  Output file: {}", output_file);
    println!();

    // Create generator with appropriate Bloom filter capacity
    let mut generator = EmailGenerator::new().with_capacity(target_count, 0.01); // 1% false positive rate

    println!("Bloom Filter Info:");
    println!("  Estimated capacity: {} items", target_count);
    println!("  False positive rate: 1%");
    println!("  Memory usage: {:.2} MB", generator.memory_usage_mb());
    println!();

    // Start generation
    println!("Generating {} emails...", target_count);
    let start_time = Instant::now();

    // Create output file
    let file = File::create(output_file).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);

    // Generate and write emails
    let mut count = 0;
    let batch_size = 100_000;

    while count < target_count {
        let batch_start = Instant::now();

        for _ in 0..batch_size.min(target_count - count) {
            let email = generator.generate();
            writeln!(writer, "{}", email).expect("Failed to write email");
            count += 1;
        }

        let batch_elapsed = batch_start.elapsed();
        let batch_rate = batch_size as f64 / batch_elapsed.as_secs_f64();

        println!(
            "  Generated {:>7} emails ({:>6.0} emails/sec, {:.2?} elapsed)",
            count,
            batch_rate,
            start_time.elapsed()
        );
    }

    writer.flush().expect("Failed to flush output");

    // Calculate statistics
    let total_elapsed = start_time.elapsed();
    let throughput = target_count as f64 / total_elapsed.as_secs_f64();

    // Print summary
    println!("\n=== Generation Complete ===");
    println!("Total emails generated: {}", count);
    println!("Total time: {:?}", total_elapsed);
    println!("Average throughput: {:.0} emails/sec", throughput);
    println!("Output file: {}", output_file);
    println!("Output file size: {}", format_file_size(output_file));
    println!("Memory usage: {:.2} MB", generator.memory_usage_mb());

    // Verify uniqueness (sample check)
    println!("\nVerifying uniqueness (sampling 10,000 emails)...");
    let verification_start = Instant::now();

    let content = std::fs::read_to_string(output_file).expect("Failed to read output file");
    let lines: Vec<&str> = content.lines().take(10_000).collect();
    let unique_count = lines.iter().collect::<std::collections::HashSet<_>>().len();

    let verification_elapsed = verification_start.elapsed();
    println!(
        "Unique in sample: {} / {} ({:.2}%)",
        unique_count,
        lines.len(),
        (unique_count as f64 / lines.len() as f64) * 100.0
    );
    println!("Verification time: {:?}", verification_elapsed);

    println!("\n=== Example Complete ===");
}

fn format_file_size(path: &str) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size > 1_000_000 {
                format!("{:.2} MB", size as f64 / 1_000_000.0)
            } else if size > 1_000 {
                format!("{:.2} KB", size as f64 / 1_000.0)
            } else {
                format!("{} bytes", size)
            }
        }
        Err(_) => "unknown".to_string(),
    }
}
