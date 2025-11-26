# rgrc - Rusty Generic Colouriser

[![Rust](https://img.shields.io/badge/rust-2024--edition-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast, Rust-based command-line tool that colorizes the output of other commands using regex-based rules with advanced count/replace functionality, similar to the classic `grc` (Generic Colouriser) utility.

**Latest Features**: Full implementation of count/replace functionality with backreference support, advanced matching controls (once/more/stop), optimized performance with intelligent caching, real-time output buffering for interactive commands, and embedded configuration files for portable deployment.

## Features

- 🚀 **High Performance**: Written in Rust with optimized regex-based colorization
- 🎨 **Rich Colorization**: Supports ANSI colors, styles, and attributes with count/replace functionality
- 🔧 **Flexible Configuration**: Compatible with grc/grcat configuration files; supports embedded configurations for portable deployment
- 🐚 **Shell Integration**: Generates aliases for popular commands
- 📖 **Comprehensive**: Supports 80+ pre-configured commands
- ⚡ **Advanced Matching**: Regex-based rules with intelligent caching, pattern optimization, and advanced count/replace controls
- 🔄 **Text Transformation**: Replace matched text with backreference support for output modification

## Quick Start

### Installation

#### From Source
```bash
git clone https://github.com/lazywalker/rgrc.git
cd rgrc
make release
sudo make install
```

#### Using Cargo
```bash
cargo install rgrc
```

### Basic Usage

```bash
# Colorize ping output
rgrc ping -c 4 google.com

# Colorize ls output
rgrc ls -la

# Colorize any command
rgrc df -h

Note on Auto-mode behavior
-------------------------

When `--color=auto` is used (the default), rgrc performs a conservative check to
decide whether to attempt colorization. In particular we support exact pseudo-command
exclusions — for example the command `rgrc ls` (pseudo-command == "ls") is explicitly
excluded from colorization in Auto mode, while `rgrc ls -l` (pseudo-command == "ls -l")
does not match the exclusion and will be colorized normally. This allows common
short forms such as `ls` to remain unmodified in Auto mode while more explicit invocations
continue to get colorization.
```

### Generate Shell Aliases

```bash
# Show aliases for supported commands
rgrc --aliases

# Generate aliases for all known commands
rgrc --all-aliases

# Add to your shell profile
echo 'eval "$(rgrc --aliases)"' >> ~/.bashrc
```

## Configuration

### Configuration Files

rgrc reads configuration from multiple locations:

```
~/.rgrc                    # User-specific config
~/.config/rgrc/rgrc.conf   # XDG config location
/usr/local/etc/rgrc.conf   # System-wide config
/etc/rgrc.conf            # Fallback system config
```

### grcat Compatibility

rgrc is fully compatible with grc/grcat configuration files:

```
~/.config/rgrc/           # rgrc-specific configs
~/.local/share/rgrc/      # User share directory
/usr/local/share/rgrc/    # Local share directory
/usr/share/rgrc/          # System share directory
~/.config/grc/            # grc compatibility
~/.local/share/grc/       # grc compatibility
/usr/local/share/grc/     # grc compatibility
/usr/share/grc/           # grc compatibility
```

### Supported Commands

rgrc comes with pre-configured colorization rules for 80+ commands:

**System Monitoring**: `df`, `free`, `ps`, `top`, `vmstat`, `iostat`
**Network Tools**: `ping`, `traceroute`, `netstat`, `ss`, `ip`, `curl`
**Development**: `gcc`, `make`, `docker`, `kubectl`, `git`
**File Operations**: `ls`, `find`, `du`, `mount`, `fdisk`
**And many more...**

## Command Line Options

```
Usage: rgrc [OPTIONS] COMMAND [ARGS...]

Options:
  --help              Show help message
  --aliases           Generate shell aliases for supported commands
  --all-aliases       Generate aliases for all known commands
  --except=CMD,...    Exclude commands from alias generation
  --color=on|off|auto Control color output (default: auto)
```

## Examples

### Basic Colorization
```bash
# Colorize ping output
rgrc ping -c 4 8.8.8.8

# Colorize disk usage
rgrc df -h

# Colorize compilation output
rgrc make

# Colorize Docker commands
rgrc docker ps
rgrc docker logs mycontainer
```

### Advanced Usage
```bash
# Force color output
rgrc --color=on ls -la

# Disable colors
rgrc --color=off ps aux

# Real-time output for interactive commands
rgrc ping -c 4 google.com  # Shows ping responses immediately

# Generate aliases for specific commands
rgrc --aliases | grep -E "(ping|ls|ps|docker)"

# Exclude certain commands from aliases
rgrc --all-aliases --except=docker,kubectl

# Configuration modes:
# Default (embedded + file system): Uses embedded configs first, falls back to file system
rgrc ls -la  # Uses embedded configs with file system fallback
```

### Shell Integration
```bash
# Bash
echo 'eval "$(rgrc --aliases)"' >> ~/.bashrc

# Zsh
echo 'eval "$(rgrc --aliases)"' >> ~/.zshrc

# Fish
rgrc --aliases > ~/.config/fish/conf.d/rgrc.fish
```

## Configuration Examples

### Custom Command Configuration

Create `~/.config/rgrc/conf.mycommand`:

```
regexp=^ERROR
colours=red,bold

regexp=^WARNING
colours=yellow

regexp=^INFO
colours=green
```

### Advanced Matching Control

Use `count` and `replace` fields for sophisticated pattern matching:

```
# Match only once per line
regexp=^\s*#
colours=cyan
count=once

# Replace matched text
regexp=(ERROR|WARN|INFO)
colours=red,yellow,green
replace=\1:

# Stop processing after first match
regexp=^FATAL
colours=red,bold
count=stop
```

**Count Options:**
- `once`: Match only the first occurrence per line
- `more`: Match all occurrences (default)
- `stop`: Match first occurrence and stop processing the line

**Replace Field:**
- Supports backreferences (`\1`, `\2`, etc.)
- Empty string removes matched text
- Can transform output content

### Extending Existing Commands

Add rules to `~/.rgrc`:

```
# Custom rules for existing commands
regexp=^CUSTOM
colours=blue,underline
```

## Performance

- **Real-time Output**: Line-buffered writer ensures immediate output for interactive commands like `ping`, `tail`, and `watch`
- **Zero-copy Operations**: Efficient memory usage with minimal allocations and streaming I/O
- **Regex Optimization**: Uses fancy-regex with advanced pattern matching, backtracking control, and result caching
- **ANSI Optimization**: Merges adjacent styles using run-length encoding to reduce escape sequences
- **Count/Replace Support**: Advanced matching control with text substitution capabilities and line reprocessing

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/lazywalker/rgrc.git
cd rgrc

# Build debug version
cargo build

# Build release version (with embedded configs - default)
cargo build --release

# Build with embedded configurations (explicit)
cargo build --release --features embed-configs

# Build without embedded configurations (file system only)
cargo build --release -no-default-features

### Embed mode when installing with cargo

When you `cargo install` rgrc you can choose whether to embed the bundled
configuration files (the `etc/rgrc.conf` and `share/conf.*` files) into the
compiled binary. Embedding is controlled by the Cargo feature `embed-configs`.

Why embed?
- Embedding makes rgrc self-contained. The rules/configs are compiled into the
  binary so the program works out-of-the-box on systems that do not provide
  these files (useful for portable builds, containers, and minimal systems).

Runtime behavior
- When built with `embed-configs` the program will extract the embedded files
  into the user cache dir (for example `~/.cache/rgrc/<version>`) on first run
  and then load configs from that cache on subsequent runs. There is a
  runtime command to flush and rebuild that cache (available only when built
  with `embed-configs`).

Examples
- Install with embedded configs enabled (explicit):

```bash
cargo install

- Install without embedding (smaller binary, requires system configs at runtime):

```bash
cargo install --no-default-features
```

If installing from a published crate on crates.io, whether `embed-configs` is
enabled by default depends on how that specific release was published. If you
need a portable, self-contained binary, explicitly pass `--features embed-configs`(by default).

# Optional: build with timing instrumentation for diagnostics
# ---------------------------------------------------------
#
# For troubleshooting we provide a small instrumentation feature that prints
# per-stage timings to stderr when enabled. Build with the `timetrace` feature
# and set the `RGRCTIME` environment variable when running to enable timings.

# Build instrumented binary:
```bash
# cargo build -p rgrc --release --features timetrace
#```

Run with timings enabled (prints to stderr):
```bash
RGRCTIME=1 target/release/rgrc ls >/dev/null

# Run all tests (126+ tests across multiple modules)
cargo test

# Run specific test modules
cargo test --lib     # Library tests (args, buffer, utils, etc.)
cargo test --bin rgrc # Binary tests
cargo test --test colorizer_tests  # Colorizer tests
cargo test --test grc_tests        # Configuration tests

# Generate documentation
cargo doc --open
```

### Project Structure

```
rgrc/
├── src/
│   ├── main.rs      # CLI entry point
│   ├── lib.rs       # Core library
│   ├── args.rs      # Command-line argument parsing
│   ├── buffer.rs    # Buffered writers for real-time output
│   ├── colorizer.rs # Colorization engine
│   ├── grc.rs       # Configuration parsing
│   └── utils.rs     # Utility functions
├── tests/
│   ├── lib_tests.rs     # Library unit tests
│   ├── colorizer_tests.rs # Colorizer functionality tests
│   └── grc_tests.rs     # Configuration parsing tests
├── doc/
│   └── rgrc.1.md    # Manual page (markdown)
├── share/           # Pre-configured rules
├── etc/             # Shell completions
└── target/          # Build artifacts
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run tests: `cargo test` (runs all 126+ tests)
5. Run specific test suites:
   - `cargo test --lib` - Core library tests
   - `cargo test --test colorizer_tests` - Colorizer tests
   - `cargo test --test grc_tests` - Configuration tests
6. Submit a pull request

## Compatibility

- **Operating Systems**: Linux, macOS, Windows (with WSL)
- **Shells**: Bash, Zsh, Fish, and others supporting ANSI escape sequences
- **Terminals**: Any ANSI-compatible terminal with 256-color support
- **grc Compatibility**: Drop-in replacement for grc with enhanced count/replace functionality and improved performance
- **Configuration Files**: Fully compatible with existing grc/grcat configuration files and directory structures

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

- Inspired by the original `grc` (Generic Colouriser) by Radovan Garabík, `grc-rs` by Lars Christensen
- Built with [Rust](https://www.rust-lang.org/) and [console](https://crates.io/crates/console)

## Related Projects

- [grc](https://github.com/garabik/grc) - Original Generic Colouriser (Python)
- [grc-rs](https://github.com/larsch/grc-rs) - Generic Colouriser in Rust (Rust)
- [lolcat](https://github.com/busyloop/lolcat) - Rainbow coloring tool
- [bat](https://github.com/sharkdp/bat) - A cat clone with syntax highlighting
