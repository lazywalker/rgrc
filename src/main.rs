// Import testable components from lib
use rgrc::{
    ColorMode,
    args::parse_args,
    buffer::LineBufferedWriter,
    colorizer::colorize_regex as colorize,
    grc::GrcatConfigEntry,
    load_rules_for_command,
    utils::{SUPPORTED_COMMANDS, command_exists, should_use_colorization_for_command_supported},
};

use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

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

#[cfg(not(feature = "embed-configs"))]
fn flush_and_rebuild_cache() {
    eprintln!("Error: --flush-cache is only available when built with embed-configs feature");
    std::process::exit(1);
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Handle --aliases and --all-aliases flags: generate shell aliases for commands.
    if args.show_aliases || args.show_all_aliases {
        let grc = std::env::current_exe().unwrap();
        let grc = grc.display();

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
                println!("alias {}='{} {}'", cmd, grc, cmd);
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

    if args.command.is_empty() {
        eprintln!("No command specified.");
        std::process::exit(1);
    }

    // Apply color mode setting
    let color_mode = args.color;
    let command_name = args.command.first().unwrap();

    // First check if console supports colors at all
    // If not, treat as Off mode - no colorization, skip piping
    let console_supports_colors = console::colors_enabled();

    let should_colorize = if !console_supports_colors {
        // Console doesn't support colors, equivalent to Off mode
        console::set_colors_enabled(false);
        false
    } else {
        // Console supports colors, apply the color mode
        console::set_colors_enabled(true);

        match color_mode {
            ColorMode::Off => false,
            ColorMode::On | ColorMode::Auto => {
                should_use_colorization_for_command_supported(command_name)
            }
        }
    };

    let pseudo_command = args.command.join(" ");

    // Load colorization rules only if we determined we should attempt colorization
    let rules: Vec<GrcatConfigEntry> = if should_colorize {
        load_rules_for_command(&pseudo_command)
    } else {
        Vec::new() // Skip expensive rule loading
    };

    // Final check: we need both the decision to colorize AND actual rules
    let should_colorize = should_colorize && !rules.is_empty();

    // Spawn the command with appropriate stdout handling
    let mut cmd = Command::new(command_name);
    cmd.args(args.command.iter().skip(1));

    // Optimization: When colorization is not needed AND output goes directly to terminal,
    // let the child process output directly to stdout. This completely avoids any piping overhead.
    // However, when output is piped (e.g., rgrc cmd | other_cmd), we must still use pipes
    // to maintain data flow integrity.
    let stdout_is_terminal = io::stdout().is_terminal();
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

    // Colorization is enabled, read from the piped stdout, apply colorization
    // rules line-by-line (or in parallel chunks), and write colored output to stdout.
    let mut stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    // Optimization: Use a larger buffer to reduce system call overhead
    // This can significantly improve performance for commands with lots of output
    let mut buffered_stdout = std::io::BufReader::with_capacity(64 * 1024, &mut stdout); // 64KB buffer

    // For real-time output commands, use line buffering to ensure output appears immediately
    // Use a smaller buffer (4KB) and flush after each line to prevent output delay
    let mut buffered_writer = std::io::BufWriter::with_capacity(4 * 1024, std::io::stdout()); // 4KB buffer for line buffering

    // Create a line-buffered writer that flushes after each line
    let mut line_buffered_writer = LineBufferedWriter::new(&mut buffered_writer);

    colorize(
        &mut buffered_stdout,
        &mut line_buffered_writer,
        rules.as_slice(),
    )?;

    // Ensure all buffered output is written
    buffered_writer.flush()?;

    // Wait for the spawned command to complete and propagate its exit code.
    let ecode = child.wait().expect("failed to wait on child");
    std::process::exit(ecode.code().expect("need an exit code"));
}
