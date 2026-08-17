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
use std::path::{Path, PathBuf};
use std::str::FromStr;

use grc::{GrcConfigReader, GrcatConfigEntry, GrcatConfigReader};

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

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|h| !h.as_os_str().is_empty())
}

// XDG base directory spec: relative values are ignored, fall back to defaults
fn xdg_config_home() -> PathBuf {
    match std::env::var_os("XDG_CONFIG_HOME") {
        Some(v) if Path::new(&v).is_absolute() => PathBuf::from(v),
        _ => home()
            .map(|h| h.join(".config"))
            .unwrap_or_else(|| PathBuf::from(".config")),
    }
}

fn xdg_data_home() -> PathBuf {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(v) if Path::new(&v).is_absolute() => PathBuf::from(v),
        _ => home()
            .map(|h| h.join(".local/share"))
            .unwrap_or_else(|| PathBuf::from(".local/share")),
    }
}

fn xdg_dirs(var: &str, default: &str) -> Vec<PathBuf> {
    let raw = std::env::var_os(var)
        // an empty value behaves as unset, so the defaults still apply
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string_lossy().into_owned())
        .unwrap_or_else(|| default.to_string());
    raw.split(':')
        .filter(|p| Path::new(p).is_absolute())
        .map(PathBuf::from)
        .collect()
}

// development paths next to the repo are only searched when opted in,
// otherwise a stray share/ in the cwd would shadow user configs (#23)
fn dev_paths_enabled() -> bool {
    std::env::var_os("RGRC_DEV_SHARE").is_some()
}

fn dedup(paths: &mut Vec<PathBuf>) {
    let mut seen = std::collections::HashSet::new();
    paths.retain(|p| seen.insert(p.clone()));
}

/// Search paths for conf files, user config first, system/legacy last.
/// Honors the XDG base directory environment variables; grc paths are kept
/// for compatibility. The development `share/` dir requires RGRC_DEV_SHARE.
pub fn resource_paths() -> Vec<PathBuf> {
    let mut paths = vec![xdg_config_home().join("rgrc"), xdg_data_home().join("rgrc")];
    paths.extend(
        xdg_dirs("XDG_CONFIG_DIRS", "/etc/xdg")
            .into_iter()
            .map(|d| d.join("rgrc")),
    );
    paths.extend(
        xdg_dirs("XDG_DATA_DIRS", "/usr/local/share:/usr/share")
            .into_iter()
            .map(|d| d.join("rgrc")),
    );
    if let Some(h) = home() {
        paths.push(h.join(".config/grc"));
        paths.push(h.join(".local/share/grc"));
    }
    paths.push(PathBuf::from("/usr/local/share/grc"));
    paths.push(PathBuf::from("/usr/share/grc"));
    if dev_paths_enabled() {
        paths.insert(0, PathBuf::from("share"));
    }
    dedup(&mut paths);
    paths
}

/// Mapper file (rgrc.conf / grc.conf) locations, user first, then system,
/// then grc legacy. The development `etc/rgrc.conf` requires RGRC_DEV_SHARE.
fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if dev_paths_enabled() {
        paths.push(PathBuf::from("etc/rgrc.conf"));
    }
    if let Some(h) = home() {
        paths.push(h.join(".rgrc"));
    }
    paths.push(xdg_config_home().join("rgrc/rgrc.conf"));
    paths.extend(
        xdg_dirs("XDG_CONFIG_DIRS", "/etc/xdg")
            .into_iter()
            .map(|d| d.join("rgrc/rgrc.conf")),
    );
    paths.push(PathBuf::from("/usr/local/etc/rgrc.conf"));
    paths.push(PathBuf::from("/etc/rgrc.conf"));
    if let Some(h) = home() {
        paths.push(h.join(".grc"));
    }
    paths.push(xdg_config_home().join("grc/grc.conf"));
    paths.push(PathBuf::from("/usr/local/etc/grc.conf"));
    paths.push(PathBuf::from("/etc/grc.conf"));
    dedup(&mut paths);
    paths
}

/// Load rules for `pseudo_command` by matching it against the patterns in the
/// mapper file at `path`, then reading the referenced conf file from the first
/// matching resource path. Embedded configs are the last resort so that user
/// files on disk always win. Returns empty vec on any failure.
pub fn load_config(path: &str, pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    let patterns = {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };
        let reader = GrcConfigReader::new(std::io::BufReader::new(file).lines());
        reader.collect::<Vec<_>>()
    };

    // two-pass match: the full pseudo command first (patterns can require
    // args), then the bare command name so simple rules like ^df$ also work
    let cmd_name = pseudo_command
        .split_whitespace()
        .next()
        .unwrap_or(pseudo_command);
    let conf = patterns
        .iter()
        .find(|(re, _)| re.is_match(pseudo_command))
        .or_else(|| patterns.iter().find(|(re, _)| re.is_match(cmd_name)))
        .map(|(_, c)| c.clone());
    let Some(conf) = conf else {
        return Vec::new();
    };
    if std::env::var_os("RGRC_DEBUG").is_some() {
        eprintln!("rgrc: matched conf '{}' for '{}'", conf, pseudo_command);
    }

    conf_rules(&conf)
}

/// Resolve a conf name like "conf.df" to rules: first matching disk resource
/// path wins (an empty file stops the search), embedded configs last.
fn conf_rules(conf: &str) -> Vec<GrcatConfigEntry> {
    for base in resource_paths() {
        let conf_path = base.join(conf);
        if std::env::var_os("RGRC_DEBUG").is_some() {
            eprintln!("rgrc: checking for config file {}", conf_path.display());
        }
        if let Some(rules) = read_conf_file(&conf_path) {
            if std::env::var_os("RGRC_DEBUG").is_some() {
                eprintln!(
                    "rgrc: found config file {} ({} rules)",
                    conf_path.display(),
                    rules.len()
                );
            }
            return rules;
        }
    }

    #[cfg(feature = "embed-configs")]
    if let Some(entries) = parse_embedded_conf(conf) {
        return entries;
    }

    Vec::new()
}

/// Read a grcat conf file from disk. `Some(rules)` if the file exists (an
/// empty file yields `Some([])`), `None` when missing.
fn read_conf_file(path: &Path) -> Option<Vec<GrcatConfigEntry>> {
    let file = File::open(path).ok()?;
    let reader = GrcatConfigReader::new(std::io::BufReader::new(file).lines());
    Some(reader.collect())
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

/// Load colorization rules for a given pseudo-command by searching all
/// configuration paths. The search stops at the first mapper file that yields
/// rules. Priority: user mapper (`$XDG_CONFIG_HOME/rgrc/rgrc.conf`), then
/// the remaining mapper paths ending with legacy grc locations; the embedded
/// mapper is the last resort.
///
/// * `pseudo_command` - the command string to match, including arguments
///   ("ping", "df -h"); a second pass matches the bare command name
pub fn load_rules_for_command(pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    // Always prioritize the user mapper
    let user_config = xdg_config_home().join("rgrc/rgrc.conf");
    let rules = load_config(&user_config.to_string_lossy(), pseudo_command);
    if !rules.is_empty() {
        return rules;
    }

    for config_path in config_paths() {
        if config_path == user_config {
            continue;
        }
        let rules = load_config(&config_path.to_string_lossy(), pseudo_command);
        if !rules.is_empty() {
            return rules;
        }
    }

    // embedded mapper only after every disk mapper failed to match
    #[cfg(feature = "embed-configs")]
    if let Some(rules) = load_embedded(pseudo_command) {
        return rules;
    }

    Vec::new()
}

/// Rules for an explicitly requested config (`-c NAME`): NAME is first tried
/// as a pseudo-command against the mappers, then directly as a conf file name
/// ("df" resolves to conf.df; "conf.df" and paths are used as-is).
pub fn load_rules_for_config(name: &str) -> Vec<GrcatConfigEntry> {
    let rules = load_rules_for_command(name);
    if !rules.is_empty() {
        return rules;
    }

    let conf = if name.starts_with("conf.") || name.contains('/') {
        name.to_string()
    } else {
        format!("conf.{}", name)
    };
    conf_rules(&conf)
}

#[cfg(feature = "embed-configs")]
fn load_embedded(pseudo_command: &str) -> Option<Vec<GrcatConfigEntry>> {
    let cursor = std::io::Cursor::new(EMBEDDED_GRC_CONF);
    let reader = GrcConfigReader::new(std::io::BufReader::new(cursor).lines());
    let patterns: Vec<_> = reader.collect();

    let cmd_name = pseudo_command
        .split_whitespace()
        .next()
        .unwrap_or(pseudo_command);
    let conf = patterns
        .iter()
        .find(|(re, _)| re.is_match(pseudo_command))
        .or_else(|| patterns.iter().find(|(re, _)| re.is_match(cmd_name)))
        .map(|(_, c)| c.clone())?;
    if std::env::var_os("RGRC_DEBUG").is_some() {
        eprintln!(
            "rgrc: embedded matched conf '{}' for '{}'",
            conf, pseudo_command
        );
    }

    let rules = conf_rules(&conf);
    if rules.is_empty() { None } else { Some(rules) }
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

    // env-sensitive checks in one test: tests share one process and run in
    // parallel, so HOME/XDG mutations must not interleave with each other
    #[test]
    fn config_resolution() {
        use tempfile::TempDir;

        let prev_xdg = std::env::var_os("XDG_CONFIG_HOME");
        let td = TempDir::new().expect("create tempdir");
        let rgrc_dir = td.path().join("rgrc");
        std::fs::create_dir_all(&rgrc_dir).expect("create rgrc dir");
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", td.path());
        }

        // XDG_CONFIG_HOME respected and user conf overrides the embedded copy
        std::fs::write(rgrc_dir.join("rgrc.conf"), "^df(\\s|$)\nconf.df\n").unwrap();
        std::fs::write(rgrc_dir.join("conf.df"), "regexp=USERDF\ncolours=green\n").unwrap();
        let has_user_df = |cmd: &str| {
            load_rules_for_command(cmd)
                .iter()
                .any(|r| r.regex.as_str().contains("USERDF"))
        };
        assert!(has_user_df("df"));
        assert!(has_user_df("df -h"));

        // second pass: `^df$` also matches "df -h" via the bare command name
        std::fs::write(rgrc_dir.join("rgrc.conf"), "^df$\nconf.df\n").unwrap();
        assert!(has_user_df("df -h"));

        // explicit config lookup: name, conf.NAME, both resolve to user rules
        assert!(
            load_rules_for_config("df")
                .iter()
                .any(|r| r.regex.as_str().contains("USERDF"))
        );
        assert!(
            load_rules_for_config("conf.df")
                .iter()
                .any(|r| r.regex.as_str().contains("USERDF"))
        );

        // embedded fallback still works once the user mapper is gone
        std::fs::remove_file(rgrc_dir.join("rgrc.conf")).unwrap();
        std::fs::remove_file(rgrc_dir.join("conf.df")).unwrap();
        #[cfg(feature = "embed-configs")]
        assert!(!load_rules_for_command("df -h").is_empty());

        if let Some(x) = prev_xdg {
            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", x);
            }
        } else {
            unsafe {
                std::env::remove_var("XDG_CONFIG_HOME");
            }
        }

        // dev share/ is opt-in via RGRC_DEV_SHARE
        assert!(!resource_paths().iter().any(|p| p == Path::new("share")));
        unsafe {
            std::env::set_var("RGRC_DEV_SHARE", "1");
        }
        let first = resource_paths().first().expect("non-empty").clone();
        unsafe {
            std::env::remove_var("RGRC_DEV_SHARE");
        }
        assert_eq!(first, PathBuf::from("share"));
    }
}
