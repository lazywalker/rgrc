# rgrc - Rusty Generic Colouriser

<!-- Repository badges -->

[![Rust](https://img.shields.io/badge/rust-2024--edition-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/lazywalker/rgrc/actions/workflows/ci.yml/badge.svg)](https://github.com/lazywalker/rgrc/actions)
[![crates.io](https://img.shields.io/crates/v/rgrc.svg)](https://crates.io/crates/rgrc)
[![docs.rs](https://docs.rs/rgrc/badge.svg)](https://docs.rs/rgrc)
[![codecov](https://codecov.io/gh/lazywalker/rgrc/branch/master/graph/badge.svg)](https://codecov.io/gh/lazywalker/rgrc)
[![Dependabot](https://img.shields.io/badge/Dependabot-enabled-brightgreen.svg)](https://github.com/lazywalker/rgrc/network/updates)
[![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/lazywalker/rgrc)

A fast, Rust-based command-line tool that colorizes the output of other commands using regex-based rules. Drop-in replacement for `grc` with better performance.

## Features

- **Fast**: 10x faster than original grc
- **Rich Colorization**: ANSI colors with count/replace support
- **Compatible**: Works with existing grc configuration files
- **Shell Integration**: Auto-generates aliases
- **80+ Commands**: Pre-configured for common tools
- **Smart Regex**: Hybrid engine with optional fancy-regex support
- **Lightweight**: Minimal dependencies (2 core deps)

## Quick Start

### Installation

**Shell (curl):**

```bash
curl -sS https://raw.githubusercontent.com/lazywalker/rgrc/master/script/install.sh | sh
```

**Cargo:**

```bash
cargo install rgrc
```

**Homebrew:**

```bash
brew tap lazywalker/rgrc
brew install rgrc
```

**Arch Linux:**

```bash
yay -S rgrc
```

**Alpine Linux:**

```bash
# Enable the testing repository (add edge testing)
doas cp /etc/apk/repositories /etc/apk/repositories.bak
echo "http://dl-cdn.alpinelinux.org/alpine/edge/testing" | doas tee -a /etc/apk/repositories
doas apk update

# Install rgrc from the testing repository
doas apk add rgrc
```

### Usage

```bash
# Colorize any command
rgrc ping -c 4 google.com
rgrc docker ps
rgrc df -h

# Set up aliases (recommended)
echo 'eval "$(rgrc --aliases)"' >> ~/.bashrc
source ~/.bashrc

# Then use commands directly
ping -c 4 google.com  # automatically colorized
docker ps             # automatically colorized
```

## Supported Commands

**System**: `df`, `free`, `ps`, `top`, `vmstat`, `iostat`, `uptime`, `mount`
**Network**: `ping`, `traceroute`, `netstat`, `ss`, `ip`, `curl`, `dig`
**Development**: `gcc`, `make`, `docker`, `kubectl`, `git`, `mvn`, `go`
**Files**: `ls`, `find`, `du`, `fdisk`, `lsof`, `stat`

[See full list in share/ directory](share/)

## Options

```bash
Usage: rgrc [OPTIONS] COMMAND [ARGS...]

Options:
  --color, --colour    Override color output (on|off|auto)
  --aliases            Output shell aliases for available binaries
  --all-aliases        Output all shell aliases
  --except CMD,..      Exclude commands from alias generation
  --completions SHELL  Print shell completion script for SHELL (bash|zsh|fish|ash)
  --flush-cache        Flush and rebuild cache directory
  --config, -c NAME    Explicit config file name (e.g., df to load conf.df)
  --help, -h           Show this help message
  --version, -V        Show installed rgrc version and exit
```

## Configuration

### Custom Rules

Create `~/.config/rgrc/conf.mycommand`:

```
regexp=^ERROR
colours=red,bold

regexp=^WARNING
colours=yellow

regexp=^INFO
colours=green
```

Add to `~/.rgrc`:

```
mycommand
conf.mycommand
```

### Shell Completions

```bash
# Bash
rgrc --completions bash > /etc/bash_completion.d/rgrc

# Zsh
rgrc --completions zsh > ~/.zfunc/_rgrc

# Fish
rgrc --completions fish > ~/.config/fish/completions/rgrc.fish
```

## Advanced Features

### Count/Replace

```
# Match only once per line
regexp=^\s*#
colours=cyan
count=once

# Replace matched text (with backreferences)
regexp=(ERROR|WARN|INFO)
colours=red,yellow,green
replace=[\1]

# Stop processing after match
regexp=^FATAL
colours=red,bold
count=stop
```

**Count options**: `once`, `more` (default), `stop`
**Replace**: Supports `\1`, `\2`, etc.

## License

MIT - see [LICENSE](LICENSE) for details.

## Credits

Inspired by [grc](https://github.com/garabik/grc) by Radovan Garabík and [grc-rs](https://github.com/larsch/grc-rs) by Lars Christensen.
