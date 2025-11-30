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
        let compiled =
            CompiledRegex::new(pattern).expect(&format!("Should compile pattern: {}", pattern));

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
