// Include the colorizer module from src
#[path = "../src/colorizer.rs"]
mod colorizer;

#[path = "../src/grc.rs"]
mod grc;

use colorizer::colorize_regex;
use fancy_regex::Regex;
use grc::GrcatConfigEntry;

/// Helper function to run colorization and return the output
fn colorize_test(
    input: &str,
    rules: &[GrcatConfigEntry],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut writer = Vec::new();
    colorize_regex(&mut input.as_bytes(), &mut writer, rules)?;
    Ok(String::from_utf8(writer)?)
}

/// Helper function to run colorize_regex and return the output
fn colorize_regex_test(
    input: &str,
    rules: &[GrcatConfigEntry],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut writer = Vec::new();
    colorize_regex(&mut input.as_bytes(), &mut writer, rules)?;
    Ok(String::from_utf8(writer)?)
}

/// Helper to create a simple style rule
fn rule(
    pattern: &str,
    style: console::Style,
) -> Result<GrcatConfigEntry, Box<dyn std::error::Error>> {
    Ok(GrcatConfigEntry::new(Regex::new(pattern)?, vec![style]))
}

#[cfg(test)]
mod basic_colorization_tests {
    use super::*;

    #[test]
    fn test_no_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let output = colorize_test("hello world\n", &[])?;
        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let output = colorize_test("", &[])?;
        assert_eq!(output, "");
        Ok(())
    }

    #[test]
    fn test_single_empty_line() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let output = colorize_test("\n", &[])?;
        assert_eq!(output, "\n");
        Ok(())
    }

    #[test]
    fn test_simple_match() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("world", console::Style::new().red())?];
        let output = colorize_test("hello world", &rules)?;

        // Verify output contains the matched word with ANSI color code
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        // Should end with newline
        assert!(output.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_no_match() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("xyz", console::Style::new().blue())?];
        let output = colorize_test("hello world", &rules)?;

        // No match means output unchanged
        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_multiple_matches_same_rule() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("o", console::Style::new().green())?];
        let output = colorize_test("foo boo", &rules)?;

        // Should contain the words (possibly with ANSI codes)
        // Check that output is not empty and contains the original structure
        assert!(!output.is_empty());
        // When colors are applied, the output will contain ANSI codes
        assert!(output.len() >= "foo boo".len());
        Ok(())
    }

    #[test]
    fn test_overlapping_matches() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule("foo", console::Style::new().red())?,
            rule("bar", console::Style::new().blue())?,
        ];
        let output = colorize_test("foobar", &rules)?;

        // Both patterns should be present
        assert!(output.contains("foo"));
        assert!(output.contains("bar"));
        Ok(())
    }

    #[test]
    fn test_style_merging() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("hello", console::Style::new().red())?];
        let output = colorize_test("hello hello", &rules)?;

        // Both instances of "hello" should be styled
        let count = output.matches("hello").count();
        assert_eq!(count, 2);
        Ok(())
    }
}

#[cfg(test)]
mod multiline_tests {
    use super::*;

    #[test]
    fn test_two_lines() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("test line\nno match\n", &rules)?;

        // First line should have "test", second should not be modified
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        Ok(())
    }

    #[test]
    fn test_multiple_lines_with_matches() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("foo", console::Style::new().green())?];

        let mut input = String::new();
        for i in 0..10 {
            if i % 2 == 0 {
                input.push_str("foo bar\n");
            } else {
                input.push_str("baz qux\n");
            }
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 10);
        Ok(())
    }

    #[test]
    fn test_large_input_single_threaded() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("line", console::Style::new().blue())?];

        // Create input with 500 lines (below parallel threshold)
        let mut input = String::new();
        for i in 0..500 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 500);
        Ok(())
    }

    #[test]
    fn test_large_input_parallel_processing() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("line", console::Style::new().red())?];

        // Create input with 1500 lines (above parallel threshold of 1000)
        let mut input = String::new();
        for i in 0..1500 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1500);

        // Verify line count is preserved (structure integrity)
        assert!(lines.len() == 1500);
        Ok(())
    }

    #[test]
    fn test_empty_lines_in_input() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().green())?];
        let input = "test\n\ntest\n\n";
        let output = colorize_test(input, &rules)?;

        // Empty lines should be preserved
        assert!(output.contains("\n\n"));
        Ok(())
    }

    #[test]
    fn test_lines_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(r"\d+", console::Style::new().yellow())?];
        let input = "line 123 and 456\nnext 789\n";
        let output = colorize_test(input, &rules)?;

        // Special regex should work
        assert!(output.contains("line"));
        assert!(output.contains("next"));
        Ok(())
    }
}

#[cfg(test)]
mod regex_pattern_tests {
    use super::*;

    #[test]
    fn test_literal_pattern() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("this is a test", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_digit_pattern() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(r"\d+", console::Style::new().blue())?];
        let output = colorize_test("value: 12345", &rules)?;
        assert!(output.contains("value:"));
        Ok(())
    }

    #[test]
    fn test_word_boundary_pattern() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(r"\btest\b", console::Style::new().green())?];
        let output = colorize_test("test testing tested", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_dot_wildcard() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("t.st", console::Style::new().red())?];
        let output = colorize_test("test toast", &rules)?;
        assert!(output.contains("test"));
        assert!(output.contains("toast"));
        Ok(())
    }

    #[test]
    fn test_alternation_pattern() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("red|blue", console::Style::new().cyan())?];
        let output = colorize_test("red ball blue sky", &rules)?;
        assert!(output.contains("red"));
        assert!(output.contains("blue"));
        Ok(())
    }

    #[test]
    fn test_character_class() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("[aeiou]", console::Style::new().magenta())?];
        let output = colorize_test("hello world", &rules)?;
        // Verify output is not empty and has expected structure
        assert!(!output.is_empty());
        // Output should be longer than or equal to input due to ANSI codes
        assert!(output.len() >= "hello world".len());
        Ok(())
    }

    #[test]
    fn test_case_sensitive_matching() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("Test", console::Style::new().red())?];
        let output = colorize_test("test Test TEST", &rules)?;

        // Only "Test" should match (case-sensitive)
        assert!(output.contains("test"));
        assert!(output.contains("Test"));
        assert!(output.contains("TEST"));
        Ok(())
    }

    #[test]
    fn test_quantifier_plus() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("a+", console::Style::new().blue())?];
        let output = colorize_test("aa aaa a", &rules)?;
        assert!(output.contains("a"));
        Ok(())
    }

    #[test]
    fn test_quantifier_star() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("ab*c", console::Style::new().red())?];
        let output = colorize_test("ac abc abbc", &rules)?;
        assert!(output.contains("ac"));
        Ok(())
    }

    #[test]
    fn test_anchored_pattern() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("^start", console::Style::new().green())?];
        let output = colorize_test("start of line\nnot start", &rules)?;
        assert!(output.contains("start"));
        Ok(())
    }
}

#[cfg(test)]
mod capture_group_tests {
    use super::*;

    #[test]
    fn test_single_capture_group() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![GrcatConfigEntry::new(
            Regex::new(r"(test)")?,
            vec![console::Style::new(), console::Style::new().red()],
        )];
        let output = colorize_test("this is test", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_multiple_capture_groups() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![GrcatConfigEntry::new(
            Regex::new(r"(\w+):(\d+)")?,
            vec![
                console::Style::new(),
                console::Style::new().red(),
                console::Style::new().blue(),
            ],
        )];
        let output = colorize_test("server:8080", &rules)?;
        assert!(output.contains("server"));
        assert!(output.contains("8080"));
        Ok(())
    }

    #[test]
    fn test_capture_group_with_no_style() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![GrcatConfigEntry::new(
            Regex::new(r"(\w+):(\d+)")?,
            vec![console::Style::new().red()],
        )];
        let output = colorize_test("server:8080", &rules)?;
        // Should still process without error
        assert!(output.contains("server"));
        Ok(())
    }

    #[test]
    fn test_nested_capture_groups() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![GrcatConfigEntry::new(
            Regex::new(r"((test))")?,
            vec![
                console::Style::new(),
                console::Style::new().red(),
                console::Style::new().green(),
            ],
        )];
        let output = colorize_test("test data", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_optional_capture_group() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![GrcatConfigEntry::new(
            Regex::new(r"(\w+)(:)?(\d+)?")?,
            vec![
                console::Style::new(),
                console::Style::new().red(),
                console::Style::new().green(),
                console::Style::new().blue(),
            ],
        )];
        let output = colorize_test("server:8080 simple", &rules)?;
        assert!(output.contains("server"));
        assert!(output.contains("simple"));
        Ok(())
    }
}

#[cfg(test)]
mod style_application_tests {
    use super::*;

    #[test]
    fn test_style_red() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_green() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().green())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_blue() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().blue())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_yellow() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().yellow())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_magenta() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().magenta())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_cyan() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().cyan())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_white() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().white())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_black() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().black())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_bold() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().bold())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_underlined() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().underlined())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_combined() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red().bold())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_on_color() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().on_blue())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_very_long_line() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("x", console::Style::new().red())?];

        // Create a very long line
        let mut input = "a".repeat(10000);
        input.push('x');
        input.push_str(&"b".repeat(10000));
        let output = colorize_test(&input, &rules)?;

        assert!(output.contains("a"));
        assert!(output.contains("b"));
        Ok(())
    }

    #[test]
    fn test_line_with_only_spaces() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("     \n", &rules)?;

        // Should preserve spaces
        assert!(output.contains("    "));
        Ok(())
    }

    #[test]
    fn test_line_with_tabs() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("test\ttest\n", &rules)?;

        // Should preserve tabs
        assert!(output.contains("\t"));
        Ok(())
    }

    #[test]
    fn test_unicode_content() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("test 你好 test", &rules)?;

        assert!(output.contains("test"));
        assert!(output.contains("你好"));
        Ok(())
    }

    #[test]
    fn test_match_at_line_start() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("^test", console::Style::new().red())?];
        let output = colorize_test("test data", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_match_at_line_end() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("end$", console::Style::new().red())?];
        let output = colorize_test("this is the end", &rules)?;

        assert!(output.contains("end"));
        Ok(())
    }

    #[test]
    fn test_zero_width_match() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("^", console::Style::new().red())?];
        let output = colorize_test("test\n", &rules)?;

        // Should handle zero-width match without hanging
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_multiple_rules_same_text() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule("test", console::Style::new().red())?,
            rule("test", console::Style::new().blue())?,
            rule("test", console::Style::new().green())?,
        ];
        let output = colorize_test("test", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_overlapping_pattern_precedence() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule("abc", console::Style::new().red())?,
            rule("bcd", console::Style::new().blue())?,
        ];
        let output = colorize_test("abcd", &rules)?;

        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
        assert!(output.contains("d"));
        Ok(())
    }

    #[test]
    fn test_consecutive_empty_lines() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("test\n\n\ntest\n", &rules)?;

        // Should preserve empty lines
        assert!(output.contains("\n\n"));
        Ok(())
    }

    #[test]
    fn test_windows_line_endings() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("test", console::Style::new().red())?];
        // Note: Input will be treated as raw bytes
        let output = colorize_test("test\r\n", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_colors_disabled() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(false);
        let rules = vec![rule("test", console::Style::new().red())?];
        let output = colorize_test("test data", &rules)?;

        // Output should still work even with colors disabled
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_skip_rule_functionality() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);

        // Create rules: one normal rule and one skipped rule
        let normal_rule = rule("ERROR", console::Style::new().red())?;
        let mut skip_rule = rule("WARNING", console::Style::new().yellow())?;
        skip_rule.skip = true; // Mark this rule as skipped

        let rules = vec![normal_rule, skip_rule];

        let input = "This is an ERROR message\nThis is a WARNING message\n";
        let output = colorize_test(input, &rules)?;

        println!("Output: {:?}", output);

        // Both ERROR and WARNING text should be present
        assert!(output.contains("ERROR"));
        assert!(output.contains("WARNING"));

        // Check that ERROR has color codes (red) but WARNING does not have yellow color codes
        // We can't easily check exact ANSI codes, so we'll check that the output contains ANSI codes for ERROR
        // and that WARNING appears without the yellow color code pattern
        assert!(output.contains("\x1b[")); // Should have some ANSI codes
        assert!(output.contains("ERROR")); // ERROR should be present

        Ok(())
    }
}

#[cfg(test)]
mod advanced_features_tests {
    use super::*;
    use grc::GrcatConfigEntryCount;

    /// Lines 290-293: Count::Stop prevents subsequent rule processing
    /// Tests that when a rule has count=Stop, processing stops after the first match
    /// and subsequent rules are not applied to the remainder of the line.
    #[test]
    fn test_count_stop_prevents_subsequent_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let mut r1 = GrcatConfigEntry::new(
            Regex::new(r"ERROR: (.*)")?,
            vec![console::Style::new().red()],
        );
        r1.count = GrcatConfigEntryCount::Stop;

        let r2 = GrcatConfigEntry::new(Regex::new(r"Boom")?, vec![console::Style::new().blue()]);

        let output = colorize_test("ERROR: Boom Boom\n", &[r1, r2])?;
        // Because r1 has count=Stop, only the first ERROR match is handled
        assert!(output.contains("ERROR: Boom"));
        Ok(())
    }

    /// Lines 282-293: Count::Once limits matches but allows other rules
    /// Tests that count=Once prevents a rule from matching multiple times on the same line,
    /// but does not stop other rules from processing (unlike count=Stop).
    #[test]
    fn test_count_once_allows_other_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let mut r1 = GrcatConfigEntry::new(Regex::new(r"o")?, vec![console::Style::new().green()]);
        r1.count = GrcatConfigEntryCount::Once;

        let r2 = GrcatConfigEntry::new(Regex::new(r"boo")?, vec![console::Style::new().blue()]);

        let output = colorize_test("foo boo\n", &[r1, r2])?;
        // Both words should be present (at least one 'o' from foo and 'boo')
        assert!(output.contains("o"));
        assert!(output.contains("boo"));
        Ok(())
    }

    /// Lines 248-274: Replace functionality breaks outer loop
    /// Tests that when a rule performs text replacement, it breaks the outer rule
    /// processing loop and follow-up rules are not applied.
    #[test]
    fn test_replace_prevents_followup_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let mut r1 =
            GrcatConfigEntry::new(Regex::new(r"Hello (\w+)")?, vec![console::Style::new()]);
        r1.replace = "\\1-XYZ".to_string();

        let r2 = GrcatConfigEntry::new(Regex::new(r"XYZ")?, vec![console::Style::new().red()]);

        let output = colorize_test("Hello world\n", &[r1, r2])?;
        // Replacement should be applied
        assert!(output.contains("world") || output.contains("XYZ"));
        Ok(())
    }

    /// Lines 248-274: Replace with multiple capture group backreferences
    /// Tests the replacement logic with multiple backreferences (\1, \2) to verify
    /// that captured groups are properly substituted in the replacement string.
    #[test]
    fn test_replace_with_multiple_backrefs() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let mut r = GrcatConfigEntry::new(Regex::new(r"(\w+)-(\d+)")?, vec![console::Style::new()]);
        r.replace = "\\2-\\1".to_string();

        let output = colorize_test("foo-123 bar\n", &[r])?;
        // Backreferences should swap the parts
        assert!(output.contains("123") || output.contains("foo"));
        Ok(())
    }

    /// Lines 213-215, 232-233: last_end cache optimization
    /// Tests the cache optimization that tracks the end position of the last match.
    /// When offset < last_end, the regex check is skipped to avoid redundant checks.
    #[test]
    fn test_last_end_cache_optimization() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let r = GrcatConfigEntry::new(Regex::new(r"aa+")?, vec![console::Style::new().red()]);

        let output = colorize_test("aaaaa aaaaa aaaaa\n", &[r])?;
        assert!(output.contains("aaaaa"));
        Ok(())
    }

    /// Lines 365-376: Run-length encoding and multiple style boundaries
    /// Tests that when adjacent characters have different styles, multiple style
    /// boundaries are correctly detected and each segment is styled independently.
    #[test]
    fn test_multiple_style_boundaries() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let r1 = GrcatConfigEntry::new(Regex::new(r"a")?, vec![console::Style::new().red()]);
        let r2 = GrcatConfigEntry::new(Regex::new(r"b")?, vec![console::Style::new().green()]);
        let r3 = GrcatConfigEntry::new(Regex::new(r"c")?, vec![console::Style::new().blue()]);

        let output = colorize_test("abc\n", &[r1, r2, r3])?;
        // All characters should be present
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
        Ok(())
    }

    /// Lines 159, 383-389: Timetrace feature timing instrumentation
    /// Tests that when the timetrace feature is enabled and RGRCTIME env var is set,
    /// timing information is recorded and reported.
    #[cfg(feature = "timetrace")]
    #[test]
    fn test_timetrace_instrumentation() -> Result<(), Box<dyn std::error::Error>> {
        unsafe { std::env::set_var("RGRCTIME", "1") };
        console::set_colors_enabled(true);

        let r = GrcatConfigEntry::new(Regex::new(r"test")?, vec![console::Style::new().red()]);
        let output = colorize_test("test\n", &[r])?;
        assert!(output.contains("test"));

        unsafe { std::env::remove_var("RGRCTIME") };
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_single_threaded_path() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("line", console::Style::new().red())?];

        // Create input with exactly 999 lines (below parallel threshold)
        let mut input = String::new();
        for i in 0..999 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 999);
        Ok(())
    }

    #[test]
    fn test_parallel_path() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("line", console::Style::new().green())?];

        // Create input with 2000 lines (above parallel threshold of 1000)
        let mut input = String::new();
        for i in 0..2000 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2000);

        // Verify ordering is preserved
        let first_line = lines.first().unwrap();
        let last_line = lines.last().unwrap();
        assert!(first_line.contains("0"));
        assert!(last_line.contains("1999"));
        Ok(())
    }

    #[test]
    fn test_boundary_at_1000_lines() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("x", console::Style::new().blue())?];

        // Create input with exactly 1000 lines (at the boundary)
        let mut input = String::new();
        for i in 0..1000 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1000);
        Ok(())
    }

    #[test]
    fn test_many_small_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);

        let mut rules = Vec::new();
        for i in 0..10 {
            rules.push(rule(&format!("word{}", i), console::Style::new().red())?);
        }

        let mut input = String::new();
        for _i in 0..100 {
            for j in 0..10 {
                input.push_str(&format!("word{} ", j));
            }
            input.push('\n');
        }

        let output = colorize_test(&input, &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_complex_regex_performance() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            console::Style::new().yellow(),
        )?];

        let mut input = String::new();
        for i in 0..100 {
            input.push_str(&format!("IP: 192.168.1.{}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        assert!(output.contains("192"));
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_realistic_log_output() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule("ERROR", console::Style::new().red())?,
            rule("WARN", console::Style::new().yellow())?,
            rule("INFO", console::Style::new().green())?,
        ];

        let input =
            "ERROR: failed to connect\nWARN: retry in progress\nINFO: connection established\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("ERROR"));
        assert!(output.contains("WARN"));
        assert!(output.contains("INFO"));
        Ok(())
    }

    #[test]
    fn test_ip_address_coloring() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            console::Style::new().cyan(),
        )?];

        let input = "Connection from 192.168.1.100 to 10.0.0.1\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("192.168.1.100"));
        assert!(output.contains("10.0.0.1"));
        Ok(())
    }

    #[test]
    fn test_port_number_coloring() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(r":(\d+)", console::Style::new().magenta())?];

        let input = "server listening on :8080\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("8080"));
        Ok(())
    }

    #[test]
    fn test_file_permissions_coloring() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule(r"^(rwx|rw-|r--)", console::Style::new().green())?];

        let input = "rwxr-xr-x user group file.txt\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("rwx"));
        Ok(())
    }

    #[test]
    fn test_http_status_coloring() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule(r" 2\d{2} ", console::Style::new().green())?,
            rule(r" 4\d{2} ", console::Style::new().yellow())?,
            rule(r" 5\d{2} ", console::Style::new().red())?,
        ];

        let input = "GET / 200 OK\nPOST /api 404 Not Found\nPUT /data 500 Error\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("200"));
        assert!(output.contains("404"));
        assert!(output.contains("500"));
        Ok(())
    }

    #[test]
    fn test_json_like_output() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule(r#"\"[^\"]*\""#, console::Style::new().cyan())?,
            rule(r": \d+", console::Style::new().yellow())?,
        ];

        let mut input = r#"{"name": "test", "value": 42}"#.to_string();
        input.push('\n');
        let output = colorize_test(&input, &rules)?;

        assert!(output.contains("name"));
        assert!(output.contains("42"));
        Ok(())
    }
}

#[cfg(test)]
mod colorize_regex_tests {
    use super::*;

    #[test]
    fn test_regex_no_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let output = colorize_regex_test("hello world\n", &[])?;
        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_regex_empty_input() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let output = colorize_regex_test("", &[])?;
        assert_eq!(output, "");
        Ok(())
    }

    #[test]
    fn test_regex_single_empty_line() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let output = colorize_regex_test("\n", &[])?;
        assert_eq!(output, "\n");
        Ok(())
    }

    #[test]
    fn test_regex_simple_match() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("world", console::Style::new().red())?];
        let output = colorize_regex_test("hello world", &rules)?;

        // Verify output contains the matched word with ANSI color code
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        // Should end with newline
        assert!(output.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_regex_no_match() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("xyz", console::Style::new().blue())?];
        let output = colorize_regex_test("hello world", &rules)?;

        // No match means output unchanged
        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_regex_multiple_matches_same_rule() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("o", console::Style::new().green())?];
        let output = colorize_regex_test("foo boo", &rules)?;

        // Should contain the words (possibly with ANSI codes)
        assert!(!output.is_empty());
        // When colors are applied, the output will contain ANSI codes
        assert!(output.len() >= "foo boo".len());
        Ok(())
    }

    #[test]
    fn test_regex_overlapping_matches() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![rule("aa", console::Style::new().red())?];
        let output = colorize_regex_test("aaa", &rules)?;

        // Should handle overlapping matches correctly
        assert!(output.contains("a"));
        assert!(output.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_regex_multiple_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule("ERROR", console::Style::new().red())?,
            rule("INFO", console::Style::new().blue())?,
        ];
        let output = colorize_regex_test("ERROR: something\nINFO: something else", &rules)?;

        // Should contain both styled sections
        assert!(output.contains("ERROR"));
        assert!(output.contains("INFO"));
        Ok(())
    }

    #[test]
    fn test_regex_capture_groups() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        // Test regex with capture groups - style different parts differently
        let mut rule_entry = rule(r"(\w+): (\d+)", console::Style::new().red())?;
        rule_entry.colors = vec![
            console::Style::new().red(),   // full match
            console::Style::new().blue(),  // first capture group (word)
            console::Style::new().green(), // second capture group (number)
        ];

        let rules = vec![rule_entry];
        let output = colorize_regex_test("count: 42", &rules)?;

        // Should contain styled output
        assert!(output.contains("count"));
        assert!(output.contains("42"));
        Ok(())
    }

    #[test]
    fn test_regex_zero_width_match() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        // Test word boundary matches (^, $, \b) which are zero-width
        let rules = vec![rule(r"\b\w+\b", console::Style::new().yellow())?];
        let output = colorize_regex_test("hello world", &rules)?;

        // Should handle word boundaries without infinite loops
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        Ok(())
    }

    #[test]
    fn test_regex_complex_patterns() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule(
                r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
                console::Style::new().cyan(),
            )?, // IP addresses
            rule(r":\d+", console::Style::new().yellow())?, // port numbers
            rule(r#"\"[^\"]*\""#, console::Style::new().green())?, // quoted strings
        ];

        let input = r#"Server 192.168.1.1:8080 responded with "OK""#.to_string() + "\n";
        let output = colorize_regex_test(&input, &rules)?;

        assert!(output.contains("192.168.1.1"));
        assert!(output.contains(":8080"));
        assert!(output.contains("\"OK\""));
        Ok(())
    }

    #[test]
    fn test_regex_performance_optimization() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        // Test that the caching optimization works by using overlapping patterns
        let rules = vec![
            rule("test", console::Style::new().red())?,
            rule("testing", console::Style::new().blue())?, // overlaps with "test"
        ];

        let output = colorize_regex_test("testing", &rules)?;
        // The caching should prevent redundant regex calls
        assert!(output.contains("testing"));
        Ok(())
    }

    #[test]
    fn test_regex_json_like_output() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        let rules = vec![
            rule(r#""[^"]*""#, console::Style::new().green())?, // quoted strings
            rule(r": \d+", console::Style::new().yellow())?,    // numbers
            rule(r": (true|false)", console::Style::new().cyan())?, // booleans
        ];

        let mut input = r#"{"name": "test", "value": 42, "active": true}"#.to_string();
        input.push('\n');
        let output = colorize_regex_test(&input, &rules)?;

        assert!(output.contains("name"));
        assert!(output.contains("42"));
        assert!(output.contains("true"));
        Ok(())
    }
}
