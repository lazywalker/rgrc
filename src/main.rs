/// Simple command existence check without external dependencies
fn command_exists(cmd: &str) -> bool {
    // Empty command is not valid
    if cmd.is_empty() {
        return false;
    }
    
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let full_path = dir.join(cmd);
            if full_path.exists() {
                return true;
            }
            // Also check with common extensions on Windows
            #[cfg(target_os = "windows")]
            {
                for ext in &[".exe", ".cmd", ".bat", ".com"] {
                    let full_path_with_ext = dir.join(format!("{}{}", cmd, ext));
                    if full_path_with_ext.exists() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

use std::process::{Command, Stdio};
use std::io::Write;

// Import testable components from lib
use rgrc::{
    ColorMode, ColorizationStrategy, colorizer::colorize_regex as colorize, grc::GrcatConfigEntry, load_rules_for_command,
};

// Use mimalloc for faster memory allocation (reduces startup overhead)
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Simple command-line argument parser to replace argparse
fn parse_args() -> Result<(ColorMode, Vec<String>, bool, bool, Vec<String>), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        std::process::exit(1);
    }

    let mut color = ColorMode::Auto;
    let mut command = Vec::new();
    let mut show_aliases = false;
    let mut show_all_aliases = false;
    let mut except_aliases = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg.starts_with("--color=") {
            // Handle --color=value format
            let value = &arg[8..]; // Skip "--color="
            color = match value {
                "on" => ColorMode::On,
                "off" => ColorMode::Off,
                "auto" => ColorMode::Auto,
                _ => return Err(format!("Invalid color mode: {}", value)),
            };
            i += 1;
        } else {
            match arg {
                "--color" => {
                    if i + 1 >= args.len() {
                        return Err("Missing value for --color".to_string());
                    }
                    color = match args[i + 1].as_str() {
                        "on" => ColorMode::On,
                        "off" => ColorMode::Off,
                        "auto" => ColorMode::Auto,
                        _ => return Err(format!("Invalid color mode: {}", args[i + 1])),
                    };
                    i += 2;
                }
                "--aliases" => {
                    show_aliases = true;
                    i += 1;
                }
                "--all-aliases" => {
                    show_all_aliases = true;
                    i += 1;
                }
                "--except" => {
                    if i + 1 >= args.len() {
                        return Err("Missing value for --except".to_string());
                    }
                    // Split comma-separated values
                    except_aliases.extend(args[i + 1].split(',').map(|s| s.trim().to_string()));
                    i += 2;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    // Everything else is treated as command arguments
                    command.extend_from_slice(&args[i..]);
                    break;
                }
            }
        }
    }

    if command.is_empty() && !show_aliases && !show_all_aliases {
        return Err("No command specified".to_string());
    }

    Ok((color, command, show_aliases, show_all_aliases, except_aliases))
}

fn print_help() {
    println!("Rusty Generic Colouriser");
    println!();
    println!("Usage: rgrc [OPTIONS] COMMAND [ARGS...]");
    println!();
    println!("Options:");
    println!("  --color MODE      Override color output (on, off, auto)");
    println!("  --aliases         Output shell aliases for available binaries");
    println!("  --all-aliases     Output all shell aliases");
    println!("  --except CMD,..   Exclude commands from alias generation");
    println!("  --help, -h        Show this help message");
    println!();
    println!("Examples:");
    println!("  rgrc ping -c 4 google.com");
    println!("  rgrc --color=off ls -la");
    println!("  rgrc --aliases");
}

/// Quick check if a command is likely to benefit from colorization (used for Smart strategy)
/// This is a lightweight check that doesn't require loading rules
fn should_use_colorization_for_command_only(command: &str) -> bool {
    // Commands that definitely benefit from colorization (have meaningful output to colorize)
    match command {
        "ant" | "blkid" | "curl" | "cvs" | "df" | "diff" | "dig" | "dnf" |
        "docker" | "du" | "env" | "esperanto" | "fdisk" | "findmnt" | "free" |
        "gcc" | "getfacl" | "getsebool" | "id" | "ifconfig" | "ip" | "iptables" |
        "irclog" | "iwconfig" | "jobs" | "kubectl" | "last" | "ldap" | "log" |
        "lolcat" | "lsattr" | "lsblk" | "lsmod" | "lsof" | "lspci" | "lsusb" |
        "mount" | "mvn" | "netstat" | "nmap" | "ntpdate" | "php" | "ping" |
        "ping2" | "proftpd" | "ps" | "pv" | "semanage" | "sensors" | "showmount" |
        "sockstat" | "sql" | "ss" | "stat" | "sysctl" | "systemctl" | "tcpdump" |
        "traceroute" | "tune2fs" | "ulimit" | "vmstat" | "wdiff" | "whois" |
        "yaml" | "go" | "iostat" | "ls" => {
            true
        }
        // For other commands, assume they don't benefit from colorization
        _ => false,
    }
}

/// Check if a command has colorization rules available (used for Always strategy)
fn should_use_colorization_for_command_supported(command: &str) -> bool {
    // Commands that are known to have colorization rules in the configuration
    match command {
        "ant" | "blkid" | "common" | "curl" | "cvs" | "df" | "diff" | "dig" | "dnf" |
        "docker" | "du" | "dummy" | "env" | "esperanto" | "fdisk" | "findmnt" | "free" |
        "gcc" | "getfacl" | "getsebool" | "id" | "ifconfig" | "ip" | "iptables" | "irclog" |
        "iwconfig" | "jobs" | "kubectl" | "last" | "ldap" | "log" | "lolcat" | "lsattr" |
        "lsblk" | "lsmod" | "lsof" | "lspci" | "lsusb" | "mount" | "mvn" | "netstat" | "nmap" |
        "ntpdate" | "php" | "ping" | "ping2" | "proftpd" | "ps" | "pv" | "semanage" |
        "sensors" | "showmount" | "sockstat" | "sql" | "ss" | "stat" | "sysctl" | "systemctl" |
        "tcpdump" | "traceroute" | "tune2fs" | "ulimit" | "uptime" | "vmstat" |
        "wdiff" | "whois" | "yaml" | "go" | "iostat" | "ls" => {
            true
        }
        // For unknown commands, assume they might have rules
        _ => true,
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
/// - --colour on|off|auto: Override color output mode.
/// - --aliases: Print shell aliases for commonly colorized commands.
/// - --all-aliases: Print shell aliases for all known commands.
/// - --except CMD1,CMD2,...: Exclude commands from alias generation.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let (color, command, show_aliases, show_all_aliases, except_aliases) = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

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
            if !except_set.contains(cmd as &str) && (show_all_aliases || command_exists(cmd))
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

    // Apply color mode setting and determine colorization strategy
    let strategy: ColorizationStrategy = color.into();
    let command_name = command.first().unwrap();

    // First check if console supports colors at all
    // If not, treat as Never strategy - no colorization, skip piping
    let console_supports_colors = console::colors_enabled();

    let should_colorize = if !console_supports_colors {
        // Console doesn't support colors, equivalent to Never strategy
        console::set_colors_enabled(false);
        false
    } else {
        // Console supports colors, apply the strategy
        console::set_colors_enabled(true);

        let should_attempt_colorization = match strategy {
            ColorizationStrategy::Always => should_use_colorization_for_command_supported(command_name),
            ColorizationStrategy::Never => false,
            ColorizationStrategy::Smart => should_use_colorization_for_command_only(command_name),
        };

        should_attempt_colorization
    };

    let pseudo_command = command.join(" ");

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
    cmd.args(command.iter().skip(1));

    // Optimization: When colorization is not needed, let the child process output directly to stdout
    // This completely avoids any piping overhead and data copying
    if !should_colorize {
        cmd.stdout(Stdio::inherit()); // Inherit parent's stdout directly
        cmd.stderr(Stdio::inherit()); // Also inherit stderr for consistency
        
        // Spawn and wait for the command
        let mut child = cmd.spawn().expect("failed to spawn command");
        let ecode = child.wait().expect("failed to wait on child");
        std::process::exit(ecode.code().expect("need an exit code"));
    }

    // Only pipe stdout when colorization is actually needed
    // This avoids unnecessary piping overhead when colors are disabled or not beneficial
    cmd.stdout(Stdio::piped());

    // Spawn the command subprocess.
    let mut child = cmd.spawn().expect("failed to spawn command");

    // Colorization is enabled, read from the piped stdout, apply colorization
    // rules line-by-line (or in parallel chunks), and write colored output to stdout.
    let mut stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    
    // Optimization: Use a larger buffer to reduce system call overhead
    // This can significantly improve performance for commands with lots of output
    let mut buffered_stdout = std::io::BufReader::with_capacity(64 * 1024, &mut stdout); // 64KB buffer
    
    // Also use a buffered writer for output to reduce write system calls
    let mut buffered_writer = std::io::BufWriter::with_capacity(32 * 1024, std::io::stdout()); // 32KB buffer
    
    colorize(&mut buffered_stdout, &mut buffered_writer, rules.as_slice())?;
    
    // Ensure all buffered output is written
    buffered_writer.flush()?;

    // Wait for the spawned command to complete and propagate its exit code.
    let ecode = child.wait().expect("failed to wait on child");
    std::process::exit(ecode.code().expect("need an exit code"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        // Test existing commands
        assert!(command_exists("echo"), "echo command should exist");
        assert!(command_exists("ls"), "ls command should exist");
        
        // Test non-existing command
        assert!(!command_exists("nonexistent_command_xyz123"), "nonexistent command should not exist");
        
        // Test with absolute path (if it exists)
        assert!(command_exists("/bin/echo") || command_exists("/usr/bin/echo"), "echo should exist in standard locations");
        
        // Test empty string
        assert!(!command_exists(""), "empty string should not be a valid command");
        
        // Test command with spaces (should not exist)
        assert!(!command_exists("command with spaces"), "commands with spaces should not exist");
    }

    #[test]
    fn test_parse_args() {
        // Test successful parsing with --color=value format
        let result = parse_args_helper(vec!["--color=on", "echo", "hello"]);
        assert!(result.is_ok());
        let (color, command, show_aliases, show_all_aliases, except_aliases) = result.unwrap();
        assert_eq!(color, ColorMode::On);
        assert_eq!(command, vec!["echo", "hello"]);
        assert!(!show_aliases);
        assert!(!show_all_aliases);
        assert!(except_aliases.is_empty());

        // Test --color value format
        let result = parse_args_helper(vec!["--color", "off", "ping", "-c", "1"]);
        assert!(result.is_ok());
        let (color, command, _, _, _) = result.unwrap();
        assert_eq!(color, ColorMode::Off);
        assert_eq!(command, vec!["ping", "-c", "1"]);

        // Test --aliases flag
        let result = parse_args_helper(vec!["--aliases"]);
        assert!(result.is_ok());
        let (color, command, show_aliases, show_all_aliases, except_aliases) = result.unwrap();
        assert_eq!(color, ColorMode::Auto); // default
        assert!(command.is_empty());
        assert!(show_aliases);
        assert!(!show_all_aliases);
        assert!(except_aliases.is_empty());

        // Test --all-aliases flag
        let result = parse_args_helper(vec!["--all-aliases"]);
        assert!(result.is_ok());
        let (_, _, show_aliases, show_all_aliases, _) = result.unwrap();
        assert!(!show_aliases);
        assert!(show_all_aliases);

        // Test --except flag
        let result = parse_args_helper(vec!["--except", "cmd1,cmd2", "--aliases"]);
        assert!(result.is_ok());
        let (_, _, _, _, except_aliases) = result.unwrap();
        assert_eq!(except_aliases, vec!["cmd1", "cmd2"]);

        // Test --help flag
        // Note: --help causes std::process::exit(0), so we can't test it directly
        // It would need integration testing

        // Test invalid color mode
        let result = parse_args_helper(vec!["--color=invalid", "echo"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid color mode"));

        // Test missing value for --color
        let result = parse_args_helper(vec!["--color"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing value for --color"));

        // Test missing value for --except
        let result = parse_args_helper(vec!["--except"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing value for --except"));

        // Test empty args (should show help and exit, but we can't test exit)
        // This would need integration testing

        // Test no command specified (when not using aliases flags)
        let result = parse_args_helper(vec!["--color=on"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No command specified"));

        // Test mixed valid args
        let result = parse_args_helper(vec!["--color=auto", "--except", "badcmd", "ls", "-la"]);
        assert!(result.is_ok());
        let (color, command, show_aliases, show_all_aliases, except_aliases) = result.unwrap();
        assert_eq!(color, ColorMode::Auto);
        assert_eq!(command, vec!["ls", "-la"]);
        assert!(!show_aliases);
        assert!(!show_all_aliases);
        assert_eq!(except_aliases, vec!["badcmd"]);

        // Test unknown flag (should be treated as command)
        let result = parse_args_helper(vec!["--unknown-flag", "echo", "test"]);
        assert!(result.is_ok());
        let (_, command, _, _, _) = result.unwrap();
        assert_eq!(command, vec!["--unknown-flag", "echo", "test"]);
    }

    // Helper function to test parse_args without std::env::args dependency
    fn parse_args_helper(args: Vec<&str>) -> Result<(ColorMode, Vec<String>, bool, bool, Vec<String>), String> {
        // Convert Vec<&str> to Vec<String> to match parse_args signature
        let args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
        
        // Create a temporary function that uses our args instead of std::env::args
        fn parse_args_test(args: Vec<String>) -> Result<(ColorMode, Vec<String>, bool, bool, Vec<String>), String> {
            if args.is_empty() {
                print_help();
                std::process::exit(1);
            }

            let mut color = ColorMode::Auto;
            let mut command = Vec::new();
            let mut show_aliases = false;
            let mut show_all_aliases = false;
            let mut except_aliases = Vec::new();

            let mut i = 0;
            while i < args.len() {
                let arg = args[i].as_str();
                if arg.starts_with("--color=") {
                    // Handle --color=value format
                    let value = &arg[8..]; // Skip "--color="
                    color = match value {
                        "on" => ColorMode::On,
                        "off" => ColorMode::Off,
                        "auto" => ColorMode::Auto,
                        _ => return Err(format!("Invalid color mode: {}", value)),
                    };
                    i += 1;
                } else {
                    match arg {
                        "--color" => {
                            if i + 1 >= args.len() {
                                return Err("Missing value for --color".to_string());
                            }
                            color = match args[i + 1].as_str() {
                                "on" => ColorMode::On,
                                "off" => ColorMode::Off,
                                "auto" => ColorMode::Auto,
                                _ => return Err(format!("Invalid color mode: {}", args[i + 1])),
                            };
                            i += 2;
                        }
                        "--aliases" => {
                            show_aliases = true;
                            i += 1;
                        }
                        "--all-aliases" => {
                            show_all_aliases = true;
                            i += 1;
                        }
                        "--except" => {
                            if i + 1 >= args.len() {
                                return Err("Missing value for --except".to_string());
                            }
                            // Split comma-separated values
                            except_aliases.extend(args[i + 1].split(',').map(|s| s.trim().to_string()));
                            i += 2;
                        }
                        "--help" | "-h" => {
                            print_help();
                            std::process::exit(0);
                        }
                        _ => {
                            // Everything else is treated as command arguments
                            command.extend_from_slice(&args[i..]);
                            break;
                        }
                    }
                }
            }

            if command.is_empty() && !show_aliases && !show_all_aliases {
                return Err("No command specified".to_string());
            }

            Ok((color, command, show_aliases, show_all_aliases, except_aliases))
        }
        
        parse_args_test(args)
    }
}
