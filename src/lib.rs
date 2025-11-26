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
//! - **Submodules**: colorizer (text colorization), grc (config file parsing)
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

pub mod args;
pub mod buffer;
pub mod colorizer;
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
    for (filename, content) in EMBEDDED_CONFIGS {
        let file_path = conf_dir.join(filename);
        std::fs::write(file_path, content).ok()?;
    }

    Some(cache_dir)
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
/// These paths are searched in order when looking for colorization rule files
/// (grcat.conf) that define how to colorize output for specific commands.
///
/// The paths support:
/// - `~` expansion (home directory)
/// - XDG Base Directory Specification compliance
/// - System-wide configuration directories
///
/// # Search Order
///
/// 1. `~/.config/rgrc` - User's rgrc config directory (XDG_CONFIG_HOME)
/// 2. `~/.local/share/rgrc` - User's rgrc data directory (XDG_DATA_HOME)
/// 3. `/usr/local/share/rgrc` - System-wide custom installations
/// 4. `/usr/share/rgrc` - Standard system location (rgrc variant)
/// 5. `~/.config/grc` - Legacy grc user config directory
/// 6. `~/.local/share/grc` - Legacy grc user data directory
/// 7. `/usr/local/share/grc` - Legacy system-wide location
/// 8. `/usr/share/grc` - Standard grc location (original)
/// 5. `/usr/share/rgrc` - Standard system location (rgrc variant)
/// 6. `~/.config/grc` - Legacy grc user config directory
/// 7. `~/.local/share/grc` - Legacy grc user data directory
/// 8. `/usr/local/share/grc` - Legacy system-wide location
/// 9. `/usr/share/grc` - Standard grc location (original)
///
/// # Examples
///
/// All paths in RESOURCE_PATHS are searched when loading configuration:
/// ```ignore
/// let config_entries = load_config("~/.config/rgrc/grc.conf", "ping");
/// // This will search in all RESOURCE_PATHS directories for matching rules
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
/// The function searches RESOURCE_PATHS for the referenced colorization files.
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
/// # Errors Handled
///
/// All errors are silently handled and result in empty or partial rule sets:
/// - File not found → returns empty vector
/// - Invalid regex → pattern not matched → returns empty vector
/// - Invalid colorization file path → skipped in collection
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
/// 5. Searches all RESOURCE_PATHS directories for the colorization file
/// 6. Loads rules from all found colorization files
pub fn load_config(path: &str, pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // First, try to load from filesystem config file
    let filesystem_result = File::open(path).ok().and_then(|f| {
        let bufreader = std::io::BufReader::new(f);
        let mut configreader = GrcConfigReader::new(bufreader.lines());
        // Find the first matching rule for this pseudo_command
        configreader
            .find(|(re, _config)| re.is_match(pseudo_command).unwrap_or(false))
            .map(|(_, config)| config)
    });

    if let Some(config) = filesystem_result {
        // Search all resource paths for the colorization file
        return RESOURCE_PATHS
            .iter()
            .map(|path| expand_tilde(path))
            .map(|path| format!("{}/{}", path, config))
            .flat_map(load_grcat_config)
            .collect();
    }

    // No configuration found
    Vec::new()
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
/// It returns a combined vector of all matching rules found across all paths.
///
/// # Arguments
///
/// * `pseudo_command` - The command string to match against configuration rules
///   (e.g., "ping", "ls", "curl")
///
/// # Returns
///
/// A vector of `GrcatConfigEntry` containing all colorization rules that apply
/// to the given pseudo-command. Rules from multiple configuration files are combined.
///
/// # Examples
///
/// ```ignore
/// let rules = load_rules_for_command("ping");
/// // Now rules contains all colorization rules for ping from all config files
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

    // Fallback to file system configuration paths
    CONFIG_PATHS
        .iter()
        .map(|s| expand_tilde(s))
        .flat_map(|s| load_config(s.as_ref(), pseudo_command))
        .collect()
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
        if let Some((_, config_file)) =
            configreader.find(|(re, _)| re.is_match(pseudo_command).unwrap_or(false))
        {
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
            unsafe { std::env::set_var("HOME", td.path()); }

            // Re-run loading after setting HOME to the tempdir-backed cache
            let rules_after = load_rules_for_command("ping");

            // Restore HOME for subsequent tests
            if let Some(h) = prev_home {
                unsafe { std::env::set_var("HOME", h); }
            } else {
                unsafe { std::env::remove_var("HOME"); }
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
