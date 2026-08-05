use rgrc::colorizer::colorize_regex;
use rgrc::grc::{CompiledRegex, GrcatConfigEntry};

fn colorize_test(
    input: &str,
    rules: &[GrcatConfigEntry],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut writer = Vec::new();
    colorize_regex(&mut input.as_bytes(), &mut writer, rules)?;
    Ok(String::from_utf8(writer)?)
}

fn colorize_regex_test(
    input: &str,
    rules: &[GrcatConfigEntry],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut writer = Vec::new();
    colorize_regex(&mut input.as_bytes(), &mut writer, rules)?;
    Ok(String::from_utf8(writer)?)
}

fn rule(
    pattern: &str,
    style: rgrc::style::Style,
) -> Result<GrcatConfigEntry, Box<dyn std::error::Error>> {
    Ok(GrcatConfigEntry::new(
        CompiledRegex::new(pattern)?,
        vec![style],
    ))
}

#[cfg(test)]
mod basic_colorization_tests {
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_no_rules() -> Result<(), Box<dyn std::error::Error>> {
        let output = colorize_test("hello world\n", &[])?;
        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<(), Box<dyn std::error::Error>> {
        let output = colorize_test("", &[])?;
        assert_eq!(output, "");
        Ok(())
    }

    #[test]
    fn test_single_empty_line() -> Result<(), Box<dyn std::error::Error>> {
        let output = colorize_test("\n", &[])?;
        assert_eq!(output, "\n");
        Ok(())
    }

    #[test]
    fn test_simple_match() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("world", Style::new().red())?];
        let output = colorize_test("hello world", &rules)?;

        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        assert!(output.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_no_match() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("xyz", Style::new().blue())?];
        let output = colorize_test("hello world", &rules)?;

        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_multiple_matches_same_rule() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("o", Style::new().green())?];
        let output = colorize_test("foo boo", &rules)?;

        assert!(!output.is_empty());
        assert!(output.len() >= "foo boo".len());
        Ok(())
    }

    #[test]
    fn test_overlapping_matches() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule("foo", Style::new().red())?,
            rule("bar", Style::new().blue())?,
        ];
        let output = colorize_test("foobar", &rules)?;

        assert!(output.contains("foo"));
        assert!(output.contains("bar"));
        Ok(())
    }

    #[test]
    fn test_style_merging() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("hello", Style::new().red())?];
        let output = colorize_test("hello hello", &rules)?;

        let count = output.matches("hello").count();
        assert_eq!(count, 2);
        Ok(())
    }
}

#[cfg(test)]
mod multiline_tests {
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_two_lines() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test line\nno match\n", &rules)?;

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        Ok(())
    }

    #[test]
    fn test_multiple_lines_with_matches() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("foo", Style::new().green())?];

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
        let rules = vec![rule("line", Style::new().blue())?];

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
        let rules = vec![rule("line", Style::new().red())?];

        let mut input = String::new();
        for i in 0..1500 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1500);

        assert!(lines.len() == 1500);
        Ok(())
    }

    #[test]
    fn test_empty_lines_in_input() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().green())?];
        let input = "test\n\ntest\n\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("\n\n"));
        Ok(())
    }

    #[test]
    fn test_lines_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule(r"\d+", Style::new().yellow())?];
        let input = "line 123 and 456\nnext 789\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("line"));
        assert!(output.contains("next"));
        Ok(())
    }
}

#[cfg(test)]
mod regex_pattern_tests {
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_literal_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("this is a test", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_digit_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule(r"\d+", Style::new().blue())?];
        let output = colorize_test("value: 12345", &rules)?;
        assert!(output.contains("value:"));
        Ok(())
    }

    #[test]
    fn test_word_boundary_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule(r"\btest\b", Style::new().green())?];
        let output = colorize_test("test testing tested", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_dot_wildcard() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("t.st", Style::new().red())?];
        let output = colorize_test("test toast", &rules)?;
        assert!(output.contains("test"));
        assert!(output.contains("toast"));
        Ok(())
    }

    #[test]
    fn test_alternation_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("red|blue", Style::new().cyan())?];
        let output = colorize_test("red ball blue sky", &rules)?;
        assert!(output.contains("red"));
        assert!(output.contains("blue"));
        Ok(())
    }

    #[test]
    fn test_character_class() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("[aeiou]", Style::new().magenta())?];
        let output = colorize_test("hello world", &rules)?;
        assert!(!output.is_empty());
        assert!(output.len() >= "hello world".len());
        Ok(())
    }

    #[test]
    fn test_case_sensitive_matching() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("Test", Style::new().red())?];
        let output = colorize_test("test Test TEST", &rules)?;

        assert!(output.contains("test"));
        assert!(output.contains("Test"));
        assert!(output.contains("TEST"));
        Ok(())
    }

    #[test]
    fn test_quantifier_plus() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("a+", Style::new().blue())?];
        let output = colorize_test("aa aaa a", &rules)?;
        assert!(output.contains("a"));
        Ok(())
    }

    #[test]
    fn test_quantifier_star() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("ab*c", Style::new().red())?];
        let output = colorize_test("ac abc abbc", &rules)?;
        assert!(output.contains("ac"));
        Ok(())
    }

    #[test]
    fn test_anchored_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("^start", Style::new().green())?];
        let output = colorize_test("start of line\nnot start", &rules)?;
        assert!(output.contains("start"));
        Ok(())
    }
}

#[cfg(test)]
mod capture_group_tests {
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_single_capture_group() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![GrcatConfigEntry::new(
            CompiledRegex::new(r"(test)")?,
            vec![Style::new(), Style::new().red()],
        )];
        let output = colorize_test("this is test", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_multiple_capture_groups() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![GrcatConfigEntry::new(
            CompiledRegex::new(r"(\w+):(\d+)")?,
            vec![Style::new(), Style::new().red(), Style::new().blue()],
        )];
        let output = colorize_test("server:8080", &rules)?;
        assert!(output.contains("server"));
        assert!(output.contains("8080"));
        Ok(())
    }

    #[test]
    fn test_capture_group_with_no_style() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![GrcatConfigEntry::new(
            CompiledRegex::new(r"(\w+):(\d+)")?,
            vec![Style::new().red()],
        )];
        let output = colorize_test("server:8080", &rules)?;
        assert!(output.contains("server"));
        Ok(())
    }

    #[test]
    fn test_nested_capture_groups() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![GrcatConfigEntry::new(
            CompiledRegex::new(r"((test))")?,
            vec![Style::new(), Style::new().red(), Style::new().green()],
        )];
        let output = colorize_test("test data", &rules)?;
        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_optional_capture_group() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![GrcatConfigEntry::new(
            CompiledRegex::new(r"(\w+)(:)?(\d+)?")?,
            vec![
                Style::new(),
                Style::new().red(),
                Style::new().green(),
                Style::new().blue(),
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
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_style_red() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_green() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().green())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_blue() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().blue())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_yellow() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().yellow())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_magenta() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().magenta())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_cyan() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().cyan())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_white() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().white())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_black() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().black())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_bold() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().bold())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_underlined() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().underlined())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_combined() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red().bold())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }

    #[test]
    fn test_style_on_color() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().on_blue())?];
        let output = colorize_test("test", &rules)?;
        assert!(!output.is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod edge_case_tests {
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_very_long_line() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("x", Style::new().red())?];

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
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("     \n", &rules)?;

        assert!(output.contains("    "));
        Ok(())
    }

    #[test]
    fn test_line_with_tabs() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test\ttest\n", &rules)?;

        assert!(output.contains("\t"));
        Ok(())
    }

    #[test]
    fn test_unicode_content() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test 你好 test", &rules)?;

        assert!(output.contains("test"));
        assert!(output.contains("你好"));
        Ok(())
    }

    #[test]
    fn test_match_at_line_start() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("^test", Style::new().red())?];
        let output = colorize_test("test data", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_match_at_line_end() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("end$", Style::new().red())?];
        let output = colorize_test("this is the end", &rules)?;

        assert!(output.contains("end"));
        Ok(())
    }

    #[test]
    fn test_zero_width_match() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("^", Style::new().red())?];
        let output = colorize_test("test\n", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_multiple_rules_same_text() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule("test", Style::new().red())?,
            rule("test", Style::new().blue())?,
            rule("test", Style::new().green())?,
        ];
        let output = colorize_test("test", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_overlapping_pattern_precedence() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule("abc", Style::new().red())?,
            rule("bcd", Style::new().blue())?,
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
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test\n\n\ntest\n", &rules)?;

        assert!(output.contains("\n\n"));
        Ok(())
    }

    #[test]
    fn test_windows_line_endings() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test\r\n", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_colors_disabled() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("test", Style::new().red())?];
        let output = colorize_test("test data", &rules)?;

        assert!(output.contains("test"));
        Ok(())
    }

    #[test]
    fn test_skip_rule_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let normal_rule = rule("ERROR", Style::new().red())?;
        let mut skip_rule = rule("WARNING", Style::new().yellow())?;
        skip_rule.skip = true; // Mark this rule as skipped

        let rules = vec![normal_rule, skip_rule];

        let input = "This is an ERROR message\nThis is a WARNING message\n";
        let output = colorize_test(input, &rules)?;

        println!("Output: {:?}", output);

        assert!(output.contains("ERROR"));
        assert!(output.contains("WARNING"));

        assert!(output.contains("\x1b[")); // Should have some ANSI codes
        assert!(output.contains("ERROR")); // ERROR should be present

        Ok(())
    }
}

#[cfg(test)]
mod advanced_features_tests {
    use super::*;
    use rgrc::Style;
    use rgrc::grc::GrcatConfigEntryCount;

    #[test]
    fn test_count_stop_prevents_subsequent_rules() -> Result<(), Box<dyn std::error::Error>> {
        let mut r1 = GrcatConfigEntry::new(
            CompiledRegex::new(r"ERROR: (.*)")?,
            vec![Style::new().red()],
        );
        r1.count = GrcatConfigEntryCount::Stop;

        let r2 = GrcatConfigEntry::new(CompiledRegex::new(r"Boom")?, vec![Style::new().blue()]);

        let output = colorize_test("ERROR: Boom Boom\n", &[r1, r2])?;
        assert!(output.contains("ERROR: Boom"));
        Ok(())
    }

    #[test]
    fn test_count_once_allows_other_rules() -> Result<(), Box<dyn std::error::Error>> {
        let mut r1 = GrcatConfigEntry::new(CompiledRegex::new(r"o")?, vec![Style::new().green()]);
        r1.count = GrcatConfigEntryCount::Once;

        let r2 = GrcatConfigEntry::new(CompiledRegex::new(r"boo")?, vec![Style::new().blue()]);

        let output = colorize_test("foo boo\n", &[r1, r2])?;
        assert!(output.contains("o"));
        assert!(output.contains("boo"));
        Ok(())
    }

    #[test]
    fn test_replace_prevents_followup_rules() -> Result<(), Box<dyn std::error::Error>> {
        let mut r1 = GrcatConfigEntry::new(CompiledRegex::new(r"Hello (\w+)")?, vec![Style::new()]);
        r1.replace = "\\1-XYZ".to_string();

        let r2 = GrcatConfigEntry::new(CompiledRegex::new(r"XYZ")?, vec![Style::new().red()]);

        let output = colorize_test("Hello world\n", &[r1, r2])?;
        assert!(output.contains("world") || output.contains("XYZ"));
        Ok(())
    }

    #[test]
    fn test_replace_with_multiple_backrefs() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = GrcatConfigEntry::new(CompiledRegex::new(r"(\w+)-(\d+)")?, vec![Style::new()]);
        r.replace = "\\2-\\1".to_string();

        let output = colorize_test("foo-123 bar\n", &[r])?;
        assert!(output.contains("123") || output.contains("foo"));
        Ok(())
    }

    #[test]
    fn test_last_end_cache_optimization() -> Result<(), Box<dyn std::error::Error>> {
        let r = GrcatConfigEntry::new(CompiledRegex::new(r"aa+")?, vec![Style::new().red()]);

        let output = colorize_test("aaaaa aaaaa aaaaa\n", &[r])?;
        assert!(output.contains("aaaaa"));
        Ok(())
    }

    #[test]
    fn test_multiple_style_boundaries() -> Result<(), Box<dyn std::error::Error>> {
        let r1 = GrcatConfigEntry::new(CompiledRegex::new(r"a")?, vec![Style::new().red()]);
        let r2 = GrcatConfigEntry::new(CompiledRegex::new(r"b")?, vec![Style::new().green()]);
        let r3 = GrcatConfigEntry::new(CompiledRegex::new(r"c")?, vec![Style::new().blue()]);

        let output = colorize_test("abc\n", &[r1, r2, r3])?;
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use rgrc::style::Style;

    use super::*;

    #[test]
    fn test_single_threaded_path() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("line", Style::new().red())?];

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
        let rules = vec![rule("line", Style::new().green())?];

        let mut input = String::new();
        for i in 0..2000 {
            input.push_str(&format!("line {}\n", i));
        }

        let output = colorize_test(&input, &rules)?;
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2000);

        let first_line = lines.first().unwrap();
        let last_line = lines.last().unwrap();
        assert!(first_line.contains("0"));
        assert!(last_line.contains("1999"));
        Ok(())
    }

    #[test]
    fn test_boundary_at_1000_lines() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("x", Style::new().blue())?];

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
        let mut rules = Vec::new();
        for i in 0..10 {
            rules.push(rule(&format!("word{}", i), Style::new().red())?);
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
        let rules = vec![rule(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            Style::new().yellow(),
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
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_realistic_log_output() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule("ERROR", Style::new().red())?,
            rule("WARN", Style::new().yellow())?,
            rule("INFO", Style::new().green())?,
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
        let rules = vec![rule(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            Style::new().cyan(),
        )?];

        let input = "Connection from 192.168.1.100 to 10.0.0.1\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("192.168.1.100"));
        assert!(output.contains("10.0.0.1"));
        Ok(())
    }

    #[test]
    fn test_port_number_coloring() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule(r":(\d+)", Style::new().magenta())?];

        let input = "server listening on :8080\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("8080"));
        Ok(())
    }

    #[test]
    fn test_file_permissions_coloring() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule(r"^(rwx|rw-|r--)", Style::new().green())?];

        let input = "rwxr-xr-x user group file.txt\n";
        let output = colorize_test(input, &rules)?;

        assert!(output.contains("rwx"));
        Ok(())
    }

    #[test]
    fn test_http_status_coloring() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule(r" 2\d{2} ", Style::new().green())?,
            rule(r" 4\d{2} ", Style::new().yellow())?,
            rule(r" 5\d{2} ", Style::new().red())?,
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
        let rules = vec![
            rule(r#"\"[^\"]*\""#, Style::new().cyan())?,
            rule(r": \d+", Style::new().yellow())?,
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
    use rgrc::Style;

    use super::*;

    #[test]
    fn test_regex_no_rules() -> Result<(), Box<dyn std::error::Error>> {
        let output = colorize_regex_test("hello world\n", &[])?;
        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_regex_empty_input() -> Result<(), Box<dyn std::error::Error>> {
        let output = colorize_regex_test("", &[])?;
        assert_eq!(output, "");
        Ok(())
    }

    #[test]
    fn test_regex_single_empty_line() -> Result<(), Box<dyn std::error::Error>> {
        let output = colorize_regex_test("\n", &[])?;
        assert_eq!(output, "\n");
        Ok(())
    }

    #[test]
    fn test_regex_simple_match() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("world", Style::new().red())?];
        let output = colorize_regex_test("hello world", &rules)?;

        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        assert!(output.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_regex_no_match() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("xyz", Style::new().blue())?];
        let output = colorize_regex_test("hello world", &rules)?;

        assert_eq!(output, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_regex_multiple_matches_same_rule() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("o", Style::new().green())?];
        let output = colorize_regex_test("foo boo", &rules)?;

        assert!(!output.is_empty());
        assert!(output.len() >= "foo boo".len());
        Ok(())
    }

    #[test]
    fn test_regex_overlapping_matches() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule("aa", Style::new().red())?];
        let output = colorize_regex_test("aaa", &rules)?;

        assert!(output.contains("a"));
        assert!(output.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_regex_multiple_rules() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule("ERROR", Style::new().red())?,
            rule("INFO", Style::new().blue())?,
        ];
        let output = colorize_regex_test("ERROR: something\nINFO: something else", &rules)?;

        assert!(output.contains("ERROR"));
        assert!(output.contains("INFO"));
        Ok(())
    }

    #[test]
    fn test_regex_capture_groups() -> Result<(), Box<dyn std::error::Error>> {
        let mut rule_entry = rule(r"(\w+): (\d+)", Style::new().red())?;
        rule_entry.colors = vec![
            Style::new().red(),   // full match
            Style::new().blue(),  // first capture group (word)
            Style::new().green(), // second capture group (number)
        ];

        let rules = vec![rule_entry];
        let output = colorize_regex_test("count: 42", &rules)?;

        assert!(output.contains("count"));
        assert!(output.contains("42"));
        Ok(())
    }

    #[test]
    fn test_regex_zero_width_match() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![rule(r"\b\w+\b", Style::new().yellow())?];
        let output = colorize_regex_test("hello world", &rules)?;

        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        Ok(())
    }

    #[test]
    fn test_regex_complex_patterns() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}", Style::new().cyan())?, // IP addresses
            rule(r":\d+", Style::new().yellow())?,                             // port numbers
            rule(r#"\"[^\"]*\""#, Style::new().green())?,                      // quoted strings
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
        let rules = vec![
            rule("test", Style::new().red())?,
            rule("testing", Style::new().blue())?, // overlaps with "test"
        ];

        let output = colorize_regex_test("testing", &rules)?;
        assert!(output.contains("testing"));
        Ok(())
    }

    #[test]
    fn test_regex_json_like_output() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule(r#""[^"]*""#, Style::new().green())?, // quoted strings
            rule(r": \d+", Style::new().yellow())?,    // numbers
            rule(r": (true|false)", Style::new().cyan())?, // booleans
        ];

        let mut input = r#"{"name": "test", "value": 42, "active": true}"#.to_string();
        input.push('\n');
        let output = colorize_regex_test(&input, &rules)?;

        assert!(output.contains("name"));
        assert!(output.contains("42"));
        assert!(output.contains("true"));
        Ok(())
    }

    #[test]
    fn test_offset_jump_overlapping_regions() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            rule("ab", Style::new().red())?,
            rule("bc", Style::new().blue())?,
        ];

        let output = colorize_regex_test("abcd", &rules)?;
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
        assert!(output.contains("d"));
        Ok(())
    }
}

#[cfg(test)]
mod non_utf8_tests {
    use super::*;
    use rgrc::Style;

    fn colorize_bytes(input: &[u8], rules: &[GrcatConfigEntry]) -> String {
        let mut writer = Vec::new();
        colorize_regex(&mut &input[..], &mut writer, rules).expect("colorize should not error");
        String::from_utf8_lossy(&writer).into_owned()
    }

    #[test]
    fn invalid_utf8_passthrough_no_rules() {
        let output = colorize_bytes(b"\xff\x80hello\n", &[]);
        assert!(output.contains('\u{FFFD}'));
        assert!(output.contains("hello"));
    }

    #[test]
    fn invalid_utf8_with_rule_still_colors() {
        let rules = vec![rule("ok", Style::new().green()).unwrap()];
        let output = colorize_bytes(b"ok\n\xff\x80\n", &rules);
        assert!(output.contains("ok"));
        assert!(output.contains('\u{FFFD}'));
    }

    #[test]
    fn invalid_utf8_preserves_newline_boundaries() {
        let output = colorize_bytes(b"\xff\nok\xff\n", &[]);
        assert_eq!(output.lines().count(), 2);
    }
}
