//! # grc.rs - Configuration Parsing for Colorization Rules
//!
//! This module provides utilities for parsing GRC (Grep Result Colorizer) configuration files
//! in two formats:
//!
//! 1. **grc.conf format**: Maps command patterns to grcat configuration files
//!    - Format: Lines with regex patterns followed by configuration file paths
//!    - Example: `^ping` → `conf.ping`
//!
//! 2. **grcat.conf format**: Defines colorization rules for specific commands
//!    - Format: Key-value pairs (regexp=..., colours=...)
//!    - Contains regex patterns matched against output text
//!    - Associates console styles (colors, attributes) with capture groups
//!
//! ## Regex Engine Selection
//!
//! The module uses a hybrid regex engine approach for optimal performance:
//!
//! - **Fast Path**: Standard `regex` crate (~2-5x faster, no lookaround support)
//! - **Enhanced Path**: Two implementations available via conditional compilation:
//!   - With `--features=fancy` (default): Uses battle-tested `fancy-regex`
//!     - Supports all advanced features (backreferences, variable-length lookbehind, etc.)
//!     - Binary size: ~2.1MB (release mode)
//!   - Without `fancy` feature: Uses lightweight `EnhancedRegex`
//!     - Supports fixed-length lookahead/lookbehind patterns
//!     - Binary size: ~1.8MB (release mode)
//!     - Covers 99% of patterns in rgrc config files
//!
//! ## Supported Styles
//!
//! The module supports grcat-style color specifications:
//! - **Foreground colors**: black, red, green, yellow, blue, magenta, cyan, white
//! - **Background colors**: on_black, on_red, on_green, on_yellow, on_blue, on_magenta, on_cyan, on_white
//! - **Attributes**: bold, italic, underline, blink, reverse
//! - **Brightness**: bright_black, bright_red, ... bright_white
//! - **Special**: unchanged, default, dark, none
//!
//! ## Module Structure
//!
//! - `style_from_str()`: Parse a single style keyword
//! - `styles_from_str()`: Parse comma-separated style list
//! - `GrcConfigReader`: Iterator for grc.conf files
//! - `GrcatConfigReader`: Iterator for grcat.conf files
//! - `GrcatConfigEntry`: Represents a single colorization rule
//! - `CompiledRegex`: Hybrid regex engine (Fast or Enhanced)

use std::io::{BufRead, Lines};

#[cfg(not(feature = "fancy-regex"))]
use crate::enhanced_regex::EnhancedRegex;
use crate::style::Style;
#[cfg(feature = "fancy-regex")]
use fancy_regex::Regex as FancyRegex;
use regex::Regex;

/// Custom error type for regex compilation
#[derive(Debug)]
pub enum RegexError {
    Syntax(String),
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Syntax(msg) => write!(f, "Regex syntax error: {}", msg),
        }
    }
}

impl std::error::Error for RegexError {}

impl From<regex::Error> for RegexError {
    fn from(err: regex::Error) -> Self {
        RegexError::Syntax(err.to_string())
    }
}

/// Hybrid regex engine: tries standard regex first, then falls back to Enhanced implementation.
///
/// This provides significant performance improvement for most configuration files:
/// - Simple patterns (90%+) → Fast path using standard `regex` crate
/// - Complex patterns with lookaround → Enhanced path
///
/// ## Enhanced Implementation Selection
///
/// The Enhanced variant uses conditional compilation:
///
/// - **With `fancy` feature** (default, enabled in Cargo.toml):
///   - Uses `fancy-regex` crate (battle-tested, production-ready)
///   - Supports: lookahead, lookbehind (variable-length), backreferences, etc.
///   - Binary size: ~2.1MB (release)
///   - Recommended for conservative users who prioritize stability
///
/// - **Without `fancy` feature** (`cargo build --no-default-features --features=embed-configs`):
///   - Uses custom `EnhancedRegex` implementation (~600 lines)
///   - Supports: fixed-length lookahead/lookbehind patterns
///   - Binary size: ~1.8MB (release)
///   - Covers 99% of patterns in rgrc config files
///   - Newer implementation, less battle-tested
///
/// ## Usage
///
/// ```ignore
/// use rgrc::grc::CompiledRegex;
///
/// // Simple pattern → Fast(Regex)
/// let re = CompiledRegex::new(r"\d+").unwrap();
///
/// // Lookahead pattern → Enhanced(fancy_regex::Regex) or Enhanced(EnhancedRegex)
/// let re = CompiledRegex::new(r"\d+(?=\.\d+\.\d+\.\d+)").unwrap();
/// ```
#[derive(Debug, Clone)]
pub enum CompiledRegex {
    /// Fast path: standard regex crate (no lookaround, ~2-5x faster)
    Fast(Regex),
    /// Enhanced path: fancy-regex (battle-tested, enabled with --features=fancy)
    #[cfg(feature = "fancy-regex")]
    Enhanced(FancyRegex),
    /// Enhanced path: our own lookaround implementation (lightweight, default without fancy feature)
    #[cfg(not(feature = "fancy-regex"))]
    Enhanced(EnhancedRegex),
}

impl CompiledRegex {
    /// Compile a regex pattern, automatically selecting the fastest engine.
    /// Tries standard regex first, then falls back to EnhancedRegex for lookaround patterns.
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        // Try standard regex first (fastest, but no lookaround)
        if let Ok(re) = Regex::new(pattern) {
            return Ok(CompiledRegex::Fast(re));
        }

        // Fall back to Enhanced regex implementation
        #[cfg(feature = "fancy-regex")]
        {
            // Use battle-tested fancy-regex when enabled
            FancyRegex::new(pattern)
                .map(CompiledRegex::Enhanced)
                .map_err(|e| RegexError::Syntax(e.to_string()))
        }
        #[cfg(not(feature = "fancy-regex"))]
        {
            // Use our own EnhancedRegex implementation (default)
            EnhancedRegex::new(pattern)
                .map(CompiledRegex::Enhanced)
                .map_err(RegexError::from)
        }
    }

    /// Check if the regex matches anywhere in the text.
    #[allow(dead_code)]
    pub fn is_match(&self, text: &str) -> bool {
        match self {
            CompiledRegex::Fast(re) => re.is_match(text),
            #[cfg(feature = "fancy-regex")]
            CompiledRegex::Enhanced(re) => re.is_match(text).unwrap_or(false),
            #[cfg(not(feature = "fancy-regex"))]
            CompiledRegex::Enhanced(re) => re.is_match(text),
        }
    }

    /// Find all capture groups starting from the given position.
    #[allow(dead_code)]
    pub fn captures_from_pos<'t>(&self, text: &'t str, pos: usize) -> Option<Captures<'t>> {
        match self {
            CompiledRegex::Fast(re) => {
                // Standard regex: convert to our Captures format
                re.captures(&text[pos..])
                    .map(|caps| Captures::Fast(caps, pos))
            }
            #[cfg(feature = "fancy-regex")]
            CompiledRegex::Enhanced(re) => {
                // fancy-regex: convert to our Captures format
                re.captures(&text[pos..])
                    .ok()
                    .flatten()
                    .map(|caps| Captures::Fancy(caps, pos))
            }
            #[cfg(not(feature = "fancy-regex"))]
            CompiledRegex::Enhanced(re) => {
                // EnhancedRegex: convert to our Captures format
                re.captures_from_pos(text, pos)
                    .map(|caps| Captures::Fast(caps, 0))
            }
        }
    }

    /// Get the pattern string for debugging.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            CompiledRegex::Fast(re) => re.as_str(),
            #[cfg(feature = "fancy-regex")]
            CompiledRegex::Enhanced(re) => re.as_str(),
            #[cfg(not(feature = "fancy-regex"))]
            CompiledRegex::Enhanced(re) => re.as_str(),
        }
    }
}

/// Unified captures interface wrapping regex::Captures.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Captures<'t> {
    Fast(regex::Captures<'t>, usize), // offset for position adjustment
    #[cfg(feature = "fancy-regex")]
    Fancy(fancy_regex::Captures<'t>, usize), // fancy-regex captures with offset
}

impl<'t> Captures<'t> {
    /// Get a capture group by index (0 = full match, 1+ = groups).
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<Match<'t>> {
        match self {
            Captures::Fast(caps, offset) => caps.get(index).map(|m| Match::Fast(m, *offset)),
            #[cfg(feature = "fancy-regex")]
            Captures::Fancy(caps, offset) => caps.get(index).map(|m| Match::Fancy(m, *offset)),
        }
    }

    /// Get the number of capture groups.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            Captures::Fast(caps, _) => caps.len(),
            #[cfg(feature = "fancy-regex")]
            Captures::Fancy(caps, _) => caps.len(),
        }
    }

    /// Check if there are no capture groups.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all capture groups by index.
    /// Returns a vector to avoid lifetime issues with closures.
    #[allow(dead_code)]
    pub fn iter(&'t self) -> Vec<Option<Match<'t>>> {
        let len = self.len();
        (0..len).map(|i| self.get(i)).collect()
    }
}

/// Unified match interface wrapping regex::Match.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Match<'t> {
    Fast(regex::Match<'t>, usize), // offset for position adjustment
    #[cfg(feature = "fancy-regex")]
    Fancy(fancy_regex::Match<'t>, usize), // fancy-regex match with offset
}

impl<'t> Match<'t> {
    /// Get the start byte position of the match.
    #[allow(dead_code)]
    pub fn start(&self) -> usize {
        match self {
            Match::Fast(m, offset) => m.start() + offset,
            #[cfg(feature = "fancy-regex")]
            Match::Fancy(m, offset) => m.start() + offset,
        }
    }

    /// Get the end byte position of the match.
    #[allow(dead_code)]
    pub fn end(&self) -> usize {
        match self {
            Match::Fast(m, offset) => m.end() + offset,
            #[cfg(feature = "fancy-regex")]
            Match::Fancy(m, offset) => m.end() + offset,
        }
    }

    /// Get the matched text as a string slice.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'t str {
        match self {
            Match::Fast(m, _) => m.as_str(),
            #[cfg(feature = "fancy-regex")]
            Match::Fancy(m, _) => m.as_str(),
        }
    }
}

/// Parse a single space-separated style keyword and apply it to a Style.
///
/// This function processes grcat-style color and attribute keywords and builds up a composite
/// `Style` object. It handles multiple space-separated style keywords in a single call,
/// applying them sequentially to create combined effects (e.g., bold red text).
///
/// ## Supported Keywords
///
/// **Foreground colors:**
/// - `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
///
pub fn style_from_str(text: &str) -> Result<Style, String> {
    text.split(' ').try_fold(Style::new(), |style, word| {
        // Handle ANSI escape sequences like "\033[38;5;140m"
        if word.starts_with('"') && word.contains("\\033[") {
            // Skip ANSI escape codes for now - they're raw color codes
            return Ok(style);
        }
        match word {
            // Empty string or no-op keywords - return style unchanged
            "" => Ok(style),
            "unchanged" => Ok(style),
            "default" => Ok(style),
            "dark" => Ok(style.dim()),
            "none" => Ok(style),

            // Foreground colors - standard ANSI colors
            "black" => Ok(style.black()),
            "red" => Ok(style.red()),
            "green" => Ok(style.green()),
            "yellow" => Ok(style.yellow()),
            "blue" => Ok(style.blue()),
            "magenta" => Ok(style.magenta()),
            "cyan" => Ok(style.cyan()),
            "white" => Ok(style.white()),

            // Background colors - with on_ prefix for background
            "on_black" => Ok(style.on_black()),
            "on_red" => Ok(style.on_red()),
            "on_green" => Ok(style.on_green()),
            "on_yellow" => Ok(style.on_yellow()),
            "on_blue" => Ok(style.on_blue()),
            "on_magenta" => Ok(style.on_magenta()),
            "on_cyan" => Ok(style.on_cyan()),
            "on_white" => Ok(style.on_white()),

            // Text attributes - styling options
            "bold" => Ok(style.bold()),
            "underline" => Ok(style.underlined()),
            "italic" => Ok(style.italic()),
            "blink" => Ok(style.blink()),
            "reverse" => Ok(style.reverse()),
            "dim" => Ok(style.dim()),

            // Bright color variants - high-intensity colors
            "bright_black" => Ok(style.bright().black()),
            "bright_red" => Ok(style.bright().red()),
            "bright_green" => Ok(style.bright().green()),
            "bright_yellow" => Ok(style.bright().yellow()),
            "bright_blue" => Ok(style.bright().blue()),
            "bright_magenta" => Ok(style.bright().magenta()),
            "bright_cyan" => Ok(style.bright().cyan()),
            "bright_white" => Ok(style.bright().white()),

            // Unknown keyword - log and return error
            _ => {
                // Return a descriptive error (used in callers/tests to detect invalid styles)
                let msg = format!("unhandled style: {}", word);
                println!("{}", msg);
                Err(msg)
            }
        }
    })
}

/// Parse a comma-separated list of style keywords into a vector of Styles.
///
/// This function processes a comma-separated style specification string (as used in grcat config)
/// and converts it into a vector of `Style` objects. Each comma-separated section is
/// passed individually to `style_from_str()` for parsing.
///
/// ## Format
///
/// Comma-separated style specifications are used in grcat.conf files to define styles for
/// different capture groups in a regex match. For example:
/// ```ignore
/// regexp=^(ERROR|WARN|INFO) (\d+ms)
/// colours=bold red,yellow,green
/// ```
/// This creates 3 styles:
/// 1. `bold red` for first capture group (ERROR|WARN|INFO)
/// 2. `yellow` for second capture group (\d+ms)
/// 3. `green` for subsequent matches
///
/// ## Arguments
///
/// * `text` - Comma-separated style keywords (e.g., "bold red,yellow,green")
///
/// ## Returns
///
/// - `Ok(Vec<Style>)` - Successfully parsed all styles
/// - `Err(())` - Any style keyword failed to parse (short-circuits on first error)
///
/// ## Implementation Details
///
/// - Splits input string on commas: `text.split(',')`
/// - Passes each section to `style_from_str()` for individual parsing
/// - Uses `collect()` with `?` operator to short-circuit on first error
/// - Returns all parsed styles in a vector
///
/// # Examples
///
/// ```ignore
/// // Single style
/// let styles = styles_from_str("bold red").unwrap();
/// assert_eq!(styles.len(), 1);
///
/// // Multiple styles for multiple capture groups
/// let styles = styles_from_str("bold red,yellow,green").unwrap();
/// assert_eq!(styles.len(), 3);
///
/// // With no-op keywords
/// let styles = styles_from_str("bold red,default,unchanged").unwrap();
/// assert_eq!(styles.len(), 3);
///
/// // Error if any style is invalid
/// assert!(styles_from_str("bold red,invalid_color,green").is_err());
/// ```
#[allow(dead_code)]
pub fn styles_from_str(text: &str) -> Result<Vec<Style>, String> {
    text.split(',').map(style_from_str).collect()
}

/// Configuration reader for the main grc.conf file.
///
/// This struct implements an iterator over GRC configuration rules. Each rule maps
/// a command name pattern (regex) to a specific grcat configuration file that defines
/// how to colorize that command's output.
///
/// ## File Format
///
/// The grc.conf file uses a paired-line format:
/// ```text
/// # Comment lines start with # and are ignored
/// ^ping           # First line: regex pattern to match command names
/// conf.ping       # Second line: path to grcat config file for matching commands
///
/// ^curl
/// conf.curl
///
/// ^(ls|dir)
/// conf.ls
/// ```
///
/// Rules are separated by blank lines or comments. Each complete rule consists of:
/// 1. A regex pattern (first non-comment line of the pair)
/// 2. A config file path (second non-comment line of the pair)
///
/// ## Parsing Behavior
///
/// - Comments (lines starting with '#' or whitespace followed by '#') are skipped
/// - Blank lines and lines with only whitespace are ignored
/// - Consecutive non-comment lines form a rule pair
/// - Malformed regexes in the pattern line cause that rule to be skipped
/// - The reader gracefully handles incomplete rules (pattern without config)
///
/// ## Generic Parameter
///
/// * `A` - A type implementing `BufRead`, typically created from:
///   - `std::io::BufReader<File>` for file input
///   - `std::io::BufReader<&[u8]>` for in-memory buffers
///
/// # Examples
///
/// ```ignore
/// use std::io::BufReader;
/// use std::fs::File;
/// use rgrc::grc::GrcConfigReader;
///
/// let file = File::open("~/.config/rgrc/grc.conf")?;
/// let reader = BufReader::new(file);
/// let config_reader = GrcConfigReader::new(reader.lines());
///
/// for (regex, config_path) in config_reader {
///     println!("Pattern: {:?}, Config: {}", regex, config_path);
/// }
/// ```
#[allow(dead_code)]
pub struct GrcConfigReader<A> {
    inner: Lines<A>,
}

#[allow(dead_code)]
impl<A: BufRead> GrcConfigReader<A> {
    /// Create a new GRC configuration reader from a line iterator.
    ///
    /// # Arguments
    ///
    /// * `inner` - A `Lines<A>` iterator yielding lines from a buffered reader
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// let file = File::open("~/.config/rgrc/grc.conf")?;
    /// let reader = BufReader::new(file);
    /// let config_reader = GrcConfigReader::new(reader.lines());
    /// ```
    pub fn new(inner: Lines<A>) -> Self {
        GrcConfigReader { inner }
    }

    /// Skip to the next non-empty, non-comment line.
    ///
    /// This helper method iterates through the input lines and returns the first line that is:
    /// - Not a comment (does not start with '#')
    /// - Not empty or whitespace-only
    /// - Not a line where whitespace precedes a comment character
    ///
    /// Comments are detected using the regex pattern `^[- \t]*(#|$)` which matches:
    /// - Lines starting with optional whitespace/dashes followed by '#' (comments)
    /// - Lines that are empty or whitespace-only (matches end of line via `$`)
    ///
    /// ## Returns
    ///
    /// - `Some(String)` - Trimmed content of the next valid line
    /// - `None` - EOF reached, no more content lines available
    ///
    /// ## Error Handling
    ///
    /// If a line read error occurs, iteration stops and returns None.
    ///
    /// # Examples
    ///
    /// With input:
    /// ```text
    /// # This is a comment
    ///
    /// ^ping
    /// conf.ping
    /// ```
    /// `next_content_line()` will skip the comment and blank line, returning `"^ping"`
    fn next_content_line(&mut self) -> Option<String> {
        // Regex pattern explanation:
        // ^[- \t]*(#|$)
        // - ^       : Start of line
        // - [- \t]* : Zero or more dashes, spaces, or tabs
        // - (#|$)   : Either a hash (comment start) or end of line (empty/whitespace only)
        //
        // This matches:
        // - Comment lines: "# comment" or "  # comment"
        // - Empty lines: "" or "   " (just whitespace)
        // But NOT:
        // - "^ping" (regex line)
        // - "conf.ping" (config path line)
        let re = Regex::new("^[- \t]*(#|$)").unwrap();
        for line in &mut self.inner {
            match line {
                Ok(line2) => {
                    // If line doesn't match the comment/empty pattern, it's a content line
                    if !re.is_match(&line2) {
                        return Some(line2.trim().to_string());
                    }
                }
                Err(_) => break, // Stop on read error
            }
        }
        None // No more content lines (EOF)
    }
}

/// Iterator that yields (regex, config_file_path) pairs from grc.conf.
///
/// This iterator reads pairs of content lines from a grc.conf file and yields them as
/// `(Regex, String)` tuples where:
/// - **Regex** is the compiled regex pattern for matching command names
/// - **String** is the path to the corresponding grcat configuration file
///
/// ## Iteration Behavior
///
/// For each complete rule pair in the config file:
/// 1. Reads the first content line (regex pattern)
/// 2. Reads the second content line (config file path)
/// 3. Compiles the regex pattern
/// 4. If compilation succeeds, yields `(regex, path)` tuple
/// 5. If regex compilation fails, logs error and moves to next rule
/// 6. If incomplete rule (only pattern, no path), stops iteration
///
/// ## Error Handling
///
/// - **Malformed regexes**: Logged via debug_println and skipped, next rule attempted
/// - **Incomplete rules**: Iteration stops
/// - **IO errors**: Treated as EOF, iteration stops
///
/// # Examples
///
/// ```ignore
/// use std::io::BufReader;
/// use std::fs::File;
///
/// let file = File::open("~/.config/rgrc/grc.conf")?;
/// let reader = BufReader::new(file);
/// let config_reader = GrcConfigReader::new(reader.lines());
///
/// for (pattern_regex, config_file) in config_reader {
///     // Each iteration yields a command pattern and its config file
///     println!("Command pattern: {:?}", pattern_regex);
///     println!("Config file: {}", config_file);
/// }
/// ```
impl<A: BufRead> Iterator for GrcConfigReader<A> {
    type Item = (Regex, String);

    /// Return the next (regex, config_file_path) pair from the grc.conf file.
    ///
    /// # Returns
    ///
    /// - `Some((Regex, String))` - Next complete rule pair
    /// - `None` - EOF or incomplete rule encountered
    ///
    /// # Implementation Notes
    ///
    /// 1. Reads pattern line using `next_content_line()`
    /// 2. Reads config path line using `next_content_line()`
    /// 3. Compiles pattern string into Regex
    /// 4. If regex is malformed, recursively calls `self.next()` to skip and try next rule
    /// 5. If successful, returns tuple of (compiled_regex, config_path)
    /// 6. If pattern or path line is missing, returns None (stops iteration)
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(regexp) = self.next_content_line() {
            if let Some(filename) = self.next_content_line() {
                if let Ok(re) = Regex::new(&regexp) {
                    // Successfully compiled regex, return the rule pair
                    Some((re, filename))
                } else {
                    // Malformed regex pattern - log and skip to next rule
                    // Recursively call self.next() to continue with next entry
                    self.next()
                }
            } else {
                // Pattern without config file - incomplete rule, stop
                None
            }
        } else {
            // No pattern line found - EOF or read error
            None
        }
    }
}

/// Reader for grcat configuration files.
///
/// This struct implements an iterator over grcat configuration rules. Each rule defines
/// a regex pattern matched against command output and associated console styles (colors,
/// attributes) to apply to matching text.
///
/// ## File Format
///
/// The grcat.conf file uses a key=value format with alphanumeric line starts:
/// ```text
/// regexp=^(ERROR|WARN|INFO)\s+(\d+ms)
/// colours=bold red,yellow,green
///
/// regexp=^(PASS|OK)\s+
/// colours=bold green
///
/// regexp=status:\s+(\w+)
/// colours=cyan
/// ```
///
/// ## Entry Structure
///
/// Each configuration entry consists of consecutive lines starting with alphanumeric characters.
/// An entry ends when a non-alphanumeric line is encountered. Each entry can contain:
///
/// **Required keys:**
/// - `regexp` - Regex pattern to match against output text
///
/// **Optional keys:**
/// - `colours` - Comma-separated console styles for capture groups
/// - Other keys are accepted but ignored
///
/// ## Parsing Behavior
///
/// - **Entry boundaries**: Marked by non-alphanumeric lines (comments, blank lines)
/// - **Comments and blanks**: Skipped (non-alphanumeric)
/// - **Invalid regexes**: Entry is skipped, iteration continues
/// - **Missing regexp key**: Entry is skipped
/// - **Missing colours key**: Entry is valid with empty color list
/// - **Key=value format**: Supports spaces around '=' (e.g., `regexp = pattern`)
///
/// ## Generic Parameter
///
/// * `A` - A type implementing `BufRead`, typically:
///   - `std::io::BufReader<File>` for file input
///   - `std::io::BufReader<&[u8]>` for in-memory buffers
///
/// # Examples
///
/// ```ignore
/// use std::io::BufReader;
/// use std::fs::File;
///
/// let file = File::open("~/.config/rgrc/conf.ping")?;
/// let reader = BufReader::new(file);
/// let grcat_reader = GrcatConfigReader::new(reader.lines());
///
/// for entry in grcat_reader {
///     println!("Pattern: {:?}", entry.regex);
///     println!("Styles: {:?}", entry.colors);
/// }
/// ```
#[allow(dead_code)]
pub struct GrcatConfigReader<A> {
    inner: Lines<A>,
}

#[allow(dead_code)]
impl<A: BufRead> GrcatConfigReader<A> {
    /// Create a new grcat configuration reader from a line iterator.
    ///
    /// # Arguments
    ///
    /// * `inner` - A `Lines<A>` iterator yielding lines from a buffered reader
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// let file = File::open("~/.config/rgrc/conf.ping")?;
    /// let reader = BufReader::new(file);
    /// let grcat_reader = GrcatConfigReader::new(reader.lines());
    /// ```
    pub fn new(inner: Lines<A>) -> Self {
        GrcatConfigReader { inner }
    }

    /// Fetch the next alphanumeric line (skipping comments/blank lines).
    ///
    /// In grcat format, configuration entries start with alphanumeric characters (a-zA-Z0-9).
    /// All other lines (comments, blank lines) are ignored. This method is used to find the
    /// start of a new configuration entry.
    ///
    /// The regex pattern `^[a-zA-Z0-9]` matches lines that start with alphanumeric characters,
    /// which indicates the beginning of a key=value line.
    ///
    /// ## Returns
    ///
    /// - `Some(String)` - Next line starting with alphanumeric character (trimmed)
    /// - `None` - EOF reached, no more entries
    ///
    /// ## Error Handling
    ///
    /// If a line read error occurs, iteration stops and returns None.
    ///
    /// # Examples
    ///
    /// With input:
    /// ```text
    /// # Comment line
    ///
    /// regexp=^ERROR
    /// colours=bold red
    ///
    /// regexp=^WARN
    /// ```
    /// `next_alphanumeric()` will skip comments and blanks, returning:
    /// 1. `"regexp=^ERROR"`
    /// 2. `"colours=bold red"`
    /// 3. `"regexp=^WARN"`
    fn next_alphanumeric(&mut self) -> Option<String> {
        // Pattern ^[a-zA-Z0-9] matches lines starting with a letter or digit
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        for line in (&mut self.inner).flatten() {
            // Skip non-matching lines (comments, blanks)
            if alphanumeric.is_match(&line) {
                return Some(line.trim().to_string());
            }
        }
        None // No more alphanumeric lines (EOF)
    }

    /// Fetch the next line if it's alphanumeric, or None to signal end of entry.
    ///
    /// This method is used during entry parsing to continue reading key=value pairs
    /// that belong to the current configuration entry. As long as lines start with
    /// alphanumeric characters, they belong to the same entry. When a non-alphanumeric
    /// line is encountered (comment, blank line), it signals the end of the current
    /// entry, and the method returns None.
    ///
    /// ## Returns
    ///
    /// - `Some(String)` - Next line if it starts with alphanumeric (still in this entry)
    /// - `None` - End of entry or EOF (non-alphanumeric line or no more input)
    ///
    /// ## Implementation Details
    ///
    /// 1. Calls `self.inner.next()` to get the next line from the buffer
    /// 2. If EOF, returns None (no next line available)
    /// 3. If line starts with alphanumeric, returns it (still in entry)
    /// 4. If line doesn't start with alphanumeric, returns None (end of entry)
    ///
    /// ## Entry Boundary Detection
    ///
    /// This method implements entry boundary detection:
    /// - Alphanumeric start → still in current entry
    /// - Non-alphanumeric start → end of entry (return None)
    /// - Examples of entry-ending lines:
    ///   - Blank line: ""
    ///   - Comment: "# This is a comment"
    ///   - Whitespace: "   "
    ///   - Any line starting with non-alphanumeric: "---", "$", etc.
    fn following(&mut self) -> Option<String> {
        // Pattern ^[a-zA-Z0-9] matches lines starting with a letter or digit
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        if let Some(Ok(line)) = self.inner.next() {
            // If line starts with alphanumeric, it's part of this entry
            if alphanumeric.is_match(&line) {
                Some(line)
            } else {
                // Non-alphanumeric line marks end of entry
                None
            }
        } else {
            // EOF reached
            None
        }
    }
}

/// A single grcat configuration entry (regex + colors).
///
/// This struct represents a complete colorization rule parsed from a grcat configuration file.
/// Each entry contains a regex pattern and associated console styles that define how to colorize
/// text matching the pattern.
///
/// ## Structure
///
/// - **regex** - A compiled `fancy_regex::Regex` pattern to match against output text
/// - **colors** - A vector of `Style` objects corresponding to capture groups
///
/// ## Semantics
///
/// When a line of output matches `regex`:
/// - The 1st capture group is styled with `colors[0]` (if present)
/// - The 2nd capture group is styled with `colors[1]` (if present)
/// - And so on for additional capture groups
/// - Non-capturing text remains unstyled
///
/// If `colors` has fewer entries than capture groups, excess groups are not styled.
/// If `colors` is empty, the regex matches but nothing is colored (useful for filtering).
///
/// ## Derived Traits
///
/// - **Debug**: Allows printing entry contents for debugging
/// - **Clone**: Enables copying entries for use in multiple places
///
/// # Examples
///
/// ```ignore
/// use fancy_regex::Regex;
/// use rgrc::style::Style;
/// use rgrc::grc::GrcatConfigEntry;
///
/// let regex = Regex::new(r"^(ERROR|WARN) (\d+ms)$").unwrap();
/// let colors = vec![
///     Style::new().bold().red(),      // ERROR|WARN
///     Style::new().yellow(),           // \d+ms
/// ];
/// let entry = GrcatConfigEntry { regex, colors };
/// ```
/// Control how many times a regex pattern should match within a single line.
///
/// This enum specifies the matching behavior for grcat configuration entries.
/// It determines whether a rule should match once, multiple times, or stop processing
/// the line after the first match.
///
/// ## Variants
///
/// - **Once**: Match only the first occurrence of the pattern in each line
/// - **More**: Match all occurrences of the pattern in each line (default)
/// - **Stop**: Match the first occurrence and stop processing the entire line
///
/// ## Usage in Configuration
///
/// In grcat configuration files, this is specified using the `count` key:
/// ```text
/// regexp=^\s*#
/// colours=cyan
/// count=once    # Only color the first comment marker
///
/// regexp=\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}
/// colours=magenta
/// count=more    # Color all IP addresses (default)
///
/// regexp=^FATAL
/// colours=red,bold
/// count=stop    # Stop processing after first fatal error
/// ```
///
/// ## Implementation Notes
///
/// The count behavior is enforced during the regex matching phase in `colorize_regex()`.
/// - `Once`: After first match, skip to next rule
/// - `More`: Continue matching within the same rule (default behavior)
/// - `Stop`: After first match, skip all remaining rules for this line
#[derive(Debug, Clone, PartialEq)]
pub enum GrcatConfigEntryCount {
    /// Match only once per line, then skip to the next rule
    Once,
    /// Match all occurrences per line (default behavior)
    More,
    /// Match once and stop processing the entire line
    Stop,
}

#[derive(Debug, Clone)]
pub struct GrcatConfigEntry {
    #[allow(dead_code)]
    /// The compiled regex pattern (hybrid: fast standard regex or fancy-regex)
    pub regex: CompiledRegex,
    /// Styles to apply to capture groups (index 0 = group 1, index 1 = group 2, etc.)
    pub colors: Vec<Style>,
    /// If true, this rule should be ignored at runtime (treated as disabled).
    pub skip: bool,
    /// How many times to apply this rule per line (Once/More/Stop).
    pub count: GrcatConfigEntryCount,
    #[allow(dead_code)]
    /// Optional replacement template used when `replace` is specified in the
    /// configuration. Placeholders like `\1` are substituted with capture groups.
    pub replace: String,
}

impl GrcatConfigEntry {
    /// Create a new GrcatConfigEntry with default count and replace values.
    ///
    /// # Arguments
    ///
    /// * `regex` - The compiled regex pattern to match against output text
    /// * `colors` - Styles to apply to capture groups
    ///
    /// # Returns
    ///
    /// A new GrcatConfigEntry with count set to GrcatConfigEntryCount::More, replace set to empty string, and skip set to false
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use fancy_regex::Regex;
    /// use rgrc::style::Style;
    /// use rgrc::grc::GrcatConfigEntry;
    ///
    /// let regex = Regex::new(r"^(ERROR|WARN) (\d+ms)$").unwrap();
    /// let colors = vec![
    ///     Style::new().bold().red(),      // ERROR|WARN
    ///     Style::new().yellow(),           // \d+ms
    /// ];
    /// let entry = GrcatConfigEntry::new(regex, colors);
    /// ```
    #[allow(dead_code)]
    pub fn new(regex: CompiledRegex, colors: Vec<Style>) -> Self {
        GrcatConfigEntry {
            regex,
            colors,
            skip: false,
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
        }
    }
}

impl<A: BufRead> Iterator for GrcatConfigReader<A> {
    type Item = GrcatConfigEntry;

    /// Parse and return the next GrcatConfigEntry from the grcat config file.
    ///
    /// This method implements the core parsing logic for grcat configuration files.
    /// It processes configuration entries consisting of key=value pairs until a
    /// non-alphanumeric line is encountered (entry boundary).
    ///
    /// ## Processing Steps
    ///
    /// 1. **Find entry start**: Call `next_alphanumeric()` to find the next entry
    /// 2. **Parse key=value pairs**: Loop through consecutive alphanumeric lines
    /// 3. **Extract keys**: Parse "regexp=..." and "colours=..." assignments
    /// 4. **Validate regex**: Compile the regexp string; skip entry if invalid
    /// 5. **Parse styles**: Convert colour specification to Style vector
    /// 6. **Check boundaries**: Use `following()` to detect end of entry
    /// 7. **Return or skip**: Yield entry if valid regex found, otherwise skip
    ///
    /// ## Key=Value Format
    ///
    /// The regex pattern matches `key = value` format:
    /// - Pattern: `^([a-z_]+)\s*=\s*(.*)$`
    /// - Supports spaces around the '=' sign
    /// - Keys are lowercase with underscores
    /// - Examples: `regexp=pattern`, `colours = style1, style2`
    ///
    /// ## Entry Requirements
    ///
    /// **Required:**
    /// - At least one line starting with `regexp=` containing a valid regex
    ///
    /// **Optional:**
    /// - `colours=` line with comma-separated style keywords
    /// - If omitted, colors default to empty vector (no styling applied)
    ///
    /// **Ignored:**
    /// - Any other keys are silently ignored
    /// - Non-alphanumeric lines between entries are skipped
    ///
    /// ## Error Handling
    ///
    /// - **Invalid regex**: Entry is skipped, iteration continues to next entry
    /// - **Invalid colors**: Log via debug_println, colors default to empty
    /// - **Missing regexp**: Entry is skipped
    /// - **Incomplete entry**: Iteration stops (None)
    ///
    /// # Examples
    ///
    /// File content:
    /// ```text
    /// regexp=^(ERROR|WARN) (\d+ms)$
    /// colours=bold red,yellow
    ///
    /// regexp=^OK\s+
    /// colours=green
    /// ```
    ///
    /// Yields two GrcatConfigEntry items:
    /// 1. regex matches ERROR/WARN with capture group for timing
    /// 2. regex matches OK status line
    fn next(&mut self) -> Option<Self::Item> {
        // Regex pattern to parse key=value lines
        // Pattern: ^([a-z_]+)\s*=\s*(.*)$
        // - ^([a-z_]+)  : Key is one or more lowercase letters/underscores
        // - \s*=\s*     : Optional whitespace around equals sign
        // - (.*)$       : Value is everything to end of line
        // Examples matched:
        // - "regexp=^ERROR"
        // - "colours = bold red, yellow"
        // - "key = value with spaces"
        let re = Regex::new("^([a-z_]+)\\s*=\\s*(.*)$").unwrap();
        let mut ln: String;

        while let Some(line) = self.next_alphanumeric() {
            ln = line;
            let mut regex: Option<CompiledRegex> = None;
            let mut colors: Option<Vec<Style>> = None;
            let mut skip: Option<bool> = None;
            let mut count: Option<GrcatConfigEntryCount> = None;
            let mut replace: Option<String> = None;

            // Loop over all consecutive alphanumeric lines belonging to this entry
            // until we hit a non-alphanumeric line (entry boundary)
            loop {
                // Parse the key=value pair from current line
                let cap = re.captures(&ln).unwrap();
                let key = cap.get(1).unwrap().as_str();
                let value = cap.get(2).unwrap().as_str();

                // Process known keys, ignore unknown ones
                match key {
                    "regexp" => {
                        // Attempt to compile the regex pattern using hybrid engine
                        // This automatically selects fast standard regex or fancy-regex
                        match CompiledRegex::new(value) {
                            Ok(re) => {
                                regex = Some(re);
                            }
                            Err(_exc) => {
                                // Log error and skip this entry (regex is required)
                                eprintln!("Failed regexp: {:?}", _exc);
                            }
                        }
                    }
                    "colours" | "colors" => {
                        // Parse comma-separated style keywords into Style vector
                        // Example: "bold red,yellow,cyan" → [Style::new().bold().red(), Style::new().yellow(), Style::new().cyan()]
                        match styles_from_str(value) {
                            Ok(styles) => colors = Some(styles),
                            Err(e) => {
                                eprintln!("Error: Invalid style in configuration: {}", e);
                                eprintln!("Skipping this rule due to style error.");
                                break;
                            }
                        }
                    }
                    "count" => {
                        // Parse count value: once/more/stop
                        count = match value {
                            "once" => Some(GrcatConfigEntryCount::Once),
                            "more" => Some(GrcatConfigEntryCount::More),
                            "stop" => Some(GrcatConfigEntryCount::Stop),
                            _ => {
                                eprintln!("Unknown count value: {}", value);
                                None
                            }
                        };
                    }
                    "replace" => {
                        // Store replace string
                        replace = Some(value.to_string());
                    }
                    "skip" => {
                        // Parse skip value: true/false
                        skip = match value.to_lowercase().as_str() {
                            "true" | "1" | "yes" => Some(true),
                            "false" | "0" | "no" => Some(false),
                            _ => {
                                eprintln!("Unknown skip value: {}, defaulting to false", value);
                                Some(false)
                            }
                        };
                    }
                    _ => {
                        // Ignore unknown keys - grcat may add new keys in future versions
                    }
                };

                // Attempt to fetch the next line in this entry
                if let Some(nline) = self.following() {
                    ln = nline; // Continue with next line of this entry
                } else {
                    // Non-alphanumeric line encountered - end of entry
                    break;
                }
            }

            // Only emit entry if we successfully parsed a regex (required)
            if let Some(regex) = regex {
                return Some(GrcatConfigEntry {
                    regex,
                    colors: colors.unwrap_or_default(), // Empty color list if not specified
                    skip: skip.unwrap_or(false),        // Default to false if not specified
                    count: count.unwrap_or(GrcatConfigEntryCount::More), // Default to More if not specified
                    replace: replace.unwrap_or_default(), // Empty string if not specified
                });
            }
            // This entry lacked a valid regex; skip and try next entry
        }
        None // No more entries (EOF)
    }
}
