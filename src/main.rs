// Import testable components from lib
use rgrc::{
    ColorMode,
    args::{get_completion_script, parse_args},
    buffer::LineBufferedWriter,
    colorizer::colorize_regex as colorize,
    grc::GrcatConfigEntry,
    load_rules_for_command,
    utils::{SUPPORTED_COMMANDS, command_exists, should_use_colorization_for_command_supported},
};

#[cfg(feature = "debug")]
use rgrc::args::DebugLevel;
#[cfg(feature = "debug")]
use rgrc::colorize_regex_with_debug;

use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};
#[cfg(feature = "timetrace")]
use std::time::Instant;

// Helper to centralize BrokenPipe handling.
// - `handle_box_error` accepts a boxed error (Box<dyn Error>), downcasts to
//   `std::io::Error` when possible and delegates to `handle_io_error`.
// - `handle_io_error` exits silently on BrokenPipe, otherwise returns the
//   error wrapped as `Box<dyn std::error::Error>` for propagation.
//
// TODO: Consider refactoring to use a custom error type for more granular control.
fn handle_box_error(e: Box<dyn std::error::Error>) -> Result<(), Box<dyn std::error::Error>> {
    match e.downcast::<std::io::Error>() {
        Ok(io_err) => handle_io_error(*io_err),
        Err(e) => Err(e),
    }
}

fn handle_io_error(e: std::io::Error) -> Result<(), Box<dyn std::error::Error>> {
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        std::process::exit(0);
    }
    Err(Box::new(e))
}

// Use mimalloc for faster memory allocation (reduces startup overhead)
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Flush and rebuild the cache directory (embed-configs only)
///
/// This function removes the existing cache directory and rebuilds it with
/// embedded configuration files. It displays the progress and results.
#[cfg(feature = "embed-configs")]
fn flush_and_rebuild_cache() {
    use rgrc::EMBEDDED_CONFIGS;

    println!("Flushing and rebuilding cache directory...");

    match rgrc::flush_and_rebuild_cache() {
        Some((cache_dir, config_count)) => {
            println!("Cache rebuild successful!");
            println!("  Location: {}", cache_dir.display());
            println!("  Main config: rgrc.conf");
            println!("  Color configs: {} files in conf/", config_count);
            println!("  Total embedded configs: {}", EMBEDDED_CONFIGS.len());
        }
        None => {
            eprintln!("Error: Failed to rebuild cache directory");
            std::process::exit(1);
        }
    }
}

/// Main entry point for the grc (generic colourizer) program.
///
/// This tool colorizes the output of command-line programs using
/// regex-based configuration rules. It works by:
/// 1. Parsing command-line arguments and configuration files.
/// 2. Spawning the target command with stdout redirected to a pipe.
/// 3. Applying colour rules to the piped output using pattern matching.
/// 4. Writing the colored output to stdout.
///
/// Configuration:
/// - Reads grc.conf to map commands to their colouring profiles.
/// - Reads grcat configuration files containing regex + style rules.
/// - Searches multiple standard paths for configuration files.
///
/// Command-line options:
/// - --color on|off|auto: Override color output mode.
/// - --aliases: Print shell aliases for commonly colorized commands.
/// - --all-aliases: Print shell aliases for all known commands.
/// - --except CMD1,CMD2,...: Exclude commands from alias generation.
/// - --completions SHELL: Print completion script for SHELL (bash|zsh|fish|ash)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Handle --version flag first: print version and exit
    if args.show_version {
        println!("rgrc {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // Handle --completions flag: print completions for the requested shell
    if let Some(shell) = args.show_completions.as_deref() {
        match get_completion_script(shell) {
            Some(script) => {
                print!("{}", script);
                std::process::exit(0);
            }
            None => {
                eprintln!("Unsupported shell for completions: {}", shell);
                std::process::exit(1);
            }
        }
    }

    // Handle --aliases and --all-aliases flags: generate shell aliases for commands.
    if args.show_aliases || args.show_all_aliases {
        let grc = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into()))
            .unwrap_or_else(|| "rgrc".to_string());

        // Build a set of excluded aliases (split comma-separated entries).
        // This allows users to exclude specific commands from the generated alias list via --except flag.
        let except_set: std::collections::HashSet<String> = args
            .except_aliases
            .iter()
            .flat_map(|s| s.split(',').map(|p| p.trim().to_string()))
            .collect();

        // Curated list of commands known to work well with grc
        for cmd in SUPPORTED_COMMANDS {
            // Output a shell alias if:
            // 1. The command is not in the exclude list, AND
            // 2. Either we're generating all aliases (--all-aliases) OR the command exists in PATH (which::which)
            if !except_set.contains(cmd as &str) && (args.show_all_aliases || command_exists(cmd)) {
                // Print shell alias in the format: alias CMD='grc CMD';
                if cmd == &"journalctl" {
                    // Special alias: run rgrc as wrapper so rgrc can control paging and coloring
                    println!("alias {}='{} journalctl --no-pager | less -R'", cmd, grc);
                } else {
                    println!("alias {}='{} {}'", cmd, grc, cmd);
                }
            }
        }
        std::process::exit(0);
    }

    // Handle --flush-cache flag: flush and rebuild cache directory
    #[cfg(feature = "embed-configs")]
    if args.flush_cache {
        flush_and_rebuild_cache();
        std::process::exit(0);
    }

    // If --config is specified, read from stdin and colorize using the specified config
    if let Some(ref config_name) = args.config {
        let color_mode = args.color;

        // Detect if stdout is a terminal (TTY)
        let stdout_is_terminal = io::stdout().is_terminal();

        // Determine if we should colorize based on color mode and TTY status
        let should_colorize = match color_mode {
            ColorMode::Off => false,
            ColorMode::On => true,
            ColorMode::Auto => stdout_is_terminal,
        };

        if !should_colorize {
            // Just pass through stdin to stdout without coloring
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = io::BufReader::new(stdin.lock());
            let mut writer = io::BufWriter::new(stdout.lock());
            match io::copy(&mut reader, &mut writer) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::BrokenPipe {
                        eprintln!("Error copying stdin to stdout: {}", e);
                    }
                    std::process::exit(0);
                }
            }
        }

        // Load colorization rules for the specified config
        let rules: Vec<GrcatConfigEntry> = load_rules_for_command(config_name);

        if rules.is_empty() {
            // No rules found, just pass through
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = io::BufReader::new(stdin.lock());
            let mut writer = io::BufWriter::new(stdout.lock());
            match io::copy(&mut reader, &mut writer) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::BrokenPipe {
                        eprintln!(
                            "Error: Failed to load rules for config '{}': No matching rules found",
                            config_name
                        );
                    }
                    std::process::exit(1);
                }
            }
        }

        // Read from stdin and colorize
        let stdin = io::stdin();
        let mut buffered_stdin = io::BufReader::with_capacity(64 * 1024, stdin.lock());
        let mut buffered_stdout = io::BufWriter::with_capacity(64 * 1024, io::stdout());
        let mut line_buffered_writer = LineBufferedWriter::new(&mut buffered_stdout);

        // Use debug colorizer if debug_level is not Off
        #[cfg(feature = "debug")]
        {
            if args.debug_level != DebugLevel::Off {
                if let Err(e) = colorize_regex_with_debug(
                    &mut buffered_stdin,
                    &mut line_buffered_writer,
                    rules.as_slice(),
                    args.debug_level,
                ) {
                    handle_box_error(e)?;
                }
            } else if let Err(e) = colorize(
                &mut buffered_stdin,
                &mut line_buffered_writer,
                rules.as_slice(),
            ) {
                handle_box_error(e)?;
            }
        }

        #[cfg(not(feature = "debug"))]
        {
            if let Err(e) = colorize(
                &mut buffered_stdin,
                &mut line_buffered_writer,
                rules.as_slice(),
            ) {
                handle_box_error(e)?;
            }
        }

        // Flush buffered output
        if let Err(e) = buffered_stdout.flush() {
            handle_io_error(e)?;
        }

        std::process::exit(0);
    }

    if args.command.is_empty() {
        eprintln!("No command specified.");
        std::process::exit(1);
    }

    // Apply color mode setting
    let color_mode = args.color;
    let command_name = args.command.first().unwrap();

    // Detect if stdout is a terminal (TTY)
    let stdout_is_terminal = io::stdout().is_terminal();

    // Determine if we should colorize based on color mode and TTY status
    let should_colorize = match color_mode {
        ColorMode::Off => false,
        ColorMode::On => should_use_colorization_for_command_supported(command_name),
        ColorMode::Auto => {
            stdout_is_terminal && should_use_colorization_for_command_supported(command_name)
        }
    };

    let pseudo_command = args.command.join(" ");

    // If we previously decided colorization should be attempted, allow an explicit
    // pseudo-command exclusion check here. This is done *before* loading rules so
    // plain `rgrc ls` (pseudo_command == "ls") can be treated as no-color while
    // `rgrc ls -l` will not match the exact exclusion and remains colorized.
    let should_colorize = if should_colorize {
        // exact match exclusions
        !rgrc::utils::pseudo_command_excluded(&pseudo_command)
    } else {
        false
    };

    // OPTIMIZATION: Load colorization rules concurrently with command preparation
    // This allows rule loading (I/O + regex compilation) to happen in parallel
    // with command spawning, reducing perceived latency
    #[cfg(feature = "timetrace")]
    let record_time = std::env::var_os("RGRCTIME").is_some();
    #[cfg(feature = "timetrace")]
    let t0 = if record_time {
        Some(Instant::now())
    } else {
        None
    };

    // Load rules if colorization is needed
    #[cfg(feature = "timetrace")]
    let t_load_start = if record_time {
        Some(Instant::now())
    } else {
        None
    };

    let rules: Vec<GrcatConfigEntry> = if should_colorize {
        load_rules_for_command(&pseudo_command)
    } else {
        Vec::new()
    };

    #[cfg(feature = "timetrace")]
    if let Some(start) = t_load_start.filter(|_| record_time) {
        eprintln!(
            "[rgrc:time] load_rules_for_command: {:} in {:?}",
            &pseudo_command,
            start.elapsed()
        );
    }

    // Spawn the command with appropriate stdout handling
    let mut cmd = Command::new(command_name);
    cmd.args(args.command.iter().skip(1));

    // Optimization: When colorization is not needed AND output goes directly to terminal,
    // let the child process output directly to stdout. This completely avoids any piping overhead.
    // However, when output is piped (e.g., rgrc cmd | other_cmd), we must still use pipes
    // to maintain data flow integrity.
    if !should_colorize && stdout_is_terminal {
        cmd.stdout(Stdio::inherit()); // Inherit parent's stdout directly
        cmd.stderr(Stdio::inherit()); // Also inherit stderr for consistency

        // Spawn and wait for the command
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                // Friendly error for missing executable
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("Error: command not found: '{}'", command_name);
                    std::process::exit(127);
                } else {
                    eprintln!("Failed to spawn '{}': {}", command_name, e);
                    std::process::exit(1);
                }
            }
        };

        let ecode = match child.wait() {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Failed while waiting for '{}': {}", command_name, e);
                std::process::exit(1);
            }
        };
        std::process::exit(ecode.code().unwrap_or(1));
    }

    // Final check: we need both the decision to colorize AND actual rules
    // If no rules were loaded, skip colorization even if it was requested
    if should_colorize && rules.is_empty() {
        // No rules found, but we're piping - just pass through without coloring
        // This handles the edge case where rule loading failed or returned empty
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        let mut child = cmd.spawn().expect("failed to spawn command");
        let ecode = child.wait().expect("failed to wait on child");
        std::process::exit(ecode.code().unwrap_or(1));
    }

    // Only pipe stdout when colorization is actually needed
    // This avoids unnecessary piping overhead when colors are disabled or not beneficial
    cmd.stdout(Stdio::piped());

    // Spawn the command subprocess.
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("Error: command not found: '{}'", command_name);
                std::process::exit(127);
            } else {
                eprintln!("Failed to spawn '{}': {}", command_name, e);
                std::process::exit(1);
            }
        }
    };

    #[cfg(feature = "timetrace")]
    if let Some(start) = t0.filter(|_| record_time) {
        eprintln!("[rgrc:time] spawn child: {:?}", start.elapsed());
    }

    // Colorization is enabled, read from the piped stdout, apply colorization
    // rules line-by-line (or in parallel chunks), and write colored output to stdout.
    let mut stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    // Optimization: Use a larger buffer to reduce system call overhead
    // This can significantly improve performance for commands with lots of output
    let mut buffered_stdout = std::io::BufReader::with_capacity(64 * 1024, &mut stdout); // 64KB buffer

    // OPTIMIZATION: Increased write buffer from 4KB to 64KB to match read buffer
    // This reduces system call overhead for large outputs while LineBufferedWriter
    // still ensures real-time line-by-line flushing for interactive commands
    let mut buffered_writer = std::io::BufWriter::with_capacity(64 * 1024, std::io::stdout()); // 64KB buffer

    // Create a line-buffered writer that flushes after each line
    let mut line_buffered_writer = LineBufferedWriter::new(&mut buffered_writer);

    // Use debug colorizer if debug_level is not Off
    #[cfg(feature = "debug")]
    {
        if args.debug_level != DebugLevel::Off {
            if let Err(e) = colorize_regex_with_debug(
                &mut buffered_stdout,
                &mut line_buffered_writer,
                rules.as_slice(),
                args.debug_level,
            ) {
                handle_box_error(e)?;
            }
        } else {
            // Measure colorize performance when requested (feature guarded)
            #[cfg(feature = "timetrace")]
            {
                if record_time {
                    let t_before_colorize = Instant::now();
                    if let Err(e) = colorize(
                        &mut buffered_stdout,
                        &mut line_buffered_writer,
                        rules.as_slice(),
                    ) {
                        handle_box_error(e)?;
                    }
                    eprintln!("[rgrc:time] colorize: {:?}", t_before_colorize.elapsed());
                } else {
                    colorize(
                        &mut buffered_stdout,
                        &mut line_buffered_writer,
                        rules.as_slice(),
                    )?;
                }
            }

            #[cfg(not(feature = "timetrace"))]
            {
                // Normal path (no instrumentation): just colorize
                if let Err(e) = colorize(
                    &mut buffered_stdout,
                    &mut line_buffered_writer,
                    rules.as_slice(),
                ) {
                    handle_box_error(e)?;
                }
            }
        }
    }

    #[cfg(not(feature = "debug"))]
    {
        // Measure colorize performance when requested (feature guarded)
        #[cfg(feature = "timetrace")]
        {
            if record_time {
                let t_before_colorize = Instant::now();
                if let Err(e) = colorize(
                    &mut buffered_stdout,
                    &mut line_buffered_writer,
                    rules.as_slice(),
                ) {
                    handle_box_error(e)?;
                }
                eprintln!("[rgrc:time] colorize: {:?}", t_before_colorize.elapsed());
            } else {
                if let Err(e) = colorize(
                    &mut buffered_stdout,
                    &mut line_buffered_writer,
                    rules.as_slice(),
                ) {
                    handle_box_error(e)?;
                }
            }
        }

        #[cfg(not(feature = "timetrace"))]
        {
            // Normal path (no instrumentation): just colorize
            if let Err(e) = colorize(
                &mut buffered_stdout,
                &mut line_buffered_writer,
                rules.as_slice(),
            ) {
                handle_box_error(e)?;
            };
        }
    }

    // Ensure all buffered output is written
    if let Err(e) = buffered_writer.flush() {
        handle_io_error(e)?;
    }

    // Wait for the spawned command to complete and propagate its exit code.
    let ecode = child.wait().expect("failed to wait on child");
    std::process::exit(ecode.code().expect("need an exit code"));
}
