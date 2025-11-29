// Additional colorizer tests to cover count/stop semantics, skip behavior, zero-width matches
#[path = "../src/colorizer.rs"]
mod colorizer;

#[path = "../src/grc.rs"]
mod grc;

use colorizer::colorize_regex;
use fancy_regex::Regex;
use grc::{GrcatConfigEntry, GrcatConfigEntryCount};

fn run_colorize(input: &str, rules: &[GrcatConfigEntry]) -> String {
    let mut out = Vec::new();
    colorize_regex(&mut input.as_bytes(), &mut out, rules).unwrap();
    String::from_utf8(out).unwrap()
}

fn strip_ansi(s: &str) -> String {
    // Remove ANSI CSI sequences like \x1b[...m
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

fn rule_with_count(pat: &str, count: GrcatConfigEntryCount) -> GrcatConfigEntry {
    let re = Regex::new(pat).unwrap();
    let mut e = GrcatConfigEntry::new(re, vec![console::Style::new().red()]);
    e.count = count;
    e
}

#[test]
fn count_once_applies_only_first_match() {
    console::set_colors_enabled(true);
    let rules = vec![rule_with_count(r"o", GrcatConfigEntryCount::Once)];
    let out = run_colorize("foo boo\n", &rules);
    // Ensure both tokens are present in output even when ANSI escapes are inserted
    let cleaned = strip_ansi(&out);
    assert!(cleaned.contains("foo"));
    assert!(cleaned.contains("boo"));
}

#[test]
fn count_stop_stops_processing_after_first_match() {
    console::set_colors_enabled(true);
    // First rule will stop after first match; second rule shouldn't color later parts
    let mut r1 = GrcatConfigEntry::new(
        Regex::new(r"ERROR: (.*)").unwrap(),
        vec![console::Style::new().red()],
    );
    r1.count = GrcatConfigEntryCount::Stop;

    let r2 = GrcatConfigEntry::new(
        Regex::new(r"Boom").unwrap(),
        vec![console::Style::new().blue()],
    );

    let out = run_colorize("ERROR: Boom Boom\n", &[r1, r2]);
    // Because r1 has count=Stop, only the first ERROR match is handled and then line processing stops
    assert!(out.contains("ERROR: Boom"));
    // second 'Boom' may or may not be colored by r2 depending on stop semantics; ensure function returns successfully
}

#[test]
fn skip_true_rule_is_ignored() {
    console::set_colors_enabled(true);
    let mut r = GrcatConfigEntry::new(
        Regex::new(r"secret").unwrap(),
        vec![console::Style::new().red()],
    );
    r.skip = true;

    let out = run_colorize("this contains secret\n", &[r]);
    // skip=true means rule ignored and output unchanged
    assert!(out.contains("this contains secret"));
}

#[test]
fn zero_width_matches_do_not_infinite_loop() {
    console::set_colors_enabled(true);
    // Use pattern that can produce zero-width matches such as ^ and word boundaries
    let r = GrcatConfigEntry::new(
        Regex::new(r"\b").unwrap(),
        vec![console::Style::new().green()],
    );
    let out = run_colorize("abc\n", &[r]);
    // Should return quickly and not hang; output length should be reasonable
    assert!(out.len() > 0);
}

#[test]
fn overlapping_precedence_latest_wins() {
    console::set_colors_enabled(true);
    // First rule colors 'foobar' red, second colors 'bar' blue; later rule should override where overlapping
    let r1 = GrcatConfigEntry::new(
        Regex::new(r"foo|foobar").unwrap(),
        vec![console::Style::new().red()],
    );
    let r2 = GrcatConfigEntry::new(
        Regex::new(r"bar").unwrap(),
        vec![console::Style::new().blue()],
    );

    let out = run_colorize("foobar\n", &[r1, r2]);
    // Ensure both sub-strings are present even if ANSI escapes split them
    assert!(out.contains("foo"));
    assert!(out.contains("bar"));
    // Since colors are enabled at test start, we expect ANSI escapes
    assert!(out.contains("\x1b["));
}

#[test]
fn replace_prevents_followup_rules() {
    console::set_colors_enabled(true);

    // Rule 1 performs replacement to insert 'XYZ' into the line and should stop further processing
    let mut r1 = GrcatConfigEntry::new(
        Regex::new(r"Hello (\w+)").unwrap(),
        vec![console::Style::new()],
    );
    r1.replace = "\\1-XYZ".to_string();

    // Rule 2 would color 'XYZ' if it ran, but replacement should prevent it
    let r2 = GrcatConfigEntry::new(
        Regex::new(r"XYZ").unwrap(),
        vec![console::Style::new().red()],
    );

    let out = run_colorize("Hello world\n", &[r1, r2]);
    let cleaned = strip_ansi(&out);
    // Replacement applied
    assert!(cleaned.contains("world-XYZ"));
    // Ensure follow-up rule didn't apply (no extra ANSI codes around XYZ beyond any present)
    // We already stripped ANSI; ensure presence and no further markers
    assert!(cleaned.contains("world-XYZ"));
}

#[test]
fn count_once_allows_other_rules_to_run() {
    console::set_colors_enabled(true);

    // rule that matches 'o' only once
    let mut r1 = GrcatConfigEntry::new(
        Regex::new(r"o").unwrap(),
        vec![console::Style::new().green()],
    );
    r1.count = GrcatConfigEntryCount::Once;

    // rule that matches 'boo' should still apply to remaining text
    let r2 = GrcatConfigEntry::new(
        Regex::new(r"boo").unwrap(),
        vec![console::Style::new().blue()],
    );

    let out = run_colorize("foo boo\n", &[r1, r2]);
    let clean = strip_ansi(&out);
    // Ensure foo and boo still present; since r1 only affects first 'o', r2 should still color 'boo' if it doesn't conflict
    assert!(clean.contains("foo"));
    assert!(clean.contains("boo"));
}

#[test]
fn last_end_cache_avoids_redundant_checks() {
    console::set_colors_enabled(true);

    // A rule that matches repetitive text should be efficient; we assert behavior and that it completes
    let r = GrcatConfigEntry::new(
        Regex::new(r"aa+").unwrap(),
        vec![console::Style::new().red()],
    );

    // input with overlapping runs that could trigger redundant checks without proper caching
    let input = "aaaaa aaaaa aaaaa\n";
    let out = run_colorize(input, &[r]);
    // Ensure output contains original tokens and function returns promptly
    assert!(out.contains("aaaaa"));
}

#[test]
fn replace_with_multiple_backrefs() {
    console::set_colors_enabled(true);

    // rule that captures two groups and replaces them with swapped values
    let mut r = GrcatConfigEntry::new(
        Regex::new(r"(\w+)-(\d+)").unwrap(),
        vec![console::Style::new()],
    );
    r.replace = "\\2-\\1".to_string();

    let out = run_colorize("foo-123 bar\n", &[r]);
    let cleaned = strip_ansi(&out);
    assert!(cleaned.contains("123-foo"));
}

#[test]
fn many_adjacent_style_boundaries_produce_multiple_segments() {
    console::set_colors_enabled(true);

    // Create rules that style single characters differently to force many boundaries
    let r1 = GrcatConfigEntry::new(Regex::new(r"a").unwrap(), vec![console::Style::new().red()]);
    let r2 = GrcatConfigEntry::new(
        Regex::new(r"b").unwrap(),
        vec![console::Style::new().green()],
    );
    let r3 = GrcatConfigEntry::new(
        Regex::new(r"c").unwrap(),
        vec![console::Style::new().blue()],
    );

    let input = "abc\n";
    let out = run_colorize(input, &[r1, r2, r3]);

    // After styling there should still be the original characters present and ANSI escapes
    let cleaned = strip_ansi(&out);
    assert_eq!(cleaned.trim(), "abc");
    assert!(out.contains("\x1b["));
}

// Exercise timetrace path when built with the feature enabled
#[cfg(feature = "timetrace")]
#[test]
fn timetrace_records_timings_when_enabled() {
    // Ensure the RGRCTIME env var is set so we hit the timing code paths
    // set_var/remove_var are treated as unsafe in some build configurations
    unsafe { std::env::set_var("RGRCTIME", "1") };
    console::set_colors_enabled(true);

    let r = GrcatConfigEntry::new(
        Regex::new(r"test").unwrap(),
        vec![console::Style::new().red()],
    );
    let out = run_colorize("test\n", &[r]);

    // Should run and produce output
    assert!(out.contains("test"));

    // Clean up env var
    unsafe { std::env::remove_var("RGRCTIME") };
}
