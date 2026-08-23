// every alias-whitelist entry must be mapped by the embedded rgrc.conf;
// catches entries like "dummy" that can never colorize anything
#[test]
#[cfg(feature = "embed-configs")]
fn whitelist_entries_are_mapped() {
    use std::io::{BufRead, Cursor};

    let reader = rgrc::grc::GrcConfigReader::new(
        std::io::BufReader::new(Cursor::new(rgrc::EMBEDDED_GRC_CONF)).lines(),
    );
    let patterns: Vec<_> = reader.collect();

    for cmd in rgrc::utils::SUPPORTED_COMMANDS {
        // the command name appears as a literal token in some pattern
        // (covers arg- or subcommand-gated mappings like `fdisk -l` or
        // `docker ps`), or a probe matches outright
        let named = patterns.iter().any(|(re, _)| {
            re.as_str()
                .split(|c: char| !c.is_alphanumeric())
                .any(|t| t == *cmd)
        });
        let probes = [(*cmd).to_string(), format!("{cmd} x")];
        let matched = probes
            .iter()
            .any(|p| patterns.iter().any(|(re, _)| re.is_match(p)));
        assert!(
            named || matched,
            "{cmd} is in SUPPORTED_COMMANDS but no rgrc.conf pattern mentions it"
        );
    }
}

#[test]
fn test_load_grcat_config_nonexistent_file() {
    let result = rgrc::load_grcat_config("/nonexistent/path/file.conf");
    assert!(
        result.is_empty(),
        "Should return empty vector for nonexistent file"
    );
}

#[test]
fn test_load_grcat_config_empty_path() {
    let result = rgrc::load_grcat_config("");
    assert!(
        result.is_empty(),
        "Should return empty vector for empty path"
    );
}

#[test]
fn test_load_grcat_config_with_tilde_path() {
    let result = rgrc::load_grcat_config("~/nonexistent.conf");
    let _ = result;
}

#[test]
#[ignore = "Skip this test as it might hang when reading a directory"]
fn test_load_grcat_config_handles_directories() {
    let result = rgrc::load_grcat_config("share");
    assert!(result.is_empty());
}

#[test]
#[ignore = "Skip this test as relative paths might cause hangs"]
fn test_load_grcat_config_with_relative_paths() {
    let result = rgrc::load_grcat_config(".");
    let _ = result;

    let result2 = rgrc::load_grcat_config("..");
    let _ = result2;
}

#[test]
fn test_load_config_nonexistent_config_file() {
    let result = rgrc::load_config("/nonexistent/grc.conf", "ls");
    assert!(
        result.is_empty(),
        "Should return empty vector when config file doesn't exist"
    );
}

#[test]
fn test_load_config_empty_config_path() {
    let result = rgrc::load_config("", "any_command");
    assert!(
        result.is_empty(),
        "Should return empty for empty config path"
    );
}

#[test]
fn test_load_config_empty_command() {
    let result = rgrc::load_config("/nonexistent/path", "");
    assert!(
        result.is_empty(),
        "Should return empty vector when given an empty command"
    );
}

#[test]
fn test_color_mode_invalid_inputs() {
    use std::str::FromStr;

    assert!(
        rgrc::ColorMode::from_str("ON").is_err(),
        "Should be case-sensitive"
    );
    assert!(rgrc::ColorMode::from_str("invalid").is_err());
    assert!(rgrc::ColorMode::from_str("").is_err());
    assert!(rgrc::ColorMode::from_str("ye").is_err());
}

#[test]
fn test_color_mode_equality() {
    use std::str::FromStr;

    let mode1 = rgrc::ColorMode::from_str("on").unwrap();
    let mode2 = rgrc::ColorMode::from_str("on").unwrap();
    assert_eq!(mode1, mode2);
}

#[test]
fn test_color_mode_all_variants() {
    use std::str::FromStr;

    let on = rgrc::ColorMode::from_str("on").unwrap();
    let off = rgrc::ColorMode::from_str("off").unwrap();
    let auto = rgrc::ColorMode::from_str("auto").unwrap();

    assert_ne!(on, off);
    assert_ne!(on, auto);
    assert_ne!(off, auto);
}

#[test]
fn test_color_mode_debug_output() {
    use std::str::FromStr;

    let mode = rgrc::ColorMode::from_str("on").unwrap();
    let debug_str = format!("{:?}", mode);
    assert_eq!(debug_str, "On");
}

#[test]
fn test_load_grcat_config_multiple_calls() {
    let result1 = rgrc::load_grcat_config("/nonexistent");
    let result2 = rgrc::load_grcat_config("/nonexistent");

    assert_eq!(result1.len(), result2.len());
    assert!(result1.is_empty() && result2.is_empty());
}

#[test]
fn test_resource_paths_constant() {
    let paths = rgrc::resource_paths();

    assert!(!paths.is_empty(), "resource_paths() should not be empty");

    let has_xdg_config = paths.iter().any(|p| p.ends_with("rgrc"));
    let has_system_paths = paths.iter().any(|p| p.starts_with("/"));
    let has_grc_compat = paths.iter().any(|p| p.ends_with("grc"));

    assert!(has_xdg_config, "Should contain rgrc paths");
    assert!(has_system_paths, "Should contain system paths (/)");
    assert!(has_grc_compat, "Should contain grc compat paths");
}

#[test]
fn test_resource_paths_no_empty_entries() {
    let paths = rgrc::resource_paths();

    for path in paths {
        assert!(
            !path.as_os_str().is_empty(),
            "resource_paths() should not contain empty entries"
        );
    }
}

#[test]
fn test_resource_paths_valid_format() {
    let paths = rgrc::resource_paths();

    for path in paths {
        assert!(
            !path.to_string_lossy().starts_with('~'),
            "Paths should be tilde-expanded: {}",
            path.display()
        );
    }
}

#[test]
fn test_load_config_with_pseudo_command() {
    let test_commands = vec!["ls", "grep", "curl -i https://example.com", "docker ps"];

    for cmd in test_commands {
        let result = rgrc::load_config("/nonexistent", cmd);
        let _ = result; // Just ensure it doesn't panic
    }
}

#[test]
fn test_color_mode_copy_semantics() {
    use std::str::FromStr;

    let mode1 = rgrc::ColorMode::from_str("on").unwrap();
    let mode2 = mode1; // Copy semantics

    assert_eq!(mode1, mode2);
}

#[test]
fn test_color_mode_clone_semantics() {
    use std::str::FromStr;

    let mode1 = rgrc::ColorMode::from_str("off").unwrap();
    let mode2 = mode1; // Copy semantics; no need to clone a Copy type

    assert_eq!(mode1, mode2);
}

#[cfg(feature = "embed-configs")]
mod embed_configs_tests {
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::{NamedTempFile, TempDir};

    static HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_home<R>(f: impl FnOnce() -> R) -> R {
        let _guard = HOME_LOCK.lock().expect("HOME_LOCK mutex poisoned");
        let td = TempDir::new().expect("create tempdir");
        let prev_home = std::env::var_os("HOME");

        let td_path = td.path().to_str().expect("tempdir path is valid utf8");
        unsafe {
            std::env::set_var("HOME", td_path);
        }

        let res = f();

        if let Some(h) = prev_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }

        res
    }

    #[test]
    fn test_embed_configs_filesystem_priority() {
        let mut temp_grc = NamedTempFile::new().unwrap();
        writeln!(temp_grc, r#"^test_command$"#).unwrap();
        writeln!(temp_grc, "conf.custom_test").unwrap();

        let mut temp_conf = NamedTempFile::new().unwrap();
        writeln!(temp_conf, "regexp=^custom_test_output$").unwrap();
        writeln!(temp_conf, "colours=red").unwrap();
        writeln!(temp_conf, "======").unwrap();

        let rules = rgrc::load_config(temp_grc.path().to_str().unwrap(), "test_command");

        let _ = rules; // Just ensure it doesn't panic
    }

    #[test]
    fn test_embed_configs_fallback_to_embedded() {
        let rules = with_temp_home(|| rgrc::load_rules_for_command("ping"));

        assert!(
            !rules.is_empty(),
            "Should fallback to embedded configs when filesystem config doesn't exist"
        );
    }

    // ~/.rgrc mapping wins over the embedded mapper (review on #34)
    #[test]
    fn test_legacy_mapper_overrides_embedded() {
        let _guard = HOME_LOCK.lock().expect("HOME_LOCK mutex poisoned");
        let td = TempDir::new().expect("create tempdir");
        let prev_home = std::env::var_os("HOME");
        let prev_xdg = std::env::var_os("XDG_CONFIG_HOME");
        let xdg_empty = td.path().join("xdgempty");
        std::fs::create_dir_all(&xdg_empty).expect("create empty xdg dir");

        unsafe {
            std::env::set_var("HOME", td.path());
            std::env::set_var("XDG_CONFIG_HOME", &xdg_empty);
        }

        std::fs::write(td.path().join(".rgrc"), "^df(\\s|$)\nconf.legacydf\n").unwrap();
        let conf_dir = xdg_empty.join("rgrc");
        std::fs::create_dir_all(&conf_dir).expect("create rgrc dir");
        std::fs::write(
            conf_dir.join("conf.legacydf"),
            "regexp=LEGACYDF\ncolours=green\n",
        )
        .unwrap();

        let used_legacy = rgrc::load_rules_for_command("df -h")
            .iter()
            .any(|r| r.regex.as_str().contains("LEGACYDF"));

        if let Some(h) = prev_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        if let Some(x) = prev_xdg {
            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", x);
            }
        } else {
            unsafe {
                std::env::remove_var("XDG_CONFIG_HOME");
            }
        }

        assert!(
            used_legacy,
            "~/.rgrc mapping should win over the embedded mapper"
        );
    }

    #[test]
    fn test_embed_configs_grcat_filesystem_priority() {
        let mut temp_conf = NamedTempFile::new().unwrap();
        writeln!(temp_conf, "regexp=^filesystem_test$").unwrap();
        writeln!(temp_conf, "colours=blue").unwrap();
        writeln!(temp_conf, "======").unwrap();

        let rules = rgrc::load_grcat_config(temp_conf.path().to_str().unwrap());

        assert!(
            !rules.is_empty(),
            "Should load rules from filesystem when file exists"
        );

        assert_eq!(rules.len(), 1, "Should have exactly one rule");
        assert_eq!(
            rules[0].regex.as_str(),
            "^filesystem_test$",
            "Regex should match filesystem content"
        );
    }

    #[test]
    fn test_embed_configs_grcat_fallback_to_embedded() {
        let rules = with_temp_home(|| rgrc::load_grcat_config("conf.ping"));

        assert!(
            !rules.is_empty(),
            "Should fallback to embedded configs for conf.ping"
        );
    }

    #[test]
    fn test_cache_population_idempotent() {
        with_temp_home(|| {
            let rules1 = rgrc::load_rules_for_command("ping");
            assert!(!rules1.is_empty(), "First call should load rules for ping");

            let rules2 = rgrc::load_rules_for_command("ping");
            assert!(
                !rules2.is_empty(),
                "Second call should also load rules for ping"
            );

            assert_eq!(
                rules1.len(),
                rules2.len(),
                "Rule counts should be identical"
            );
            for (rule1, rule2) in rules1.iter().zip(rules2.iter()) {
                assert_eq!(
                    rule1.regex.as_str(),
                    rule2.regex.as_str(),
                    "Regex patterns should be identical"
                );
            }
        });
    }

    #[test]
    fn test_load_config_from_embedded_unknown_command() {
        let rules = rgrc::load_rules_for_command("definitely_not_a_real_command_12345");

        assert!(
            rules.is_empty(),
            "Should return empty rules for unknown commands"
        );
    }

    #[test]
    fn test_load_config_from_embedded_empty_command() {
        let rules = rgrc::load_rules_for_command("");

        assert!(
            rules.is_empty(),
            "Should return empty rules for empty command"
        );
    }

    #[test]
    fn test_cache_creation_failure_fallback() {
        let rules_normal = with_temp_home(|| rgrc::load_rules_for_command("ping"));
        assert!(!rules_normal.is_empty(), "Normal operation should work");

        let rules_fallback = rgrc::load_grcat_config("nonexistent_config_file");
        assert!(
            rules_fallback.is_empty(),
            "Should gracefully handle non-existent config files"
        );
    }

    #[test]
    fn test_resource_paths_priority_over_cache() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_file_path = temp_dir.path().join("conf.test_grcat_priority");
        let config_content = "regexp=^grcat_filesystem_test$\ncolours=red\n======\n";
        fs::write(&config_file_path, config_content).unwrap();

        let rules = rgrc::load_grcat_config(config_file_path.to_str().unwrap());

        assert!(
            !rules.is_empty(),
            "Should load rules from filesystem when file exists"
        );

        assert_eq!(rules.len(), 1, "Should have exactly one rule");
        assert_eq!(
            rules[0].regex.as_str(),
            "^grcat_filesystem_test$",
            "Regex should match filesystem content, not cache content"
        );

        drop(temp_dir);
    }

    #[test]
    fn test_config_paths_priority_over_cache() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("conf");
        fs::create_dir_all(&config_dir).unwrap();

        let custom_config_path = config_dir.join("conf.test_config_priority");
        let custom_config_content = "regexp=^config_path_test$\ncolours=blue\n======\n";
        fs::write(&custom_config_path, custom_config_content).unwrap();

        let mut temp_grc = NamedTempFile::new().unwrap();
        writeln!(temp_grc, r#"^test_config_priority$"#).unwrap();
        writeln!(temp_grc, "conf.test_config_priority").unwrap();

        let rules = rgrc::load_config(temp_grc.path().to_str().unwrap(), "test_config_priority");

        let _ = rules; // Just ensure it doesn't panic

        drop(temp_grc);
        drop(temp_dir);
    }
}

#[cfg(not(feature = "embed-configs"))]
mod no_embed_configs_tests {

    #[test]
    fn test_no_embed_configs_filesystem_only() {
        let rules = rgrc::load_config("/nonexistent/grc.conf", "ping");

        assert!(
            rules.is_empty(),
            "Should return empty when no embed-configs and filesystem config doesn't exist"
        );
    }

    #[test]
    fn test_no_embed_configs_grcat_filesystem_only() {
        let rules = rgrc::load_grcat_config("/nonexistent/conf.ping");

        assert!(
            rules.is_empty(),
            "Should return empty when no embed-configs and filesystem config doesn't exist"
        );
    }
}
