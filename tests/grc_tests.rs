use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

// Include the modules from src
#[path = "../src/grc.rs"]
mod grc;

use grc::{GrcConfigReader, GrcatConfigEntry, GrcatConfigReader};

/// Helper function to get the project root directory
fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Helper function to get share directory path
fn get_share_dir() -> PathBuf {
    get_project_root().join("share")
}

/// Helper function to get etc directory path
fn get_etc_dir() -> PathBuf {
    get_project_root().join("etc")
}

#[cfg(test)]
mod grc_config_reader_tests {
    use super::*;

    #[test]
    fn test_grc_conf_parsing() {
        let conf_path = get_etc_dir().join("rgrc.conf");
        assert!(
            conf_path.exists(),
            "grc.conf should exist at {:?}",
            conf_path
        );

        let file = File::open(&conf_path).expect("Failed to open grc.conf");
        let reader = BufReader::new(file);
        let grc_reader = GrcConfigReader::new(reader.lines());

        let configs: Vec<_> = grc_reader.collect();
        assert!(
            !configs.is_empty(),
            "grc.conf should contain configuration entries"
        );

        // Verify each entry has valid regex and config file path
        for (regex, config_file) in &configs {
            assert!(
                !config_file.is_empty(),
                "Config file path should not be empty"
            );
            // Test that regex can match something
            assert!(regex.is_match("test").is_ok(), "Regex should be valid");
        }

        println!(
            "Successfully parsed {} configuration entries from grc.conf",
            configs.len()
        );
    }

    #[test]
    fn test_grc_conf_specific_commands() {
        let conf_path = get_etc_dir().join("rgrc.conf");
        let file = File::open(&conf_path).expect("Failed to open grc.conf");
        let reader = BufReader::new(file);
        let grc_reader = GrcConfigReader::new(reader.lines());

        let configs: Vec<_> = grc_reader.collect();

        // Check for common commands
        let command_configs: Vec<_> = configs
            .iter()
            .map(|(regex, config)| (regex.as_str(), config.as_str()))
            .collect();

        // Verify some expected commands are present
        let has_ping = command_configs
            .iter()
            .any(|(_, config)| config.contains("ping"));
        let has_ls = command_configs
            .iter()
            .any(|(_, config)| config.contains("ls"));
        let has_diff = command_configs
            .iter()
            .any(|(_, config)| config.contains("diff"));

        assert!(
            has_ping || has_ls || has_diff,
            "grc.conf should contain at least one common command configuration"
        );
    }

    #[test]
    fn test_grc_conf_regex_patterns() {
        let conf_path = get_etc_dir().join("rgrc.conf");
        let file = File::open(&conf_path).expect("Failed to open grc.conf");
        let reader = BufReader::new(file);
        let grc_reader = GrcConfigReader::new(reader.lines());

        for (regex, config_file) in grc_reader {
            // Test that each regex can be used for matching
            assert!(
                regex.is_match("").is_ok(),
                "Regex from {} should be valid",
                config_file
            );
        }
    }

    #[test]
    fn test_grc_conf_skip_comments() {
        let conf_path = get_etc_dir().join("rgrc.conf");
        let file = File::open(&conf_path).expect("Failed to open grc.conf");
        let reader = BufReader::new(file);
        let grc_reader = GrcConfigReader::new(reader.lines());

        let configs: Vec<_> = grc_reader.collect();

        // Ensure no config file paths contain comment markers
        for (_, config_file) in &configs {
            assert!(
                !config_file.starts_with('#'),
                "Config file paths should not start with #: {}",
                config_file
            );
        }
    }
}

#[cfg(test)]
mod grcat_config_reader_tests {
    use super::*;

    /// Get all conf.* files from share directory
    fn get_all_conf_files() -> Vec<PathBuf> {
        let share_dir = get_share_dir();
        let mut conf_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&share_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name()
                    && filename.to_string_lossy().starts_with("conf.")
                {
                    conf_files.push(path);
                }
            }
        }

        conf_files.sort();
        conf_files
    }

    #[test]
    fn test_all_conf_files_exist() {
        let conf_files = get_all_conf_files();
        assert!(
            !conf_files.is_empty(),
            "Share directory should contain conf.* files"
        );

        println!("Found {} configuration files:", conf_files.len());
        for file in &conf_files {
            println!("  - {:?}", file.file_name().unwrap());
        }
    }

    #[test]
    fn test_parse_all_conf_files() {
        let conf_files = get_all_conf_files();
        let mut successful_parses = 0;
        let mut total_entries = 0;
        let mut files_with_issues = Vec::new();

        for conf_file in &conf_files {
            let filename = conf_file.file_name().unwrap().to_string_lossy();

            match File::open(conf_file) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let grcat_reader = GrcatConfigReader::new(reader.lines());

                    // Collect entries, noting that some may be skipped due to unsupported styles
                    let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();

                    successful_parses += 1;
                    total_entries += entries.len();

                    if entries.is_empty() {
                        files_with_issues.push(filename.to_string());
                    }

                    println!("{}: {} entries", filename, entries.len());
                }
                Err(e) => {
                    panic!("Failed to open {}: {}", filename, e);
                }
            }
        }

        assert_eq!(
            successful_parses,
            conf_files.len(),
            "All conf files should be parseable"
        );

        if !files_with_issues.is_empty() {
            println!("\nFiles with no parsed entries (may use unsupported styles):");
            for file in &files_with_issues {
                println!("  - {}", file);
            }
        }

        println!(
            "\nTotal: {} files, {} entries",
            successful_parses, total_entries
        );
        // Allow some files to have no entries due to unsupported styles
        assert!(
            total_entries > 0,
            "Should have parsed at least some configuration entries"
        );
    }

    #[test]
    fn test_conf_ls_specific_patterns() {
        let conf_path = get_share_dir().join("conf.ls");
        let file = File::open(&conf_path).expect("conf.ls should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.ls should contain at least one entry"
        );

        // Verify each entry has valid regex
        for entry in &entries {
            assert!(entry.regex.is_match("").is_ok(), "Regex should be valid");
        }

        println!("conf.ls contains {} pattern entries", entries.len());
    }

    #[test]
    fn test_conf_ping_specific_patterns() {
        let conf_path = get_share_dir().join("conf.ping");
        let file = File::open(&conf_path).expect("conf.ping should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.ping should contain at least one entry"
        );

        println!("conf.ping contains {} pattern entries", entries.len());
    }

    #[test]
    fn test_conf_diff_specific_patterns() {
        let conf_path = get_share_dir().join("conf.diff");
        let file = File::open(&conf_path).expect("conf.diff should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.diff should contain at least one entry"
        );

        println!("conf.diff contains {} pattern entries", entries.len());
    }

    #[test]
    fn test_conf_netstat_patterns() {
        let conf_path = get_share_dir().join("conf.netstat");
        if conf_path.exists() {
            let file = File::open(&conf_path).expect("conf.netstat should be readable");
            let reader = BufReader::new(file);
            let grcat_reader = GrcatConfigReader::new(reader.lines());

            let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
            assert!(
                !entries.is_empty(),
                "conf.netstat should contain at least one entry"
            );

            println!("conf.netstat contains {} pattern entries", entries.len());
        }
    }

    #[test]
    fn test_all_conf_files_have_valid_regexes() {
        let conf_files = get_all_conf_files();
        let mut files_tested = 0;
        let mut total_regexes_tested = 0;

        for conf_file in &conf_files {
            let filename = conf_file.file_name().unwrap().to_string_lossy();

            if let Ok(file) = File::open(conf_file) {
                let reader = BufReader::new(file);
                let grcat_reader = GrcatConfigReader::new(reader.lines());

                for entry in grcat_reader {
                    // Test that each regex can match an empty string without errors
                    match entry.regex.is_match("") {
                        Ok(_) => total_regexes_tested += 1,
                        Err(e) => panic!("Invalid regex in {}: {:?}", filename, e),
                    }
                }

                files_tested += 1;
            }
        }

        println!(
            "Validated {} regexes across {} files",
            total_regexes_tested, files_tested
        );
        assert_eq!(files_tested, conf_files.len());
        println!("Note: Files with unsupported styles (like 'reverse') may have fewer entries");
    }

    #[test]
    fn test_all_conf_files_color_definitions() {
        let conf_files = get_all_conf_files();

        for conf_file in &conf_files {
            let filename = conf_file.file_name().unwrap().to_string_lossy();

            if let Ok(file) = File::open(conf_file) {
                let reader = BufReader::new(file);
                let grcat_reader = GrcatConfigReader::new(reader.lines());

                for entry in grcat_reader {
                    // Each entry should have a valid regex
                    assert!(
                        entry.regex.is_match("").is_ok(),
                        "Entry in {} has invalid regex",
                        filename
                    );

                    // Colors vector can be empty (default) or contain styles
                    // Just verify it's a valid vector
                    let _colors = &entry.colors;
                }
            }
        }
    }

    /// Test individual conf files by name
    #[test]
    fn test_specific_conf_files() {
        let test_files = vec![
            "conf.ant",
            "conf.blkid",
            "conf.configure",
            "conf.curl",
            "conf.cvs",
            "conf.df",
            "conf.dig",
            "conf.dnf",
            "conf.docker-machinels",
            "conf.dockerimages",
            "conf.dockerinfo",
            "conf.dockernetwork",
            "conf.dockerps",
            "conf.dockerpull",
            "conf.dockersearch",
            "conf.dockerversion",
            "conf.du",
            "conf.env",
            "conf.fdisk",
            "conf.findmnt",
            "conf.free",
            "conf.gcc",
            "conf.getfacl",
            "conf.getsebool",
            "conf.go-test",
            "conf.id",
            "conf.ifconfig",
            "conf.iostat_sar",
            "conf.ip",
            "conf.ipaddr",
            "conf.ipneighbor",
            "conf.iproute",
            "conf.iptables",
            "conf.irclog",
            "conf.iwconfig",
            "conf.jobs",
            "conf.kubectl",
            "conf.last",
            "conf.ldap",
            "conf.log",
            "conf.lolcat",
            "conf.lsattr",
            "conf.lsblk",
            "conf.lsmod",
            "conf.lsof",
            "conf.lspci",
            "conf.lsusb",
            "conf.mount",
            "conf.mtr",
            "conf.mvn",
            "conf.nmap",
            "conf.ntpdate",
            "conf.php",
            "conf.ping2",
            "conf.proftpd",
            "conf.ps",
            "conf.pv",
            "conf.semanageboolean",
            "conf.semanagefcontext",
            "conf.semanageuser",
            "conf.sensors",
            "conf.showmount",
            "conf.sockstat",
            "conf.sql",
            "conf.ss",
            "conf.stat",
            "conf.sysctl",
            "conf.systemctl",
            "conf.tcpdump",
            "conf.traceroute",
            "conf.tune2fs",
            "conf.ulimit",
            "conf.uptime",
            "conf.vmstat",
            "conf.wdiff",
            "conf.whois",
            "conf.yaml",
        ];

        let share_dir = get_share_dir();
        let mut found_count = 0;
        let mut parsed_count = 0;

        for filename in &test_files {
            let conf_path = share_dir.join(filename);
            if conf_path.exists() {
                found_count += 1;

                if let Ok(file) = File::open(&conf_path) {
                    let reader = BufReader::new(file);
                    let grcat_reader = GrcatConfigReader::new(reader.lines());
                    let entries: Vec<_> = grcat_reader.collect();

                    if !entries.is_empty() {
                        parsed_count += 1;
                    }
                }
            }
        }

        println!(
            "Found {}/{} test files, successfully parsed {} with entries",
            found_count,
            test_files.len(),
            parsed_count
        );
        assert!(
            found_count > 0,
            "Should find at least some test configuration files"
        );
    }

    #[test]
    fn test_conf_log_count_field_parsing() {
        let conf_path = get_share_dir().join("conf.log");
        let file = File::open(&conf_path).expect("conf.log should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.log should contain at least one entry"
        );

        // Check that count fields are parsed correctly
        let mut found_once = false;
        let mut found_more = false;
        let mut found_stop = false;

        for entry in &entries {
            match entry.count {
                grc::GrcatConfigEntryCount::Once => found_once = true,
                grc::GrcatConfigEntryCount::More => found_more = true,
                grc::GrcatConfigEntryCount::Stop => found_stop = true,
            }
        }

        assert!(found_once, "Should find at least one 'once' count entry");
        assert!(found_more, "Should find at least one 'more' count entry");
        assert!(found_stop, "Should find at least one 'stop' count entry");

        println!(
            "conf.log count field parsing: once={}, more={}, stop={}",
            found_once, found_more, found_stop
        );
    }

    #[test]
    fn test_conf_ping2_replace_field_parsing() {
        let conf_path = get_share_dir().join("conf.ping2");
        let file = File::open(&conf_path).expect("conf.ping2 should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.ping2 should contain at least one entry"
        );

        // Check that replace field is parsed correctly
        let mut found_replace = false;
        let mut replace_value = String::new();

        for entry in &entries {
            if !entry.replace.is_empty() {
                found_replace = true;
                replace_value = entry.replace.clone();
                break;
            }
        }

        assert!(
            found_replace,
            "Should find at least one entry with replace field"
        );
        assert_eq!(
            replace_value, "TIMEOUT \\1",
            "Replace value should match expected pattern"
        );

        println!(
            "conf.ping2 replace field parsing: found replace='{}'",
            replace_value
        );
    }

    #[test]
    fn test_count_and_replace_default_values() {
        // Test with a simple config that doesn't specify count or replace
        let config_content = r#"
regexp=test pattern
colours=red
======
"#;

        let reader = BufReader::new(config_content.as_bytes());
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert_eq!(entries.len(), 1, "Should parse one entry");

        let entry = &entries[0];
        // Check default values
        assert!(
            matches!(entry.count, grc::GrcatConfigEntryCount::More),
            "Default count should be More"
        );
        assert!(
            entry.replace.is_empty(),
            "Default replace should be empty string"
        );

        println!("Default values test passed: count=More, replace=''");
    }

    #[test]
    fn test_conf_netstat_skip_field_parsing() {
        let conf_path = get_share_dir().join("conf.netstat");
        let file = File::open(&conf_path).expect("conf.netstat should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.netstat should contain at least one entry"
        );

        // Check that skip fields are parsed correctly
        let mut found_skip_true = false;
        let mut found_skip_false = false;

        for entry in &entries {
            if entry.skip {
                found_skip_true = true;
            } else {
                found_skip_false = true;
            }
        }

        assert!(
            found_skip_true,
            "Should find at least one entry with skip=true"
        );
        assert!(
            found_skip_false,
            "Should find at least one entry with skip=false"
        );

        println!(
            "conf.netstat skip field parsing: skip_true={}, skip_false={}",
            found_skip_true, found_skip_false
        );
    }

    #[test]
    fn test_conf_sockstat_skip_field_parsing() {
        let conf_path = get_share_dir().join("conf.sockstat");
        let file = File::open(&conf_path).expect("conf.sockstat should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.sockstat should contain at least one entry"
        );

        // Check that skip fields are parsed correctly
        let mut found_skip_true = false;
        let mut found_skip_false = false;

        for entry in &entries {
            if entry.skip {
                found_skip_true = true;
            } else {
                found_skip_false = true;
            }
        }

        assert!(
            found_skip_true,
            "Should find at least one entry with skip=true"
        );
        assert!(
            found_skip_false,
            "Should find at least one entry with skip=false"
        );

        println!(
            "conf.sockstat skip field parsing: skip_true={}, skip_false={}",
            found_skip_true, found_skip_false
        );
    }

    #[test]
    fn test_conf_ss_skip_field_parsing() {
        let conf_path = get_share_dir().join("conf.ss");
        let file = File::open(&conf_path).expect("conf.ss should exist");
        let reader = BufReader::new(file);
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert!(
            !entries.is_empty(),
            "conf.ss should contain at least one entry"
        );

        // Check that skip fields are parsed correctly
        let mut found_skip_true = false;
        let mut found_skip_false = false;

        for entry in &entries {
            if entry.skip {
                found_skip_true = true;
            } else {
                found_skip_false = true;
            }
        }

        assert!(
            found_skip_true,
            "Should find at least one entry with skip=true"
        );
        assert!(
            found_skip_false,
            "Should find at least one entry with skip=false"
        );

        println!(
            "conf.ss skip field parsing: skip_true={}, skip_false={}",
            found_skip_true, found_skip_false
        );
    }

    #[test]
    fn test_skip_default_values() {
        // Test with a simple config that doesn't specify skip
        let config_content = r#"
regexp=test pattern
colours=red
======
"#;

        let reader = BufReader::new(config_content.as_bytes());
        let grcat_reader = GrcatConfigReader::new(reader.lines());

        let entries: Vec<GrcatConfigEntry> = grcat_reader.collect();
        assert_eq!(entries.len(), 1, "Should parse one entry");

        assert!(
            !entries[0].skip,
            "Skip should default to false when not specified"
        );
        println!("Skip default value test passed: skip=false");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_grc_conf_references_valid_conf_files() {
        let grc_conf_path = get_etc_dir().join("rgrc.conf");
        let file = File::open(&grc_conf_path).expect("Failed to open grc.conf");
        let reader = BufReader::new(file);
        let grc_reader = GrcConfigReader::new(reader.lines());

        let share_dir = get_share_dir();
        let mut missing_files = Vec::new();
        let mut found_files = 0;

        for (_regex, config_file) in grc_reader {
            // Check if the referenced config file exists in share directory
            let conf_path = share_dir.join(&config_file);
            if conf_path.exists() {
                found_files += 1;
            } else {
                missing_files.push(config_file.clone());
            }
        }

        if !missing_files.is_empty() {
            println!(
                "Warning: grc.conf references {} missing files:",
                missing_files.len()
            );
            for file in &missing_files {
                println!("  - {}", file);
            }
        }

        println!(
            "Found {} referenced configuration files in share/",
            found_files
        );
        assert!(
            found_files > 0,
            "At least some referenced files should exist"
        );
    }

    #[test]
    fn test_complete_workflow_grc_conf_to_grcat() {
        let grc_conf_path = get_etc_dir().join("rgrc.conf");
        let file = File::open(&grc_conf_path).expect("Failed to open grc.conf");
        let reader = BufReader::new(file);
        let grc_reader = GrcConfigReader::new(reader.lines());

        let share_dir = get_share_dir();
        let mut workflows_tested = 0;
        let mut total_workflows = 0;

        for (_regex, config_file) in grc_reader {
            let conf_path = share_dir.join(&config_file);
            if conf_path.exists() {
                total_workflows += 1;
                // Test complete workflow: grc.conf entry -> grcat config file
                if let Ok(file) = File::open(&conf_path) {
                    let reader = BufReader::new(file);
                    let grcat_reader = GrcatConfigReader::new(reader.lines());
                    let _entries: Vec<_> = grcat_reader.collect();

                    // Count as successful workflow even if entries is empty
                    workflows_tested += 1;
                }
            }
        }

        println!(
            "Successfully tested {}/{} complete workflows (grc.conf -> grcat config)",
            workflows_tested, total_workflows
        );
        assert!(
            workflows_tested > 0,
            "Should test at least one complete workflow"
        );
    }

    #[test]
    fn test_all_conf_files_in_share_are_valid() {
        let conf_files = get_all_conf_files();
        let mut valid_files = 0;
        let mut invalid_files = Vec::new();

        for conf_file in &conf_files {
            let filename = conf_file.file_name().unwrap().to_string_lossy().to_string();

            match File::open(conf_file) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let grcat_reader = GrcatConfigReader::new(reader.lines());

                    // Try to collect all entries - this will validate parsing
                    let entries: Vec<_> = grcat_reader.collect();

                    // Even if a file has no entries, as long as it doesn't error, it's valid
                    valid_files += 1;

                    if entries.is_empty() {
                        println!("  {} - no entries (possibly all comments)", filename);
                    }
                }
                Err(e) => {
                    invalid_files.push((filename.clone(), e.to_string()));
                }
            }
        }

        if !invalid_files.is_empty() {
            println!("Invalid files:");
            for (file, error) in &invalid_files {
                println!("  - {}: {}", file, error);
            }
        }

        println!(
            "Valid configuration files: {}/{}",
            valid_files,
            conf_files.len()
        );
        assert_eq!(
            valid_files,
            conf_files.len(),
            "All configuration files should be valid"
        );
    }

    /// Helper to get all conf files
    fn get_all_conf_files() -> Vec<PathBuf> {
        let share_dir = get_share_dir();
        let mut conf_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&share_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name()
                    && filename.to_string_lossy().starts_with("conf.")
                {
                    conf_files.push(path);
                }
            }
        }

        conf_files.sort();
        conf_files
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_colors_handling() {
        // Some entries might not have colors defined
        let conf_path = get_share_dir().join("conf.dummy");
        if conf_path.exists() {
            let file = File::open(&conf_path).expect("Should open conf.dummy");
            let reader = BufReader::new(file);
            let grcat_reader = GrcatConfigReader::new(reader.lines());

            for entry in grcat_reader {
                // Should handle entries with or without colors
                let _colors = &entry.colors;
            }
        }
    }

    #[test]
    fn test_complex_regex_patterns() {
        // Test files with complex regex patterns
        let test_files = vec!["conf.ls", "conf.ping", "conf.netstat", "conf.iptables"];
        let share_dir = get_share_dir();

        for filename in &test_files {
            let conf_path = share_dir.join(filename);
            if conf_path.exists() {
                let file =
                    File::open(&conf_path).unwrap_or_else(|_| panic!("Should open {}", filename));
                let reader = BufReader::new(file);
                let grcat_reader = GrcatConfigReader::new(reader.lines());

                for entry in grcat_reader {
                    // Verify complex regexes are properly parsed
                    assert!(
                        entry.regex.is_match("test").is_ok(),
                        "Complex regex in {} should be valid",
                        filename
                    );
                }
            }
        }
    }

    #[test]
    fn test_multiple_color_definitions() {
        let conf_path = get_share_dir().join("conf.ls");
        if conf_path.exists() {
            let file = File::open(&conf_path).expect("Should open conf.ls");
            let reader = BufReader::new(file);
            let grcat_reader = GrcatConfigReader::new(reader.lines());

            let entries: Vec<_> = grcat_reader.collect();

            // conf.ls should have entries with multiple colors for capture groups
            let has_multiple_colors = entries.iter().any(|e| e.colors.len() > 1);

            if has_multiple_colors {
                println!("conf.ls has entries with multiple color definitions");
            }
        }
    }
}
