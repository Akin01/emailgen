//! Wordlist and domain list loading utilities.
//!
//! This module provides functionality to load names and domains
//! from text files for use in email generation.

use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;

/// Load names from a text file (one name per line).
///
/// # Arguments
/// * `path` - Path to the text file
///
/// # Returns
/// A vector of names (trimmed, non-empty lines)
///
/// # Example
/// ```no_run
/// use emailgen::wordlist::load_names;
///
/// let names = load_names("data/names.txt").unwrap();
/// ```
pub fn load_names<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let names: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(names)
}

/// Load domains from a text file (one domain per line).
///
/// # Arguments
/// * `path` - Path to the text file
///
/// # Returns
/// A vector of domains (trimmed, non-empty lines)
///
/// # Example
/// ```no_run
/// use emailgen::wordlist::load_domains;
///
/// let domains = load_domains("data/domains.txt").unwrap();
/// ```
pub fn load_domains<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let domains: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && line.contains('.'))
        .collect();

    Ok(domains)
}

/// Load names from multiple files or directories.
///
/// # Arguments
/// * `paths` - Paths to files or directories
///
/// # Returns
/// A combined vector of all names from all files
pub fn load_names_from_multiple<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<String>> {
    let mut all_names = Vec::new();

    for path in paths {
        let path_ref = path.as_ref();

        if path_ref.is_dir() {
            // Load all .txt files from directory
            if let Ok(entries) = std::fs::read_dir(path_ref) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.extension().is_some_and(|ext| ext == "txt") {
                        if let Ok(names) = load_names(&entry_path) {
                            all_names.extend(names);
                        }
                    }
                }
            }
        } else if path_ref.is_file() {
            if let Ok(names) = load_names(path_ref) {
                all_names.extend(names);
            }
        }
    }

    Ok(all_names)
}

/// Built-in fallback names for when no wordlist is provided.
pub const FALLBACK_NAMES: &[&str] = &[
    "James",
    "Mary",
    "John",
    "Patricia",
    "Robert",
    "Jennifer",
    "Michael",
    "Linda",
    "William",
    "Elizabeth",
    "David",
    "Barbara",
    "Richard",
    "Susan",
    "Joseph",
    "Jessica",
    "Thomas",
    "Sarah",
    "Charles",
    "Karen",
    "Christopher",
    "Nancy",
    "Daniel",
    "Lisa",
    "Matthew",
    "Betty",
    "Anthony",
    "Margaret",
    "Donald",
    "Sandra",
    "Mark",
    "Ashley",
    "Paul",
    "Kimberly",
    "Steven",
    "Emily",
    "Andrew",
    "Donna",
    "Kenneth",
    "Michelle",
    "Joshua",
    "Dorothy",
    "Kevin",
    "Carol",
    "Brian",
    "Amanda",
    "George",
    "Melissa",
    "Edward",
    "Deborah",
    "Ronald",
    "Stephanie",
    "Timothy",
    "Rebecca",
    "Jason",
    "Sharon",
    "Jeffrey",
    "Laura",
    "Ryan",
    "Cynthia",
    "Jacob",
    "Kathleen",
    "Gary",
    "Amy",
    "Nicholas",
    "Angela",
    "Eric",
    "Shirley",
    "Jonathan",
    "Anna",
    "Stephen",
    "Brenda",
];

/// Built-in fallback domains for when no domain list is provided.
pub const FALLBACK_DOMAINS: &[&str] = &[
    "gmail.com",
    "yahoo.com",
    "hotmail.com",
    "outlook.com",
    "example.com",
    "test.org",
    "mail.com",
    "proton.me",
    "icloud.com",
    "aol.com",
];

/// Get default names (fallback if no wordlist provided).
pub fn get_default_names() -> Vec<String> {
    FALLBACK_NAMES.iter().map(|s| s.to_string()).collect()
}

/// Get default domains (fallback if no domain list provided).
pub fn get_default_domains() -> Vec<String> {
    FALLBACK_DOMAINS.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_names() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("names.txt");

        fs::write(&file_path, "John\nJane\nBob\n").unwrap();

        let names = load_names(&file_path).unwrap();
        assert_eq!(names, vec!["John", "Jane", "Bob"]);
    }

    #[test]
    fn test_load_domains() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("domains.txt");

        fs::write(&file_path, "example.com\ntest.org\ninvalid\nmail.com\n").unwrap();

        let domains = load_domains(&file_path).unwrap();
        assert_eq!(domains, vec!["example.com", "test.org", "mail.com"]);
    }

    #[test]
    fn test_fallback_data() {
        let names = get_default_names();
        assert!(!names.is_empty());

        let domains = get_default_domains();
        assert!(!domains.is_empty());
    }
}
