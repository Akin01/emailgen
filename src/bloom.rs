//! Bloom filter wrapper for efficient email uniqueness checking.
//!
//! This module provides a Bloom filter implementation for tracking
//! generated emails with minimal memory usage.

use bloomfilter::Bloom;

/// A Bloom filter wrapper for checking email uniqueness.
///
/// Bloom filters are space-efficient probabilistic data structures
/// that can tell you definitively if an element has NOT been seen,
/// or probably if an element HAS been seen.
pub struct EmailBloomFilter {
    bloom: Bloom<String>,
    estimated_items: usize,
    false_positive_rate: f64,
}

impl EmailBloomFilter {
    /// Create a new Bloom filter for email uniqueness checking.
    ///
    /// # Arguments
    /// * `estimated_items` - Expected number of items to store
    /// * `false_positive_rate` - Acceptable false positive rate (0.0 to 1.0)
    ///
    /// # Returns
    /// A new EmailBloomFilter instance
    ///
    /// # Example
    /// ```
    /// use emailgen::bloom::EmailBloomFilter;
    ///
    /// // Create filter for 1 million emails with 1% false positive rate
    /// let filter = EmailBloomFilter::new(1_000_000, 0.01);
    /// ```
    pub fn new(estimated_items: usize, false_positive_rate: f64) -> Self {
        // Calculate optimal bloom filter size
        // m = -(n * ln(p)) / (ln(2)^2)
        let ln2_sq = 2.0_f64.ln().powi(2);
        let bitmap_size =
            ((-(estimated_items as f64) * false_positive_rate.ln()) / ln2_sq).ceil() as usize;

        let bloom = Bloom::new(bitmap_size, estimated_items);
        Self {
            bloom,
            estimated_items,
            false_positive_rate,
        }
    }

    /// Check if an email might already exist in the filter.
    ///
    /// # Arguments
    /// * `email` - Email address to check
    ///
    /// # Returns
    /// `true` if the email might exist (possible false positive),
    /// `false` if the email definitely doesn't exist
    ///
    /// # Example
    /// ```
    /// use emailgen::bloom::EmailBloomFilter;
    ///
    /// let mut filter = EmailBloomFilter::new(1000, 0.01);
    /// assert!(!filter.contains("test@example.com"));
    /// filter.insert("test@example.com");
    /// assert!(filter.contains("test@example.com"));
    /// ```
    pub fn contains(&self, email: &str) -> bool {
        self.bloom.check(&email.to_string())
    }

    /// Insert an email into the filter.
    ///
    /// # Arguments
    /// * `email` - Email address to insert
    ///
    /// # Returns
    /// `true` if the email was already in the filter (possible false positive),
    /// `false` if the email was newly inserted
    pub fn insert(&mut self, email: &str) -> bool {
        let email_string = email.to_string();
        let already_exists = self.bloom.check(&email_string);
        self.bloom.set(&email_string);
        already_exists
    }

    /// Check and insert an email atomically.
    ///
    /// # Arguments
    /// * `email` - Email address to check and insert
    ///
    /// # Returns
    /// `true` if the email was newly inserted (unique),
    /// `false` if the email might already exist
    pub fn check_and_insert(&mut self, email: &str) -> bool {
        if self.bloom.check(&email.to_string()) {
            false
        } else {
            self.bloom.set(&email.to_string());
            true
        }
    }

    /// Get the estimated number of items this filter can hold.
    pub fn estimated_capacity(&self) -> usize {
        self.estimated_items
    }

    /// Get the false positive rate.
    pub fn false_positive_rate(&self) -> f64 {
        self.false_positive_rate
    }

    /// Get the approximate memory usage in bytes.
    pub fn memory_usage_bytes(&self) -> usize {
        // Bloom filter uses approximately n * log2(1/fpr) / ln(2) bits
        // where n is estimated items and fpr is false positive rate
        let bits = (self.estimated_items as f64 * (-self.false_positive_rate.ln())
            / (2.0_f64.ln().powi(2)))
        .ceil() as usize;
        bits / 8
    }

    /// Get the approximate memory usage in megabytes.
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes() as f64 / (1024.0 * 1024.0)
    }

    /// Clear the filter (reset to empty state).
    pub fn clear(&mut self) {
        let ln2_sq = 2.0_f64.ln().powi(2);
        let bitmap_size = ((-(self.estimated_items as f64) * self.false_positive_rate.ln())
            / ln2_sq)
            .ceil() as usize;
        self.bloom = Bloom::new(bitmap_size, self.estimated_items);
    }
}

impl Default for EmailBloomFilter {
    /// Create a default Bloom filter for 100,000 items with 1% false positive rate.
    fn default() -> Self {
        Self::new(100_000, 0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut filter = EmailBloomFilter::new(1000, 0.01);

        // Initially should not contain anything
        assert!(!filter.contains("test@example.com"));

        // Insert and check
        assert!(filter.check_and_insert("test@example.com"));
        assert!(filter.contains("test@example.com"));

        // Second insert should return false
        assert!(!filter.check_and_insert("test@example.com"));
    }

    #[test]
    fn test_memory_estimation() {
        let filter = EmailBloomFilter::new(1_000_000, 0.01);
        let memory_mb = filter.memory_usage_mb();

        // Should be roughly 9-12 MB for 1M items at 1% FPR
        // Formula: m = -(n * ln(p)) / (ln(2)^2) bits
        assert!(memory_mb > 1.0 && memory_mb < 20.0);
    }

    #[test]
    fn test_clear() {
        let mut filter = EmailBloomFilter::new(1000, 0.01);
        filter.insert("test@example.com");
        assert!(filter.contains("test@example.com"));

        filter.clear();
        assert!(!filter.contains("test@example.com"));
    }
}
