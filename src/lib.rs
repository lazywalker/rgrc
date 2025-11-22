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

pub mod colorizer;
pub mod grc;

use std::fs::File;
use std::io::BufRead;
use std::str::FromStr;

use grc::{GrcConfigReader, GrcatConfigEntry, GrcatConfigReader};
use shellexpand;

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
///                      (e.g., "ping", "ls", "curl")
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
    File::open(path)
        .ok()
        .and_then(|f| {
            let bufreader = std::io::BufReader::new(f);
            let mut configreader = GrcConfigReader::new(bufreader.lines());
            // Find the first matching rule for this pseudo_command
            configreader
                .find(|(re, _config)| re.is_match(pseudo_command).unwrap_or(false))
                .map(|(_, config)| config)
        })
        .map(|config| {
            // Search all resource paths for the colorization file
            RESOURCE_PATHS
                .iter()
                .map(|path| shellexpand::tilde(path))
                .map(|path| format!("{}/{}", path, config))
                .flat_map(|filename| load_grcat_config(filename))
                .collect()
        })
        .unwrap_or_default()
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
///                Supports paths with ~ for home directory expansion
///                Can be a path within RESOURCE_PATHS directories
///                Example: "~/.config/rgrc/conf.ping"
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
    File::open(filename.as_ref())
        .ok()
        .map(|grcat_config_file| {
            let bufreader = std::io::BufReader::new(grcat_config_file);
            // Parse all rules from the configuration file
            let configreader = GrcatConfigReader::new(bufreader.lines());
            configreader.collect()
        })
        .unwrap_or_default()
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
///                      (e.g., "ping", "ls", "curl")
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
pub fn load_rules_for_command(pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    CONFIG_PATHS
        .iter()
        .map(|s| shellexpand::tilde(s))
        .flat_map(|s| load_config(s.as_ref(), pseudo_command))
        .collect()
}
