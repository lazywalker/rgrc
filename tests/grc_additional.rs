// Additional grc parsing and style parsing edge-case tests
#[path = "../src/enhanced_regex.rs"]
mod enhanced_regex;

#[path = "../src/grc.rs"]
mod grc;

use grc::{GrcConfigReader, GrcatConfigEntryCount, GrcatConfigReader};
use std::io::BufRead;

// Note: private helpers style_from_str / styles_from_str are exercised indirectly
// by parsing grcat config entries (see tests below).
#[test]
fn grcconfigreader_skips_comments_and_handles_incomplete_pair() {
    let data =
        "# comment\n  # another comment\n^cmd1\nconf.cmd1\n^incomplete_only\n# nothing after\n";
    let reader = std::io::Cursor::new(data);
    let mut r = GrcConfigReader::new(std::io::BufReader::new(reader).lines());

    // first yields a valid pair
    let first = r.next().expect("expected first pair");
    assert!(first.0.is_match("cmd1")); // is_match now returns bool directly
    assert_eq!(first.1, "conf.cmd1");

    // then we hit an incomplete pattern-only rule; iterator should stop (None)
    assert!(r.next().is_none());
}

#[test]
fn grcatreader_parses_count_replace_and_skip_values() {
    // Because entries are detected via alphanumeric line start we need simpler content
    let input = "regexp=^A (\\d+)\\ncolours=red\ncount=once\nreplace=\nskip=true\n\nregexp=^B\\ncolours=green\ncount=stop\nreplace=sub\nskip=false\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());

    // first entry should parse and reflect count and skip
    let e1 = it.next().expect("first entry");
    match e1.count {
        GrcatConfigEntryCount::Once => {}
        _ => panic!("expected count=Once for first entry"),
    }
    // skip was set true -> e1.skip should be true
    assert!(e1.skip, "expected skip=true for first entry");

    // second entry
    let e2 = it.next().expect("second entry");
    // count should be Stop for second entry (but types don't implement Debug Display here)
    match e2.count {
        GrcatConfigEntryCount::Stop => {}
        _ => panic!("expected count=Stop for second entry"),
    }
}

#[test]
#[should_panic]
fn grcatreader_invalid_colours_panics() {
    // colours line contains an invalid token -> styles_from_str unwrap will panic
    let input = "regexp=^ERR\ncolours=not_a_known_color\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());
    // calling next should panic when attempting to parse invalid colours
    let _ = it.next();
}

#[test]
fn grcatreader_unknown_count_defaults_to_more() {
    let input = "regexp=^X\ncolours=red\ncount=weird\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());
    let e = it.next().expect("entry");
    // unknown count should fallback to More
    match e.count {
        GrcatConfigEntryCount::More => {}
        other => panic!("expected More, found {:?}", other),
    }
}

#[test]
fn grcatreader_skip_parsing_variants() {
    // Test several skip variations
    let input = "regexp=^A\ncolours=red\nskip=yes\n\nregexp=^B\ncolours=blue\nskip=0\n\nregexp=^C\ncolours=green\nskip=unknown\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());

    let e1 = it.next().unwrap();
    assert!(e1.skip, "skip=yes expected true");

    let e2 = it.next().unwrap();
    assert!(!e2.skip, "skip=0 expected false");

    let e3 = it.next().unwrap();
    // unknown skip values default to false
    assert!(!e3.skip, "unknown skip value should default to false");
}

#[test]
fn grcatreader_missing_colours_is_empty_vector() {
    let input = "regexp=^Z\n# no colours line\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());
    let e = it.next().expect("entry");
    assert!(
        e.colors.is_empty(),
        "missing colours should lead to empty colors vector"
    );
}
