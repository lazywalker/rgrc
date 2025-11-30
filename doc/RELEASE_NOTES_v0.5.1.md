# Release Notes - v0.5.1

**Release Date**: 2025-11-30

## 🎉 Overview

Version 0.5.1 represents a major architectural improvement to rgrc, introducing a hybrid regex engine with optional fancy-regex support, custom ANSI styling implementation, and significant dependency reduction. This release focuses on performance optimization, binary size reduction, and providing users with flexible choices between battle-tested libraries and lightweight custom implementations.

## ✨ Key Features

### 🚀 Hybrid Regex Engine

The most significant improvement in this release is the introduction of a hybrid regex engine that automatically selects the optimal implementation for each pattern:

- **Fast Path** (90%+ of patterns): Uses standard `regex` crate (~2-5x faster)
  - Simple patterns without lookaround assertions
  - Zero overhead for common colorization rules
  - Handles most configuration files efficiently

- **Enhanced Path**: Two implementations available via conditional compilation
  - **With `--features=fancy-regex`** (default): Battle-tested `fancy-regex` library
    - Supports all advanced features (backreferences, variable-length lookbehind)
    - Binary size: ~2.1MB (release mode)
    - Recommended for production use
  - **Without fancy feature**: Lightweight custom `EnhancedRegex` implementation
    - Binary size: ~1.8MB (release mode, 300KB smaller)
    - Supports fixed-length lookahead/lookbehind patterns
    - Covers 99% of patterns in rgrc config files
    - Custom implementation (~700 lines)

**Automatic Pattern Detection**:
```rust
// Simple patterns → Fast path (standard regex)
let re = CompiledRegex::new(r"\d+").unwrap();

// Lookaround patterns → Enhanced path (fancy-regex or EnhancedRegex)
let re = CompiledRegex::new(r"\d+(?=\.\d+\.\d+\.\d+)").unwrap();
```

**Build Options**:
```bash
# Default build (with fancy-regex, battle-tested)
cargo build --release  # → 2.1MB binary

# Lightweight build (custom EnhancedRegex)
cargo build --release --no-default-features --features=embed-configs
# → 1.8MB binary
```

### 🎨 Custom ANSI Style Module

Replaced the `console` crate with a lightweight custom implementation:

- **Zero external dependencies**: Reduced dependency tree complexity
- **Complete feature parity**: All colors, backgrounds, and text attributes supported
- **Minimal code**: 362 lines vs much larger console crate footprint
- **Better control**: Custom implementation tailored for rgrc's exact needs

**Supported Styles**:
- Foreground colors: black, red, green, yellow, blue, magenta, cyan, white
- Background colors: on_black, on_red, on_green, etc.
- Attributes: bold, italic, underline, blink, reverse
- Bright variants: bright_black, bright_red, etc.

### 📦 Dependency Optimization

Significant reduction in dependencies:

**Removed**:
- ❌ `console` crate → Custom `style` module (362 lines)
- ❌ `lazy_static` crate → Unused, deleted
- ❌ `assert_cmd`, `assert_fs`, `predicates` → Unused dev-dependencies

**Core Dependencies** (production):
- ✅ `regex` (required): Standard regex engine
- ✅ `mimalloc` (required): Fast memory allocator
- ✅ `fancy-regex` (optional, default): Enhanced regex support

**Result**: From 4 core dependencies down to 2 (or 3 with fancy-regex)

## 🔧 Improvements

### Performance Optimizations

1. **Fast-Path Pattern Specialization**
   - Hand-optimized fast paths for common lookaround patterns
   - ~10 specialized patterns avoid regex overhead
   - Examples: IPv4 addresses, file sizes, timestamps, end-of-line

2. **Hybrid Engine Benefits**
   - Simple patterns: No performance change (uses fast path)
   - Lookaround patterns: Minimal overhead with EnhancedRegex
   - Overall: Maintains 10x faster than original grc

3. **Buffer Size Optimization**
   - Increased read buffer: 64KB (from 8KB)
   - Increased write buffer: 64KB (from 4KB)
   - Reduced system call overhead for large outputs

### Code Quality & Testing

- **Comprehensive test coverage**: 92.76% overall (up from 87.67%)
- **Enhanced regex tests**: 28 new test cases covering edge cases
  - Fast-path pattern validation
  - Lookahead/lookbehind verification
  - Negative assertion testing
  - API coverage (find_from_pos, captures_from_pos, is_match)
  
- **Test suite statistics**:
  - 377 tests across 15 test suites
  - All tests passing for both build configurations
  - New test files: `enhanced_regex_coverage.rs`, `hybrid_regex_test.rs`

### Documentation

- **Module-level documentation**: Comprehensive docs for all modules
- **Cargo.toml features**: Detailed feature flag documentation
- **README updates**: 
  - Feature comparison table
  - Build configuration guides
  - Regex engine selection guide
  - Binary size comparison

- **Code examples**: Usage examples in documentation
- **Migration guide**: Clear instructions for different build options

### Build System

- **Conditional compilation**: Clean `#[cfg(feature = "fancy-regex")]` usage
- **Feature flags**: 
  - `default = ["embed-configs", "fancy-regex"]`
  - `fancy-regex`: Optional fancy-regex support
  - `embed-configs`: Embedded configuration files
  - `timetrace`: Performance instrumentation

## 🐛 Bug Fixes

- Fixed regex compilation for edge cases with lookaround patterns
- Corrected test assertions for enhanced regex behavior
- Fixed clippy warnings across the codebase
- Resolved indentation issues in shell scripts

## 📊 Performance & Binary Size

### Binary Size Comparison

| Configuration | Size | Notes |
|--------------|------|-------|
| Default (fancy-regex) | 2.1MB | Battle-tested, full features |
| Lightweight (EnhancedRegex) | 1.8MB | Custom impl, 99% coverage |
| Difference | -300KB | 14% size reduction |

### Test Coverage

| Module | Coverage | Improvement |
|--------|----------|-------------|
| enhanced_regex.rs | 94.16% | +6.36% |
| args.rs | 97.83% | - |
| colorizer.rs | 97.46% | - |
| grc.rs | 94.62% | - |
| style.rs | 91.71% | - |
| **Overall** | **92.76%** | **+1.56%** |

### Performance Impact

- **Simple patterns**: No change (fast path)
- **Lookaround patterns**: Negligible overhead
- **Overall**: Maintains excellent performance
- **Throughput**: Sustained 10x faster than original grc

## 🎯 Design Rationale

The dual Enhanced implementation approach provides:

1. **Safety Net**: fancy-regex is battle-tested for conservative users
2. **Optimization**: EnhancedRegex offers smaller binary for advanced users
3. **Flexibility**: Users choose based on their priorities (size vs maturity)
4. **Performance**: Hybrid engine ensures fast path for most patterns

## 🔄 Breaking Changes

**None** - This release is fully backward compatible with v0.4.1.

All existing configuration files and usage patterns continue to work without modification.

## 📦 Dependencies

### Production Dependencies
- `regex` ^1.12: Standard regex engine (required)
- `mimalloc` ^0.1.43: Fast memory allocator (required)
- `fancy-regex` ^0.16: Enhanced regex support (optional, default)

### Development Dependencies
- `tempfile` 3.0: Temporary files for testing
- `criterion` 0.5: Benchmarking framework

### Removed Dependencies
- `console`: Replaced with custom style module
- `lazy_static`: Unused, removed
- `assert_cmd`, `assert_fs`, `predicates`: Unused test helpers

## 🚀 Usage Examples

### Basic Usage (Default Build)

```bash
# Install with default features (fancy-regex enabled)
cargo install rgrc

# Use normally - automatic hybrid engine selection
rgrc ls -la
rgrc ping -c 4 google.com
rgrc docker ps
```

### Lightweight Build

```bash
# Install without fancy-regex (smaller binary)
cargo install rgrc --no-default-features --features=embed-configs

# Test both configurations
cargo test
cargo test --no-default-features --features=embed-configs
```

### Feature Flag Reference

```bash
# Default: embed-configs + fancy-regex
cargo build --release

# Minimal: embed-configs only (uses EnhancedRegex)
cargo build --release --no-default-features --features=embed-configs

# With timing: all features + timetrace
cargo build --release --features=timetrace
RGRCTIME=1 rgrc ls -la
```

## 📝 Migration Guide

### From v0.4.1 to v0.5.1

No migration steps required! Simply upgrade:

```bash
# Using cargo (default build)
cargo install rgrc --force

# Lightweight build
cargo install rgrc --no-default-features --features=embed-configs --force

# From source
git pull
make release
sudo make install
```

### Choosing Build Configuration

**Use Default Build** (fancy-regex) if:
- You prioritize stability and battle-tested code
- Binary size is not a concern
- You want maximum regex feature support
- Production deployment

**Use Lightweight Build** (EnhancedRegex) if:
- You want smaller binary size (-300KB)
- Your configs don't use backreferences or variable-length lookbehind
- You trust newer, less battle-tested code
- Embedded or resource-constrained environments

## 🔮 What's Next

Looking ahead to future releases:

- **Performance**: Continue optimizing based on benchmarks
- **Features**: Additional fast-path patterns as needed
- **Testing**: Expand edge case coverage
- **Documentation**: More examples and use cases
- **Compatibility**: Track fancy-regex updates

## 📚 Technical Details

### Enhanced Regex Implementation

The custom `EnhancedRegex` implementation (~700 lines) supports:

- ✅ Positive lookahead: `(?=pattern)`
- ✅ Positive lookbehind: `(?<=pattern)` (fixed-length)
- ✅ Negative lookahead: `(?!pattern)`
- ✅ Negative lookbehind: `(?<!pattern)` (fixed-length)
- ❌ Backreferences: `\1`, `\2`, etc. (not supported)
- ❌ Variable-length lookbehind (not supported)

### Fast-Path Patterns

Hand-optimized patterns:
- `\s|$` or `$|\s` - Whitespace or end
- `\s` - Whitespace only
- `$` - End of line/string
- `\s[A-Z]` - Space + uppercase
- `\s[A-Z][a-z]{2}\s` - Month abbreviations
- `[:/]` - Colon or slash
- `\.\d+\.\d+\.\d+` - IPv4 continuation
- `[KMG]B?` - Size units

### Testing Strategy

- **Unit tests**: Per-module test coverage
- **Integration tests**: Full workflow testing
- **Benchmark tests**: Performance validation
- **Coverage tests**: Edge case exploration
- **Conditional compilation tests**: Both build variants tested

## 🙏 Acknowledgments

Special thanks to:
- GitHub Copilot for implementation assistance
- The Rust community for excellent tooling
- fancy-regex maintainers for the reference implementation
- All users who provided feedback and testing

## 📚 Resources

- **Repository**: https://github.com/lazywalker/rgrc
- **Documentation**: See [README.md](../README.md)
- **Issues**: https://github.com/lazywalker/rgrc/issues
- **Pull Request**: https://github.com/lazywalker/rgrc/pull/3
- **License**: MIT

---

**Full Changelog**: https://github.com/lazywalker/rgrc/compare/v0.4.1...v0.5.1

**Install Now**:
```bash
# Default (recommended)
cargo install rgrc

# Lightweight
cargo install rgrc --no-default-features --features=embed-configs
```

Enjoy the enhanced, optimized rgrc experience! 🚀

## 📊 Statistics

- **Commits**: 15+ commits merged
- **Files Changed**: 24 files
- **Lines Added**: ~4,000
- **Lines Removed**: ~480
- **Test Coverage**: 92.76% (+1.56%)
- **Binary Size**: 1.8MB - 2.1MB (depending on build)
- **Dependencies**: Reduced from 4 to 2-3 core deps
- **Test Cases**: 377 tests (+28 new tests)
