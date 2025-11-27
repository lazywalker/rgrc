//! Integration tests comparing regex-based and SIMD-based colorizers
//! to ensure they produce identical outputs for literal patterns.
#[cfg(feature = "simd")]
mod test {
    use console::Style;
    use fancy_regex::Regex;
    use rgrc::colorizer::{colorize_regex, colorize_simd};
    use rgrc::grc::{GrcatConfigEntry, GrcatConfigEntryCount};
    use std::io::Cursor;

    fn create_test_rules() -> Vec<GrcatConfigEntry> {
        vec![
            GrcatConfigEntry {
                regex: Regex::new("ERROR").unwrap(),
                colors: vec![Style::new().red().bold()],
                skip: false,
                count: GrcatConfigEntryCount::More,
                replace: String::new(),
            },
            GrcatConfigEntry {
                regex: Regex::new("WARNING").unwrap(),
                colors: vec![Style::new().yellow()],
                skip: false,
                count: GrcatConfigEntryCount::More,
                replace: String::new(),
            },
            GrcatConfigEntry {
                regex: Regex::new("INFO").unwrap(),
                colors: vec![Style::new().cyan()],
                skip: false,
                count: GrcatConfigEntryCount::More,
                replace: String::new(),
            },
        ]
    }

    #[test]
    fn test_simd_matches_regex_simple() {
        let test_input = "INFO: System starting\nERROR: Connection failed\nWARNING: Low memory\n";
        let rules = create_test_rules();

        // Test regex implementation
        let mut regex_reader = Cursor::new(test_input);
        let mut regex_output = Vec::new();
        colorize_regex(&mut regex_reader, &mut regex_output, &rules).unwrap();

        // Test SIMD implementation
        let mut simd_reader = Cursor::new(test_input);
        let mut simd_output = Vec::new();
        colorize_simd(&mut simd_reader, &mut simd_output, &rules).unwrap();

        // Both should produce identical output for literal patterns
        assert_eq!(
            String::from_utf8_lossy(&regex_output),
            String::from_utf8_lossy(&simd_output),
            "SIMD and regex implementations should produce identical output for literal patterns"
        );
    }

    #[test]
    fn test_simd_matches_regex_multiple_matches() {
        let test_input = "ERROR ERROR ERROR\nINFO WARNING INFO\n";
        let rules = create_test_rules();

        let mut regex_reader = Cursor::new(test_input);
        let mut regex_output = Vec::new();
        colorize_regex(&mut regex_reader, &mut regex_output, &rules).unwrap();

        let mut simd_reader = Cursor::new(test_input);
        let mut simd_output = Vec::new();
        colorize_simd(&mut simd_reader, &mut simd_output, &rules).unwrap();

        assert_eq!(
            String::from_utf8_lossy(&regex_output),
            String::from_utf8_lossy(&simd_output),
            "Multiple matches should be handled identically"
        );
    }

    #[test]
    fn test_simd_empty_input() {
        let test_input = "";
        let rules = create_test_rules();

        let mut regex_reader = Cursor::new(test_input);
        let mut regex_output = Vec::new();
        colorize_regex(&mut regex_reader, &mut regex_output, &rules).unwrap();

        let mut simd_reader = Cursor::new(test_input);
        let mut simd_output = Vec::new();
        colorize_simd(&mut simd_reader, &mut simd_output, &rules).unwrap();

        assert_eq!(
            regex_output, simd_output,
            "Empty input should be handled identically"
        );
    }

    #[test]
    fn test_simd_no_matches() {
        let test_input = "Nothing interesting here\nJust plain text\n";
        let rules = create_test_rules();

        let mut regex_reader = Cursor::new(test_input);
        let mut regex_output = Vec::new();
        colorize_regex(&mut regex_reader, &mut regex_output, &rules).unwrap();

        let mut simd_reader = Cursor::new(test_input);
        let mut simd_output = Vec::new();
        colorize_simd(&mut simd_reader, &mut simd_output, &rules).unwrap();

        assert_eq!(
            String::from_utf8_lossy(&regex_output),
            String::from_utf8_lossy(&simd_output),
            "Lines without matches should be handled identically"
        );
    }
}
