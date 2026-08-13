# Development Guide

## Add new commands

1. Create a new config file in `share/` (e.g., `conf.mycommand`)
2. Add rules in the format:
   ```ini
   regexp=REGEX
   colours=COLOR1,COLOR2,...
   ```
3. Test with:
   ```bash
   echo "ERROR: Something went wrong" | RGRC_DEV_SHARE=1 cargo run -- --color on -c conf.mycommand
   ```
   `RGRC_DEV_SHARE=1` makes rgrc search `share/` next to the repo; without it
   only user (`$XDG_CONFIG_HOME/rgrc`) and system paths are searched.
   `-c` also runs a command when one is given:
   ```bash
   RGRC_DEV_SHARE=1 cargo run -- --color on -c conf.mycommand mycommand --arg
   ```
4. Enable command in `etc/rgrc.conf` to load the new config file. after that, the command will be available as `rgrc mycommand` (or via alias if configured).

## Config resolution

Conf files are searched in this order (first match wins, embedded configs last):

1. `$XDG_CONFIG_HOME/rgrc` (default `~/.config/rgrc`)
2. `$XDG_DATA_HOME/rgrc` (default `~/.local/share/rgrc`)
3. `$XDG_CONFIG_DIRS/rgrc`, `$XDG_DATA_DIRS/rgrc` (system defaults apply)
4. grc compat paths (`~/.config/grc`, `/usr/share/grc`, ...)
5. configs embedded at build time

Mapper files (`rgrc.conf`) are searched the same way. Patterns match the full
command line first, then the bare command name, so `^df$` also matches `df -h`.

## Testing

```bash
# Run tests
make test
# Run tests with coverage
make cov
```

## Code Formatting and Linting

```bash
make check
```

## Local Installation

```bash
make install

# Uninstall
make uninstall
```

## Advanced Features

### Count/Replace

```ini
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

### 256-color and truecolor

Beyond the 8 base colors and their `bright_*` variants:

```ini
# 256-color (0-255), grc-compatible
colours=colour_140
colours=on_colour_140

# truecolor hex
colours=rgb:ff8800
colours=on_rgb:ff8800
```

Both `colour_` and `color_` (American spelling) are accepted.
