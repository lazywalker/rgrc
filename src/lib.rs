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

// Include build-time generated preprocessed configs
include!(concat!(env!("OUT_DIR"), "/preprocessed_configs.rs"));

// Embedded configuration files
// These are compiled into the binary for cargo install compatibility
macro_rules! embed_config {
    ($name:expr) => {
        include_str!(concat!("../share/", $name))
    };
}

pub const EMBEDDED_CONFIGS: &[(&str, &str)] = &[
    ("conf.ant", embed_config!("conf.ant")),
    ("conf.blkid", embed_config!("conf.blkid")),
    ("conf.common", embed_config!("conf.common")),
    ("conf.configure", embed_config!("conf.configure")),
    ("conf.curl", embed_config!("conf.curl")),
    ("conf.cvs", embed_config!("conf.cvs")),
    ("conf.df", embed_config!("conf.df")),
    ("conf.diff", embed_config!("conf.diff")),
    ("conf.dig", embed_config!("conf.dig")),
    ("conf.dnf", embed_config!("conf.dnf")),
    ("conf.docker-machinels", embed_config!("conf.docker-machinels")),
    ("conf.dockerimages", embed_config!("conf.dockerimages")),
    ("conf.dockerinfo", embed_config!("conf.dockerinfo")),
    ("conf.dockernetwork", embed_config!("conf.dockernetwork")),
    ("conf.dockerps", embed_config!("conf.dockerps")),
    ("conf.dockerpull", embed_config!("conf.dockerpull")),
    ("conf.dockersearch", embed_config!("conf.dockersearch")),
    ("conf.dockerversion", embed_config!("conf.dockerversion")),
    ("conf.du", embed_config!("conf.du")),
    ("conf.dummy", embed_config!("conf.dummy")),
    ("conf.env", embed_config!("conf.env")),
    ("conf.esperanto", embed_config!("conf.esperanto")),
    ("conf.fdisk", embed_config!("conf.fdisk")),
    ("conf.findmnt", embed_config!("conf.findmnt")),
    ("conf.free", embed_config!("conf.free")),
    ("conf.gcc", embed_config!("conf.gcc")),
    ("conf.getfacl", embed_config!("conf.getfacl")),
    ("conf.getsebool", embed_config!("conf.getsebool")),
    ("conf.go-test", embed_config!("conf.go-test")),
    ("conf.id", embed_config!("conf.id")),
    ("conf.ifconfig", embed_config!("conf.ifconfig")),
    ("conf.iostat_sar", embed_config!("conf.iostat_sar")),
    ("conf.ip", embed_config!("conf.ip")),
    ("conf.ipaddr", embed_config!("conf.ipaddr")),
    ("conf.ipneighbor", embed_config!("conf.ipneighbor")),
    ("conf.iproute", embed_config!("conf.iproute")),
    ("conf.iptables", embed_config!("conf.iptables")),
    ("conf.irclog", embed_config!("conf.irclog")),
    ("conf.iwconfig", embed_config!("conf.iwconfig")),
    ("conf.jobs", embed_config!("conf.jobs")),
    ("conf.kubectl", embed_config!("conf.kubectl")),
    ("conf.last", embed_config!("conf.last")),
    ("conf.ldap", embed_config!("conf.ldap")),
    ("conf.log", embed_config!("conf.log")),
    ("conf.lolcat", embed_config!("conf.lolcat")),
    ("conf.ls", embed_config!("conf.ls")),
    ("conf.lsattr", embed_config!("conf.lsattr")),
    ("conf.lsblk", embed_config!("conf.lsblk")),
    ("conf.lsmod", embed_config!("conf.lsmod")),
    ("conf.lsof", embed_config!("conf.lsof")),
    ("conf.lspci", embed_config!("conf.lspci")),
    ("conf.lsusb", embed_config!("conf.lsusb")),
    ("conf.mount", embed_config!("conf.mount")),
    ("conf.mtr", embed_config!("conf.mtr")),
    ("conf.mvn", embed_config!("conf.mvn")),
    ("conf.netstat", embed_config!("conf.netstat")),
    ("conf.nmap", embed_config!("conf.nmap")),
    ("conf.ntpdate", embed_config!("conf.ntpdate")),
    ("conf.php", embed_config!("conf.php")),
    ("conf.ping", embed_config!("conf.ping")),
    ("conf.ping2", embed_config!("conf.ping2")),
    ("conf.proftpd", embed_config!("conf.proftpd")),
    ("conf.ps", embed_config!("conf.ps")),
    ("conf.pv", embed_config!("conf.pv")),
    ("conf.semanageboolean", embed_config!("conf.semanageboolean")),
    ("conf.semanagefcontext", embed_config!("conf.semanagefcontext")),
    ("conf.semanageuser", embed_config!("conf.semanageuser")),
    ("conf.sensors", embed_config!("conf.sensors")),
    ("conf.showmount", embed_config!("conf.showmount")),
    ("conf.sockstat", embed_config!("conf.sockstat")),
    ("conf.sql", embed_config!("conf.sql")),
    ("conf.ss", embed_config!("conf.ss")),
    ("conf.stat", embed_config!("conf.stat")),
    ("conf.sysctl", embed_config!("conf.sysctl")),
    ("conf.systemctl", embed_config!("conf.systemctl")),
    ("conf.tcpdump", embed_config!("conf.tcpdump")),
    ("conf.traceroute", embed_config!("conf.traceroute")),
    ("conf.tune2fs", embed_config!("conf.tune2fs")),
    ("conf.ulimit", embed_config!("conf.ulimit")),
    ("conf.uptime", embed_config!("conf.uptime")),
    ("conf.vmstat", embed_config!("conf.vmstat")),
    ("conf.wdiff", embed_config!("conf.wdiff")),
    ("conf.whois", embed_config!("conf.whois")),
    ("conf.yaml", embed_config!("conf.yaml")),
];

pub const EMBEDDED_GRC_CONF: &str = include_str!("../etc/rgrc.conf");

// Cached parsed configurations for performance - now truly persistent across calls
lazy_static::lazy_static! {
    static ref PARSED_EMBEDDED_CONFIGS: std::sync::RwLock<std::collections::HashMap<String, Vec<GrcatConfigEntry>>> = 
        std::sync::RwLock::new(std::collections::HashMap::new());
    
    static ref PARSED_EMBEDDED_GRC: Vec<fancy_regex::Regex> = {
        PRECOMPILED_GRC_RULES.iter()
            .filter_map(|(regex_str, _)| fancy_regex::Regex::new(regex_str).ok())
            .collect()
    };
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
/// - **Test mode**: Includes `./share` for CI/testing environments
///
/// # Search Order
///
/// ## Normal Mode
/// 1. `~/.config/rgrc` - User's rgrc config directory (XDG_CONFIG_HOME)
/// 2. `~/.local/share/rgrc` - User's rgrc data directory (XDG_DATA_HOME)
/// 3. `/usr/local/share/rgrc` - System-wide custom installations
/// 4. `/usr/share/rgrc` - Standard system location (rgrc variant)
/// 5. `~/.config/grc` - Legacy grc user config directory
/// 6. `~/.local/share/grc` - Legacy grc user data directory
/// 7. `/usr/local/share/grc` - Legacy system-wide location
/// 8. `/usr/share/grc` - Standard grc location (original)
///
/// ## Test Mode (when running `cargo test`)
/// 1. `./share` - **Project share directory (for CI/testing)**
/// 2. `~/.config/rgrc` - User's rgrc config directory (XDG_CONFIG_HOME)
/// 3. `~/.local/share/rgrc` - User's rgrc data directory (XDG_DATA_HOME)
/// 4. `/usr/local/share/rgrc` - System-wide custom installations
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
#[cfg(test)]
pub const RESOURCE_PATHS: &[&str] = &[
    "./share", // Test mode: include project share directory first
    "~/.config/rgrc",
    "~/.local/share/rgrc",
    "/usr/local/share/rgrc",
    "/usr/share/rgrc",
    "~/.config/grc",
    "~/.local/share/grc",
    "/usr/local/share/grc",
    "/usr/share/grc",
];

#[cfg(not(test))]
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
    // First, try to use embedded grc.conf
    let embedded_result = {
        let bufreader = std::io::BufReader::new(EMBEDDED_GRC_CONF.as_bytes());
        let mut configreader = GrcConfigReader::new(bufreader.lines());
        // Find the first matching rule for this pseudo_command
        configreader
            .find(|(re, _config)| re.is_match(pseudo_command).unwrap_or(false))
            .map(|(_, config)| config)
    };
    
    if let Some(config) = embedded_result {
        // Search all resource paths for the colorization file
        return RESOURCE_PATHS
            .iter()
            .map(|path| shellexpand::tilde(path))
            .map(|path| format!("{}/{}", path, config))
            .flat_map(load_grcat_config)
            .collect();
    }
    
    // Fallback to file system
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
                .flat_map(load_grcat_config)
                .collect()
        })
        .unwrap_or_default()
}/// Load colorization rules from a grcat.conf-style configuration file.
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
    
    // Extract config name from path (e.g., "conf.ping" from "/path/to/conf.ping")
    let config_name = filename_str
        .split('/')
        .last()
        .unwrap_or(filename_str);
    
    // First, try to find cached embedded config
    {
        let cache = PARSED_EMBEDDED_CONFIGS.read().unwrap();
        if let Some(cached_entries) = cache.get(config_name) {
            return cached_entries.clone();
        }
    }
    
    // Not in cache, try to find embedded config and parse it
    if let Some((_, embedded_content)) = EMBEDDED_CONFIGS.iter().find(|(name, _)| *name == config_name) {
        let bufreader = std::io::BufReader::new(embedded_content.as_bytes());
        let configreader = GrcatConfigReader::new(bufreader.lines());
        let entries: Vec<_> = configreader.collect();
        
        // Cache the parsed result
        let mut cache = PARSED_EMBEDDED_CONFIGS.write().unwrap();
        cache.insert(config_name.to_string(), entries.clone());
        
        return entries;
    }
    
    // Fallback to file system
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
/// **Test mode**: Includes `./etc/rgrc.conf` for CI/testing environments.
#[cfg(test)]
const CONFIG_PATHS: &[&str] = &[
    "./etc/rgrc.conf", // Test mode: include project etc directory first
    "~/.rgrc",
    "~/.config/rgrc/rgrc.conf",
    "/usr/local/etc/rgrc.conf",
    "/etc/rgrc.conf",
    "~/.grc",
    "~/.config/grc/grc.conf",
    "/usr/local/etc/grc.conf",
    "/etc/grc.conf",
];

#[cfg(not(test))]
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
    // First, try to load from embedded configuration
    let embedded_rules = load_config_from_embedded(pseudo_command);
    if !embedded_rules.is_empty() {
        return embedded_rules;
    }
    
    // Fallback to file system configuration paths
    CONFIG_PATHS
        .iter()
        .map(|s| shellexpand::tilde(s))
        .flat_map(|s| load_config(s.as_ref(), pseudo_command))
        .collect()
}

/// Load colorization rules from embedded configuration only.
/// This is a fast path that avoids file system access.
fn load_config_from_embedded(pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // Find the first matching rule for this pseudo_command in precompiled config
    for (i, (_, config_file)) in PRECOMPILED_GRC_RULES.iter().enumerate() {
        if let Some(regex) = PARSED_EMBEDDED_GRC.get(i) {
            if regex.is_match(pseudo_command).unwrap_or(false) {
                // Load the corresponding embedded grcat config
                return load_grcat_config_from_embedded(config_file);
            }
        }
    }
    Vec::new()
}

/// Load grcat config from embedded configs only (no file system fallback).
fn load_grcat_config_from_embedded(config_name: &str) -> Vec<GrcatConfigEntry> {
    // Try to find cached embedded config first
    {
        let cache = PARSED_EMBEDDED_CONFIGS.read().unwrap();
        if let Some(cached_entries) = cache.get(config_name) {
            return cached_entries.clone();
        }
    }
    
    // Not in cache, find embedded config and parse it
    if let Some((_, embedded_content)) = EMBEDDED_CONFIGS.iter().find(|(name, _)| *name == config_name) {
        let bufreader = std::io::BufReader::new(embedded_content.as_bytes());
        let configreader = GrcatConfigReader::new(bufreader.lines());
        let entries: Vec<_> = configreader.collect();
        
        // Cache the parsed result
        let mut cache = PARSED_EMBEDDED_CONFIGS.write().unwrap();
        cache.insert(config_name.to_string(), entries.clone());
        
        return entries;
    }
    
    Vec::new()
}

// Note: These tests are documentation-based since the main() function
// cannot be directly tested. The actual behavior would need to be tested
// through integration tests that run the binary.
#[cfg(test)]
#[test]
fn test_load_rules_for_command() {
    // Test loading rules for a known command that should have configuration
    let rules = load_rules_for_command("ping");

    // Since we have embedded rgrc.conf and conf.ping, we should get some rules
    // The exact number may vary, but it should be non-empty for a common command
    assert!(!rules.is_empty(), "Should load rules for ping command from embedded configs");

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
        
        // Should be reasonably fast (< 50ms per call in release mode)
        println!("Average time to load ping rules: {:?}", avg_time);
        assert!(avg_time.as_millis() < 50, "Loading rules should be reasonably fast (< 50ms)");
    }
}
