use rgrc::grc::CompiledRegex;

#[test]
fn test_simple_pattern_uses_fast_regex() {
    let pattern = r"\bhello\b";
    let compiled = CompiledRegex::new(pattern).expect("Should compile simple pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            println!("✓ Simple pattern uses Fast regex engine");
        }
        CompiledRegex::Enhanced(_) => {
            panic!("Simple pattern should use Fast regex, not Enhanced");
        }
    }
}

#[test]
fn test_complex_pattern_uses_enhanced() {
    let pattern = r"hello(?=\d+)";
    let compiled = CompiledRegex::new(pattern).expect("Should compile complex pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Complex pattern with lookahead should use Enhanced regex, not Fast");
        }
        CompiledRegex::Enhanced(_) => {
            println!("✓ Lookahead pattern uses Enhanced regex engine");
        }
    }
}

#[test]
fn test_lookbehind_pattern_uses_enhanced() {
    let pattern = r"(?<=\d{3})hello";
    let compiled = CompiledRegex::new(pattern).expect("Should compile lookbehind pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Complex pattern with lookbehind should use Enhanced regex, not Fast");
        }
        CompiledRegex::Enhanced(_) => {
            println!("✓ Lookbehind pattern uses Enhanced regex engine");
        }
    }
}

#[test]
#[cfg(not(feature = "fancy-regex"))]
fn test_backreference_fails() {
    let pattern = r"(\w+)\s+\1";
    let compiled = CompiledRegex::new(pattern);

    assert!(
        compiled.is_err(),
        "Backreference pattern should fail to compile"
    );
    println!("✓ Backreference pattern correctly fails to compile (not supported)");
}

#[test]
#[cfg(feature = "fancy-regex")]
fn test_backreference_works_with_fancy() {
    let pattern = r"(\w+)\s+\1";
    let compiled = CompiledRegex::new(pattern);

    assert!(
        compiled.is_ok(),
        "Backreference pattern should compile with fancy-regex"
    );
    println!("✓ Backreference pattern works with fancy-regex");

    let regex = compiled.unwrap();
    assert!(
        regex.is_match("hello hello"),
        "Should match duplicated word"
    );
    assert!(
        !regex.is_match("hello world"),
        "Should not match different words"
    );
}

#[test]
fn test_multiple_simple_patterns() {
    let simple_patterns = vec![
        r"\d+",          // digits
        r"[a-z]+",       // letters
        r"^\w+",         // word at start
        r"\d+$",         // digits at end
        r"foo|bar",      // alternation (simple)
        r"hello.*world", // simple wildcard
    ];

    for pattern in simple_patterns {
        let compiled = CompiledRegex::new(pattern)
            .unwrap_or_else(|_| panic!("Should compile pattern: {}", pattern));

        match compiled {
            CompiledRegex::Fast(_) => {
                println!("✓ Pattern '{}' uses Fast regex", pattern);
            }
            CompiledRegex::Enhanced(_) => {
                panic!(
                    "Simple pattern '{}' should use Fast regex, not Enhanced",
                    pattern
                );
            }
        }
    }
}

#[test]
fn test_ipv4_pattern_simple_case() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(
        regex.is_match("192.168.1.1"),
        "Should match first octet in IPv4"
    );
}

#[test]
fn test_ipv4_pattern_with_prefix() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(
        regex.is_match("IP: 192.168.1.1"),
        "Should match with prefix"
    );
}

#[test]
fn test_ipv4_pattern_embedded() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(
        regex.is_match("text 10.0.0.255 more"),
        "Should match embedded IPv4"
    );
}

#[test]
fn test_ipv4_pattern_incomplete() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    assert!(
        !regex.is_match("192.168"),
        "Should not match two octets only"
    );
    assert!(!regex.is_match("192"), "Should not match single octet");
}

#[test]
fn test_ipv4_pattern_multiple_addresses() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    let text = "Connect from 192.168.1.1 to 10.0.0.1";
    assert!(regex.is_match(text), "Should find IPv4 addresses in text");
}

#[test]
fn test_enhanced_ipv4_find_match() {
    use rgrc::enhanced_regex::EnhancedRegex;

    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = EnhancedRegex::new(pattern).unwrap();
    let text = "192.168.1.1";

    let mat = regex.find_from_pos(text, 0);
    assert!(mat.is_some(), "Should find match in IPv4 address");

    let mat = mat.unwrap();
    assert_eq!(mat.start(), 0, "Match should start at position 0");
    assert_eq!(mat.end(), 3, "Match should end at position 3");
    assert_eq!(mat.as_str(), "192", "Should match first octet '192'");
}

#[test]
fn test_enhanced_ipv4_is_match() {
    use rgrc::enhanced_regex::EnhancedRegex;

    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = EnhancedRegex::new(pattern).unwrap();

    assert!(regex.is_match("192.168.1.1"), "is_match should return true");
    assert!(
        regex.is_match("10.0.0.1"),
        "is_match should work for different IPs"
    );
    assert!(
        !regex.is_match("192.168"),
        "is_match should return false for incomplete IP"
    );
}

#[test]
fn test_enhanced_ipv4_find_from_different_positions() {
    use rgrc::enhanced_regex::EnhancedRegex;

    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = EnhancedRegex::new(pattern).unwrap();
    let text = "Server at 192.168.1.1 and client at 10.0.0.1";

    let mat1 = regex.find_from_pos(text, 0);
    assert!(mat1.is_some(), "Should find first IPv4");
    assert_eq!(mat1.unwrap().as_str(), "192", "First match should be '192'");

    let mat2 = regex.find_from_pos(text, 14);
    assert!(mat2.is_some(), "Should find second IPv4");
    assert_eq!(mat2.unwrap().as_str(), "10", "Second match should be '10'");
}
