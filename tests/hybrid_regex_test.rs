// Test to verify hybrid regex engine is working correctly
// Simple patterns should use fast regex::Regex
// Complex patterns (with lookahead/lookbehind) should use fancy-regex

use rgrc::grc::CompiledRegex;

#[test]
fn test_simple_pattern_uses_fast_regex() {
    // Simple pattern without lookahead/lookbehind should compile to Fast variant
    let pattern = r"\bhello\b";
    let compiled = CompiledRegex::new(pattern).expect("Should compile simple pattern");

    // We can't directly test which variant it is without exposing internals,
    // but we can verify it works correctly
    match compiled {
        CompiledRegex::Fast(_) => {
            // Success! Simple pattern uses fast regex
            println!("✓ Simple pattern uses Fast regex engine");
        }
        CompiledRegex::Fancy(_) => {
            panic!("Simple pattern should use Fast regex, not Fancy");
        }
    }
}

#[test]
fn test_complex_pattern_uses_fancy_regex() {
    // Pattern with lookahead should compile to Fancy variant
    let pattern = r"hello(?=\d+)";
    let compiled = CompiledRegex::new(pattern).expect("Should compile complex pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Complex pattern with lookahead should use Fancy regex, not Fast");
        }
        CompiledRegex::Fancy(_) => {
            // Success! Complex pattern uses fancy-regex
            println!("✓ Complex pattern uses Fancy regex engine");
        }
    }
}

#[test]
fn test_lookbehind_pattern_uses_fancy_regex() {
    // Pattern with lookbehind (constant length) should compile to Fancy variant
    let pattern = r"(?<=\d{3})hello";
    let compiled = CompiledRegex::new(pattern).expect("Should compile lookbehind pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Pattern with lookbehind should use Fancy regex, not Fast");
        }
        CompiledRegex::Fancy(_) => {
            // Success!
            println!("✓ Lookbehind pattern uses Fancy regex engine");
        }
    }
}

#[test]
fn test_backreference_uses_fancy_regex() {
    // Pattern with backreference should compile to Fancy variant
    let pattern = r"(\w+)\s+\1";
    let compiled = CompiledRegex::new(pattern).expect("Should compile backreference pattern");

    match compiled {
        CompiledRegex::Fast(_) => {
            panic!("Pattern with backreference should use Fancy regex, not Fast");
        }
        CompiledRegex::Fancy(_) => {
            // Success!
            println!("✓ Backreference pattern uses Fancy regex engine");
        }
    }
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
            CompiledRegex::Fancy(_) => {
                panic!("Simple pattern '{}' should use Fast regex", pattern);
            }
        }
    }
}
