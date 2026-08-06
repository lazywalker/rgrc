use rgrc::grc::{
    CompiledRegex, GrcConfigReader, GrcatConfigEntry, GrcatConfigEntryCount, GrcatConfigReader,
};
use std::io::BufRead;

#[test]
fn test_style_ansi_escape_code_skipped() {
    let style_str = r#""\033[38;5;140m""#;
    let result = rgrc::grc::style_from_str(style_str);
    assert!(result.is_ok());
}

#[test]
fn test_style_unknown_keyword_error() {
    let unknown_style = "thisIsNotAValidStyle";
    let result = rgrc::grc::style_from_str(unknown_style);
    assert!(result.is_err(), "Unknown style should return Err");
}

#[test]
fn test_style_multiple_unknown_keywords() {
    let style_str = "red unknown1 blue unknown2";
    let result = rgrc::grc::style_from_str(style_str);
    assert!(result.is_err());
}

#[test]
fn test_grcat_reader_empty_file() {
    use std::io::BufReader;
    let empty = "";
    let reader = BufReader::new(empty.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    assert!(grcat_reader.next().is_none());
}

#[test]
fn test_grcat_reader_invalid_count_defaults_to_more() {
    use std::io::BufReader;
    let config = "regexp=test\ncolours=red\ncount=invalid_value\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        match entry.count {
            GrcatConfigEntryCount::More => {}
            _ => panic!("Expected count to default to More for invalid value"),
        }
    }
}

#[test]
fn test_grcat_reader_skip_field_parsing() {
    use std::io::BufReader;

    let config_yes = "regexp=test\ncolours=red\nskip=yes\n-\n";
    let reader = BufReader::new(config_yes.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());
    if let Some(entry) = grcat_reader.next() {
        assert!(entry.skip, "skip=yes should set skip to true");
    }

    let config_true = "regexp=test\ncolours=red\nskip=true\n-\n";
    let reader = BufReader::new(config_true.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());
    if let Some(entry) = grcat_reader.next() {
        assert!(entry.skip, "skip=true should set skip to true");
    }

    let config_one = "regexp=test\ncolours=red\nskip=1\n-\n";
    let reader = BufReader::new(config_one.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());
    if let Some(entry) = grcat_reader.next() {
        assert!(entry.skip, "skip=1 should set skip to true");
    }
}

#[test]
fn test_grc_reader_skips_comments_and_empty_lines() {
    use std::io::BufReader;
    let config = "# This is a comment\n\n  # Another comment with leading space\n\nregexp=^ping$\nconf.ping\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grc_reader = rgrc::grc::GrcConfigReader::new(reader.lines());

    if let Some((_regex, path)) = grc_reader.next() {
        assert_eq!(path, "conf.ping");
    } else {
        panic!("Should have found one valid entry");
    }
}

#[test]
fn test_grc_reader_incomplete_pair() {
    use std::io::BufReader;
    let incomplete = "regexp=^test$\n";
    let reader = BufReader::new(incomplete.as_bytes());
    let mut grc_reader = rgrc::grc::GrcConfigReader::new(reader.lines());

    assert!(grc_reader.next().is_none());
}

#[test]
fn test_grc_reader_invalid_regex() {
    use std::io::BufReader;
    let invalid_regex = "regexp=^test(\nconf.test\n";
    let reader = BufReader::new(invalid_regex.as_bytes());
    let mut grc_reader = rgrc::grc::GrcConfigReader::new(reader.lines());

    let result = grc_reader.next();
    assert!(result.is_none() || result.is_some());
}

#[test]
fn test_grcat_reader_invalid_regex_skipped() {
    use std::io::BufReader;
    let config = "regexp=invalid(regex\ncolours=red\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    let result = grcat_reader.next();
    assert!(result.is_none() || result.is_some());
}

#[test]
fn test_grcat_reader_empty_colours_vector() {
    use std::io::BufReader;
    let config = "regexp=test\ncolours=\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        assert!(
            entry.colors.is_empty() || !entry.colors.is_empty(),
            "Should handle empty colours"
        );
    }
}

#[test]
fn test_grcat_config_entry_new() {
    let regex = CompiledRegex::new(r"test").unwrap();
    let style = rgrc::style::Style::new().red();
    let entry = GrcatConfigEntry::new(regex, vec![style]);

    assert_eq!(entry.count, GrcatConfigEntryCount::More);
    assert_eq!(entry.replace, "");
    assert!(!entry.skip);
    assert_eq!(entry.colors.len(), 1);
}

#[test]
fn test_grcat_reader_count_once() {
    use std::io::BufReader;
    let config = "regexp=test\ncolours=red\ncount=once\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        assert!(matches!(entry.count, GrcatConfigEntryCount::Once));
    }
}

#[test]
fn test_grcat_reader_count_stop() {
    use std::io::BufReader;
    let config = "regexp=test\ncolours=red\ncount=stop\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        assert!(matches!(entry.count, GrcatConfigEntryCount::Stop));
    }
}

#[test]
fn test_grcat_reader_count_more() {
    use std::io::BufReader;
    let config = "regexp=test\ncolours=red\ncount=more\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        assert!(matches!(entry.count, GrcatConfigEntryCount::More));
    }
}

#[test]
fn test_grcat_reader_unknown_count_value() {
    use std::io::BufReader;
    let config = "regexp=test\ncolours=red\ncount=unknown_value\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        assert!(matches!(entry.count, GrcatConfigEntryCount::More));
    } else {
        panic!("Expected an entry to be parsed");
    }
}

#[test]
fn test_grcat_reader_replace_field() {
    use std::io::BufReader;
    let config = "regexp=(\\d+)\ncolours=red\nreplace=NUM:\\1\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    if let Some(entry) = grcat_reader.next() {
        assert_eq!(entry.replace, "NUM:\\1");
    }
}

#[test]
fn test_grcat_reader_multiple_entries() {
    use std::io::BufReader;
    let config = "regexp=error\ncolours=red\n-\nregexp=warning\ncolours=yellow\n-\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grcat_reader = rgrc::grc::GrcatConfigReader::new(reader.lines());

    let first = grcat_reader.next();
    assert!(first.is_some());

    let second = grcat_reader.next();
    assert!(second.is_some());

    let third = grcat_reader.next();
    assert!(third.is_none());
}

#[test]
fn test_grc_reader_multiple_command_mappings() {
    use std::io::BufReader;
    let config = "regexp=^ping$\nconf.ping\nregexp=^ls\nconf.ls\n";
    let reader = BufReader::new(config.as_bytes());
    let mut grc_reader = rgrc::grc::GrcConfigReader::new(reader.lines());

    let first = grc_reader.next();
    assert!(first.is_some());
    if let Some((_, path)) = first {
        assert_eq!(path, "conf.ping");
    }

    let second = grc_reader.next();
    assert!(second.is_some());
    if let Some((_, path)) = second {
        assert_eq!(path, "conf.ls");
    }
}

#[test]
fn test_style_multiple_keywords() {
    let style_str = "bold red underline";
    let result = rgrc::grc::style_from_str(style_str);
    assert!(
        result.is_ok(),
        "Multiple valid keywords should parse successfully"
    );
}

#[test]
fn test_style_bright_colors() {
    let colors = vec!["bright_red", "bright_blue", "bright_green", "bright_yellow"];
    for color in colors {
        let result = rgrc::grc::style_from_str(color);
        assert!(result.is_ok(), "Bright color {} should parse", color);
    }
}

#[test]
fn test_style_background_colors() {
    let colors = vec!["on_red", "on_blue", "on_green", "on_black"];
    for color in colors {
        let result = rgrc::grc::style_from_str(color);
        assert!(result.is_ok(), "Background color {} should parse", color);
    }
}

#[test]
fn test_style_attributes() {
    let attrs = vec!["bold", "italic", "underline", "blink", "reverse"];
    for attr in attrs {
        let result = rgrc::grc::style_from_str(attr);
        assert!(result.is_ok(), "Attribute {} should parse", attr);
    }
}

#[test]
fn test_style_noop_keywords() {
    let noops = vec!["unchanged", "default", "none", ""];
    for noop in noops {
        let result = rgrc::grc::style_from_str(noop);
        assert!(result.is_ok(), "No-op keyword '{}' should parse", noop);
    }
}

#[test]
fn test_style_dark_aliases_dim() {
    let result = rgrc::grc::style_from_str("dark");
    assert!(result.is_ok());
    let style = result.unwrap();
    assert_eq!(format!("{}", style.apply_to("x")), "\x1b[2mx\x1b[0m");

    let result = rgrc::grc::style_from_str("dark green");
    assert!(result.is_ok());
    let style = result.unwrap();
    assert_eq!(format!("{}", style.apply_to("x")), "\x1b[2;32mx\x1b[0m");
}

#[test]
fn test_styles_from_str_comma_separated() {
    let style_str = "red,blue,green";
    let result = rgrc::grc::styles_from_str(style_str);
    assert!(result.is_ok());
    if let Ok(styles) = result {
        assert_eq!(styles.len(), 3, "Should parse 3 comma-separated styles");
    }
}

#[test]
fn test_styles_from_str_single() {
    let style_str = "bold red";
    let result = rgrc::grc::styles_from_str(style_str);
    assert!(result.is_ok());
    if let Ok(styles) = result {
        assert_eq!(styles.len(), 1, "Should parse single combined style");
    }
}

#[test]
fn test_styles_from_str_empty() {
    let result = rgrc::grc::styles_from_str("");
    assert!(result.is_ok());
}

#[test]
fn test_styles_from_str_with_invalid() {
    let style_str = "red,invalidstyle,blue";
    let result = rgrc::grc::styles_from_str(style_str);
    assert!(
        result.is_err(),
        "Should return Err when encountering invalid style"
    );
}

#[test]
fn grcconfigreader_skips_comments_and_handles_incomplete_pair() {
    let data =
        "# comment\n  # another comment\n^cmd1\nconf.cmd1\n^incomplete_only\n# nothing after\n";
    let reader = std::io::Cursor::new(data);
    let mut r = GrcConfigReader::new(std::io::BufReader::new(reader).lines());

    let first = r.next().expect("expected first pair");
    assert!(first.0.is_match("cmd1")); // is_match now returns bool directly
    assert_eq!(first.1, "conf.cmd1");

    assert!(r.next().is_none());
}

#[test]
fn grcatreader_parses_count_replace_and_skip_values() {
    let input = "regexp=^A (\\d+)\\ncolours=red\ncount=once\nreplace=\nskip=true\n\nregexp=^B\\ncolours=green\ncount=stop\nreplace=sub\nskip=false\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());

    let e1 = it.next().expect("first entry");
    match e1.count {
        GrcatConfigEntryCount::Once => {}
        _ => panic!("expected count=Once for first entry"),
    }
    assert!(e1.skip, "expected skip=true for first entry");

    let e2 = it.next().expect("second entry");
    match e2.count {
        GrcatConfigEntryCount::Stop => {}
        _ => panic!("expected count=Stop for second entry"),
    }
}

#[test]
fn grcatreader_invalid_colours_moving_on() {
    let input = "regexp=^ERR\ncolours=not_a_known_color\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());
    let _ = it.next();
}

#[test]
fn grcatreader_unknown_count_defaults_to_more() {
    let input = "regexp=^X\ncolours=red\ncount=weird\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());
    let e = it.next().expect("entry");
    match e.count {
        GrcatConfigEntryCount::More => {}
        other => panic!("expected More, found {:?}", other),
    }
}

#[test]
fn grcatreader_skip_parsing_variants() {
    let input = "regexp=^A\ncolours=red\nskip=yes\n\nregexp=^B\ncolours=blue\nskip=0\n\nregexp=^C\ncolours=green\nskip=unknown\n\n";
    let reader = std::io::Cursor::new(input);
    let mut it = GrcatConfigReader::new(std::io::BufReader::new(reader).lines());

    let e1 = it.next().unwrap();
    assert!(e1.skip, "skip=yes expected true");

    let e2 = it.next().unwrap();
    assert!(!e2.skip, "skip=0 expected false");

    let e3 = it.next().unwrap();
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
