//! # mailgen
//!
//! High-performance email generator using Markov chains and Bloom filters.
//!
//! This library provides tools for generating realistic, unique email addresses
//! at scale using:
//! - **Markov chains** (via markovify-rs) for realistic name generation
//! - **Bloom filters** for memory-efficient uniqueness checking
//! - **Configurable patterns** for varied email formats
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mailgen::EmailGenerator;
//!
//! let mut generator = EmailGenerator::new();
//! let email = generator.generate();
//! println!("Generated: {}", email);
//! ```
//!
//! ## Generate Many Emails
//!
//! ```rust,no_run
//! use mailgen::EmailGenerator;
//!
//! let mut generator = EmailGenerator::new();
//! let emails = generator.generate_many(1000);
//! ```
//!
//! ## Custom Names and Domains
//!
//! ```rust,no_run
//! use mailgen::EmailGenerator;
//! use mailgen::wordlist::{load_names, load_domains};
//!
//! let names = load_names("data/names.txt").unwrap();
//! let domains = load_domains("data/domains.txt").unwrap();
//!
//! let mut generator = EmailGenerator::with_names_and_domains(names, domains);
//! let emails = generator.generate_many(10000);
//! ```
//!
//! ## Generate to File
//!
//! ```rust,no_run
//! use mailgen::EmailGenerator;
//!
//! let mut generator = EmailGenerator::new();
//! generator.generate_to_file(1_000_000, "emails.txt").unwrap();
//! ```

pub mod bloom;
pub mod generator;
pub mod wordlist;

pub use bloom::EmailBloomFilter;
pub use generator::{
    EmailGenerator, EmailPattern, FastGenerationConfig, GeneratorConfig, NameSourceConfig,
};
pub use wordlist::{load_domains, load_names, load_names_from_multiple};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
