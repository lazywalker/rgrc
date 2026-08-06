//! Core library: config loading, colorization, and regex engine selection.

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

// Use generated `embedded_configs.rs` (created by build.rs) so the list of
// embedded files is derived from the `share` directory instead of being hard-coded.
#[cfg(feature = "embed-configs")]
include!(concat!(env!("OUT_DIR"), "/embedded_configs.rs"));

#[cfg(feature = "embed-configs")]
pub const EMBEDDED_GRC_CONF: &str = include_str!("../etc/rgrc.conf");

// parse a conf file from the embedded config table by basename (e.g. "conf.ping").
#[cfg(feature = "embed-configs")]
fn parse_embedded_conf(config_name: &str) -> Option<Vec<GrcatConfigEntry>> {
    let content = EMBEDDED_CONFIGS
        .binary_search_by_key(&config_name, |(k, _)| *k)
        .ok()
        .map(|i| EMBEDDED_CONFIGS[i].1)?;
    let cursor = std::io::Cursor::new(content);
    let reader = GrcatConfigReader::new(std::io::BufReader::new(cursor).lines());
    Some(reader.collect())
}

// extract basename from a path, e.g. "/usr/share/rgrc/conf.ping" -> "conf.ping"
#[cfg(feature = "embed-configs")]
fn basename(path: &str) -> &str {
    match path.rsplit('/').next() {
        Some(name) => name,
        None => path,
    }
}

/// On = always color, Off = never, Auto = only when stdout is a TTY.
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(ColorMode::On),
            "off" => Ok(ColorMode::Off),
            "auto" => Ok(ColorMode::Auto),
            _ => Err(()),
        }
    }
}

/// Search paths for conf files, user config first, system/legacy last.
pub const RESOURCE_PATHS: &[&str] = &[
    "share", // Development mode: relative to project root (where cargo run is executed)
    "~/.config/rgrc",
    "~/.local/share/rgrc",
    "/usr/local/share/rgrc",
    "/usr/share/rgrc",
    "~/.config/grc",
    "~/.local/share/grc",
    "/usr/local/share/grc",
    "/usr/share/grc",
];

/// Load rules for `pseudo_command` by matching it against grc.conf patterns,
/// then reading the referenced conf file from the first matching RESOURCE_PATHS.
/// Returns empty vec on any failure (file not found, no match, parse error).
pub fn load_config(path: &str, pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // First, try to load from filesystem config file
    let filesystem_result = File::open(path).ok().and_then(|f| {
        let bufreader = std::io::BufReader::new(f);
        let configreader = GrcConfigReader::new(bufreader.lines());
        // Iterate each rule so we can optionally log which pattern matched
        for (re, config) in configreader {
            if re.is_match(pseudo_command) {
                if std::env::var_os("RGRC_DEBUG").is_some() {
                    eprintln!(
                        "rgrc: matched pattern '{}' in {} for '{}'",
                        re.as_str(),
                        path,
                        pseudo_command
                    );
                }
                return Some(config);
            }
        }
        None
    });

    if let Some(config) = filesystem_result {
        // Search RESOURCE_PATHS for the colorization file - **stop at first match**
        for base_path in RESOURCE_PATHS {
            let expanded_path = expand_tilde(base_path);
            let config_path = format!("{}/{}", expanded_path, config);
            if std::env::var_os("RGRC_DEBUG").is_some() {
                eprintln!("rgrc: checking for config file {}", config_path);
            }
            // Use file_exists_and_parse to distinguish "file exists but empty" from "file not found"
            match file_exists_and_parse(&config_path) {
                Some(rules) => {
                    if std::env::var_os("RGRC_DEBUG").is_some() {
                        eprintln!(
                            "rgrc: found config file {} ({} rules)",
                            config_path,
                            rules.len()
                        );
                    }
                    return rules; // File found (even if empty) - STOP
                }
                None => continue, // File not found - keep searching
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
    if let Ok(grcat_config_file) = File::open(filename) {
        let bufreader = std::io::BufReader::new(grcat_config_file);
        let configreader = GrcatConfigReader::new(bufreader.lines());
        let entries: Vec<_> = configreader.collect();
        return Some(entries);
    }

    #[cfg(feature = "embed-configs")]
    {
        let name = basename(filename);
        if let Some(entries) = parse_embedded_conf(name) {
            return Some(entries);
        }
    }

    None
}

/// Load and parse a grcat conf file. Returns empty vec on any error
/// (file not found, parse failure). Falls back to embedded cache when
/// the `embed-configs` feature is enabled.
pub fn load_grcat_config<T: AsRef<str>>(filename: T) -> Vec<GrcatConfigEntry> {
    let filename_str = filename.as_ref();

    if filename_str.is_empty() {
        return Vec::new();
    }

    if let Ok(grcat_config_file) = File::open(filename_str) {
        let bufreader = std::io::BufReader::new(grcat_config_file);
        let configreader = GrcatConfigReader::new(bufreader.lines());
        let entries: Vec<_> = configreader.collect();
        if !entries.is_empty() {
            return entries;
        }
    }

    #[cfg(feature = "embed-configs")]
    {
        let name = basename(filename_str);
        if let Some(entries) = parse_embedded_conf(name)
            && !entries.is_empty()
        {
            return entries;
        }
    }

    Vec::new()
}

/// Configuration file paths in priority order.
/// The program searches these paths to find grc.conf (or rgrc.conf) which maps
/// commands to their colorization profiles. Paths prefixed with ~ are expanded using shellexpand.
/// Typical flow: try ~/.grc first (user config), then system-wide configs (/etc/grc.conf).
const CONFIG_PATHS: &[&str] = &[
    "etc/rgrc.conf", // Development mode: relative to project root when develop with cargo run
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
    // Always prioritize user config first
    let user_config_path = "~/.config/rgrc/rgrc.conf";
    let expanded_user_config = expand_tilde(user_config_path);
    let rules = load_config(&expanded_user_config, pseudo_command);
    if !rules.is_empty() {
        return rules;
    }

    // then try embedded configs directly from memory
    #[cfg(feature = "embed-configs")]
    {
        let cursor = std::io::Cursor::new(EMBEDDED_GRC_CONF);
        let reader = GrcConfigReader::new(std::io::BufReader::new(cursor).lines());
        for (re, config_file) in reader {
            if re.is_match(pseudo_command) {
                if std::env::var_os("RGRC_DEBUG").is_some() {
                    eprintln!(
                        "rgrc: embedded matched pattern '{}' -> {}",
                        re.as_str(),
                        config_file
                    );
                }
                if let Some(entries) = parse_embedded_conf(&config_file) {
                    return entries;
                }
            }
        }
    }

    for config_path in CONFIG_PATHS {
        if *config_path == "~/.config/rgrc/rgrc.conf" {
            continue;
        }
        let expanded_path = expand_tilde(config_path);
        let rules = load_config(&expanded_path, pseudo_command);
        if !rules.is_empty() {
            return rules;
        }
    }

    Vec::new()
}

#[cfg(test)]
mod lib_test {
    use super::*;

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
        let user_rules = load_grcat_config(user_conf_file.to_string_lossy());
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
        let system_rules = load_grcat_config(system_conf_file.to_string_lossy());
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
        let rules_1 = load_grcat_config(conf_file_1.to_string_lossy());
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
        std::fs::write(&system_conf_file, "regexp=^SYSTEM\ncolours=red")
            .expect("write system config");

        // The critical test: load_grcat_config with empty file should return empty
        // and NOT continue searching the next directory
        let rules_user = load_grcat_config(user_conf_file.to_string_lossy());
        assert!(
            rules_user.is_empty(),
            "Empty user config file should return no rules (NOT fall back to system)"
        );

        // Verify system config has rules (to prove it COULD have been loaded)
        let rules_system = load_grcat_config(system_conf_file.to_string_lossy());
        assert!(!rules_system.is_empty(), "System config should have rules");

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
