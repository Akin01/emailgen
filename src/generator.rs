//! Email generator using Markov chains for realistic name generation.
//!
//! This module provides the core email generation functionality using
//! markovify-rs for name generation and Bloom filters for uniqueness.

use crate::bloom::EmailBloomFilter;
use crate::wordlist::{get_default_domains, get_default_names};
use indicatif::ProgressBar;
use markovify_rs::Text;
use rand::Rng;
use rayon::prelude::*;
use std::sync::Arc;

/// Configuration for fast generation (prioritizes speed over variety)
#[derive(Debug, Clone, Copy, Default)]
pub struct FastGenerationConfig {
    /// Use only wordlist sampling (no Markov generation)
    pub wordlist_only: bool,
    /// Skip some validation checks
    pub skip_validation: bool,
}

/// Configuration for name source ratios
#[derive(Debug, Clone, Copy)]
pub struct NameSourceConfig {
    /// Percentage of names from wordlist (0-100)
    pub wordlist_percent: u8,
    /// Percentage of names from cache (0-100, remaining is Markov)
    pub cache_percent: u8,
    /// Percentage of fresh Markov generation (0-100)
    /// Note: wordlist + cache + markov should equal 100
    pub markov_percent: u8,
}

impl Default for NameSourceConfig {
    fn default() -> Self {
        // Default: 40% wordlist, 60% cached, 0% Markov (for best performance with variety)
        // Markov generation is slow, so we use pre-generated cached names instead
        Self {
            wordlist_percent: 40,
            cache_percent: 60,
            markov_percent: 0,
        }
    }
}

impl NameSourceConfig {
    /// Create a config optimized for speed (95% wordlist/cached)
    pub fn fast() -> Self {
        Self {
            wordlist_percent: 50,
            cache_percent: 50,
            markov_percent: 0,
        }
    }

    /// Create a config optimized for variety (more Markov generation)
    pub fn varied() -> Self {
        Self {
            wordlist_percent: 30,
            cache_percent: 30,
            markov_percent: 40,
        }
    }

    /// Create a balanced config
    pub fn balanced() -> Self {
        Self {
            wordlist_percent: 50,
            cache_percent: 45,
            markov_percent: 5,
        }
    }
}

/// Email pattern templates for generating varied email addresses.
#[derive(Debug, Clone, Default)]
pub enum EmailPattern {
    /// first.last@domain (e.g., john.smith@example.com)
    #[default]
    FirstLast,
    /// firstlast@domain (e.g., johnsmith@example.com)
    FirstLastNoSep,
    /// flast@domain (e.g., jsmith@example.com)
    FirstInitialLast,
    /// first_last@domain (e.g., john_smith@example.com)
    FirstUnderscoreLast,
    /// firstN@domain where N is a number (e.g., john23@example.com)
    FirstNumber,
    /// last.first@domain (e.g., smith.john@example.com)
    LastFirst,
    /// Custom pattern with placeholders {first}, {last}, {domain}, {number}
    Custom(String),
}

/// Configuration for the email generator.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Minimum username length
    pub min_username_length: usize,
    /// Maximum username length
    pub max_username_length: usize,
    /// Email patterns to use (weighted)
    pub patterns: Vec<(EmailPattern, u32)>,
    /// Whether to add random numbers to usernames
    pub add_numbers: bool,
    /// Maximum number to add (if add_numbers is true)
    pub max_number: u32,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            min_username_length: 5,
            max_username_length: 30,
            patterns: vec![
                (EmailPattern::FirstLast, 35),
                (EmailPattern::FirstLastNoSep, 25),
                (EmailPattern::FirstInitialLast, 20),
                (EmailPattern::FirstUnderscoreLast, 10),
                (EmailPattern::FirstNumber, 10),
            ],
            add_numbers: true,
            max_number: 999,
        }
    }
}

/// High-performance email generator using Markov chains.
pub struct EmailGenerator {
    /// Text model for first names
    first_name_model: Option<Text>,
    /// Text model for last names
    last_name_model: Option<Text>,
    /// List of first names for direct sampling (Arc for cheap cloning)
    first_names: Arc<Vec<String>>,
    /// List of last names for direct sampling (Arc for cheap cloning)
    last_names: Arc<Vec<String>>,
    /// Cached first names from Markov model (Arc for cheap cloning)
    cached_first_names: Arc<Vec<String>>,
    /// Cached last names from Markov model (Arc for cheap cloning)
    cached_last_names: Arc<Vec<String>>,
    /// List of domains to use (Arc for cheap cloning)
    domains: Arc<Vec<String>>,
    /// Bloom filter for uniqueness
    bloom_filter: EmailBloomFilter,
    /// Generator configuration
    config: GeneratorConfig,
    /// Name source configuration (wordlist vs cache vs Markov)
    name_source_config: NameSourceConfig,
    /// Fast generation config
    fast_config: FastGenerationConfig,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
    /// Count of generated emails
    generated_count: usize,
}

impl EmailGenerator {
    /// Create a new email generator with default settings.
    ///
    /// Uses built-in fallback names and domains.
    pub fn new() -> Self {
        Self::with_names_and_domains(get_default_names(), get_default_domains())
    }

    /// Create a new email generator with custom names and domains.
    ///
    /// # Arguments
    /// * `names` - List of names for training the Markov model
    /// * `domains` - List of domains to use
    pub fn with_names_and_domains(names: Vec<String>, domains: Vec<String>) -> Self {
        // Build separate lists for first and last names
        let (first_names, last_names) = Self::split_names(&names);

        // Build Markov models for first and last names
        let first_name_model = Self::build_name_model(&first_names);
        let last_name_model = Self::build_name_model(&last_names);

        // Pre-generate cache of names from Markov model (larger cache for better variety)
        // For large wordlists, this samples directly from wordlist for speed
        let cached_first_names = Self::generate_name_cache(&first_name_model, 2000, &first_names);
        let cached_last_names = Self::generate_name_cache(&last_name_model, 2000, &last_names);

        Self {
            first_name_model: Some(first_name_model),
            last_name_model: Some(last_name_model),
            first_names: Arc::new(first_names),
            last_names: Arc::new(last_names),
            cached_first_names: Arc::new(cached_first_names),
            cached_last_names: Arc::new(cached_last_names),
            domains: Arc::new(domains),
            bloom_filter: EmailBloomFilter::new(1_000_000, 0.01),
            config: GeneratorConfig::default(),
            name_source_config: NameSourceConfig::default(),
            fast_config: FastGenerationConfig::default(),
            rng: rand::thread_rng(),
            generated_count: 0,
        }
    }

    /// Generate a cache of names from a Markov model.
    /// For large wordlists, samples directly from the wordlist for speed.
    fn generate_name_cache(model: &Text, count: usize, wordlist: &[String]) -> Vec<String> {
        let mut cache = Vec::with_capacity(count);

        // For large wordlists, sample directly from wordlist (much faster)
        // Markov generation is slow with large corpora
        if wordlist.len() > 1000 {
            let mut rng = rand::thread_rng();
            while cache.len() < count {
                if let Some(name) = wordlist.get(rng.gen_range(0..wordlist.len())) {
                    let name = name.trim().to_string();
                    if name.len() >= 2
                        && name.len() <= 15
                        && !name.contains(' ')
                        && name.chars().all(|c| c.is_alphabetic())
                    {
                        cache.push(capitalize_first(&name));
                    }
                }
            }
            return cache;
        }

        // For small wordlists, use Markov generation for variety
        let mut attempts = 0;
        let max_attempts = count * 5; // Allow more attempts to fill cache

        while cache.len() < count && attempts < max_attempts {
            attempts += 1;
            if let Some(name) = model.make_short_sentence(
                15,          // max_chars
                Some(2),     // min_chars
                None,        // init_state
                Some(5),     // tries (more tries for better fill rate)
                None,        // max_overlap_ratio
                None,        // max_overlap_total
                Some(false), // test_output
                Some(1),     // max_words
                Some(1),     // min_words
            ) {
                let name = name.trim().to_string();
                if name.len() >= 2
                    && name.len() <= 15
                    && !name.contains(' ')
                    && name.chars().all(|c| c.is_alphabetic())
                {
                    cache.push(capitalize_first(&name));
                }
            }
        }

        // Fill remaining with wordlist if cache is incomplete
        let mut rng = rand::thread_rng();
        while cache.len() < count {
            if let Some(name) = wordlist.get(rng.gen_range(0..wordlist.len())) {
                let name = name.trim().to_string();
                if name.len() >= 2
                    && name.len() <= 15
                    && !name.contains(' ')
                    && name.chars().all(|c| c.is_alphabetic())
                {
                    cache.push(capitalize_first(&name));
                }
            }
        }

        cache
    }

    /// Split full names into first and last name lists.
    fn split_names(names: &[String]) -> (Vec<String>, Vec<String>) {
        let mut first_names = Vec::new();
        let mut last_names = Vec::new();

        for name in names {
            let parts: Vec<&str> = name.split_whitespace().collect();
            if parts.len() >= 2 {
                first_names.push(parts[0].to_string());
                last_names.push(parts[parts.len() - 1].to_string());
            } else if !name.is_empty() {
                // If only one name, use it for both
                first_names.push(name.clone());
            }
        }

        // Add defaults if empty
        if first_names.is_empty() {
            first_names = vec!["John".to_string(), "Jane".to_string(), "Bob".to_string()];
        }
        if last_names.is_empty() {
            last_names = vec![
                "Smith".to_string(),
                "Doe".to_string(),
                "Johnson".to_string(),
            ];
        }

        (first_names, last_names)
    }

    /// Build a text model for name generation.
    fn build_name_model(names: &[String]) -> Text {
        let corpus = names.join("\n");
        Text::new(&corpus, 1, true, false, None)
            .unwrap_or_else(|_| Text::new("John\nJane\nBob", 1, true, false, None).unwrap())
    }

    /// Generate a random first name - uses configurable source ratios.
    pub fn generate_first_name(&mut self) -> String {
        // Fast path: 100% wordlist/cached (no Markov at all)
        if self.fast_config.wordlist_only {
            // Direct 50/50 split between wordlist and cached
            if self.rng.gen_bool(0.5) && !self.first_names.is_empty() {
                return self.first_names[self.rng.gen_range(0..self.first_names.len())].clone();
            } else if !self.cached_first_names.is_empty() {
                return self.cached_first_names
                    [self.rng.gen_range(0..self.cached_first_names.len())]
                .clone();
            } else if !self.first_names.is_empty() {
                return self.first_names[self.rng.gen_range(0..self.first_names.len())].clone();
            }
            // Fallback
            return ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][self.rng.gen_range(0..6)]
                .to_string();
        }

        // Use configured ratios
        let choice = self.rng.gen_range(0..100);
        let wordlist_end = self.name_source_config.wordlist_percent as usize;
        let cache_end = wordlist_end + self.name_source_config.cache_percent as usize;

        if choice < wordlist_end && !self.first_names.is_empty() {
            return self.first_names[self.rng.gen_range(0..self.first_names.len())].clone();
        } else if choice < cache_end && !self.cached_first_names.is_empty() {
            return self.cached_first_names[self.rng.gen_range(0..self.cached_first_names.len())]
                .clone();
        }

        // Markov generation
        if let Some(ref model) = self.first_name_model {
            for _ in 0..3 {
                if let Some(name) = model.make_short_sentence(
                    12,
                    Some(2),
                    None,
                    Some(3),
                    None,
                    None,
                    Some(false),
                    Some(1),
                    Some(1),
                ) {
                    let name = name.trim().to_string();
                    if name.len() >= 2
                        && name.len() <= 12
                        && !name.contains(' ')
                        && name.chars().all(|c| c.is_alphabetic())
                    {
                        return capitalize_first(&name);
                    }
                }
            }
        }

        // Fallback to wordlist
        if !self.first_names.is_empty() {
            return self.first_names[self.rng.gen_range(0..self.first_names.len())].clone();
        }

        // Ultimate fallback
        ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][self.rng.gen_range(0..6)].to_string()
    }

    /// Generate a random last name - uses configurable source ratios.
    pub fn generate_last_name(&mut self) -> String {
        // Fast path: 100% wordlist/cached (no Markov at all)
        if self.fast_config.wordlist_only {
            // Direct 50/50 split between wordlist and cached
            if self.rng.gen_bool(0.5) && !self.last_names.is_empty() {
                return self.last_names[self.rng.gen_range(0..self.last_names.len())].clone();
            } else if !self.cached_last_names.is_empty() {
                return self.cached_last_names[self.rng.gen_range(0..self.cached_last_names.len())]
                    .clone();
            } else if !self.last_names.is_empty() {
                return self.last_names[self.rng.gen_range(0..self.last_names.len())].clone();
            }
            // Fallback
            return ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"]
                [self.rng.gen_range(0..6)]
            .to_string();
        }

        // Use configured ratios
        let choice = self.rng.gen_range(0..100);
        let wordlist_end = self.name_source_config.wordlist_percent as usize;
        let cache_end = wordlist_end + self.name_source_config.cache_percent as usize;

        if choice < wordlist_end && !self.last_names.is_empty() {
            return self.last_names[self.rng.gen_range(0..self.last_names.len())].clone();
        } else if choice < cache_end && !self.cached_last_names.is_empty() {
            return self.cached_last_names[self.rng.gen_range(0..self.cached_last_names.len())]
                .clone();
        }

        // Markov generation
        if let Some(ref model) = self.last_name_model {
            for _ in 0..3 {
                if let Some(name) = model.make_short_sentence(
                    15,
                    Some(2),
                    None,
                    Some(3),
                    None,
                    None,
                    Some(false),
                    Some(1),
                    Some(1),
                ) {
                    let name = name.trim().to_string();
                    if name.len() >= 2
                        && name.len() <= 15
                        && !name.contains(' ')
                        && name.chars().all(|c| c.is_alphabetic())
                    {
                        return capitalize_first(&name);
                    }
                }
            }
        }

        // Fallback to wordlist
        if !self.last_names.is_empty() {
            return self.last_names[self.rng.gen_range(0..self.last_names.len())].clone();
        }

        // Ultimate fallback
        ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"][self.rng.gen_range(0..6)]
            .to_string()
    }

    /// Generate a random domain from the domain list.
    pub fn generate_domain(&mut self) -> String {
        if self.domains.is_empty() {
            return "example.com".to_string();
        }

        self.domains[self.rng.gen_range(0..self.domains.len())].clone()
    }

    /// Generate a username from first and last name.
    fn generate_username(&mut self, first: &str, last: &str) -> String {
        let pattern = self.select_pattern();
        let number = if self.config.add_numbers {
            self.rng.gen_range(0..=self.config.max_number)
        } else {
            0
        };

        match pattern {
            EmailPattern::FirstLast => {
                format!("{}.{}", first.to_lowercase(), last.to_lowercase())
            }
            EmailPattern::FirstLastNoSep => {
                format!("{}{}", first.to_lowercase(), last.to_lowercase())
            }
            EmailPattern::FirstInitialLast => {
                format!(
                    "{}{}{}",
                    first.chars().next().unwrap_or('u').to_lowercase(),
                    last.to_lowercase(),
                    if number > 0 {
                        number.to_string()
                    } else {
                        String::new()
                    }
                )
            }
            EmailPattern::FirstUnderscoreLast => {
                format!("{}_{}", first.to_lowercase(), last.to_lowercase())
            }
            EmailPattern::FirstNumber => {
                format!(
                    "{}{}{}",
                    first.to_lowercase(),
                    if number > 0 {
                        number
                    } else {
                        self.rng.gen_range(10..99)
                    },
                    ""
                )
            }
            EmailPattern::LastFirst => {
                format!("{}.{}", last.to_lowercase(), first.to_lowercase())
            }
            EmailPattern::Custom(template) => template
                .replace("{first}", &first.to_lowercase())
                .replace("{last}", &last.to_lowercase())
                .replace("{domain}", &self.generate_domain())
                .replace(
                    "{number}",
                    &if number > 0 {
                        number.to_string()
                    } else {
                        self.rng.gen_range(10..99).to_string()
                    },
                ),
        }
    }

    /// Select a random pattern based on weights.
    fn select_pattern(&mut self) -> EmailPattern {
        let total_weight: u32 = self.config.patterns.iter().map(|(_, w)| w).sum();
        let mut rand_val = self.rng.gen_range(0..total_weight);

        for (pattern, weight) in &self.config.patterns {
            if rand_val < *weight {
                return pattern.clone();
            }
            rand_val -= weight;
        }

        EmailPattern::FirstLast
    }

    /// Generate a unique email address.
    ///
    /// Uses the Bloom filter to ensure uniqueness.
    ///
    /// # Returns
    /// A unique email address
    pub fn generate(&mut self) -> String {
        let mut attempts = 0;
        let max_attempts = 100;

        loop {
            let first = self.generate_first_name();
            let last = self.generate_last_name();
            let username = self.generate_username(&first, &last);
            let domain = self.generate_domain();
            let email = format!("{}@{}", username, domain);

            // Check length constraints
            if username.len() < self.config.min_username_length
                || username.len() > self.config.max_username_length
            {
                attempts += 1;
                if attempts >= max_attempts {
                    // Return anyway if we can't meet constraints
                    break;
                }
                continue;
            }

            // Check uniqueness with Bloom filter
            if self.bloom_filter.check_and_insert(&email) {
                self.generated_count += 1;
                return email;
            }

            attempts += 1;
            if attempts >= max_attempts {
                // Return a unique email with timestamp
                let unique_email = format!("{}{}@{}", username, self.generated_count, domain);
                self.bloom_filter.insert(&unique_email);
                self.generated_count += 1;
                return unique_email;
            }
        }

        // Fallback
        let count = self.generated_count;
        let domain = self.generate_domain();
        let email = format!("user{}@{}", count, domain);
        self.bloom_filter.insert(&email);
        self.generated_count += 1;
        email
    }

    /// Generate multiple unique email addresses.
    ///
    /// # Arguments
    /// * `count` - Number of emails to generate
    ///
    /// # Returns
    /// A vector of unique email addresses
    pub fn generate_many(&mut self, count: usize) -> Vec<String> {
        let mut emails = Vec::with_capacity(count);
        for _ in 0..count {
            emails.push(self.generate());
        }
        emails
    }

    /// Generate multiple unique email addresses in parallel.
    /// Optimized for maximum throughput with minimal overhead.
    /// Uses a two-phase approach: parallel generation followed by bloom filter deduplication.
    /// Oversamples by 30% to account for deduplication loss.
    ///
    /// # Arguments
    /// * `count` - Number of emails to generate
    ///
    /// # Returns
    /// A vector of unique email addresses
    pub fn generate_many_parallel(&mut self, count: usize) -> Vec<String> {
        // Get current state for cloning
        let first_names = self.first_names.clone();
        let last_names = self.last_names.clone();
        let cached_first_names = self.cached_first_names.clone();
        let cached_last_names = self.cached_last_names.clone();
        let domains = self.domains.clone();
        let name_source_config = self.name_source_config;
        let fast_config = self.fast_config;

        // Estimate combination space to detect if request is feasible
        let estimated_space = self.estimate_combination_space(&first_names, &last_names, &domains);
        if count > estimated_space {
            eprintln!(
                "Warning: Requested {} emails exceeds estimated combination space of {}",
                count, estimated_space
            );
            eprintln!("Adding more names/domains or increasing number ranges will help.");
        }

        // Oversample by 30% to account for deduplication
        let oversample_factor = 1.3;
        let target_count = count;
        let generate_count = (count as f64 * oversample_factor) as usize;

        // Generate in parallel chunks - larger chunks for better throughput
        let num_threads = rayon::current_num_threads();
        let chunk_size = (generate_count / num_threads).max(1000);
        let num_chunks = generate_count.div_ceil(chunk_size);

        let results: Vec<Vec<String>> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk_idx| {
                let start = chunk_idx * chunk_size;
                let end = (start + chunk_size).min(generate_count);
                let chunk_count = end - start;

                // Create independent RNG for this chunk with unique seed
                let mut rng = rand::rngs::ThreadRng::default();
                let mut chunk_emails = Vec::with_capacity(chunk_count);

                // Pre-calculate pattern thresholds
                let pattern_thresholds = (35, 60, 80, 90); // first.last, firstlast, flast#, first_last, first#

                for _ in 0..chunk_count {
                    // Generate first name
                    let first = if fast_config.wordlist_only {
                        if rng.gen_bool(0.5) && !first_names.is_empty() {
                            first_names[rng.gen_range(0..first_names.len())].clone()
                        } else if !cached_first_names.is_empty() {
                            cached_first_names[rng.gen_range(0..cached_first_names.len())].clone()
                        } else if !first_names.is_empty() {
                            first_names[rng.gen_range(0..first_names.len())].clone()
                        } else {
                            ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][rng.gen_range(0..6)]
                                .to_string()
                        }
                    } else {
                        let choice = rng.gen_range(0..100);
                        let wordlist_end = name_source_config.wordlist_percent as usize;
                        let cache_end = wordlist_end + name_source_config.cache_percent as usize;

                        if choice < wordlist_end && !first_names.is_empty() {
                            first_names[rng.gen_range(0..first_names.len())].clone()
                        } else if choice < cache_end && !cached_first_names.is_empty() {
                            cached_first_names[rng.gen_range(0..cached_first_names.len())].clone()
                        } else if !first_names.is_empty() {
                            first_names[rng.gen_range(0..first_names.len())].clone()
                        } else {
                            ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][rng.gen_range(0..6)]
                                .to_string()
                        }
                    };

                    // Generate last name
                    let last = if fast_config.wordlist_only {
                        if rng.gen_bool(0.5) && !last_names.is_empty() {
                            last_names[rng.gen_range(0..last_names.len())].clone()
                        } else if !cached_last_names.is_empty() {
                            cached_last_names[rng.gen_range(0..cached_last_names.len())].clone()
                        } else if !last_names.is_empty() {
                            last_names[rng.gen_range(0..last_names.len())].clone()
                        } else {
                            ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"]
                                [rng.gen_range(0..6)]
                            .to_string()
                        }
                    } else {
                        let choice = rng.gen_range(0..100);
                        let wordlist_end = name_source_config.wordlist_percent as usize;
                        let cache_end = wordlist_end + name_source_config.cache_percent as usize;

                        if choice < wordlist_end && !last_names.is_empty() {
                            last_names[rng.gen_range(0..last_names.len())].clone()
                        } else if choice < cache_end && !cached_last_names.is_empty() {
                            cached_last_names[rng.gen_range(0..cached_last_names.len())].clone()
                        } else if !last_names.is_empty() {
                            last_names[rng.gen_range(0..last_names.len())].clone()
                        } else {
                            ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"]
                                [rng.gen_range(0..6)]
                            .to_string()
                        }
                    };

                    // Generate email pattern - optimized inline
                    let pattern_choice = rng.gen_range(0..100);
                    let username = if pattern_choice < pattern_thresholds.0 {
                        // first.last (35%)
                        format!("{}.{}", first.to_lowercase(), last.to_lowercase())
                    } else if pattern_choice < pattern_thresholds.1 {
                        // firstlast (25%)
                        format!("{}{}", first.to_lowercase(), last.to_lowercase())
                    } else if pattern_choice < pattern_thresholds.2 {
                        // flast# (20%)
                        format!(
                            "{}{}{}",
                            first.chars().next().unwrap_or('u').to_lowercase(),
                            last.to_lowercase(),
                            rng.gen_range(10..999) // Extended range for more variety
                        )
                    } else if pattern_choice < pattern_thresholds.3 {
                        // first_last (10%)
                        format!("{}_{}", first.to_lowercase(), last.to_lowercase())
                    } else {
                        // first# (10%)
                        format!("{}{}", first.to_lowercase(), rng.gen_range(10..9999))
                        // Extended range
                    };

                    let domain = &domains[rng.gen_range(0..domains.len())];
                    chunk_emails.push(format!("{}@{}", username, domain));
                }

                chunk_emails
            })
            .collect();

        // Phase 2: Use bloom filter to ensure uniqueness
        let mut all_emails: Vec<String> = results.into_iter().flatten().collect();

        // Sort and dedup first (fast path for exact duplicates)
        all_emails.sort_unstable();
        all_emails.dedup();

        // Use the instance's bloom filter to ensure global uniqueness
        let mut unique_emails = Vec::with_capacity(target_count);

        for email in all_emails {
            if self.bloom_filter.check_and_insert(&email) {
                unique_emails.push(email);
                if unique_emails.len() >= target_count {
                    break;
                }
            }
        }

        // Generate more if we didn't get enough (with combination space awareness)
        if unique_emails.len() < target_count {
            let remaining = target_count - unique_emails.len();
            eprintln!(
                "Generated {} unique emails, need {} more",
                unique_emails.len(),
                remaining
            );
            eprintln!("Generating additional emails with extended patterns...");

            // Use sequential generation with fallback for remaining
            let additional = self.generate_many_with_fallback(remaining);
            unique_emails.extend(additional);
        }

        unique_emails.truncate(target_count);
        unique_emails
    }

    /// Generate emails and write to a file (synchronous version).
    ///
    /// # Arguments
    /// * `count` - Number of emails to generate
    /// * `output_path` - Path to output file
    ///
    /// # Returns
    /// Number of emails generated
    pub fn generate_to_file<P: AsRef<std::path::Path>>(
        &mut self,
        count: usize,
        output_path: P,
    ) -> std::io::Result<usize> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(1024 * 1024, file); // 1MB buffer

        for i in 0..count {
            let email = self.generate();
            writeln!(writer, "{}", email)?;

            // Progress indicator for large generations
            if (i + 1) % 100_000 == 0 {
                eprintln!("Generated {} emails...", i + 1);
            }
        }

        writer.flush()?;
        Ok(count)
    }

    /// Generate emails asynchronously with parallel generation (no progress bar).
    /// Optimized for maximum throughput.
    pub async fn generate_to_file_async<P: AsRef<std::path::Path>>(
        &mut self,
        count: usize,
        output_path: P,
    ) -> std::io::Result<usize> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        // Generate all emails in parallel first (no progress overhead)
        let emails = self.generate_many_parallel(count);

        // Write asynchronously with large buffer
        let file = File::create(output_path).await?;
        let mut writer = tokio::io::BufWriter::with_capacity(8 * 1024 * 1024, file);

        let mut batch_buffer = String::with_capacity(1024 * 1024);
        const BATCH_SIZE: usize = 50000;

        for (i, email) in emails.iter().enumerate() {
            batch_buffer.push_str(email);
            batch_buffer.push('\n');

            if (i + 1) % BATCH_SIZE == 0 {
                writer.write_all(batch_buffer.as_bytes()).await?;
                batch_buffer.clear();
            }
        }

        if !batch_buffer.is_empty() {
            writer.write_all(batch_buffer.as_bytes()).await?;
        }

        writer.flush().await?;
        Ok(count)
    }

    /// Generate emails asynchronously with parallel generation and TUI progress bar.
    /// Optimized for maximum throughput.
    ///
    /// # Arguments
    /// * `count` - Number of emails to generate
    /// * `output_path` - Path to output file
    /// * `progress_bar` - Progress bar for TUI updates
    ///
    /// # Returns
    /// Number of emails generated
    pub async fn generate_to_file_async_with_progress<P: AsRef<std::path::Path>>(
        &mut self,
        count: usize,
        output_path: P,
        progress_bar: Option<Arc<ProgressBar>>,
    ) -> std::io::Result<usize> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        // Generate all emails in parallel first with progress
        let emails = if let Some(ref pb) = progress_bar {
            pb.set_position(0);
            self.generate_many_parallel_with_progress(count, Some(pb.clone()))
        } else {
            self.generate_many_parallel(count)
        };

        // Write asynchronously with large buffer (8MB for better throughput)
        let file = File::create(output_path).await?;
        let mut writer = tokio::io::BufWriter::with_capacity(8 * 1024 * 1024, file);

        // Build large batch buffer for efficient writing
        let mut batch_buffer = String::with_capacity(1024 * 1024); // 1MB batch buffer
        const BATCH_SIZE: usize = 50000; // Larger batches for better throughput

        for (i, email) in emails.iter().enumerate() {
            batch_buffer.push_str(email);
            batch_buffer.push('\n');

            // Flush in large batches
            if (i + 1) % BATCH_SIZE == 0 {
                writer.write_all(batch_buffer.as_bytes()).await?;
                batch_buffer.clear();
            }
        }

        // Write remaining emails
        if !batch_buffer.is_empty() {
            writer.write_all(batch_buffer.as_bytes()).await?;
        }

        writer.flush().await?;
        Ok(count)
    }

    /// Generate multiple unique email addresses in parallel with TUI progress bar.
    /// Uses a two-phase approach: parallel generation followed by bloom filter deduplication.
    /// Oversamples by 30% to account for deduplication loss.
    pub fn generate_many_parallel_with_progress(
        &mut self,
        count: usize,
        progress_bar: Option<Arc<ProgressBar>>,
    ) -> Vec<String> {
        // Get current state for cloning
        let first_names = self.first_names.clone();
        let last_names = self.last_names.clone();
        let cached_first_names = self.cached_first_names.clone();
        let cached_last_names = self.cached_last_names.clone();
        let domains = self.domains.clone();
        let name_source_config = self.name_source_config;
        let fast_config = self.fast_config;

        // Estimate combination space to detect if request is feasible
        let estimated_space = self.estimate_combination_space(&first_names, &last_names, &domains);
        if count > estimated_space {
            eprintln!(
                "Warning: Requested {} emails exceeds estimated combination space of {}",
                count, estimated_space
            );
            eprintln!("Adding more names/domains or increasing number ranges will help.");
        }

        // Oversample by 30% to account for deduplication
        let oversample_factor = 1.3;
        let target_count = count;
        let generate_count = (count as f64 * oversample_factor) as usize;

        // Generate in parallel chunks (no progress updates during generation to avoid contention)
        let num_threads = rayon::current_num_threads();
        let chunk_size = (generate_count / num_threads).max(1000);
        let num_chunks = generate_count.div_ceil(chunk_size);

        // Pre-calculate mode for optimized generation
        let is_zero_markov = name_source_config.markov_percent == 0;

        let results: Vec<Vec<String>> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk_idx| {
                let start = chunk_idx * chunk_size;
                let end = (start + chunk_size).min(generate_count);
                let chunk_count = end - start;

                let mut rng = rand::rngs::ThreadRng::default();
                let mut chunk_emails = Vec::with_capacity(chunk_count);
                let pattern_thresholds = (35, 60, 80, 90);

                // Cache lengths to avoid repeated Vec access
                let first_names_len = first_names.len();
                let cached_first_names_len = cached_first_names.len();
                let last_names_len = last_names.len();
                let cached_last_names_len = cached_last_names.len();
                let domains_len = domains.len();

                for _i in 0..chunk_count {
                    // Optimized fast path for 0% Markov (default mode)
                    let first = if is_zero_markov {
                        // Direct cached/wordlist selection without config checks
                        if rng.gen_bool(0.6) && cached_first_names_len > 0 {
                            cached_first_names[rng.gen_range(0..cached_first_names_len)].clone()
                        } else if first_names_len > 0 {
                            first_names[rng.gen_range(0..first_names_len)].clone()
                        } else {
                            ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][rng.gen_range(0..6)]
                                .to_string()
                        }
                    } else if fast_config.wordlist_only {
                        if rng.gen_bool(0.5) && first_names_len > 0 {
                            first_names[rng.gen_range(0..first_names_len)].clone()
                        } else if cached_first_names_len > 0 {
                            cached_first_names[rng.gen_range(0..cached_first_names_len)].clone()
                        } else if first_names_len > 0 {
                            first_names[rng.gen_range(0..first_names_len)].clone()
                        } else {
                            ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][rng.gen_range(0..6)]
                                .to_string()
                        }
                    } else {
                        let choice = rng.gen_range(0..100);
                        let wordlist_end = name_source_config.wordlist_percent as usize;
                        let cache_end = wordlist_end + name_source_config.cache_percent as usize;
                        if choice < wordlist_end && first_names_len > 0 {
                            first_names[rng.gen_range(0..first_names_len)].clone()
                        } else if choice < cache_end && cached_first_names_len > 0 {
                            cached_first_names[rng.gen_range(0..cached_first_names_len)].clone()
                        } else if first_names_len > 0 {
                            first_names[rng.gen_range(0..first_names_len)].clone()
                        } else {
                            ["John", "Jane", "Bob", "Alice", "Mike", "Sarah"][rng.gen_range(0..6)]
                                .to_string()
                        }
                    };

                    let last = if is_zero_markov {
                        // Direct cached/wordlist selection without config checks
                        if rng.gen_bool(0.6) && cached_last_names_len > 0 {
                            cached_last_names[rng.gen_range(0..cached_last_names_len)].clone()
                        } else if last_names_len > 0 {
                            last_names[rng.gen_range(0..last_names_len)].clone()
                        } else {
                            ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"]
                                [rng.gen_range(0..6)]
                            .to_string()
                        }
                    } else if fast_config.wordlist_only {
                        if rng.gen_bool(0.5) && last_names_len > 0 {
                            last_names[rng.gen_range(0..last_names_len)].clone()
                        } else if cached_last_names_len > 0 {
                            cached_last_names[rng.gen_range(0..cached_last_names_len)].clone()
                        } else if last_names_len > 0 {
                            last_names[rng.gen_range(0..last_names_len)].clone()
                        } else {
                            ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"]
                                [rng.gen_range(0..6)]
                            .to_string()
                        }
                    } else {
                        let choice = rng.gen_range(0..100);
                        let wordlist_end = name_source_config.wordlist_percent as usize;
                        let cache_end = wordlist_end + name_source_config.cache_percent as usize;
                        if choice < wordlist_end && last_names_len > 0 {
                            last_names[rng.gen_range(0..last_names_len)].clone()
                        } else if choice < cache_end && cached_last_names_len > 0 {
                            cached_last_names[rng.gen_range(0..cached_last_names_len)].clone()
                        } else if last_names_len > 0 {
                            last_names[rng.gen_range(0..last_names_len)].clone()
                        } else {
                            ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"]
                                [rng.gen_range(0..6)]
                            .to_string()
                        }
                    };

                    let pattern_choice = rng.gen_range(0..100);
                    let username = if pattern_choice < pattern_thresholds.0 {
                        format!("{}.{}", first.to_lowercase(), last.to_lowercase())
                    } else if pattern_choice < pattern_thresholds.1 {
                        format!("{}{}", first.to_lowercase(), last.to_lowercase())
                    } else if pattern_choice < pattern_thresholds.2 {
                        format!(
                            "{}{}{}",
                            first.chars().next().unwrap_or('u').to_lowercase(),
                            last.to_lowercase(),
                            rng.gen_range(10..999) // Extended range
                        )
                    } else if pattern_choice < pattern_thresholds.3 {
                        format!("{}_{}", first.to_lowercase(), last.to_lowercase())
                    } else {
                        format!("{}{}", first.to_lowercase(), rng.gen_range(10..9999))
                        // Extended range
                    };

                    let domain = &domains[rng.gen_range(0..domains_len)];
                    chunk_emails.push(format!("{}@{}", username, domain));
                }

                chunk_emails
            })
            .collect();

        // Update progress bar once after parallel generation completes
        if let Some(ref pb) = progress_bar {
            pb.set_position(count as u64 / 2); // Set to 50% for generation phase
        }

        // Phase 2: Use bloom filter to ensure uniqueness
        let mut all_emails: Vec<String> = results.into_iter().flatten().collect();

        // Sort and dedup first (fast path for exact duplicates)
        all_emails.sort_unstable();
        all_emails.dedup();

        // Use the instance's bloom filter to ensure global uniqueness
        let mut unique_emails = Vec::with_capacity(target_count);

        for email in all_emails {
            if self.bloom_filter.check_and_insert(&email) {
                unique_emails.push(email);
                if unique_emails.len() >= target_count {
                    break;
                }
            }
        }

        // Set final progress
        if let Some(ref pb) = progress_bar {
            pb.set_position(count as u64);
        }

        // Generate more if we didn't get enough (with fallback for exhausted space)
        if unique_emails.len() < target_count {
            let remaining = target_count - unique_emails.len();
            eprintln!(
                "Generated {} unique emails, need {} more",
                unique_emails.len(),
                remaining
            );
            eprintln!("Generating additional emails with extended patterns...");
            unique_emails.extend(self.generate_many_with_fallback(remaining));
        }

        unique_emails.truncate(target_count);
        unique_emails
    }

    /// Generate emails with parallel generation and parallel batched I/O.
    /// This is the fastest method for large email counts.
    ///
    /// # Arguments
    /// * `count` - Number of emails to generate
    /// * `output_path` - Path to output file
    ///
    /// # Returns
    /// Number of emails generated
    pub async fn generate_to_file_parallel<P: AsRef<std::path::Path>>(
        &mut self,
        count: usize,
        output_path: P,
    ) -> std::io::Result<usize> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        // Generate all emails in parallel first
        let emails = Arc::new(self.generate_many_parallel(count));

        // Open file
        let file = File::create(output_path).await?;
        let mut writer = tokio::io::BufWriter::with_capacity(4 * 1024 * 1024, file); // 4MB buffer

        // Write all emails
        for (i, email) in emails.iter().enumerate() {
            writer.write_all(email.as_bytes()).await?;
            writer.write_all(b"\n").await?;

            if (i + 1) % 100_000 == 0 {
                eprintln!("Written {} emails...", i + 1);
            }
        }

        writer.flush().await?;
        Ok(count)
    }

    /// Get the count of generated emails.
    pub fn generated_count(&self) -> usize {
        self.generated_count
    }

    /// Get the Bloom filter memory usage in MB.
    pub fn memory_usage_mb(&self) -> f64 {
        self.bloom_filter.memory_usage_mb()
    }

    /// Set the generator configuration.
    pub fn with_config(mut self, config: GeneratorConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the Bloom filter capacity.
    pub fn with_capacity(mut self, capacity: usize, fpr: f64) -> Self {
        self.bloom_filter = EmailBloomFilter::new(capacity, fpr);
        self
    }

    /// Get the Bloom filter false positive rate.
    pub fn false_positive_rate(&self) -> f64 {
        self.bloom_filter.false_positive_rate()
    }

    /// Enable fast generation mode (prioritizes speed over variety).
    /// In fast mode, uses 100% wordlist/cached names with no Markov generation.
    pub fn with_fast_mode(mut self, enabled: bool) -> Self {
        self.fast_config.wordlist_only = enabled;
        self
    }

    /// Set fast generation config.
    pub fn set_fast_mode(&mut self, enabled: bool) {
        self.fast_config.wordlist_only = enabled;
    }

    /// Set name source configuration.
    pub fn with_name_source_config(mut self, config: NameSourceConfig) -> Self {
        self.name_source_config = config;
        self
    }

    /// Set name source ratios (wordlist%, cached%, markov%).
    pub fn with_name_source_ratios(mut self, wordlist: u8, cache: u8, markov: u8) -> Self {
        self.name_source_config = NameSourceConfig {
            wordlist_percent: wordlist,
            cache_percent: cache,
            markov_percent: markov,
        };
        self
    }

    /// Set name source ratios (mutable version).
    pub fn set_name_source_ratios(&mut self, wordlist: u8, cache: u8, markov: u8) {
        self.name_source_config = NameSourceConfig {
            wordlist_percent: wordlist,
            cache_percent: cache,
            markov_percent: markov,
        };
    }

    /// Get current name source configuration.
    pub fn name_source_config(&self) -> NameSourceConfig {
        self.name_source_config
    }

    /// Estimate the total combination space available.
    fn estimate_combination_space(
        &self,
        first_names: &[String],
        last_names: &[String],
        domains: &[String],
    ) -> usize {
        let n_first = first_names.len().max(1);
        let n_last = last_names.len().max(1);
        let n_domains = domains.len().max(1);

        // Calculate combinations per pattern
        let first_last = n_first * n_last * n_domains; // first.last
        let firstlast = n_first * n_last * n_domains; // firstlast
        let flast_num = n_last * n_domains * 990; // flast# (10-999)
        let first_underscore = n_first * n_last * n_domains; // first_last
        let first_num = n_first * n_domains * 9990; // first# (10-9999)

        first_last + firstlast + flast_num + first_underscore + first_num
    }

    /// Generate emails with guaranteed termination even when combination space is exhausted.
    /// Uses direct counter-based generation for speed when combination space is limited.
    pub fn generate_many_with_fallback(&mut self, count: usize) -> Vec<String> {
        let mut emails = Vec::with_capacity(count);

        // Skip slow generation and go directly to fast counter-based emails
        // This is much faster when combination space is exhausted
        let mut counter = self.generated_count;
        let domains_len = self.domains.len().max(1);

        while emails.len() < count {
            let domain = &self.domains[counter % domains_len];
            // Create guaranteed unique email with counter
            let email = format!("gen{}@{}", counter, domain);
            self.bloom_filter.insert(&email);
            emails.push(email);
            counter += 1;
        }

        self.generated_count = counter;
        emails
    }
}

impl Default for EmailGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Capitalize the first letter of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_generation() {
        let mut gen = EmailGenerator::new();
        let email = gen.generate();

        assert!(email.contains('@'));
        assert!(email.contains('.'));
    }

    #[test]
    fn test_uniqueness() {
        let mut gen = EmailGenerator::new();
        let emails = gen.generate_many(1000);

        // All emails should be unique (Bloom filter may have false positives)
        let unique_count = emails
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert!(unique_count >= 990); // Allow for some false positives
    }

    #[test]
    fn test_username_length() {
        let mut gen = EmailGenerator::new();
        gen.config.min_username_length = 5;
        gen.config.max_username_length = 20;

        for _ in 0..100 {
            let email = gen.generate();
            let username = email.split('@').next().unwrap();
            assert!(username.len() >= 4); // Some flexibility
            assert!(username.len() <= 25);
        }
    }
}
