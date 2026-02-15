# Changelog

## v0.6.12

- feat(rules): add new asdf, gpg, iprule, json, phpunit commands and update dig, fdisk, id, ifconfig, ip, iproute, last, ls, nmap, ps, uptime from newmaster branch of original grc
- feat(docker): change container id, layer_id/hash and sha256 digest to dark style

## v0.6.11

- feat(diskutil): add support for diskutil command (macos)
- feat(bin): remove rgrv validator from release packaging add keep it for development builds only

## v0.6.9

- refactor(regex): replace regex with rigex-lite to reduce bin size(~1.8MB vs ~0.7MB) and improve performance
- refactor(feature-gate): merge timetrace and debug features to reduce code complexity
- refactor(debug): use --verbose flag and -v/-vv for different levels of debug output instead of --debug flag
- feat(bin): add default-run to cargo.toml to save our time in develpment

## v0.6.8

- fix(regex): bugfix #17 kubectl not working properly
- feat(args): support both --color and --colours mentioned at #18
- feat(conf): add an option to explicitly specify the conf file and pipling just like grcat #16

## v0.6.7

- fix(color): respect TTY status in auto mode to support piping by @sedlund in #13

## v0.6.6

- feat: add configuration for tailing modern log files (conf.rlog) with support for Rust/Go log formats and add auto-detection for tail commands on .log files
- fix: use program name instead of absolute path for aliases to improve portability

## v0.6.5

- feat: add support for journalctl command with special alias
- feat: add podman as alias for docker commands
- fix: prioritize user configuration in load_rules_for_command function

## v0.6.4

- feat: add "dim" style and treat "dark" as dim attribute
- fix: change bright_black to use ansi_90 for better visibility on black terminals

## v0.6.2

- fix: refine config loading logic to handle empty files correctly
- docs: add documentation for new options like shell completions and debug output

## v0.6.1

- feat: implement EnhancedRegex preprocessing for improved regex compatibility
- feat: add rgrv standalone validator for configuration files
- test: add unit tests for EnhancedRegex preprocessing

## v0.5.1

- feat: introduce hybrid regex engine with fast path using standard regex and enhanced path with fancy-regex or custom EnhancedRegex
- feat: replace console crate with custom ANSI style module for zero external dependencies
- perf: optimize performance with fast-path specialization and increased buffer sizes
- refactor: reduce dependencies and improve build system with conditional compilation
- test: increase test coverage to 92.76% with comprehensive regex tests
- docs: update documentation with feature guides and migration instructions
- fix: resolve regex compilation edge cases and clippy warnings

## v0.4.3

- feat: add installation script with platform and architecture detection
- feat: enhance shell completion for commands and files
- feat: add support for shell completions in command-line arguments
- feat: simplify packaging process and improve cross-platform builds
- fix: correct URL construction for release artifacts
- cleanup: remove obsolete zsh completion file

## v0.4.2

- feat: add support for aarch64 target on macOS in release workflow
- feat: add support for building .deb packages with Docker and GitHub Actions
- ci: improve CI workflows with cross toolchain installation and artifact naming
- cli: add --version / -V flag
- docs: add AUR install instructions
- test: fixes for embedded configs and integration tests
- fix: various cache and release fixes

## v0.4.1

- feat: smart Auto-mode with pseudo-command exclusions for better performance
- feat: embedded configuration files (embed-configs) for portable binaries
- feat: performance diagnostics with timetrace feature
- perf: optimized build settings and faster cold starts
- test: robust CI testing with isolated temporary directories
- docs: comprehensive README updates and installation guide
- fix: cache directory handling and CI test failures
