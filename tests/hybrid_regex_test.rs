// Test to verify hybrid regex engine is working correctly
// Simple patterns should use fast regex::Regex
// Complex patterns (with lookahead/lookbehind) use Enhanced regex

use rgrc::grc::CompiledRegex;

#[test]
fn test_simple_pattern_uses_fast_regex() {
    // Simple pattern without lookahead/lookbehind should compile to Fast variant
    let pattern = r"\bhello\b";
    let compiled = CompiledRegex::new(pattern).expect("Should compile simple pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            // Success! Simple pattern uses fast regex
            println!("✓ Simple pattern uses Fast regex engine");
        }
        CompiledRegex::Enhanced(_) => {
            panic!("Simple pattern should use Fast regex, not Enhanced");
        }
    }
}

#[test]
fn test_complex_pattern_uses_enhanced() {
    // Pattern with lookahead should compile to Enhanced variant
    let pattern = r"hello(?=\d+)";
    let compiled = CompiledRegex::new(pattern).expect("Should compile complex pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Complex pattern with lookahead should use Enhanced regex, not Fast");
        }
        CompiledRegex::Enhanced(_) => {
            // Success! EnhancedRegex can handle lookahead
            println!("✓ Lookahead pattern uses Enhanced regex engine");
        }
    }
}

#[test]
fn test_lookbehind_pattern_uses_enhanced() {
    // Pattern with lookbehind (constant length) should compile to Enhanced variant
    let pattern = r"(?<=\d{3})hello";
    let compiled = CompiledRegex::new(pattern).expect("Should compile lookbehind pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Complex pattern with lookbehind should use Enhanced regex, not Fast");
        }
        CompiledRegex::Enhanced(_) => {
            // Success! EnhancedRegex can handle lookbehind
            println!("✓ Lookbehind pattern uses Enhanced regex engine");
        }
    }
}

#[test]
fn test_backreference_fails() {
    // Pattern with backreference is not supported
    let pattern = r"(\w+)\s+\1";
    let compiled = CompiledRegex::new(pattern);

    // Backreferences are not supported, should fail to compile
    assert!(
        compiled.is_err(),
        "Backreference pattern should fail to compile"
    );
    println!("✓ Backreference pattern correctly fails to compile (not supported)");
}

#[test]
fn test_multiple_simple_patterns() {
    // Test that various common simple patterns use Fast regex
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

// ============================================================================
// IPv4 Pattern Tests (from examples/test_ipv4.rs)
// ============================================================================

#[test]
fn test_ipv4_pattern_simple_case() {
    // Pattern \d+(?=\.\d+\.\d+\.\d+) should match first octet of IPv4 address
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    // Should match "192" in "192.168.1.1" because it's followed by ".168.1.1"
    assert!(
        regex.is_match("192.168.1.1"),
        "Should match first octet in IPv4"
    );
}

#[test]
fn test_ipv4_pattern_with_prefix() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    // Should find IPv4 address even with prefix text
    assert!(
        regex.is_match("IP: 192.168.1.1"),
        "Should match with prefix"
    );
}

#[test]
fn test_ipv4_pattern_embedded() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    // Should find IPv4 address embedded in text
    assert!(
        regex.is_match("text 10.0.0.255 more"),
        "Should match embedded IPv4"
    );
}

#[test]
fn test_ipv4_pattern_incomplete() {
    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = CompiledRegex::new(pattern).unwrap();

    // Should NOT match incomplete IP addresses
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

    // Should match first octet of multiple IP addresses
    let text = "Connect from 192.168.1.1 to 10.0.0.1";
    assert!(regex.is_match(text), "Should find IPv4 addresses in text");
}

// ============================================================================
// Enhanced Regex Direct Tests (from examples/debug_ipv4_enhanced.rs)
// ============================================================================

#[test]
fn test_enhanced_ipv4_find_match() {
    use rgrc::enhanced_regex::EnhancedRegex;

    let pattern = r"\d+(?=\.\d+\.\d+\.\d+)";
    let regex = EnhancedRegex::new(pattern).unwrap();
    let text = "192.168.1.1";

    // Should find match at start of IPv4 address
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

    // Test is_match method
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

    // Find first match
    let mat1 = regex.find_from_pos(text, 0);
    assert!(mat1.is_some(), "Should find first IPv4");
    assert_eq!(mat1.unwrap().as_str(), "192", "First match should be '192'");

    // Find second match (starting after first)
    let mat2 = regex.find_from_pos(text, 14);
    assert!(mat2.is_some(), "Should find second IPv4");
    assert_eq!(mat2.unwrap().as_str(), "10", "Second match should be '10'");
}
