//! Common test utilities shared across colorizer tests

use rgrc::colorizer::colorize_regex;
use rgrc::grc::GrcatConfigEntry;
use std::io::Cursor;

/// Helper function to run colorize_regex and return the output as a String
///
/// # Arguments
/// * `input` - The input text to colorize
/// * `rules` - Vector of GrcatConfigEntry rules to apply
///
/// # Returns
/// The colorized output as a UTF-8 string
pub fn run_colorize(input: &str, rules: Vec<GrcatConfigEntry>) -> String {
    let mut output = Vec::new();
    let mut reader = Cursor::new(input.as_bytes());
    colorize_regex(&mut reader, &mut output, &rules).unwrap();
    String::from_utf8(output).unwrap()
}

/// Helper function to strip ANSI escape codes from a string
///
/// Removes all ANSI CSI sequences (like `\x1b[...m`) for clean text comparison
///
/// # Arguments
/// * `s` - The string containing ANSI escape codes
///
/// # Returns
/// The string with all ANSI codes removed
pub fn strip_ansi(s: &str) -> String {
    let re = regex_lite::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

// (helper for creating rules with count removed - use explicit construction in tests)
