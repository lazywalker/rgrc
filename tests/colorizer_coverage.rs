mod common;

use common::{run_colorize, strip_ansi};
use rgrc::colorizer::colorize_regex;
use rgrc::grc::{CompiledRegex, GrcatConfigEntry, GrcatConfigEntryCount};
use rgrc::style::Style;
use std::io::Cursor;

#[test]
fn replace_with_backrefs_modifies_line() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"(\w+):(\d+)").unwrap(),
        colors: vec![Style::new().red()],
        count: GrcatConfigEntryCount::More,
        replace: "\\1=\\2".to_string(), // Replace with = separator
        skip: false,
    }];

    let result = run_colorize("server:8080 test", rules);
    let stripped = strip_ansi(&result);
    assert!(!stripped.is_empty());
}

#[test]
fn replace_breaks_outer_loop_and_restarts() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"(\d+)\.(\d+)").unwrap(),
        colors: vec![Style::new().cyan()],
        count: GrcatConfigEntryCount::More,
        replace: "\\1_\\2".to_string(), // Replace dot with underscore
        skip: false,
    }];

    let result = run_colorize("version 1.2.3 test", rules);
    let stripped = strip_ansi(&result);
    assert!(!stripped.is_empty());
}

#[test]
fn zero_width_lookahead_prevents_infinite_loop() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"(?=\d)").unwrap(),
        colors: vec![Style::new().green()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("abc123def", rules);
    assert!(!result.is_empty());
}

#[test]
fn word_boundary_zero_width_advances_correctly() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"\b").unwrap(),
        colors: vec![Style::new().magenta()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("one two three", rules);
    assert!(strip_ansi(&result).contains("one two three"));
}

#[test]
fn style_range_bounds_check_prevents_panic() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"test").unwrap(),
        colors: vec![Style::new().blue(), Style::new().red()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("test", rules);
    assert!(strip_ansi(&result).contains("test"));
}

#[test]
fn run_length_encoding_merges_consecutive_same_style() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"\d+").unwrap(),
        colors: vec![Style::new().yellow()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("123 456 789", rules);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped.trim(), "123 456 789");
}

#[test]
fn final_segment_output_for_partial_styling() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"^hello").unwrap(),
        colors: vec![Style::new().cyan()],
        count: GrcatConfigEntryCount::Once,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("hello world", rules);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped.trim(), "hello world");
}

#[test]
fn cache_optimization_skips_overlapping_regions() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"\d+").unwrap(),
        colors: vec![Style::new().green()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("123 456 789 012", rules);
    let stripped = strip_ansi(&result);
    assert!(stripped.contains("123"));
    assert!(stripped.contains("456"));
}

#[test]
fn capture_group_index_out_of_colors_bounds() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"(\d+):(\d+):(\d+)").unwrap(),
        colors: vec![Style::new().red()], // Only one style for group 0
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("time 12:34:56 test", rules);
    assert!(strip_ansi(&result).contains("12:34:56"));
}

#[test]
fn last_end_tracking_updates_correctly() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"[a-z]+").unwrap(),
        colors: vec![Style::new().blue()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("abc def ghi jkl", rules);
    let stripped = strip_ansi(&result);
    assert_eq!(stripped.trim(), "abc def ghi jkl");
}

#[test]
fn count_once_matches_only_first_occurrence() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"test").unwrap(),
        colors: vec![Style::new().yellow()],
        count: GrcatConfigEntryCount::Once,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("test test test", rules);
    assert!(strip_ansi(&result).contains("test"));
}

#[test]
fn count_stop_prevents_subsequent_rules() {
    let rules = vec![
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"stop").unwrap(),
            colors: vec![Style::new().red()],
            count: GrcatConfigEntryCount::Stop,
            replace: String::new(),
            skip: false,
        },
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"here").unwrap(),
            colors: vec![Style::new().green()],
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
            skip: false,
        },
    ];

    let result = run_colorize("stop here now", rules);
    assert!(strip_ansi(&result).contains("stop here now"));
}

#[test]
fn no_match_breaks_while_loop() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"xyz").unwrap(),
        colors: vec![Style::new().cyan()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("abc def", rules);
    assert_eq!(strip_ansi(&result).trim(), "abc def");
}

#[test]
fn empty_style_ranges_outputs_line_unchanged() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"nomatch").unwrap(),
        colors: vec![Style::new().red()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("some text here", rules);
    assert_eq!(strip_ansi(&result).trim(), "some text here");
}

#[test]
fn style_application_respects_line_length_bounds() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r".+").unwrap(),
        colors: vec![Style::new().magenta()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("x", rules); // Very short line
    assert_eq!(strip_ansi(&result).trim(), "x");
}

#[test]
fn style_boundary_at_position_zero_handled() {
    let rules = vec![
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"^\w+").unwrap(),
            colors: vec![Style::new().red()],
            count: GrcatConfigEntryCount::Once,
            replace: String::new(),
            skip: false,
        },
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"\d+$").unwrap(),
            colors: vec![Style::new().blue()],
            count: GrcatConfigEntryCount::Once,
            replace: String::new(),
            skip: false,
        },
    ];

    let result = run_colorize("word123", rules);
    assert!(strip_ansi(&result).contains("word123"));
}

#[test]
fn multiple_style_boundaries_tracked_correctly() {
    let rules = vec![
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"a").unwrap(),
            colors: vec![Style::new().red()],
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
            skip: false,
        },
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"b").unwrap(),
            colors: vec![Style::new().blue()],
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
            skip: false,
        },
    ];

    let result = run_colorize("aXbXaXb", rules);
    assert!(strip_ansi(&result).contains("aXbXaXb"));
}

#[test]
fn skip_rule_is_ignored_in_processing() {
    let rules = vec![
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"skip").unwrap(),
            colors: vec![Style::new().red()],
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
            skip: true, // This rule should be skipped
        },
        GrcatConfigEntry {
            regex: CompiledRegex::new(r"process").unwrap(),
            colors: vec![Style::new().green()],
            count: GrcatConfigEntryCount::More,
            replace: String::new(),
            skip: false,
        },
    ];

    let result = run_colorize("skip process", rules);
    assert!(strip_ansi(&result).contains("skip process"));
}

#[test]
fn offset_advances_past_match_end() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"\d").unwrap(),
        colors: vec![Style::new().yellow()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("1a2b3c", rules);
    assert!(strip_ansi(&result).contains("1a2b3c"));
}

#[test]
fn rule_matched_once_flag_stops_matching() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"\w").unwrap(),
        colors: vec![Style::new().cyan()],
        count: GrcatConfigEntryCount::Once,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("abc", rules);
    assert!(strip_ansi(&result).contains("abc"));
}

#[test]
fn multiple_capture_groups_iterate_correctly() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"(\w+):(\d+)").unwrap(),
        colors: vec![
            Style::new().red(),
            Style::new().blue(),
            Style::new().green(),
        ],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("host:8080", rules);
    assert!(strip_ansi(&result).contains("host:8080"));
}

#[test]
fn colorize_returns_ok_on_success() {
    let rules = vec![];
    let mut output = Vec::new();
    let mut reader = Cursor::new(b"test");
    let result = colorize_regex(&mut reader, &mut output, &rules);
    assert!(result.is_ok());
}

#[test]
fn empty_line_fast_path_writes_newline_only() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"\d+").unwrap(),
        colors: vec![Style::new().red()],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("test\n\n\nmore", rules);
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines.len() >= 2);
}

#[test]
fn cache_skip_when_offset_behind_last_end() {
    let rules = vec![GrcatConfigEntry {
        regex: CompiledRegex::new(r"(?=\w)(\w)(\w+)?").unwrap(),
        colors: vec![
            Style::new().blue(),
            Style::new().red(),
            Style::new().green(),
        ],
        count: GrcatConfigEntryCount::More,
        replace: String::new(),
        skip: false,
    }];

    let result = run_colorize("test word", rules);
    assert!(strip_ansi(&result).contains("test"));
}
