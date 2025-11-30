//! Enhanced Regex Implementation
//!
//! This module provides a lightweight enhancement over the standard `regex` crate
//! to support common lookahead and lookbehind patterns without requiring the full
//! `fancy-regex` dependency.
//!
//! The implementation handles ~80% of lookaround use cases found in rgrc config files
//! with minimal code (~200-300 lines) and negligible performance overhead.

use regex::Regex;
use std::fmt;

/// Represents a lookaround assertion (lookahead or lookbehind)
#[derive(Debug, Clone)]
pub enum Lookaround {
    /// Positive lookahead: (?=pattern)
    /// Asserts that the position is followed by pattern
    Ahead { pattern: String, regex: Regex },

    /// Positive lookbehind: (?<=pattern)
    /// Asserts that the position is preceded by pattern
    Behind {
        #[allow(dead_code)]
        pattern: String,
        regex: Regex,
    },

    /// Negative lookahead: (?!pattern)
    /// Asserts that the position is NOT followed by pattern
    /// check_at_start: if true, check at match_start instead of match_end
    NegAhead {
        #[allow(dead_code)]
        pattern: String,
        regex: Regex,
        check_at_start: bool,
    },

    /// Negative lookbehind: (?<!pattern)
    /// Asserts that the position is NOT preceded by pattern
    NegBehind {
        #[allow(dead_code)]
        pattern: String,
        regex: Regex,
    },
}

impl Lookaround {
    /// Create a new lookahead assertion
    pub fn ahead(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Lookaround::Ahead {
            pattern: pattern.to_string(),
            regex: Regex::new(pattern)?,
        })
    }

    /// Create a new lookbehind assertion
    pub fn behind(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Lookaround::Behind {
            pattern: pattern.to_string(),
            regex: Regex::new(pattern)?,
        })
    }

    /// Create a new negative lookahead assertion
    pub fn neg_ahead(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Lookaround::NegAhead {
            pattern: pattern.to_string(),
            regex: Regex::new(pattern)?,
            check_at_start: false,
        })
    }

    /// Create a negative lookahead that checks at match start
    pub fn neg_ahead_at_start(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Lookaround::NegAhead {
            pattern: pattern.to_string(),
            regex: Regex::new(pattern)?,
            check_at_start: true,
        })
    }

    /// Create a new negative lookbehind assertion
    pub fn neg_behind(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Lookaround::NegBehind {
            pattern: pattern.to_string(),
            regex: Regex::new(pattern)?,
        })
    }

    /// Verify if the lookaround condition is satisfied at the given match position
    pub fn verify(&self, text: &str, match_start: usize, match_end: usize) -> bool {
        match self {
            Lookaround::Ahead { regex, pattern } => {
                // Fast path for common patterns - avoid regex compilation overhead
                match pattern.as_str() {
                    // Pattern: \s|$ or $|\s (whitespace or end)
                    r"\s|$" | r"$|\s" => {
                        if match_end >= text.len() {
                            return true;
                        }
                        let ch = text.as_bytes()[match_end];
                        return ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r';
                    }
                    // Pattern: \s (just whitespace)
                    r"\s" => {
                        if match_end >= text.len() {
                            return false;
                        }
                        let ch = text.as_bytes()[match_end];
                        return ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r';
                    }
                    // Pattern: $ (just end of line/string)
                    "$" => {
                        return match_end >= text.len();
                    }
                    // Pattern: \s[A-Z] (whitespace followed by uppercase letter)
                    r"\s[A-Z]" => {
                        if match_end + 1 >= text.len() {
                            return false;
                        }
                        let bytes = text.as_bytes();
                        let ch1 = bytes[match_end];
                        let ch2 = bytes[match_end + 1];
                        return (ch1 == b' ' || ch1 == b'\t' || ch1 == b'\n' || ch1 == b'\r')
                            && ch2.is_ascii_uppercase();
                    }
                    // Pattern: \s[A-Z][a-z]{2}\s (e.g., " Nov ")
                    r"\s[A-Z][a-z]{2}\s" => {
                        if match_end + 4 >= text.len() {
                            return false;
                        }
                        let bytes = text.as_bytes();
                        let ch0 = bytes[match_end];
                        let ch1 = bytes[match_end + 1];
                        let ch2 = bytes[match_end + 2];
                        let ch3 = bytes[match_end + 3];
                        let ch4 = bytes[match_end + 4];
                        return (ch0 == b' ' || ch0 == b'\t')
                            && ch1.is_ascii_uppercase()
                            && ch2.is_ascii_lowercase()
                            && ch3.is_ascii_lowercase()
                            && (ch4 == b' ' || ch4 == b'\t');
                    }
                    // Pattern: [:/] (colon or slash)
                    "[:/]" => {
                        if match_end >= text.len() {
                            return false;
                        }
                        let ch = text.as_bytes()[match_end];
                        return ch == b':' || ch == b'/';
                    }
                    // Pattern: \.\d+\.\d+\.\d+ (IPv4 address continuation)
                    r"\.\d+\.\d+\.\d+" => {
                        // Minimum: ".1.1.1" = 6 chars, but could be longer like ".168.1.1" = 8 chars
                        if match_end + 6 > text.len() {
                            return false;
                        }
                        let bytes = &text.as_bytes()[match_end..];
                        // Quick check: must start with '.'
                        if bytes.is_empty() || bytes[0] != b'.' {
                            return false;
                        }
                        // Use regex for full validation (complex pattern)
                        let remaining = &text[match_end..];
                        if let Some(mat) = regex.find(remaining) {
                            return mat.start() == 0;
                        }
                        return false;
                    }
                    // Pattern: [KMG]B? (size unit like KB, M, GB)
                    r"[KMG]B?" => {
                        if match_end >= text.len() {
                            return false;
                        }
                        let bytes = text.as_bytes();
                        let ch1 = bytes[match_end];
                        if ch1 == b'K' || ch1 == b'M' || ch1 == b'G' {
                            // Check if followed by optional 'B'
                            if match_end + 1 < bytes.len() && bytes[match_end + 1] == b'B' {
                                return true;
                            }
                            // Or just the unit letter
                            return true;
                        }
                        return false;
                    }
                    // Pattern: [KMGT] (size unit without B)
                    "[KMGT]" => {
                        if match_end >= text.len() {
                            return false;
                        }
                        let ch = text.as_bytes()[match_end];
                        return ch == b'K' || ch == b'M' || ch == b'G' || ch == b'T';
                    }
                    _ => {
                        // Fall through to regex matching
                    }
                }

                // Check if the pattern matches at the position after match_end
                // For lookahead, we need to check if there's a match starting at match_end
                let remaining = &text[match_end..];
                if let Some(mat) = regex.find(remaining) {
                    // The match must start at position 0 (right after our match)
                    mat.start() == 0
                } else {
                    false
                }
            }
            Lookaround::Behind { regex, .. } => {
                // Check if text before match_start matches the pattern
                if match_start == 0 {
                    return regex.is_match("");
                }
                // For lookbehind, we need to check if the pattern matches at the end
                // of the prefix. We use find to get the rightmost match.
                if let Some(last_match) = regex.find_iter(&text[..match_start]).last() {
                    // The match must end exactly at match_start
                    last_match.end() == match_start
                } else {
                    false
                }
            }
            Lookaround::NegAhead {
                regex,
                check_at_start,
                ..
            } => {
                // Opposite of positive lookahead
                // For patterns like ^(?:(?!...)), check at match_start instead of match_end
                let check_pos = if *check_at_start {
                    match_start
                } else {
                    match_end
                };
                let remaining = &text[check_pos..];
                if let Some(mat) = regex.find(remaining) {
                    // The match should NOT start at position 0
                    mat.start() != 0
                } else {
                    true // No match means negative lookahead succeeds
                }
            }
            Lookaround::NegBehind { regex, .. } => {
                // Opposite of positive lookbehind
                if match_start == 0 {
                    return !regex.is_match("");
                }
                if let Some(last_match) = regex.find_iter(&text[..match_start]).last() {
                    last_match.end() != match_start
                } else {
                    true // No match means negative lookbehind succeeds
                }
            }
        }
    }
}

/// An enhanced regex that supports basic lookaround assertions
///
/// This struct wraps a standard `regex::Regex` and adds support for common
/// lookahead and lookbehind patterns by post-processing matches.
#[derive(Clone)]
pub struct EnhancedRegex {
    /// The main regex pattern (with lookarounds removed)
    main_regex: Regex,
    /// Optional lookaround assertions to verify
    lookarounds: Vec<Lookaround>,
    /// Original pattern for debugging
    original_pattern: String,
}

impl fmt::Debug for EnhancedRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnhancedRegex")
            .field("pattern", &self.original_pattern)
            .field("lookarounds", &self.lookarounds.len())
            .finish()
    }
}

impl EnhancedRegex {
    /// Create a new EnhancedRegex from a pattern string
    ///
    /// This will attempt to parse and extract lookaround assertions,
    /// then compile the main pattern separately.
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let (main_pattern, lookarounds) = parse_pattern(pattern)?;

        Ok(EnhancedRegex {
            main_regex: Regex::new(&main_pattern)?,
            lookarounds,
            original_pattern: pattern.to_string(),
        })
    }

    /// Find the first match in the text, starting from position `start`
    pub fn find_from_pos<'t>(&self, text: &'t str, start: usize) -> Option<regex::Match<'t>> {
        // Fast path: no lookarounds
        if self.lookarounds.is_empty() {
            return self.main_regex.find_at(text, start);
        }

        let mut pos = start;

        while pos < text.len() {
            if let Some(mat) = self.main_regex.find_at(text, pos) {
                let match_start = mat.start();
                let match_end = mat.end();

                // Try this match first
                if self.verify_lookarounds(text, match_start, match_end) {
                    return Some(mat);
                }

                // Backtrack: try shorter matches from the same start position
                // This is needed because greedy quantifiers might match too much
                // Optimize: only backtrack last 5 chars for patterns > 10 chars
                let min_length = 1;
                let backtrack_chars = if match_end - match_start > 10 {
                    5
                } else {
                    match_end - match_start - min_length
                };
                let backtrack_start = match_end.saturating_sub(backtrack_chars);

                if backtrack_start > match_start {
                    for try_end in (match_start + min_length..=backtrack_start).rev() {
                        let substring = &text[match_start..try_end];
                        // Quick check: does substring match pattern at all?
                        if let Some(sub_mat) = self.main_regex.find(substring)
                            && sub_mat.start() == 0
                            && sub_mat.end() == substring.len()
                        {
                            // Valid substring match, verify lookarounds
                            if self.verify_lookarounds(text, match_start, try_end) {
                                // Return the shortened match
                                let restricted_text = &text[..try_end];
                                if let Some(final_mat) =
                                    self.main_regex.find_at(restricted_text, match_start)
                                    && final_mat.start() == match_start
                                    && final_mat.end() == try_end
                                {
                                    return Some(final_mat);
                                }
                            }
                        }
                    }
                }

                // No valid match from this start position, try next
                pos = match_start + 1;
            } else {
                break;
            }
        }
        None
    }

    /// Find all matches in the text
    pub fn find_iter<'t>(&self, text: &'t str) -> EnhancedMatches<'_, 't> {
        EnhancedMatches {
            regex: self,
            text,
            last_pos: 0,
        }
    }

    /// Get captures for the first match, starting from position `start`
    pub fn captures_from_pos<'t>(
        &self,
        text: &'t str,
        start: usize,
    ) -> Option<regex::Captures<'t>> {
        let mut pos = start;

        while pos < text.len() {
            if let Some(caps) = self.main_regex.captures_at(text, pos) {
                let mat = caps.get(0).unwrap();
                // Verify all lookaround conditions
                if self.verify_lookarounds(text, mat.start(), mat.end()) {
                    return Some(caps);
                }
                // Move past this match and continue searching
                pos = mat.start() + 1;
            } else {
                break;
            }
        }
        None
    }

    /// Verify all lookaround conditions for a match
    #[inline]
    fn verify_lookarounds(&self, text: &str, match_start: usize, match_end: usize) -> bool {
        // Fast path: no lookarounds
        if self.lookarounds.is_empty() {
            return true;
        }

        // Check each lookaround
        for lookaround in &self.lookarounds {
            if !lookaround.verify(text, match_start, match_end) {
                return false;
            }
        }
        true
    }

    /// Check if the pattern matches the text
    pub fn is_match(&self, text: &str) -> bool {
        self.find_from_pos(text, 0).is_some()
    }

    /// Get the original pattern string
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.original_pattern
    }
}

/// Iterator over all matches in a text
pub struct EnhancedMatches<'r, 't> {
    regex: &'r EnhancedRegex,
    text: &'t str,
    last_pos: usize,
}

impl<'r, 't> Iterator for EnhancedMatches<'r, 't> {
    type Item = regex::Match<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mat) = self.regex.find_from_pos(self.text, self.last_pos) {
            self.last_pos = mat.end();
            Some(mat)
        } else {
            None
        }
    }
}

/// Extract lookaround with proper bracket matching
fn extract_lookaround_content(pattern: &str, start: usize) -> Option<(usize, String)> {
    let chars: Vec<char> = pattern.chars().collect();
    if start + 2 >= chars.len() {
        return None;
    }

    // Should start with "(?"
    if chars[start] != '(' || chars[start + 1] != '?' {
        return None;
    }

    // Find matching closing paren
    let mut depth = 1;
    let mut i = start + 2;

    while i < chars.len() && depth > 0 {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            '\\' => i += 1, // Skip escaped char
            _ => {}
        }
        i += 1;
    }

    if depth == 0 {
        let inner_start = start + 2; // Skip "(?"
        // Find where the lookaround type ends
        let mut type_end = inner_start;
        while type_end < chars.len() && "=!<".contains(chars[type_end]) {
            type_end += 1;
        }
        let content: String = chars[type_end..i - 1].iter().collect();
        Some((i, content))
    } else {
        None
    }
}

/// Parse a regex pattern and extract lookaround assertions
///
/// Returns: (main_pattern, lookarounds)
fn parse_pattern(pattern: &str) -> Result<(String, Vec<Lookaround>), regex::Error> {
    let mut main_pattern = pattern.to_string();
    let mut lookarounds = Vec::new();

    // Extract lookarounds in order (important for correct behavior)
    // We need to handle multiple lookarounds including nested parentheses

    // Collect all lookarounds first (we'll remove them in reverse order to maintain positions)
    let mut found_lookarounds = Vec::new();

    // Manual parsing to handle nested parentheses
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 2 < chars.len() && chars[i] == '(' && chars[i + 1] == '?' {
            // Check lookaround type
            let lookaround_type = if i + 3 < chars.len() && chars[i + 2] == '<' {
                if i + 4 < chars.len() {
                    if chars[i + 3] == '=' {
                        Some(("<=", 4))
                    } else if chars[i + 3] == '!' {
                        Some(("<!", 4))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if i + 3 < chars.len() {
                if chars[i + 2] == '=' {
                    Some(("=", 3))
                } else if chars[i + 2] == '!' {
                    Some(("!", 3))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((type_str, _type_len)) = lookaround_type
                && let Some((end_pos, inner_pattern)) = extract_lookaround_content(pattern, i)
            {
                let lookaround = match type_str {
                    "=" => Lookaround::ahead(&inner_pattern)?,
                    "!" => {
                        // Special case: ^(?:(?!...)) should check at match start
                        let prefix = &pattern[..i];
                        let is_at_start = prefix.trim_start_matches('^').trim_start() == "(?:";
                        if is_at_start {
                            Lookaround::neg_ahead_at_start(&inner_pattern)?
                        } else {
                            Lookaround::neg_ahead(&inner_pattern)?
                        }
                    }
                    "<=" => Lookaround::behind(&inner_pattern)?,
                    "<!" => Lookaround::neg_behind(&inner_pattern)?,
                    _ => unreachable!(),
                };

                found_lookarounds.push((i, end_pos, lookaround));
                i = end_pos;
                continue;
            }
        }
        i += 1;
    }

    // Remove lookarounds from the pattern (in reverse order to maintain indices)
    found_lookarounds.sort_by_key(|(start, _, _)| *start);
    for (_, _, lookaround) in &found_lookarounds {
        lookarounds.push(lookaround.clone());
    }

    // Remove lookarounds from pattern (reverse order)
    for (start, end, _) in found_lookarounds.iter().rev() {
        main_pattern.replace_range(*start..*end, "");
    }

    // Convert lazy quantifiers to greedy when lookarounds are present
    // This is necessary because we need to try different match lengths
    // to find one that satisfies the lookaround conditions.
    // Lazy quantifiers cause find_at to only return the shortest match.
    if !lookarounds.is_empty() {
        main_pattern = main_pattern
            .replace("+?", "+")
            .replace("*?", "*")
            .replace("??", "?");
    }

    Ok((main_pattern, lookarounds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_lookahead() {
        let re = EnhancedRegex::new(r"\d+(?=\s)").unwrap();
        assert!(re.is_match("123 "));
        assert!(!re.is_match("123"));
        assert!(!re.is_match("123a"));
    }

    #[test]
    fn test_simple_lookbehind() {
        let re = EnhancedRegex::new(r"(?<=\s)\d+").unwrap();
        assert!(re.is_match(" 123"));
        assert!(!re.is_match("123"));
        assert!(!re.is_match("a123"));
    }

    #[test]
    fn test_negative_lookahead() {
        let re = EnhancedRegex::new(r"\d+(?!\s)").unwrap();
        assert!(re.is_match("123"));
        assert!(re.is_match("123a"));
        // "123 " should match "12" because greedy \d+ matches "123" first,
        // lookahead fails (followed by space), then backtracking tries "12"
        // and succeeds (followed by "3", not a space)
        assert!(re.is_match("123 "));

        // To test "no match after digits", use a pattern that matches the full string
        let re2 = EnhancedRegex::new(r"^\d+(?!\s)$").unwrap();
        assert!(!re2.is_match("123 "));
    }

    #[test]
    fn test_negative_lookbehind() {
        let re = EnhancedRegex::new(r"(?<!\s)\d+").unwrap();
        assert!(re.is_match("123"));
        assert!(re.is_match("a123"));
        // Note: " 123" actually DOES match "23" (not preceded by space)
        // Even fancy-regex matches this way
        assert!(re.is_match(" 123"));
        // To truly test negative lookbehind, check the match position
        if let Some(m) = re.find_from_pos(" 123", 0) {
            // Should match "23", not "123"
            assert_eq!(m.as_str(), "23");
            assert_eq!(m.start(), 2);
        }
    }

    #[test]
    fn test_common_pattern_end_boundary() {
        // Pattern: (?=\s|$) - match at space or end of string
        let re = EnhancedRegex::new(r"\d+(?=\s|$)").unwrap();
        assert!(re.is_match("123 "));
        assert!(re.is_match("123"));
        assert!(!re.is_match("123a"));
    }

    #[test]
    fn test_common_pattern_ls_size() {
        // From conf.ls: match file size followed by month abbreviation
        let re = EnhancedRegex::new(r"\d{7}(?=\s[A-Z][a-z]{2}\s)").unwrap();
        assert!(re.is_match("1234567 Mar "));
        assert!(re.is_match("9876543 Nov 30"));
        assert!(!re.is_match("1234567 "));
        assert!(!re.is_match("1234567 123"));
    }

    #[test]
    fn test_captures() {
        let re = EnhancedRegex::new(r"(\d+)(?=\s)").unwrap();
        if let Some(caps) = re.captures_from_pos("abc 123 def", 0) {
            assert_eq!(caps.get(1).unwrap().as_str(), "123");
        } else {
            panic!("Expected to find match");
        }
    }

    #[test]
    fn test_find_iter() {
        let re = EnhancedRegex::new(r"\d+(?=\s)").unwrap();
        let matches: Vec<_> = re.find_iter("123 456 789").map(|m| m.as_str()).collect();
        assert_eq!(matches, vec!["123", "456"]);
    }

    #[test]
    fn test_no_lookaround() {
        // Pattern without lookaround should work normally
        let re = EnhancedRegex::new(r"\d+").unwrap();
        assert!(re.is_match("123"));
        assert!(re.is_match("abc123def"));
    }

    #[test]
    fn test_multiple_lookarounds() {
        // Pattern with both lookbehind and lookahead
        let re = EnhancedRegex::new(r"(?<=\s)\d+(?=\s)").unwrap();
        assert!(re.is_match(" 123 "));
        assert!(!re.is_match("123 "));
        assert!(!re.is_match(" 123"));
        assert!(!re.is_match("123"));
    }
}
