// Import testable components from lib
use rgrc::{
    args::{get_completion_script, parse_args},
    run::{
        eligible_for_colorize, filter_stdin_exit, rules_for, run_colorized, run_passthrough,
        stdin_plan,
    },
    utils::{set_process_title, write_aliases},
};

use std::io::{self, IsTerminal, Write};
use std::process::Command;

// Use mimalloc for faster memory allocation (reduces startup overhead)
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // Parse command-line arguments
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Handle --version flag first: print version and exit.
    // write-and-ignore instead of println!: a closed pipe must not panic
    if args.show_version {
        let _ = writeln!(io::stdout().lock(), "rgrc {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // Handle --completions flag: print completions for the requested shell
    if let Some(shell) = args.show_completions.as_deref() {
        match get_completion_script(shell) {
            Some(script) => {
                let _ = write!(io::stdout().lock(), "{}", script);
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

        // Comma-separated exclusions from --except
        let except_set: std::collections::HashSet<String> = args
            .except_aliases
            .iter()
            .flat_map(|s| s.split(',').map(|p| p.trim().to_string()))
            .collect();

        let mut out = io::BufWriter::new(io::stdout().lock());
        let _ = write_aliases(&mut out, &grc, args.show_all_aliases, &except_set);
        let _ = out.flush();
        std::process::exit(0);
    }

    // --config without a command: grcat-style stdin filter
    // --config with a command: run the command and colorize its output (#23)
    if let Some(ref config_name) = args.config
        && args.command.is_empty()
    {
        let plan = stdin_plan(config_name, args.color, io::stdout().is_terminal());
        std::process::exit(filter_stdin_exit(plan));
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

    let stdout_is_terminal = io::stdout().is_terminal();
    let pseudo_command = args.command.join(" ");

    // Whether we actually colorize is decided by rule loading: a command no
    // rgrc.conf pattern maps to loads no rules and falls through to passthrough
    let should_colorize = eligible_for_colorize(
        args.color,
        stdout_is_terminal,
        &pseudo_command,
        explicit_config.is_some(),
    );

    let rules = if should_colorize {
        rules_for(explicit_config, &pseudo_command)
    } else {
        Vec::new()
    };

    let mut cmd = Command::new(command_name);
    cmd.args(args.command.iter().skip(1));

    if !should_colorize || rules.is_empty() {
        std::process::exit(run_passthrough(&mut cmd, command_name));
    }
    std::process::exit(run_colorized(&mut cmd, command_name, &rules));
}
