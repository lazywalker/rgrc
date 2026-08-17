// Import testable components from lib
use rgrc::{
    ColorMode,
    args::{get_completion_script, parse_args},
    buffer::LineBufferedWriter,
    colorizer::colorize_regex as colorize,
    grc::GrcatConfigEntry,
    load_rules_for_command, load_rules_for_config,
    utils::{
        SUPPORTED_COMMANDS, command_exists, set_process_title,
        should_use_colorization_for_command_supported,
    },
};

use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

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

// stdin-to-stdout passthrough; `ok` is the exit code on success, `err` on copy error
fn copy_stdin(ok: i32, err: i32) -> ! {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut writer = io::BufWriter::new(stdout.lock());
    let res = io::copy(&mut reader, &mut writer);
    let _ = writer.flush();
    match res {
        Ok(_) => std::process::exit(ok),
        Err(_) => std::process::exit(err),
    }
}

// grcat-style stdin filter for `rgrc -c NAME` without a trailing command
fn filter_stdin(config_name: &str, color_mode: ColorMode) -> ! {
    let stdout_is_terminal = io::stdout().is_terminal();
    let should_colorize = match color_mode {
        ColorMode::Off => false,
        ColorMode::On => true,
        ColorMode::Auto => stdout_is_terminal,
    };

    if !should_colorize {
        copy_stdin(0, 0);
    }

    let rules: Vec<GrcatConfigEntry> = load_rules_for_config(config_name);
    if rules.is_empty() {
        eprintln!(
            "Error: Failed to load rules for config '{}': No matching rules found",
            config_name
        );
        copy_stdin(0, 1);
    }

    let stdin = io::stdin();
    let mut buffered_stdin = io::BufReader::with_capacity(64 * 1024, stdin.lock());
    let mut buffered_stdout = io::BufWriter::with_capacity(64 * 1024, io::stdout());
    let mut line_buffered_writer = LineBufferedWriter::new(&mut buffered_stdout);

    if let Err(e) = colorize(
        &mut buffered_stdin,
        &mut line_buffered_writer,
        rules.as_slice(),
    ) && let Err(e) = handle_box_error(e)
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = buffered_stdout.flush()
        && let Err(e) = handle_io_error(e)
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    std::process::exit(0);
}

// Use mimalloc for faster memory allocation (reduces startup overhead)
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
                // plain alias for every command; piping to less in the alias
                // breaks trailing args like `journalctl -f` (#32)
                println!("alias {}='{} {}'", cmd, grc, cmd);
            }
        }
        std::process::exit(0);
    }

    // --config without a command: grcat-style stdin filter
    // --config with a command: run the command and colorize its output (#23)
    if let Some(ref config_name) = args.config
        && args.command.is_empty()
    {
        filter_stdin(config_name, args.color);
    }

    if args.command.is_empty() {
        eprintln!("No command specified.");
        std::process::exit(1);
    }

    let explicit_config = args.config.as_deref();
    let command_name = args.command.first().unwrap();

    // Update process title to show the wrapped command instead of "rgrc"
    // This makes tmux, ps, top etc. display the actual command being run
    set_process_title(command_name);

    // Detect if stdout is a terminal (TTY)
    let stdout_is_terminal = io::stdout().is_terminal();

    // An explicit -c config bypasses the supported-commands whitelist and
    // the pseudo-command exclusions: the user asked for this config
    let eligible =
        explicit_config.is_some() || should_use_colorization_for_command_supported(command_name);
    let should_colorize = match args.color {
        ColorMode::Off => false,
        ColorMode::On => eligible,
        ColorMode::Auto => stdout_is_terminal && eligible,
    };

    let pseudo_command = args.command.join(" ");

    // check pseudo-command exclusions before loading rules so bare `rgrc ls`
    // skips coloring (ls colorizes its own output) while `rgrc ls -l` does not.
    let should_colorize = if should_colorize {
        explicit_config.is_some() || !rgrc::utils::pseudo_command_excluded(&pseudo_command)
    } else {
        false
    };

    let rules: Vec<GrcatConfigEntry> = if should_colorize {
        match explicit_config {
            Some(name) => load_rules_for_config(name),
            None => load_rules_for_command(&pseudo_command),
        }
    } else {
        Vec::new()
    };

    // Spawn the command with appropriate stdout handling
    let mut cmd = Command::new(command_name);
    cmd.args(args.command.iter().skip(1));

    // When not colorizing (or no rules resolved), let the child write
    // directly to our stdout. Going through a pipe here only risks corrupting
    // binary output (e.g. `docker save > file`, see #31) and adds copying
    // overhead for no gain.
    if !should_colorize || rules.is_empty() {
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

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

        let ecode = match child.wait() {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Failed while waiting for '{}': {}", command_name, e);
                std::process::exit(1);
            }
        };
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

    let mut stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    let mut buffered_stdout = std::io::BufReader::with_capacity(64 * 1024, &mut stdout);
    let mut buffered_writer = std::io::BufWriter::with_capacity(64 * 1024, std::io::stdout());
    let mut line_buffered_writer = LineBufferedWriter::new(&mut buffered_writer);

    if let Err(e) = colorize(
        &mut buffered_stdout,
        &mut line_buffered_writer,
        rules.as_slice(),
    ) {
        handle_box_error(e)?;
    }

    // Ensure all buffered output is written
    if let Err(e) = buffered_writer.flush() {
        handle_io_error(e)?;
    }

    // Wait for the spawned command to complete and propagate its exit code.
    let ecode = child.wait().expect("failed to wait on child");
    std::process::exit(ecode.code().expect("need an exit code"));
}
