# Release Notes — v0.2.3

**Release date:** 2025-11-23

## Overview

v0.2.3 is the first public release that brings the basic rgrc core to life: a fast, Rust-native command wrapper that colorizes the output of other commands using regex-driven rules. This release focuses on usability, testability and packaging—adding runtime rules loading, an installable Makefile, thorough documentation, and CI automation.

## Highlights

- ✅ Core colourizer engine
  - Regex-based rules with advanced count/replace support and robust matching behaviour
  - The colorizer is exercised by unit tests included in the release

- ✅ Configuration and rule loading
  - Implemented `load_rules_for_command` to streamline configuration discovery and loading
  - Support for configuration files and aliases for many common commands

- ✅ CLI, docs, and packaging
  - Comprehensive `README.md` with installation, usage and configuration guidance
  - Added `Makefile` for building and packaging convenience (`make release`, `make install`)
  - Man page (rgrc.1) and a converter script to generate the man page from Markdown

- ✅ CI / test automation
  - Added GitHub Actions workflows to run tests, formatting checks and release packaging
  - Dependabot configuration created to keep dependencies up-to-date

## Bug fixes and polish

- Adjusted test fixtures and resource paths to make unit tests deterministic
- Fixed help and usage text, corrected whitespace and formatting issues
- CI and release workflow updates (build cross compilation improvements and artifact naming)

## Getting started

Install locally:

```bash
git clone https://github.com/lazywalker/rgrc.git
cd rgrc
make release
sudo make install
```

If you prefer Rust-native install for a development workflow:

```bash
cargo install
```

## Contributors & notes

Thanks to everyone who contributed in the early development phase for helping stabilize the core features, tests, and packaging.

---

Full commit history up to this tag is available at: https://github.com/lazywalker/rgrc/commits/v0.2.3
