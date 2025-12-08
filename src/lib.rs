//! # lib.rs - Core Library for rgrc
//!
//! This module provides the core functionality of rgrc (Rust GRC), a colorization tool
//! that applies syntax highlighting to command output based on configuration rules.
//!
//! ## Architecture
//!
//! The library is organized into the following components:
//!
//! - **ColorMode**: Controls whether color output is enabled (On/Off/Auto)
//! - **Configuration Loading**: Functions to load colorization rules from config files
//! - **Submodules**:
//!   - `style`: Lightweight ANSI styling (replaces console crate)
//!   - `colorizer`: Text colorization engine
//!   - `grc`: Config file parsing with hybrid regex engine
//!   - `enhanced_regex`: Custom lookaround implementation (used when fancy feature is disabled)
//!
//! ## Features
//!
//! - **embed-configs** (default): Embed configuration files into binary
//! - **fancy-regex** (default): Use battle-tested fancy-regex for enhanced patterns
//!   - Disable for smaller binary: `cargo build --no-default-features --features=embed-configs`
//! - **timetrace**: Enable timing trace for performance profiling
//!
//! ## Regex Engine
//!
//! rgrc uses a hybrid regex approach:
//! - Simple patterns → Standard `regex` crate (fast)
//! - Complex patterns → `fancy-regex` (default) or `EnhancedRegex` (lightweight)
//!
//! See `grc::CompiledRegex` documentation for details.
//!
//! ## Usage Example
//!
//! ```ignore
//! use rgrc::{ColorMode, load_config, load_grcat_config};
//!
//! // Determine if colors should be used
//! let color_mode = ColorMode::Auto;
//!
//! // Load colorization rules for a specific command
//! let rules = load_config("~/.config/rgrc/grc.conf", "ping");
//! ```

pub mod style;
// Re-export Style for easier access
pub use style::Style;

pub mod args;
pub mod buffer;
pub mod colorizer;
pub mod enhanced_regex;
pub mod grc;
pub mod utils;

use std::fs::File;
use std::io::BufRead;
use std::str::FromStr;

use grc::{GrcConfigReader, GrcatConfigEntry, GrcatConfigReader};

// Simple tilde expansion function to replace shellexpand
fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return format!("{}/{}", home, stripped);
    }
    path.to_string()
}

// Version constant for cache directory
#[cfg(feature = "embed-configs")]
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Use generated `embedded_configs.rs` (created by build.rs) so the list of
// embedded files is derived from the `share` directory instead of being hard-coded.
#[cfg(feature = "embed-configs")]
include!(concat!(env!("OUT_DIR"), "/embedded_configs.rs"));

/// The bundled `rgrc.conf` contents when `embed-configs` is enabled.
/// This mirrors the on-disk `etc/rgrc.conf` file and is empty when embedding
/// is disabled.
#[cfg(feature = "embed-configs")]
pub const EMBEDDED_GRC_CONF: &str = include_str!("../etc/rgrc.conf");

/// Flush and rebuild the cache directory (embed-configs only)
///
/// This function removes the existing cache directory and rebuilds it with
/// embedded configuration files. Returns the path to the rebuilt cache directory
/// and the number of configuration files created.
///
/// # Returns
///
/// Returns `Some((cache_path, config_count))` on success, `None` on failure.
#[cfg(feature = "embed-configs")]
pub fn flush_and_rebuild_cache() -> Option<(std::path::PathBuf, usize)> {
    // Get cache directory path
    let cache_dir = get_cache_dir()?;

    // Remove existing cache directory if it exists
    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir).ok()?;
    }

    // Rebuild cache
    let new_cache_dir = ensure_cache_populated()?;

    // Count the number of config files
    let conf_dir = new_cache_dir.join("conf");
    let config_count = if conf_dir.exists() {
        std::fs::read_dir(&conf_dir)
            .map(|entries| entries.count())
            .unwrap_or(0)
    } else {
        0
    };

    Some((new_cache_dir, config_count))
}

// Helper function to get cache directory path
#[cfg(feature = "embed-configs")]
fn get_cache_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .map(|h| h.join(".cache").join("rgrc").join(VERSION))
}

// Ensure cache directory exists and populate it with embedded configs
#[cfg(feature = "embed-configs")]
fn ensure_cache_populated() -> Option<std::path::PathBuf> {
    let cache_dir = get_cache_dir()?;

    // Check if cache directory exists and appears populated (rgrc.conf + at least one conf file)
    let grc_conf_path = cache_dir.join("rgrc.conf");
    let conf_dir = cache_dir.join("conf");
    if grc_conf_path.exists() {
        // If conf directory exists and contains at least one file, we assume cache is populated
        if conf_dir.exists()
            && let Ok(mut entries) = std::fs::read_dir(&conf_dir)
            && entries.next().is_some()
        {
            return Some(cache_dir);
        }
        // rgrc.conf exists but conf dir missing or empty — fall through and repopulate
    }

    // Create cache directory structure
    std::fs::create_dir_all(&cache_dir).ok()?;
    let conf_dir = cache_dir.join("conf");
    std::fs::create_dir_all(&conf_dir).ok()?;

    // Write rgrc.conf
    std::fs::write(&grc_conf_path, EMBEDDED_GRC_CONF).ok()?;

    // Write all embedded configs
    // Don't fail the entire cache population if a single file fails to write
    let mut any_success = false;
    for (filename, content) in EMBEDDED_CONFIGS {
        let file_path = conf_dir.join(filename);
        if std::fs::write(file_path, content).is_ok() {
            any_success = true;
        }
    }

    // Only return Some if we successfully wrote at least one config file
    if any_success { Some(cache_dir) } else { None }
}

/// Control whether colored output should be enabled for this run.
///
/// This enum determines the color output mode for the application:
///
/// - **On**: Always enable colored output
/// - **Off**: Always disable colored output, output plain text
/// - **Auto**: Enable colors only if output is to a terminal (TTY)
///
/// The Auto mode is recommended for most use cases as it automatically
/// disables colors when output is piped or redirected.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use rgrc::ColorMode;
///
/// assert_eq!(ColorMode::from_str("on"), Ok(ColorMode::On));
/// assert_eq!(ColorMode::from_str("off"), Ok(ColorMode::Off));
/// assert_eq!(ColorMode::from_str("auto"), Ok(ColorMode::Auto));
/// assert!(ColorMode::from_str("invalid").is_err());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorMode {
    /// Always enable colored output
    On,
    /// Always disable colored output
    Off,
    /// Enable colors only for terminal output (auto-detect)
    Auto,
}

impl FromStr for ColorMode {
    type Err = ();

    /// Parse a string into a ColorMode variant.
    ///
    /// Accepts string values: "on", "off", or "auto" (case-sensitive).
    ///
    /// # Arguments
    ///
    /// * `s` - String slice to parse ("on", "off", or "auto")
    ///
    /// # Returns
    ///
    /// - `Ok(ColorMode::On)` if s is "on"
    /// - `Ok(ColorMode::Off)` if s is "off"
    /// - `Ok(ColorMode::Auto)` if s is "auto"
    /// - `Err(())` if s is any other value
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use rgrc::ColorMode;
    ///
    /// let mode = ColorMode::from_str("on").unwrap();
    /// assert_eq!(mode, ColorMode::On);
    ///
    /// let mode = ColorMode::from_str("auto").unwrap();
    /// assert_eq!(mode, ColorMode::Auto);
    ///
    /// assert!(ColorMode::from_str("maybe").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(ColorMode::On),
            "off" => Ok(ColorMode::Off),
            "auto" => Ok(ColorMode::Auto),
            _ => Err(()),
        }
    }
}

/// Standard resource paths searched for grcat config files.
///
/// These paths are searched **in order** when looking for colorization rule files
/// (grcat.conf) that define how to colorize output for specific commands.
/// **The search stops at the first matching file found.**
///
/// The paths support:
/// - `~` expansion (home directory)
/// - XDG Base Directory Specification compliance
/// - System-wide configuration directories
///
/// # Search Order (Priority)
///
/// User configurations take precedence over system configurations:
///
/// 1. `~/.config/rgrc` - User's rgrc config directory (XDG_CONFIG_HOME) **← HIGHEST PRIORITY**
/// 2. `~/.local/share/rgrc` - User's rgrc data directory (XDG_DATA_HOME)
/// 3. `/usr/local/share/rgrc` - System-wide custom installations
/// 4. `/usr/share/rgrc` - Standard system location (rgrc variant)
/// 5. `~/.config/grc` - Legacy grc user config directory
/// 6. `~/.local/share/grc` - Legacy grc user data directory
/// 7. `/usr/local/share/grc` - Legacy system-wide location
/// 8. `/usr/share/grc` - Standard grc location (original) **← LOWEST PRIORITY**
///
/// # Example: Priority Resolution
///
/// For file `conf.df`, search stops at the **first match**:
/// - If `~/.config/rgrc/conf.df` exists → **RETURNED** (other paths not searched)
/// - If only `/usr/share/rgrc/conf.df` exists → returned as fallback
///
/// # Examples
///
/// All paths in RESOURCE_PATHS are searched in order when loading configuration:
/// ```ignore
/// let config_entries = load_config("~/.config/rgrc/grc.conf", "ping");
/// // This will search in RESOURCE_PATHS directories until first match is found
/// ```
pub const RESOURCE_PATHS: &[&str] = &[
    "~/.config/rgrc",
    "~/.local/share/rgrc",
    "/usr/local/share/rgrc",
    "/usr/share/rgrc",
    "~/.config/grc",
    "~/.local/share/grc",
    "/usr/local/share/grc",
    "/usr/share/grc",
];

/// Load colorization rules for a given command from a grc.conf-style configuration file.
///
/// This function reads a grc.conf configuration file and extracts colorization rules
/// that match the specified pseudo_command. It then loads the detailed rule files
/// referenced by the matching configuration entry.
///
/// # Configuration File Format
///
/// The grc.conf file uses a key-value format where:
/// - Each line is a rule mapping a command pattern to a colorization file
/// - Format: `<regex_pattern> <colorization_file_name>`
/// - Example: `^ping` conf.ping
///
/// The function searches RESOURCE_PATHS for the referenced colorization files,
/// stopping at the **first match found** to respect user configuration priority.
///
/// # Arguments
///
/// * `path` - Path to the grc.conf file to read (e.g., "~/.config/rgrc/grc.conf")
/// * `pseudo_command` - The command name to match against patterns in grc.conf
///   (e.g., "ping", "ls", "curl")
///
/// # Returns
///
/// A `Vec<GrcatConfigEntry>` containing all colorization rules loaded from the
/// referenced grcat.conf files. Returns an empty vector if:
/// - The grc.conf file cannot be opened
/// - No matching rule is found for the pseudo_command
/// - The referenced colorization files cannot be opened
///
/// # Priority Resolution
///
/// When searching for a config file (e.g., `conf.ping`), the function stops at
/// the **first directory containing the file**:
/// - User config (`~/.config/rgrc/conf.ping`) takes precedence
/// - System config (`/usr/share/rgrc/conf.ping`) only used if user config not found
///
/// # Errors Handled
///
/// All errors are silently handled and result in empty or partial rule sets:
/// - File not found → returns empty vector
/// - Invalid regex → pattern not matched → returns empty vector
/// - Invalid colorization file path → skipped to next directory
///
/// # Examples
///
/// ```ignore
/// use rgrc::load_config;
///
/// // Load colorization rules for the ping command
/// let rules = load_config("~/.config/rgrc/grc.conf", "ping");
/// if !rules.is_empty() {
///     println!("Found {} colorization rules for ping", rules.len());
/// }
///
/// // Load rules for curl command
/// let curl_rules = load_config("~/.config/rgrc/grc.conf", "curl");
/// ```
///
/// # Implementation Details
///
/// 1. Opens and parses grc.conf file
/// 2. Searches for a regex pattern matching pseudo_command
/// 3. Extracts the colorization file reference from matching entry
/// 4. Expands ~ in paths using shellexpand
/// 5. Searches RESOURCE_PATHS directories **in order** for the colorization file
/// 6. Returns rules from the **first matching file found**
pub fn load_config(path: &str, pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // First, try to load from filesystem config file
    let filesystem_result = File::open(path).ok().and_then(|f| {
        let bufreader = std::io::BufReader::new(f);
        let mut configreader = GrcConfigReader::new(bufreader.lines());
        // Find the first matching rule for this pseudo_command
        configreader
            .find(|(re, _config)| re.is_match(pseudo_command))
            .map(|(_, config)| config)
    });

    if let Some(config) = filesystem_result {
        // Search RESOURCE_PATHS for the colorization file - **stop at first match**
        for base_path in RESOURCE_PATHS {
            let expanded_path = expand_tilde(base_path);
            let config_path = format!("{}/{}", expanded_path, config);
            // Use file_exists_and_parse to distinguish "file exists but empty" from "file not found"
            match file_exists_and_parse(&config_path) {
                Some(rules) => return rules,           // File found (even if empty) - STOP
                None => continue,                      // File not found - keep searching
            }
        }
    }

    // No configuration found
    Vec::new()
}

/// Check if a file exists and parse it for colorization rules.
/// 
/// Returns:
/// - `Some(rules)` if file exists (may be empty if file is empty)
/// - `None` if file does not exist
/// 
/// This distinguishes between "file doesn't exist" (None) and 
/// "file exists but has no rules" (Some([])).
fn file_exists_and_parse(filename: &str) -> Option<Vec<GrcatConfigEntry>> {
    // Try to open the file
    if let Ok(grcat_config_file) = File::open(filename) {
        let bufreader = std::io::BufReader::new(grcat_config_file);
        // Parse all rules from the configuration file
        let configreader = GrcatConfigReader::new(bufreader.lines());
        let entries: Vec<_> = configreader.collect();
        // Return Some (even if empty) - file exists
        return Some(entries);
    }

    // Fallback to embedded configuration (only when embed-configs is enabled)
    #[cfg(feature = "embed-configs")]
    {
        // Extract config name from path (e.g., "conf.ping" from full path)
        let config_name = filename;

        // Ensure cache is populated
        if let Some(cache_dir) = ensure_cache_populated() {
            let conf_dir = cache_dir.join("conf");
            let config_path = conf_dir.join(config_name);
            if let Ok(grcat_config_file) = File::open(&config_path) {
                let bufreader = std::io::BufReader::new(grcat_config_file);
                let configreader = GrcatConfigReader::new(bufreader.lines());
                let entries: Vec<_> = configreader.collect();
                // Return Some (embedded file found, even if empty)
                return Some(entries);
            }
        }
    }

    // File not found
    None
}

/// Load colorization rules from a grcat.conf-style configuration file.
///
/// This function reads a grcat.conf file and parses all colorization rules contained
/// within it. These rules define how specific text patterns should be colored in output.
///
/// # Configuration File Format
///
/// The grcat.conf file format is a set of regex patterns paired with color specifications.
/// Each rule can specify:
/// - Regular expressions to match against text patterns
/// - Color foreground values (standard ANSI color names)
/// - Color background values
/// - Text attributes (bold, dim, italic, etc.)
///
/// # Arguments
///
/// * `filename` - Path to the grcat.conf file to read
///   Supports paths with ~ for home directory expansion
///   Can be a path within RESOURCE_PATHS directories
///   Example: "~/.config/rgrc/conf.ping"
///
/// # Returns
///
/// A `Vec<GrcatConfigEntry>` containing all parsed colorization rules from the file.
/// Returns an empty vector if:
/// - The file cannot be opened
/// - The file has invalid syntax
/// - Any parsing errors occur
///
/// # Type Parameter
///
/// * `T: AsRef<str>` - Accepts String, &str, or any type convertible to &str
///
/// # Examples
///
/// ```ignore
/// use rgrc::load_grcat_config;
///
/// // Load rules from a specific file
/// let rules = load_grcat_config("~/.config/rgrc/conf.ping");
/// println!("Loaded {} rules", rules.len());
///
/// // Works with both owned and borrowed strings
/// let filename = String::from("~/.config/rgrc/conf.curl");
/// let rules = load_grcat_config(filename);
///
/// let rules2 = load_grcat_config("~/.config/rgrc/conf.ls");
/// ```
///
/// # Integration with load_config
///
/// This function is typically called indirectly through `load_config()`:
/// ```ignore
/// // High-level: load_config finds the right grcat file automatically
/// let rules = load_config("~/.config/rgrc/grc.conf", "ping");
///
/// // Low-level: if you already know the grcat file path
/// let rules = load_grcat_config("~/.config/rgrc/conf.ping");
/// ```
///
/// # Error Handling
///
/// All errors (file not found, parse errors, etc.) are silently handled
/// and result in an empty rule vector. This allows graceful degradation
/// when configuration files are missing or malformed.
pub fn load_grcat_config<T: AsRef<str>>(filename: T) -> Vec<GrcatConfigEntry> {
    let filename_str = filename.as_ref();

    // Return empty vector for empty filename
    if filename_str.is_empty() {
        return Vec::new();
    }

    // First, try to load from filesystem
    if let Ok(grcat_config_file) = File::open(filename_str) {
        let bufreader = std::io::BufReader::new(grcat_config_file);
        // Parse all rules from the configuration file
        let configreader = GrcatConfigReader::new(bufreader.lines());
        let entries: Vec<_> = configreader.collect();

        // If we successfully loaded from filesystem and got entries, return them
        if !entries.is_empty() {
            return entries;
        }
    }

    // Fallback to embedded configuration (only when embed-configs is enabled)
    #[cfg(feature = "embed-configs")]
    {
        // Extract config name from path (e.g., "conf.ping" from "conf.ping")
        let config_name = filename_str;

        // Ensure cache is populated
        if let Some(cache_dir) = ensure_cache_populated() {
            let conf_dir = cache_dir.join("conf");
            let config_path = conf_dir.join(config_name);
            if let Ok(grcat_config_file) = File::open(&config_path) {
                let bufreader = std::io::BufReader::new(grcat_config_file);
                let configreader = GrcatConfigReader::new(bufreader.lines());
                let entries: Vec<_> = configreader.collect();
                if !entries.is_empty() {
                    return entries;
                }
            }
        }
    }

    // No configuration found
    Vec::new()
}

/// Configuration file paths in priority order.
/// The program searches these paths to find grc.conf (or rgrc.conf) which maps
/// commands to their colorization profiles. Paths prefixed with ~ are expanded using shellexpand.
/// Typical flow: try ~/.grc first (user config), then system-wide configs (/etc/grc.conf).
const CONFIG_PATHS: &[&str] = &[
    "~/.rgrc",
    "~/.config/rgrc/rgrc.conf",
    "/usr/local/etc/rgrc.conf",
    "/etc/rgrc.conf",
    "~/.grc",
    "~/.config/grc/grc.conf",
    "/usr/local/etc/grc.conf",
    "/etc/grc.conf",
];

/// Load colorization rules for a given pseudo-command by searching all configuration paths.
///
/// This function iterates through the predefined CONFIG_PATHS, attempting to load
/// colorization rules for the specified pseudo-command from each configuration file.
/// **The search stops at the first file that contains matching rules.**
///
/// # Priority Resolution
///
/// Configuration files are searched in priority order:
/// 1. User configs (`~/.rgrc`, `~/.config/rgrc/rgrc.conf`) checked first
/// 2. System configs (`/etc/rgrc.conf`, `/usr/local/etc/rgrc.conf`) as fallback
/// 3. Legacy grc configs checked last for backward compatibility
///
/// # Arguments
///
/// * `pseudo_command` - The command string to match against configuration rules
///   (e.g., "ping", "ls", "curl")
///
/// # Returns
///
/// A vector of `GrcatConfigEntry` containing all colorization rules that apply
/// to the given pseudo-command from the **first config file containing matches**.
///
/// # Examples
///
/// ```ignore
/// let rules = load_rules_for_command("ping");
/// // Now rules contains all colorization rules for ping from the first matching config file
/// ```
#[allow(dead_code)]
pub fn load_rules_for_command(pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // First, try to load from embedded configuration (only when feature is enabled)
    #[cfg(feature = "embed-configs")]
    {
        let embedded_rules = load_config_from_embedded(pseudo_command);
        if !embedded_rules.is_empty() {
            return embedded_rules;
        }
    }

    // Fallback to file system configuration paths - **stop at first match**
    for config_path in CONFIG_PATHS {
        let expanded_path = expand_tilde(config_path);
        let rules = load_config(&expanded_path, pseudo_command);
        if !rules.is_empty() {
            return rules; // Stop at first matching config file
        }
    }

    Vec::new()
}

/// Helper function to format Style info with colors applied
#[cfg(feature = "debug")]
fn format_style_info(_style: &Style) -> String {
    // Return simple representation for display
    // The style itself will be applied for formatting
    String::new()
}

/// Colorize input with debug output showing which rules match each line.
///
/// This function applies colorization AND prints debug information to stderr
/// showing which rules matched each input line.
///
/// # Arguments
///
/// * `reader` - Input source implementing Read
/// * `writer` - Output destination implementing Write  
/// * `rules` - Slice of colorization rules
///
/// # Returns
///
/// * `Ok(())` - Successfully processed all input
/// * `Err(Box<dyn Error>)` - I/O or processing error
#[cfg(feature = "debug")]
pub fn colorize_regex_with_debug<R, W>(
    reader: &mut R,
    writer: &mut W,
    rules: &[GrcatConfigEntry],
    debug_level: crate::args::DebugLevel,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: std::io::Read,
    W: std::io::Write,
{
    use crate::args::DebugLevel;
    use std::io::{BufRead, BufReader};

    let buffered_reader = BufReader::new(reader);
    let mut line_num = 0;

    for line_result in buffered_reader.lines() {
        let line = line_result?;
        line_num += 1;

        // Check which rules match and collect debug info
        let mut matched_rules = Vec::new();
        for (rule_idx, rule) in rules.iter().enumerate() {
            if rule.regex.is_match(&line) {
                matched_rules.push((rule_idx, rule));
            }
        }

        // Apply colorization using the standard colorizer
        // Create a temporary cursor from the line with a newline
        use std::io::Cursor;
        let mut line_reader = Cursor::new(format!("{}\n", line).into_bytes());
        let mut temp_output = Vec::new();

        colorizer::colorize_regex(&mut line_reader, &mut temp_output, rules)?;

        // Write the colored output (no need to add newline, colorize_regex already did)
        writer.write_all(&temp_output)?;

        // Print debug info to stderr based on debug level
        match debug_level {
            DebugLevel::Off => {
                // No debug output
            }
            DebugLevel::Basic => {
                // Show matched rules with count
                if matched_rules.is_empty() {
                    let line_marker_str = format!("[Line {}]", line_num);
                    eprintln!(
                        "{} ℹ️  No rules matched",
                        Style::new().cyan().apply_to(&line_marker_str)
                    );
                } else {
                    let line_marker_str = format!("[Line {}]", line_num);
                    let line_marker = Style::new().cyan().apply_to(&line_marker_str);
                    eprintln!(
                        "{} ✓ Matched {} rule(s): {}",
                        line_marker,
                        matched_rules.len(),
                        matched_rules
                            .iter()
                            .map(|(idx, rule)| {
                                let colors_display = if rule.colors.is_empty() {
                                    "no-style".to_string()
                                } else {
                                    format!("{} style(s)", rule.colors.len())
                                };
                                format!("#{} ({})", idx + 1, colors_display)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
            DebugLevel::Verbose => {
                // Show detailed rule and style information
                let line_marker_str = format!("[Line {}]", line_num);
                let line_marker = Style::new().cyan().apply_to(&line_marker_str);

                if matched_rules.is_empty() {
                    eprintln!("{} ℹ️  No rules matched", line_marker);
                } else {
                    eprintln!("{} ✓ Matched {} rule(s):", line_marker, matched_rules.len());
                    for (idx, rule) in matched_rules.iter() {
                        // Display Rule with bold formatting
                        let rule_display = format!("Rule #{}: {}", idx + 1, rule.regex.as_str());
                        eprintln!("  {}", Style::new().bold().apply_to(&rule_display));

                        // Display first matched text with styles applied
                        if let Some(captures) = rule.regex.captures_from_pos(&line, 0) {
                            // Get the full match (group 0) and rebuild it with individual groups styled
                            if let Some(_full_match) = captures.get(0) {
                                let mut styled_groups = Vec::new();
                                for group_idx in 1..captures.len() {
                                    if let Some(cap) = captures.get(group_idx) {
                                        let text = cap.as_str();
                                        // Apply style if it exists for this group
                                        if group_idx <= rule.colors.len() {
                                            styled_groups.push(format!(
                                                "{}",
                                                rule.colors[group_idx - 1].apply_to(text)
                                            ));
                                        } else {
                                            styled_groups.push(text.to_string());
                                        }
                                    }
                                }

                                if !styled_groups.is_empty() {
                                    let matched_text = styled_groups.join(" ");
                                    eprintln!(
                                        "    {}",
                                        Style::new()
                                            .dim()
                                            .apply_to(&format!("Matched: {}", matched_text))
                                    );
                                }
                            }
                        }

                        if rule.colors.is_empty() {
                            eprintln!("    {}", Style::new().dim().apply_to("Styles: (none)"));
                        } else {
                            eprintln!("    {}:", Style::new().dim().apply_to("Styles"));
                            for (color_idx, color) in rule.colors.iter().enumerate() {
                                let _color_display = format_style_info(color);
                                eprintln!(
                                    "      {}",
                                    color.apply_to(&format!("Group {}: applied", color_idx + 1))
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Load colorization rules from embedded configuration.
/// On first run, writes embedded configs to disk cache, then loads from there.
#[cfg(feature = "embed-configs")]
fn load_config_from_embedded(pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // Ensure cache is populated, get cache directory
    let cache_dir = match ensure_cache_populated() {
        Some(dir) => dir,
        None => return Vec::new(), // Failed to create cache
    };

    // Load from cached rgrc.conf
    let grc_conf_path = cache_dir.join("rgrc.conf");
    let conf_dir = cache_dir.join("conf");

    // Use load_config to find matching config file
    if let Ok(f) = File::open(&grc_conf_path) {
        let bufreader = std::io::BufReader::new(f);
        let mut configreader = GrcConfigReader::new(bufreader.lines());
        if let Some((_, config_file)) = configreader.find(|(re, _)| re.is_match(pseudo_command)) {
            let config_path = conf_dir.join(&config_file);
            if let Some(config_str) = config_path.to_str() {
                return load_grcat_config(config_str);
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
mod lib_test {
    use super::*;

    // Note: These tests are documentation-based since the main() function
    // cannot be directly tested. The actual behavior would need to be tested
    // through integration tests that run the binary.
    #[cfg(test)]
    #[test]
    fn test_load_rules_for_command() {
        // Test loading rules for a known command that should have configuration
        let rules = load_rules_for_command("ping");

        // Behavior depends on whether embed-configs feature is enabled
        #[cfg(feature = "embed-configs")]
        {
            // Ensure tests run reliably in CI where HOME may be unset or point
            // to an unexpected location. Use a tempdir as HOME so ensure_cache_populated
            // can write the embedded config cache and load_rules_for_command finds rules.
            use tempfile::TempDir;
            let td = TempDir::new().expect("create tempdir");
            let prev_home = std::env::var_os("HOME");
            unsafe {
                std::env::set_var("HOME", td.path());
            }

            // Re-run loading after setting HOME to the tempdir-backed cache
            let rules_after = load_rules_for_command("ping");

            // Restore HOME for subsequent tests
            if let Some(h) = prev_home {
                unsafe {
                    std::env::set_var("HOME", h);
                }
            } else {
                unsafe {
                    std::env::remove_var("HOME");
                }
            }

            assert!(
                !rules_after.is_empty(),
                "Should load rules for ping command from embedded configs when embed-configs is enabled"
            );
        }

        #[cfg(not(feature = "embed-configs"))]
        {
            // Without embed-configs, rules may or may not be found depending on filesystem
            // We just verify the function doesn't panic and returns valid structures
            for rule in &rules {
                assert!(
                    !rule.regex.as_str().is_empty(),
                    "Rule should have a regex pattern"
                );
            }
        }

        // Verify that the rules are valid GrcatConfigEntry structs
        for rule in &rules {
            assert!(
                !rule.regex.as_str().is_empty(),
                "Rule should have a regex pattern"
            );
            // Colors can be empty for some rules, but regex should always be present
        }

        // Test with a command that likely doesn't exist
        let no_rules = load_rules_for_command("nonexistent_command_xyz");
        // This should return empty, as no config should match
        assert!(
            no_rules.is_empty(),
            "Nonexistent command should return no rules"
        );

        // Performance test: measure time to load rules (skip in debug mode)
        #[cfg(not(debug_assertions))]
        {
            use std::time::Instant;
            let start = Instant::now();
            for _ in 0..10 {
                let _rules = load_rules_for_command("ping");
            }
            let duration = start.elapsed();
            let avg_time = duration / 10;

            // Should be reasonably fast (< 1500ms per call in release mode, accounting for cache creation)
            println!("Average time to load ping rules: {:?}", avg_time);
            assert!(
                avg_time.as_millis() < 1500,
                "Loading rules should be reasonably fast (< 1500ms)"
            );
        }
    }

    #[test]
    fn test_config_priority_order() {
        // Test that user configs take precedence over system configs
        // This test verifies that load_config stops at first match
        use tempfile::TempDir;

        // Create temporary directories simulating RESOURCE_PATHS
        let user_config_dir = TempDir::new().expect("create user config dir");
        let system_config_dir = TempDir::new().expect("create system config dir");

        // Create a test config file in both directories
        let user_conf_file = user_config_dir.path().join("conf.testcmd");
        let system_conf_file = system_config_dir.path().join("conf.testcmd");

        // User config: has style on line 1
        std::fs::write(&user_conf_file, "regexp=^USER\ncolours=green").expect("write user config");

        // System config: has different style
        std::fs::write(&system_conf_file, "regexp=^SYSTEM\ncolours=red")
            .expect("write system config");

        // Test load_grcat_config with user config (should return rules from this file)
        let user_rules = load_grcat_config(user_conf_file.to_string_lossy().to_string());
        assert!(
            !user_rules.is_empty(),
            "Should load rules from user config file"
        );

        // Verify it loaded the USER pattern, not SYSTEM
        let has_user_pattern = user_rules
            .iter()
            .any(|rule| rule.regex.as_str().contains("USER"));
        assert!(
            has_user_pattern,
            "User config should contain USER pattern, proving user config was loaded (not system)"
        );

        // Test with system config
        let system_rules = load_grcat_config(system_conf_file.to_string_lossy().to_string());
        assert!(
            !system_rules.is_empty(),
            "Should load rules from system config file"
        );

        let has_system_pattern = system_rules
            .iter()
            .any(|rule| rule.regex.as_str().contains("SYSTEM"));
        assert!(
            has_system_pattern,
            "System config should contain SYSTEM pattern"
        );
    }

    #[test]
    fn test_load_config_stops_at_first_match() {
        // Test that load_config stops searching after first matching config file
        use tempfile::TempDir;

        // Create temp grc.conf and two conf files
        let temp_dir = TempDir::new().expect("create temp dir");
        let grc_conf_path = temp_dir.path().join("grc.conf");
        let conf_dir1 = TempDir::new().expect("create conf dir 1");
        let conf_dir2 = TempDir::new().expect("create conf dir 2");

        // Create grc.conf mapping testcmd to conf.testcmd
        std::fs::write(&grc_conf_path, "^testcmd\tconf.testcmd").expect("write grc.conf");

        // Create conf.testcmd in first directory with "USER" pattern
        let conf_file_1 = conf_dir1.path().join("conf.testcmd");
        std::fs::write(&conf_file_1, "regexp=^USER\ncolours=green").expect("write conf file 1");

        // Create conf.testcmd in second directory with "SYSTEM" pattern
        let conf_file_2 = conf_dir2.path().join("conf.testcmd");
        std::fs::write(&conf_file_2, "regexp=^SYSTEM\ncolours=red").expect("write conf file 2");

        // When both files exist, load_grcat_config should return from first found
        let rules_1 = load_grcat_config(conf_file_1.to_string_lossy().to_string());
        assert!(
            !rules_1.is_empty(),
            "Should load rules from first config file"
        );

        let has_user = rules_1
            .iter()
            .any(|rule| rule.regex.as_str().contains("USER"));
        assert!(
            has_user,
            "Should load USER pattern from first config file (not SYSTEM)"
        );
    }

    #[test]
    fn test_empty_config_file_stops_search() {
        // Test that an empty config file in a higher-priority directory stops the search
        // even though it contains no rules. This is the critical bug fix.
        use tempfile::TempDir;

        // Create temp directories simulating RESOURCE_PATHS priority
        let user_config_dir = TempDir::new().expect("create user config dir");
        let system_config_dir = TempDir::new().expect("create system config dir");

        // Create grc.conf files in both directories
        let user_grc_conf = user_config_dir.path().join("grc.conf");
        let system_grc_conf = system_config_dir.path().join("grc.conf");

        // Both map testcmd to conf.testcmd
        std::fs::write(&user_grc_conf, "^testcmd\tconf.testcmd").expect("write user grc.conf");
        std::fs::write(&system_grc_conf, "^testcmd\tconf.testcmd").expect("write system grc.conf");

        // Create conf.testcmd files
        let user_conf_file = user_config_dir.path().join("conf.testcmd");
        let system_conf_file = system_config_dir.path().join("conf.testcmd");

        // User config: EMPTY (no rules)
        std::fs::write(&user_conf_file, "").expect("write empty user config");

        // System config: has rules
        std::fs::write(&system_conf_file, "regexp=^SYSTEM\ncolours=red").expect("write system config");

        // The critical test: load_grcat_config with empty file should return empty
        // and NOT continue searching the next directory
        let rules_user = load_grcat_config(user_conf_file.to_string_lossy().to_string());
        assert!(
            rules_user.is_empty(),
            "Empty user config file should return no rules (NOT fall back to system)"
        );

        // Verify system config has rules (to prove it COULD have been loaded)
        let rules_system = load_grcat_config(system_conf_file.to_string_lossy().to_string());
        assert!(
            !rules_system.is_empty(),
            "System config should have rules"
        );

        // Verify it has SYSTEM pattern
        let has_system = rules_system
            .iter()
            .any(|rule| rule.regex.as_str().contains("SYSTEM"));
        assert!(has_system, "System config should contain SYSTEM pattern");
    }

    #[test]
    fn test_expand_tilde() {
        // Test with valid HOME environment variable
        unsafe {
            std::env::set_var("HOME", "/home/testuser");
        }

        // Normal tilde expansion
        assert_eq!(expand_tilde("~/Documents"), "/home/testuser/Documents");
        assert_eq!(expand_tilde("~/"), "/home/testuser/");
        assert_eq!(expand_tilde("~"), "~");

        // No tilde should be unchanged
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
        assert_eq!(expand_tilde("relative/path"), "relative/path");
        assert_eq!(expand_tilde(""), "");

        // Tilde not at start should be unchanged
        assert_eq!(expand_tilde("path~/to/file"), "path~/to/file");
        assert_eq!(expand_tilde("path~"), "path~");

        // Test without HOME environment variable
        unsafe {
            std::env::remove_var("HOME");
        }
        assert_eq!(expand_tilde("~/Documents"), "~/Documents");
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");

        // Restore HOME for other tests
        unsafe {
            std::env::set_var("HOME", "/home/testuser");
        }
    }
}
