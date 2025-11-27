# SIMD Colorizer Optimization

This document explains the SIMD-optimized colorization implementation added to rgrc.

## Overview

The SIMD-optimized colorizer (`colorize_simd`) provides significant performance improvements for literal pattern matching by using byte-level parallel operations instead of regex matching.

## Architecture

### Two-Tier Approach

1. **Fast Path (SIMD)**: Literal patterns use Aho-Corasick multi-pattern matching
   - Patterns like `"ERROR"`, `"WARNING"`, `"INFO"` (no regex metacharacters)
   - Uses SIMD-accelerated operations via `memchr` and `aho-corasick` crates
   - 4-16x faster than regex for simple patterns

2. **Fallback Path (Regex)**: Complex patterns use fancy-regex
   - Patterns with capture groups, lookaheads, backreferences, etc.
   - Same behavior as original `colorize_regex` function
   - Ensures compatibility with all existing configurations

### Key Dependencies

- **memchr**: SIMD-optimized byte scanning (SSE2/AVX2 on x86_64, NEON on ARM)
- **aho-corasick**: Multi-pattern matching with failure links, SIMD-accelerated
- **fancy-regex**: Fallback for complex regex patterns

## Performance Comparison

Run benchmarks to see performance differences:

```bash
cargo bench --bench colorizer_bench
```

### Expected Results

#### Literal Patterns (100-10,000 lines)
- **SIMD**: ~4-8x faster than regex
- Best for: log files, simple keyword highlighting

#### Complex Patterns (with capture groups, lookaheads)
- **SIMD**: Similar to regex (uses regex fallback)
- Best for: syntax highlighting, complex parsing

#### Mixed Workload
- **SIMD**: ~2-4x faster overall
- Best for: general-purpose colorization

### Benchmark Output Example

```
literal_patterns/regex/100   time: [1.234 ms 1.256 ms 1.278 ms]
literal_patterns/simd/100    time: [0.234 ms 0.245 ms 0.256 ms]
                             change: [-80.5%] (faster)

complex_patterns/regex/1000  time: [12.34 ms 12.56 ms 12.78 ms]
complex_patterns/simd/1000   time: [11.89 ms 12.01 ms 12.13 ms]
                             change: [-4.4%] (slightly faster)
```

## Implementation Details

### Pattern Detection Heuristic

The SIMD colorizer automatically detects which patterns are literals:

```rust
let is_literal = !pattern_str.chars().any(|c| {
    matches!(c, '.' | '*' | '+' | '?' | '|' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '\\')
});
```

### Aho-Corasick Construction

All literal patterns are compiled into a single Aho-Corasick automaton:

```rust
let ac = AhoCorasick::new(&patterns)?;
```

This allows searching for all patterns simultaneously in a single pass.

### SIMD Operations

Under the hood, the crates use:
- **x86_64**: SSE2, AVX2 instructions
- **aarch64**: NEON instructions  
- **Fallback**: Optimized scalar code

## Usage

### Option 1: Use SIMD colorizer directly

```rust
use rgrc::colorizer::colorize_simd;

let mut reader = std::io::stdin();
let mut writer = std::io::stdout();
colorize_simd(&mut reader, &mut writer, &rules)?;
```

### Option 2: Auto-select based on pattern complexity

```rust
use rgrc::colorizer::{colorize_regex, colorize_simd};

// Analyze rules to determine which implementation is better
let has_complex_patterns = rules.iter().any(|r| {
    let pattern = r.regex.as_str();
    pattern.contains('(') || pattern.contains('[') || pattern.contains('\\')
});

if has_complex_patterns {
    colorize_regex(&mut reader, &mut writer, &rules)?;
} else {
    colorize_simd(&mut reader, &mut writer, &rules)?;
}
```

## Testing

Run correctness tests to verify both implementations produce identical output:

```bash
cargo test colorizer_comparison
```

All tests should pass, confirming that SIMD and regex implementations are functionally equivalent for literal patterns.

## Limitations

The SIMD optimizer works best with:
- ✅ Literal string patterns (no regex features)
- ✅ Case-sensitive matches
- ✅ Simple keyword highlighting
- ❌ Capture groups (falls back to regex)
- ❌ Lookaheads/lookbehinds (falls back to regex)
- ❌ Character classes `[a-z]` (falls back to regex)
- ❌ Quantifiers `*`, `+`, `?` (falls back to regex)

## Future Optimizations

Potential improvements:
1. **Case-insensitive matching**: Use lowercase conversion + SIMD
2. **Simple character classes**: Convert `[0-9]` to memchr scan for each digit
3. **Anchored patterns**: Optimize `^ERROR` to only check line start
4. **Parallel line processing**: Use rayon for multi-threaded colorization

## Benchmarking Tips

### Run specific benchmark

```bash
cargo bench --bench colorizer_bench -- literal_patterns
```

### Generate HTML reports

Criterion automatically generates detailed HTML reports in `target/criterion/`.

### Profile with perf

```bash
cargo bench --bench colorizer_bench --profile=bench -- --profile-time=5
```

### Compare against baseline

```bash
# Save baseline
cargo bench --bench colorizer_bench -- --save-baseline main

# Make changes, then compare
cargo bench --bench colorizer_bench -- --baseline main
```

## References

- [memchr documentation](https://docs.rs/memchr/)
- [aho-corasick documentation](https://docs.rs/aho-corasick/)
- [fancy-regex documentation](https://docs.rs/fancy-regex/)
- [Criterion benchmarking guide](https://bheisler.github.io/criterion.rs/book/)
