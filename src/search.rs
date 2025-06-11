// src/search.rs
use std::collections::{HashSet, HashMap};
use regex::Regex;

// Keyword search functions
pub fn search_commands_by_keyword(commands: &[String], pattern: &str, case_sensitive: bool) -> Vec<String> {
    let mut results = Vec::new();
    let regex_pattern = if case_sensitive {
        Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap())
    } else {
        Regex::new(&format!("(?i){}", pattern)).unwrap_or_else(|_| Regex::new("").unwrap())
    };

    for cmd in commands {
        if regex_pattern.is_match(cmd) {
            results.push(cmd.clone());
        }
    }

    results
}

pub fn search_words_by_keyword(words: &[String], pattern: &str, case_sensitive: bool) -> HashSet<String> {
    let mut results = HashSet::new();
    let regex_pattern = if case_sensitive {
        Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap())
    } else {
        Regex::new(&format!("(?i){}", pattern)).unwrap_or_else(|_| Regex::new("").unwrap())
    };

    for word in words {
        if regex_pattern.is_match(word) {
            results.insert(word.clone());
        }
    }

    results
}

// Category search function
pub fn search_by_category(
    commands: &[String],
    category_pattern: &str,
    case_sensitive: bool,
    category_counts: &HashMap<String, usize>
) -> (Vec<String>, Vec<String>) {
    let mut matching_commands = Vec::new();
    let mut matching_categories = Vec::new();
    let regex_pattern = if case_sensitive {
        Regex::new(category_pattern).unwrap_or_else(|_| Regex::new("").unwrap())
    } else {
        Regex::new(&format!("(?i){}", category_pattern)).unwrap_or_else(|_| Regex::new("").unwrap())
    };

    // Find matching categories
    for category in category_counts.keys() {
        if regex_pattern.is_match(category) {
            matching_categories.push(category.clone());
        }
    }

    // Find commands that belong to matching categories
    for cmd in commands {
        let categories = super::categorize_command(cmd);
        for category in categories {
            if regex_pattern.is_match(&category) {
                matching_commands.push(cmd.clone());
                break;
            }
        }
    }

    (matching_commands, matching_categories)
}

// Print functions
pub fn print_keyword_search_results(commands: &[String], words: &HashSet<String>) {
    println!("\n\x1b[1;34m=== KEYWORD SEARCH RESULTS ===\x1b[0m");
    
    if !commands.is_empty() {
        println!("\n\x1b[1mMatching Commands:\x1b[0m");
        for (i, cmd) in commands.iter().enumerate() {
            println!("{}. {}", i + 1, cmd);
        }
    }
    
    if !words.is_empty() {
        println!("\n\x1b[1mMatching Keywords:\x1b[0m");
        for (i, word) in words.iter().enumerate() {
            println!("{}. {}", i + 1, word);
        }
    }
    
    if commands.is_empty() && words.is_empty() {
        println!("\nNo matches found.");
    }
}

pub fn print_category_search_results(commands: &[String], categories: &[String]) {
    println!("\n\x1b[1;34m=== CATEGORY SEARCH RESULTS ===\x1b[0m");
    
    if !categories.is_empty() {
        println!("\n\x1b[1mMatching Categories:\x1b[0m");
        for (i, category) in categories.iter().enumerate() {
            println!("{}. {}", i + 1, category);
        }
    }
    
    if !commands.is_empty() {
        println!("\n\x1b[1mMatching Commands:\x1b[0m");
        for (i, cmd) in commands.iter().enumerate() {
            println!("{}. {}", i + 1, cmd);
        }
    }
    
    if commands.is_empty() && categories.is_empty() {
        println!("\nNo matches found.");
    }
}
