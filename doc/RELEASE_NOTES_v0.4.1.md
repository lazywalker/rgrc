# Release Notes - v0.4.1

**Release Date**: 2025-11-26

## 🎉 Overview

Version 0.4.1 brings significant improvements to rgrc's performance diagnostics, smart colorization behavior, embedded configuration support (portable binaries via embed-configs), and packaging/deployment flexibility. This release focuses on making rgrc more intelligent about when to colorize output, easier to install & package with embedded configs, and simpler to diagnose performance hotspots.

## ✨ New Features

This release highlights three major improvements: embedded configuration support (embed-configs), smarter Auto-mode behavior, and optional performance instrumentation (timetrace).

### 🎨 Smart Auto-Mode with Pseudo-Command Exclusions

Auto-mode now intelligently excludes certain commands from colorization to improve user experience:

- **Exact matching exclusion list**: Commands like `ls` (without arguments) are excluded from colorization in Auto mode
- **Granular control**: `rgrc ls` remains uncolored while `rgrc ls -l` gets full colorization
- **Configurable**: Exclusion list can be extended via `PSEUDO_NO_COLOR` constant

**Use Case**: Common short-form commands remain fast and unmodified, while explicit invocations with options get rich colorization.

```bash
# No colorization in Auto mode (fast path)
rgrc ls

# Full colorization in Auto mode
rgrc ls -l
```

### 📦 Embedded configuration files (embed-configs)

rgrc supports embedding its default set of configuration files into the binary at build time (feature: `embed-configs`, enabled by default). This makes installed binaries portable, predictable, and fast on first run.

- **Build-time generation**: During `cargo build` / `cargo install` a small build script (`build.rs`) scans the repository `share/` configuration files (e.g. `share/conf.*`) and emits an `embedded_configs.rs` module using `include_str!` for each matched config. The binary contains both the config sources and a list of names.
- **Runtime cache**: On first run, rgrc will populate a runtime cache directory so embedded configs are available to code that expects real files. The cache is located under the platform XDG cache directory (e.g. `$XDG_CACHE_HOME/rgrc` or `~/.cache/rgrc`).
- **Why we cache**: The cache keeps the runtime API happy (which expects files on disk) and avoids repeated extraction work on subsequent runs. Cache entries are cheap — the binary stores the canonical sources, extraction is done only once per install.
- **Disable embedding**: If you prefer a tiny binary that reads configs only from the filesystem, disable the feature at install-time:

```bash
# Install with embedded configs (default - portable, self-contained)
cargo install rgrc

# Install without embedded configs (smaller binary - filesystem only)
cargo install rgrc --no-default-features
```

- **Managing the cache**: Use `rgrc --flush-cache` to rebuild the runtime cache from embedded configs; useful after upgrading or when injecting new embedded files into a binary.

- **Packaging and maintainers**: Packagers can decide whether to ship embedded configs (most convenient) or rely on distro-provided config files. If embedding is disabled packagers must provide the `share/` configuration files at a supported filesystem location.

- **Developer note**: The embed step is strictly compile-time; disabling the `embed-configs` feature removes the build-time generation step and shrinks the binary. The embedded code and cache handling are covered by unit tests and CI so packaging differences are exercised in automated checks.

### ⏱️ Performance Diagnostics (Timetrace Feature)

New optional instrumentation for performance analysis and debugging:

- **Stage-level timing**: Measures rule loading, process spawning, and colorization phases
- **Line-by-line metrics**: Tracks throughput (lines processed per second)
- **Zero overhead**: Completely removed at compile time when feature is disabled
- **Easy activation**: Build with `--features timetrace` and run with `RGRCTIME=1`

**Usage**:
```bash
# Build with timetrace instrumentation
cargo build --release --features timetrace

# Run with timing enabled
RGRCTIME=1 target/release/rgrc ls -la
```

**Sample Output**:
```
[RGRC] Rule loading: 2.345ms
[RGRC] Spawn command: 1.234ms
[RGRC] Colorize: 15.678ms, lines: 42, throughput: 2680 lines/sec
```

## 🔧 Improvements

### Code Quality & Testing

- **Robust CI testing**: Tests now use isolated temporary directories for reliable CI execution
- **Environment handling**: Improved test isolation with proper `HOME` directory management
- **Refactored logic**: Streamlined pseudo-command exclusion and environment variable handling
- **Better error messages**: Enhanced assertions and test failure diagnostics

### Documentation

- **Comprehensive README updates**: Added sections on Auto-mode behavior, embed-configs usage, and timetrace feature
- **Installation guide**: Clear instructions for cargo install with/without embedded configs
- **Performance guide**: Documentation on using timetrace for performance analysis
- **Shell completion**: Updated zsh completion support

### Build System

- **Build-time code generation**: `build.rs` automatically generates embedded config constants
- **Feature flags**: Clean separation between default and optional features
- **Optimized profiles**: Enhanced release profile with LTO and panic=abort for smaller binaries

## 🐛 Bug Fixes

- Fixed cache directory path handling on systems with non-standard `HOME` configurations
- Resolved CI test failures related to embedded config cache population
- Corrected environment variable restoration in test teardown
- Fixed minor documentation formatting issues

## 📊 Performance

- **Reduced binary size**: Optimized build settings and dependency tree
- **Faster cold starts**: Embedded configs eliminate filesystem lookups on first run
- **Improved throughput**: Colorization engine optimizations based on timetrace measurements
- **Smart fast-paths**: Auto-mode exclusions bypass colorization entirely for common commands

## 🔄 Breaking Changes

**None** - This release is fully backward compatible with v0.2.3.

## 📦 Dependencies

- Bumped `fancy-regex` to ^0.16.2 for improved regex performance
- Added `tempfile` (dev dependency) for robust test isolation
- Updated `console` to ^0.16.1

## 🙏 Acknowledgments

Special thanks to:
- GitHub Copilot for assisting with implementation and testing
- All contributors who reported issues and suggested improvements
- The Rust community for excellent tooling and libraries

## 📝 Migration Guide

### From v0.2.3 to v0.4.1

No migration steps required! Simply upgrade:

```bash
# Using cargo
cargo install rgrc --force

# From source
git pull
make release
sudo make install
```

### Enabling New Features

**Embedded Configs** (enabled by default):
```bash
cargo install rgrc
# First run will populate cache automatically
```

**Timetrace Diagnostics** (opt-in):
```bash
cargo install rgrc --features timetrace
# Use RGRCTIME=1 to enable timing output
```

## 🔮 What's Next

Looking ahead to future releases:

- User-configurable exclusion lists (environment variables or config files)
- Additional performance optimizations based on timetrace insights
- Expanded shell completion support (fish, bash)
- More pre-configured command patterns

## 📚 Resources

- **Repository**: https://github.com/lazywalker/rgrc
- **Documentation**: See [README.md](README.md)
- **Issues**: https://github.com/lazywalker/rgrc/issues
- **License**: MIT

---

**Full Changelog**: https://github.com/lazywalker/rgrc/compare/v0.2.3...v0.4.1

**Install Now**:
```bash
cargo install rgrc
```

Enjoy the enhanced rgrc experience! 🚀
