# Release Notes - v0.6.1

**Release Date**: 2025-12-01

## Overview

Version 0.6.1 focuses on improving regex compatibility and robustness and on providing a standalone validator for configuration files. This release completes and stabilizes the EnhancedRegex preprocessing pipeline, adds the `rgrv` validation tool, increases test coverage, and cleans up lints and minor issues.

## Key Improvements

### EnhancedRegex preprocessing and compatibility

- Problem: Several patterns in `share/conf.*` failed to compile with the standard `regex` crate (invalid escapes, boundary escapes inside character classes, and variable-length lookbehind with alternation), while compiling under `fancy-regex`. These cases required deterministic preprocessing so the standard `regex` engine can be used by default.
- Implementation: `src/enhanced_regex.rs` now provides a preprocessing pipeline that:
  - fixes invalid escapes outside character classes
  - normalizes/repairs escapes inside character classes and boundary escapes
  - rewrites or simplifies variable-length lookbehind / alternation cases to forms accepted by `regex` where possible
  - handles common edge cases around captures and grouped alternations
- Result: Most patterns that previously required `fancy-regex` now validate correctly with the standard `regex` crate after preprocessing.

### `rgrv` — standalone validator

- Added `src/bin/rgrv.rs`, a small CLI that validates `grc.conf` and `conf.*` files and prints structured, user-friendly errors (path, line, error type, suggestion).
- Commands:
  - `rgrv grc [PATH]` — validate `grc.conf`
  - `rgrv conf [PATH ...]` — validate one or more `conf.*` files
- Improvements: Supports legacy compact one-line format `pattern<TAB|space>styles` and normalizes hyphenated style names to underscores for validation (e.g. `bright-red` → `bright_red`).

### Tests and quality

- Added and extended tests:
  - Unit tests for EnhancedRegex preprocessing (21 cases)
  - `tests/rgrv_coverage.rs` to exercise many `rgrv` branches
  - `tests/rgrv_additional.rs` for previously untested error branches
- All new tests pass locally.
- Fixed clippy warnings (e.g. `collapsible-if`, `needless-range-loop`) in modified modules.

### Binary size and build notes

- Explored a `profile.minimal` (opt-level = "z") for smaller `rgrv` binaries. Example sizes observed locally:
  - `target/release/rgrv` (release): ~1.6 MB
  - `target/minimal/rgrv` (minimal profile): ~1.3 MB
- Optionally making `mimalloc` optional was explored; that is left as a packaging decision.

## Bug fixes and details

- Fixed patterns that failed with the standard `regex` (invalid escapes, character-class boundary escapes, variable-length lookbehind cases).
- `rgrv` now tolerates and validates the compact `pattern<TAB|space>styles` format and reports clearer FormatError / RegexError / StyleError messages.
- Improved `ValidationError` output formatting to make suggestions and file locations more discoverable.

## Packaging / Distribution notes

- Packaging maintainers: use `--profile minimal` to minimize binary size for distribution. If full regex compatibility is required (backreferences, variable-length lookbehind), enable the `fancy-regex` feature.
- Homebrew: formula updates exist under `homebrew-rgrc/Formula/rgrc.rb` (I can open a PR if desired).

## Usage examples

```bash
# Validate a single conf file
rgrv conf share/conf.ping

# Validate multiple conf files
rgrv conf share/conf.*

# Validate grc.conf
rgrv grc /etc/rgrc/rgrc.conf
```
