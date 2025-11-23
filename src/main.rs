use std::process::{Command, Stdio};

// Import testable components from lib
use rgrc::{
    ColorMode, colorizer::colorize_regex as colorize, grc::GrcatConfigEntry, load_rules_for_command,
};

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
/// - --colour on|off|auto: Override color output mode.
/// - --aliases: Print shell aliases for commonly colorized commands.
/// - --all-aliases: Print shell aliases for all known commands.
/// - --except CMD1,CMD2,...: Exclude commands from alias generation.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut command: Vec<String> = Vec::new();
    let mut color = ColorMode::Auto;
    let mut show_all_aliases = false;
    let mut except_aliases: Vec<String> = Vec::new();
    let mut show_aliases = false;

    // Parse command-line arguments using the argparse crate.
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Rusty Generic Colouriser");
        ap.stop_on_first_argument(true);
        ap.refer(&mut color).add_option(
            &["--color"],
            argparse::Store,
            "Override color output (on, off, auto)",
        );
        ap.refer(&mut command).required().add_argument(
            "command",
            argparse::Collect,
            "Command to run",
        );
        ap.refer(&mut show_aliases).add_option(
            &["--aliases"],
            argparse::StoreTrue,
            "Output shell aliases for available binaries",
        );
        ap.refer(&mut show_all_aliases).add_option(
            &["--all-aliases"],
            argparse::StoreTrue,
            "Output all shell aliases",
        );
        ap.refer(&mut except_aliases).add_option(
            &["--except"],
            argparse::Collect,
            "Exclude alias from generated list (multiple or comma-separated allowed)",
        );

        // If no arguments provided, show help
        if std::env::args().len() == 1 {
            ap.print_help("rgrc", &mut std::io::stdout())?;
            std::process::exit(1);
        }
        ap.parse_args_or_exit();
    }

    // Handle --aliases and --all-aliases flags: generate shell aliases for commands.
    if show_aliases || show_all_aliases {
        let grc = std::env::current_exe().unwrap();
        let grc = grc.display();

        // Build a set of excluded aliases (split comma-separated entries).
        // This allows users to exclude specific commands from the generated alias list via --except flag.
        let except_set: std::collections::HashSet<String> = except_aliases
            .iter()
            .flat_map(|s| s.split(',').map(|p| p.trim().to_string()))
            .collect();

        // Curated list of commands known to work well with grc
        for cmd in &[
            "ant",
            "blkid",
            "common",
            "curl",
            "cvs",
            "df",
            "diff",
            "dig",
            "dnf",
            "docker",
            "du",
            "dummy",
            "env",
            "esperanto",
            "fdisk",
            "findmnt",
            "free",
            "gcc",
            "getfacl",
            "getsebool",
            "id",
            "ifconfig",
            "ip",
            "iptables",
            "irclog",
            "iwconfig",
            "jobs",
            "kubectl",
            "last",
            "ldap",
            "log",
            "lolcat",
            "lsattr",
            "lsblk",
            "lsmod",
            "lsof",
            "lspci",
            "mount",
            "mvn",
            "netstat",
            "nmap",
            "ntpdate",
            "php",
            "ping",
            "ping2",
            "proftpd",
            "ps",
            "pv",
            "semanage",
            "sensors",
            "showmount",
            "sockstat",
            "sql",
            "ss",
            "stat",
            "sysctl",
            "systemctl",
            "tail",
            "tcpdump",
            "traceroute",
            "tune2fs",
            "ulimit",
            "uptime",
            "vmstat",
            "wdiff",
            "whois",
            "yaml",
            "docker",
            "go",
            "iostat",
            "lsusb",
        ] {
            // Output a shell alias if:
            // 1. The command is not in the exclude list, AND
            // 2. Either we're generating all aliases (--all-aliases) OR the command exists in PATH (which::which)
            if !except_set.contains(cmd as &str)
                && (show_all_aliases || which::which(cmd).is_ok())
            {
                // Print shell alias in the format: alias CMD='grc CMD';
                println!("alias {}='{} {}';", cmd, grc, cmd);
            }
        }
        std::process::exit(0);
    }

    if command.is_empty() {
        eprintln!("No command specified.");
        std::process::exit(1);
    }

    // Apply color mode setting
    match color {
        ColorMode::On => console::set_colors_enabled(true),
        ColorMode::Off => console::set_colors_enabled(false),
        ColorMode::Auto => {} // Default behavior based on TTY detection
    }

    let pseudo_command = command.join(" ");

    // Load colorization rules: iterate through config paths, find matching command regex,
    // then load the associated grcat configuration file (containing regex + color style rules).
    // Rules from all matching configs are collected into a single vector for colorization.
    let rules: Vec<GrcatConfigEntry> = load_rules_for_command(&pseudo_command);

    // Spawn the command with appropriate stdout handling
    let mut cmd = Command::new(command.first().unwrap().as_str());
    cmd.args(command.iter().skip(1));

    // If we have colorization rules, pipe the command's stdout so we can intercept and colorize it.
    if !rules.is_empty() {
        cmd.stdout(Stdio::piped());
    }

    // Spawn the command subprocess.
    let mut child = cmd.spawn().expect("failed to spawn command");

    // If colorization rules exist, read from the piped stdout, apply colorization
    // rules line-by-line (or in parallel chunks), and write colored output to stdout.
    if !rules.is_empty() {
        let mut stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");
        colorize(&mut stdout, &mut std::io::stdout(), rules.as_slice())?;
    }

    // Wait for the spawned command to complete and propagate its exit code.
    let ecode = child.wait().expect("failed to wait on child");
    std::process::exit(ecode.code().expect("need an exit code"));
}
