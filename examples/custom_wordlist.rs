//! Example: Generate emails with custom wordlist
//!
//! This example demonstrates how to use custom name and domain wordlists
//! for generating targeted email addresses.
//!
//! Run with: cargo run --example custom_wordlist

use mailgen::wordlist::{load_domains, load_names};
use mailgen::{EmailGenerator, EmailPattern, GeneratorConfig};
use std::path::Path;

fn main() {
    println!("=== Custom Wordlist Email Generator ===\n");

    // Example 1: Using default wordlists
    println!("1. Using Default Wordlists");
    println!("{}", "-".repeat(40));

    let mut gen1 = EmailGenerator::new();
    println!("Generated 5 emails with defaults:");
    for i in 1..=5 {
        println!("  {}. {}", i, gen1.generate());
    }
    println!();

    // Example 2: Create custom wordlists inline
    println!("2. Using Inline Custom Wordlists");
    println!("{}", "-".repeat(40));

    let custom_names: Vec<String> = vec![
        "Alice Johnson",
        "Bob Smith",
        "Carol Williams",
        "David Brown",
        "Eva Davis",
        "Frank Miller",
        "Grace Wilson",
        "Henry Moore",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let custom_domains: Vec<String> = vec![
        "techcorp.com",
        "startup.io",
        "enterprise.net",
        "business.org",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut gen2 =
        EmailGenerator::with_names_and_domains(custom_names.clone(), custom_domains.clone());

    println!("Generated 5 emails with custom lists:");
    for i in 1..=5 {
        println!("  {}. {}", i, gen2.generate());
    }
    println!();

    // Example 3: Custom patterns
    println!("3. Using Custom Email Patterns");
    println!("{}", "-".repeat(40));

    let mut gen3 =
        EmailGenerator::with_names_and_domains(custom_names.clone(), custom_domains.clone())
            .with_config(GeneratorConfig {
                min_username_length: 4,
                max_username_length: 20,
                patterns: vec![
                    (
                        EmailPattern::Custom("{first}_{last}@{domain}".to_string()),
                        50,
                    ),
                    (
                        EmailPattern::Custom("{first}.{last}{number}@{domain}".to_string()),
                        30,
                    ),
                    (EmailPattern::FirstLast, 20),
                ],
                add_numbers: true,
                max_number: 99,
            });

    println!("Generated 5 emails with custom patterns:");
    for i in 1..=5 {
        println!("  {}. {}", i, gen3.generate());
    }
    println!();

    // Example 4: Load from files (if they exist)
    println!("4. Loading from Files");
    println!("{}", "-".repeat(40));

    // Create example files
    std::fs::create_dir_all("target/example_data").ok();

    std::fs::write(
        "target/example_data/names.txt",
        "John Smith\nJane Doe\nBob Johnson\nAlice Williams\nCharlie Brown\n",
    )
    .ok();

    std::fs::write(
        "target/example_data/domains.txt",
        "example.com\ntest.org\nmail.net\ndemo.io\n",
    )
    .ok();

    let names_path = "target/example_data/names.txt";
    let domains_path = "target/example_data/domains.txt";

    if Path::new(names_path).exists() && Path::new(domains_path).exists() {
        let names = load_names(names_path).expect("Failed to load names");
        let domains = load_domains(domains_path).expect("Failed to load domains");

        println!(
            "Loaded {} names and {} domains from files",
            names.len(),
            domains.len()
        );

        let mut gen4 = EmailGenerator::with_names_and_domains(names, domains);

        println!("Generated 5 emails from file wordlists:");
        for i in 1..=5 {
            println!("  {}. {}", i, gen4.generate());
        }
    } else {
        println!("Example files not found, skipping file loading demo.");
    }
    println!();

    // Example 5: Generate many emails
    println!("5. Bulk Generation Performance");
    println!("{}", "-".repeat(40));

    let mut gen5 = EmailGenerator::with_names_and_domains(custom_names, custom_domains);

    let count = 100_000;
    let start = std::time::Instant::now();
    let emails = gen5.generate_many(count);
    let elapsed = start.elapsed();

    println!("Generated {} emails in {:?}", count, elapsed);
    println!(
        "Throughput: {:.0} emails/sec",
        count as f64 / elapsed.as_secs_f64()
    );
    println!("Memory usage: {:.2} MB", gen5.memory_usage_mb());

    // Show sample
    println!("\nSample of generated emails:");
    for (i, email) in emails.iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, email);
    }

    println!("\n=== Example Complete ===");
}
