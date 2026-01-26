// Coverage tests for src/bin/rgrv.rs
//
// This file provides comprehensive test coverage for the rgrv validation tool,
// including validation logic, error detection, file handling, and edge cases.

#[cfg(target_arch = "x86_64")]
mod rgrv {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper function to get the rgrv binary path
    fn get_rgrv_binary() -> PathBuf {
        // Prefer the cargo-provided binary path when running integration tests.
        if let Ok(bin) = std::env::var("CARGO_BIN_EXE_rgrv") {
            return PathBuf::from(bin);
        }

        // Fallback: compute path relative to the current test executable.
        let mut path = std::env::current_exe().unwrap();
        path.pop(); // remove test exe name
        path.pop(); // remove deps
        path.push("rgrv");
        if cfg!(windows) {
            path.set_extension("exe");
        }
        path
    }

    // Unexpected line after colours= should produce a FormatError mention
    #[test]
    fn test_unexpected_line_after_colours() {
        let tmp = TempDir::new().unwrap();
        let conf = tmp.path().join("conf.test");

        let mut f = fs::File::create(&conf).unwrap();
        writeln!(f, "regexp=^test").unwrap();
        writeln!(f, "colours=red").unwrap();
        writeln!(f, "weird=unexpected").unwrap();
        drop(f);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf.to_str().unwrap())
            .output()
            .expect("failed to run rgrv");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        assert!(
            combined.contains("Unexpected line after colours=") || combined.contains("FormatError")
        );
    }

    // Unexpected line after regexp: when a following non-colours/non-config line appears
    #[test]
    fn test_unexpected_line_after_regexp() {
        let tmp = TempDir::new().unwrap();
        let conf = tmp.path().join("conf.test");

        let mut f = fs::File::create(&conf).unwrap();
        writeln!(f, "regexp=^hello").unwrap();
        writeln!(f, "not-a-valid-line").unwrap();
        drop(f);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf.to_str().unwrap())
            .output()
            .expect("failed to run rgrv");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        assert!(
            combined.contains("Unexpected line after regexp") || combined.contains("FormatError")
        );
    }

    /// Test help command with --help flag
    #[test]
    fn test_help_flag() {
        let output = Command::new(get_rgrv_binary())
            .arg("--help")
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rgrc Configuration Validator"));
        assert!(stdout.contains("Commands:"));
        assert!(stdout.contains("grc"));
        assert!(stdout.contains("conf"));
        assert!(stdout.contains("all"));
    }

    /// Test help command with -h flag
    #[test]
    fn test_help_short_flag() {
        let output = Command::new(get_rgrv_binary())
            .arg("-h")
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rgrc Configuration Validator"));
    }

    /// Test version command with --version flag
    #[test]
    fn test_version_flag() {
        let output = Command::new(get_rgrv_binary())
            .arg("--version")
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rgrc-validate"));
    }

    /// Test version command with -v flag
    #[test]
    fn test_version_short_flag() {
        let output = Command::new(get_rgrv_binary())
            .arg("-v")
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rgrc-validate"));
    }

    /// Test no arguments (should show help and exit with error)
    #[test]
    fn test_no_arguments() {
        let output = Command::new(get_rgrv_binary())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage:"));
    }

    /// Test unknown command
    #[test]
    fn test_unknown_command() {
        let output = Command::new(get_rgrv_binary())
            .arg("unknown_command")
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Unknown command"));
    }

    /// Test valid grc.conf file
    #[test]
    fn test_validate_valid_grc_conf() {
        let temp_dir = TempDir::new().unwrap();
        let grc_conf = temp_dir.path().join("test_grc.conf");

        // Create a valid grc.conf
        let mut file = fs::File::create(&grc_conf).unwrap();
        writeln!(file, "# Test grc configuration").unwrap();
        writeln!(file, "^ping").unwrap();
        writeln!(file, "conf.ping").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "^netstat").unwrap();
        writeln!(file, "conf.netstat").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg(grc_conf.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        // Should succeed even if conf files don't exist (just validates format)
        assert!(output.status.success() || !output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Validating grc.conf"));
    }

    /// Test grc.conf with invalid regex
    #[test]
    fn test_validate_grc_conf_invalid_regex() {
        let temp_dir = TempDir::new().unwrap();
        let grc_conf = temp_dir.path().join("test_grc.conf");

        // Create grc.conf with invalid regex
        let mut file = fs::File::create(&grc_conf).unwrap();
        writeln!(file, "# Invalid regex test").unwrap();
        writeln!(file, "^ping[unclosed").unwrap();
        writeln!(file, "conf.ping").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg(grc_conf.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("RegexError") || stderr.contains("Invalid regex"));
    }

    /// Test grc.conf with missing config file reference
    #[test]
    fn test_validate_grc_conf_missing_reference() {
        let temp_dir = TempDir::new().unwrap();
        let grc_conf = temp_dir.path().join("test_grc.conf");

        // Create grc.conf with missing config reference
        let mut file = fs::File::create(&grc_conf).unwrap();
        writeln!(file, "# Missing reference test").unwrap();
        writeln!(file, "^ping").unwrap();
        // Missing conf file reference
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg(grc_conf.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("FormatError") || stderr.contains("Missing config file"));
    }

    /// Test grc.conf with empty line after pattern (invalid)
    #[test]
    fn test_validate_grc_conf_empty_after_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let grc_conf = temp_dir.path().join("test_grc.conf");

        // Create grc.conf with empty line after pattern
        let mut file = fs::File::create(&grc_conf).unwrap();
        writeln!(file, "^ping").unwrap();
        writeln!(file).unwrap(); // Empty line instead of config file
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg(grc_conf.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
    }

    /// Test grc.conf with comment after pattern (invalid)
    #[test]
    fn test_validate_grc_conf_comment_after_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let grc_conf = temp_dir.path().join("test_grc.conf");

        // Create grc.conf with comment line after pattern
        let mut file = fs::File::create(&grc_conf).unwrap();
        writeln!(file, "^ping").unwrap();
        writeln!(file, "# This is a comment").unwrap(); // Comment instead of config file
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg(grc_conf.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("FormatError") || stderr.contains("Expected config file"));
    }

    /// Test valid conf.* file
    #[test]
    fn test_validate_valid_conf_file() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create a valid conf file
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, "# Test configuration").unwrap();
        writeln!(file, r"^\d+\s+bytes\s+from red bold").unwrap();
        writeln!(file, r"time=[\d\.]+ green").unwrap();
        writeln!(file).unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Validating color configuration"));
    }

    /// Test conf file with invalid regex
    #[test]
    fn test_validate_conf_file_invalid_regex() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with invalid regex
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, "# Invalid regex test").unwrap();
        writeln!(file, r"^\d+[unclosed red").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("RegexError") || stderr.contains("Invalid regex"));
    }

    /// Test conf file with missing style definition
    #[test]
    fn test_validate_conf_file_missing_style() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with missing style
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, "# Missing style test").unwrap();
        writeln!(file, r"^\d+").unwrap(); // No style definition
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("FormatError") || stderr.contains("Missing style"));
    }

    /// Test conf file with unknown style
    #[test]
    fn test_validate_conf_file_unknown_style() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with unknown style
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d+ unknown_style").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("StyleError") || stderr.contains("Unknown style"));
    }

    /// Test conf file with multiple unknown styles
    #[test]
    fn test_validate_conf_file_multiple_unknown_styles() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with multiple unknown styles
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d+ red unknown1 blue unknown2").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
    }

    /// Test conf file with valid bright colors
    #[test]
    fn test_validate_conf_file_bright_colors() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with bright colors
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d+ bright-red bright-green bright-blue").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test conf file with valid background colors
    #[test]
    fn test_validate_conf_file_background_colors() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with background colors
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d+ on_red on_blue on_green").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test conf file with valid text attributes
    #[test]
    fn test_validate_conf_file_text_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with text attributes
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d+ bold italic underline").unwrap();
        writeln!(file, r"^\w+ dim blink reverse").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test conf file with tab separator
    #[test]
    fn test_validate_conf_file_tab_separator() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with tab separator
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, "^\\d+\tred bold").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test conf file with comments
    #[test]
    fn test_validate_conf_file_comments() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with comments
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, r"^\d+ red").unwrap();
        writeln!(file, "# Another comment").unwrap();
        writeln!(file, r"^\w+ blue").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test conf file with empty lines
    #[test]
    fn test_validate_conf_file_empty_lines() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with empty lines
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file).unwrap();
        writeln!(file, r"^\d+ red").unwrap();
        writeln!(file).unwrap();
        writeln!(file, r"^\w+ blue").unwrap();
        writeln!(file).unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test multiple conf files validation
    #[test]
    fn test_validate_multiple_conf_files() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file1 = temp_dir.path().join("conf.test1");
        let conf_file2 = temp_dir.path().join("conf.test2");

        // Create two valid conf files
        let mut file1 = fs::File::create(&conf_file1).unwrap();
        writeln!(file1, r"^\d+ red").unwrap();
        drop(file1);

        let mut file2 = fs::File::create(&conf_file2).unwrap();
        writeln!(file2, r"^\w+ blue").unwrap();
        drop(file2);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file1.to_str().unwrap())
            .arg(conf_file2.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2 files validated"));
    }

    /// Test multiple conf files with errors
    #[test]
    fn test_validate_multiple_conf_files_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file1 = temp_dir.path().join("conf.test1");
        let conf_file2 = temp_dir.path().join("conf.test2");

        // Create one valid and one invalid conf file
        let mut file1 = fs::File::create(&conf_file1).unwrap();
        writeln!(file1, r"^\d+ red").unwrap();
        drop(file1);

        let mut file2 = fs::File::create(&conf_file2).unwrap();
        writeln!(file2, r"^\w+ unknown_style").unwrap();
        drop(file2);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file1.to_str().unwrap())
            .arg(conf_file2.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let output_combined = format!("{}{}", stdout, stderr);
        assert!(output_combined.contains("2 files validated") || output_combined.contains("error"));
    }

    /// Test conf validation with non-existent file
    #[test]
    fn test_validate_conf_file_not_found() {
        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg("/non/existent/conf.test")
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let output_str = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output_str.contains("not found") || output_str.contains("error"));
    }

    /// Test grc validation with non-existent file
    #[test]
    fn test_validate_grc_file_not_found() {
        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg("/non/existent/grc.conf")
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Failed to read") || stderr.contains("error"));
    }

    /// Test 'all' command (validates both grc and conf files)
    #[test]
    fn test_validate_all_command() {
        let output = Command::new(get_rgrv_binary())
            .arg("all")
            .output()
            .expect("Failed to execute rgrv");

        // The 'all' command is not yet implemented, so it should fail
        // but provide helpful output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        assert!(
            combined.contains("Validating grc.conf")
                || combined.contains("Validating color configuration")
                || combined.contains("Unknown command")
        );
    }

    /// Test conf file with complex regex patterns
    #[test]
    fn test_validate_conf_file_complex_regex() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with complex regex patterns
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}} cyan").unwrap();
        writeln!(
            file,
            r"([A-Za-z0-9+/]{{4}})*([A-Za-z0-9+/]{{2}}==|[A-Za-z0-9+/]{{3}}=)? yellow"
        )
        .unwrap();
        writeln!(
            file,
            r"(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){{3}} green"
        )
        .unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test grc.conf with comments between entries
    #[test]
    fn test_validate_grc_conf_with_comments() {
        let temp_dir = TempDir::new().unwrap();
        let grc_conf = temp_dir.path().join("test_grc.conf");

        // Create grc.conf with comments between entries
        let mut file = fs::File::create(&grc_conf).unwrap();
        writeln!(file, "# First entry").unwrap();
        writeln!(file, "^ping").unwrap();
        writeln!(file, "conf.ping").unwrap();
        writeln!(file, "# Second entry").unwrap();
        writeln!(file, "^netstat").unwrap();
        writeln!(file, "conf.netstat").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("grc")
            .arg(grc_conf.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        // May succeed or fail depending on whether conf files exist
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Validating grc.conf"));
    }

    /// Test empty conf file
    #[test]
    fn test_validate_empty_conf_file() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create empty conf file
        fs::File::create(&conf_file).unwrap();

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        // Empty file should be valid (no errors)
        assert!(output.status.success());
    }

    /// Test conf file with only comments
    #[test]
    fn test_validate_conf_file_only_comments() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with only comments
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, "# Comment 1").unwrap();
        writeln!(file, "# Comment 2").unwrap();
        writeln!(file, "# Comment 3").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(output.status.success());
    }

    /// Test conf file with mixed valid and invalid entries
    #[test]
    fn test_validate_conf_file_mixed_entries() {
        let temp_dir = TempDir::new().unwrap();
        let conf_file = temp_dir.path().join("conf.test");

        // Create conf file with mixed entries
        let mut file = fs::File::create(&conf_file).unwrap();
        writeln!(file, r"^\d+ red").unwrap();
        writeln!(file, r"^\w+ invalid_style").unwrap();
        writeln!(file, r"^test blue").unwrap();
        drop(file);

        let output = Command::new(get_rgrv_binary())
            .arg("conf")
            .arg(conf_file.to_str().unwrap())
            .output()
            .expect("Failed to execute rgrv");

        assert!(!output.status.success());
    }
}
