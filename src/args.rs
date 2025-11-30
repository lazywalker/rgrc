//! # args.rs - Command-line argument parsing for rgrc
//!
//! This module handles parsing command-line arguments and provides structured
//! access to the parsed options.

use crate::ColorMode;

/// Parsed command-line arguments for the `rgrc` binary.
///
/// This structure contains the semantic options extracted from the raw
/// command-line invocation. It is returned by `parse_args()` for use by
/// the main application logic.
///
/// # Fields
///
/// - `color`: Color mode requested by the user (`On`, `Off`, `Auto`).
/// - `command`: The command and its arguments to run (first element is the
///   executable name).
/// - `show_aliases`: Whether to print shell aliases for available commands.
/// - `show_all_aliases`: Whether to print aliases for all known commands.
/// - `except_aliases`: Comma-separated list of commands to exclude when
///   generating aliases.
/// - `flush_cache`: Whether to flush and rebuild the cache directory (embed-configs only).
///
/// # Example
///
/// ```ignore
/// let args = rgrc::args::parse_args()?;
/// println!("Color mode: {:?}", args.color);
/// ```
#[derive(Debug, PartialEq)]
pub struct Args {
    /// Requested color mode (on/off/auto)
    pub color: ColorMode,
    /// Command to execute and its arguments
    pub command: Vec<String>,
    /// Print aliases for detected commands in PATH
    pub show_aliases: bool,
    /// Print aliases for all supported commands
    pub show_all_aliases: bool,
    /// Commands to exclude from alias generation
    pub except_aliases: Vec<String>,
    /// Flush and rebuild cache directory (embed-configs only)
    pub flush_cache: bool,
    /// Print the CLI version and exit
    pub show_version: bool,
    /// Print shell completions for specified shell (bash|zsh|fish|ash)
    pub show_completions: Option<String>,
}

/// Parse command-line arguments
///
/// Returns parsed arguments or an error message
/// Parse command-line arguments and return an `Args` structure.
///
/// This function reads `std::env::args()` (skipping the program name) and
/// supports flags documented in the CLI help. On invalid usage it returns an
/// `Err(String)` describing the problem.
///
/// # Examples
///
/// ```ignore
/// // Simulated invocation: rgrc --color=on ping -c 1 google.com
/// let args = rgrc::args::parse_args().expect("valid args");
/// ```
pub fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    parse_args_impl(args)
}

/// Internal implementation of argument parsing
///
/// This function contains the core argument parsing logic and can be used
/// both by `parse_args()` (which gets args from environment) and by tests
/// (which pass args directly).
fn parse_args_impl(args: Vec<String>) -> Result<Args, String> {
    if args.is_empty() {
        print_help();
        std::process::exit(1);
    }

    // Helper function to parse argument value from either "--arg value" or "--arg=value"
    fn parse_arg_value<'a>(
        args: &'a [String],
        index: usize,
        arg_name: &str,
    ) -> Result<(&'a str, usize), String> {
        let arg = args[index].as_str();
        let prefix = format!("--{}=", arg_name);

        if let Some(value) = arg.strip_prefix(&prefix) {
            // Handle --arg=value format
            if value.is_empty() {
                return Err(format!("Missing value for --{}", arg_name));
            }
            Ok((value, index + 1))
        } else if arg == format!("--{}", arg_name) {
            // Handle --arg value format
            if index + 1 >= args.len() {
                return Err(format!("Missing value for --{}", arg_name));
            }
            Ok((args[index + 1].as_str(), index + 2))
        } else {
            Err(format!("Unexpected argument format: {}", arg))
        }
    }

    let mut color = ColorMode::Auto;
    let mut command = Vec::new();
    let mut show_aliases = false;
    let mut show_all_aliases = false;
    let mut except_aliases = Vec::new();
    let mut flush_cache = false;
    let mut show_version = false;
    let mut show_completions: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            arg if arg.starts_with("--color") => {
                let (value, next_i) = parse_arg_value(&args, i, "color")?;
                color = match value {
                    "on" => ColorMode::On,
                    "off" => ColorMode::Off,
                    "auto" => ColorMode::Auto,
                    _ => return Err(format!("Invalid color mode: {}", value)),
                };
                i = next_i;
            }
            arg if arg.starts_with("--except") => {
                let (value, next_i) = parse_arg_value(&args, i, "except")?;
                // Split comma-separated values
                except_aliases.extend(value.split(',').map(|s| s.trim().to_string()));
                i = next_i;
            }
            arg if arg.starts_with("--completions") => {
                let (value, next_i) = parse_arg_value(&args, i, "completions")?;
                show_completions = Some(value.to_string());
                i = next_i;
            }
            "--aliases" => {
                show_aliases = true;
                i += 1;
            }
            "--all-aliases" => {
                show_all_aliases = true;
                i += 1;
            }
            "--flush-cache" => {
                flush_cache = true;
                i += 1;
            }
            "--version" | "-v" => {
                show_version = true;
                i += 1;
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

    if command.is_empty()
        && !show_aliases
        && !show_all_aliases
        && !flush_cache
        && !show_version
        && show_completions.is_none()
    {
        return Err("No command specified".to_string());
    }

    Ok(Args {
        color,
        command,
        show_aliases,
        show_all_aliases,
        except_aliases,
        flush_cache,
        show_version,
        show_completions,
    })
}

/// Return a shell completion script for a supported shell, or None for an unsupported
/// shell name.
pub fn get_completion_script(shell: &str) -> Option<&'static str> {
    match shell {
        "bash" => Some(
            r#"_rgrc_completions() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"

    if [[ ${COMP_CWORD} -gt 0 && ${COMP_WORDS[COMP_CWORD-1]} == "--completions" ]]; then
        COMPREPLY=( $(compgen -W "bash zsh fish ash" -- "$cur") )
        return 0
    fi

    if [[ ${cur} == --* ]]; then
        COMPREPLY=( $(compgen -W "--color --aliases --all-aliases --except --flush-cache --help -h --version -v --completions" -- "$cur") )
        return 0
    fi

    # Complete commands and files
    COMPREPLY=( $(compgen -c -f -- "$cur") )
}

complete -F _rgrc_completions rgrc
"#,
        ),
        "zsh" => Some(
            r#"#compdef rgrc
_rgrc() {
  _arguments \
    '--color=[Override color output]:mode:(on off auto)' \
    '--aliases[Output shell aliases for available binaries]' \
    '--all-aliases[Output all shell aliases]' \
    '--except=[Exclude commands from alias generation]:commands:' \
    '--flush-cache[Flush and rebuild cache dir]' \
    '--help[Show help]' \
    '--version[Show version]' \
    '--completions=[Print completions for shell]:shell:(bash zsh fish ash)' \
    '1:command:_command_names -e' \
    '*::args:_files'
}
compdef _rgrc rgrc
"#,
        ),
        "fish" => Some(
            r#"# fish completion for rgrc
complete -c rgrc -l color -d 'Override color output (on,off,auto)'
complete -c rgrc -l aliases -d 'Output shell aliases for detected binaries'
complete -c rgrc -l all-aliases -d 'Output all aliases'
complete -c rgrc -l except -r -d 'Exclude commands from alias generation' -a '(__rgrc_list_commands)'
complete -c rgrc -l flush-cache -d 'Flush cache (embed-configs only)'
complete -c rgrc -l help -d 'Show help'
complete -c rgrc -l version -s v -d 'Show version'
complete -c rgrc -l completions -d 'Print completions for shell' -a 'bash zsh fish ash'

# Complete commands and files for arguments
complete -c rgrc -f -a '(__fish_complete_command)'
complete -c rgrc -F

function __rgrc_list_commands
    # no-op placeholder for future dynamic completions
    echo ""
end
"#,
        ),
        "ash" => Some(
            r#"# ash / sh completion helper (simple - may need shell support)
complete -W "--color --aliases --all-aliases --except --flush-cache --help -h --version -v --completions" rgrc
"#,
        ),
        _ => None,
    }
}

/// Print help message to stdout
fn print_help() {
    println!("Rusty Generic Colouriser");
    println!();
    println!("Usage: rgrc [OPTIONS] COMMAND [ARGS...]");
    println!();
    println!("Options:");
    println!("  --color MODE         Override color output (on, off, auto)");
    println!("  --aliases            Output shell aliases for available binaries");
    println!("  --all-aliases        Output all shell aliases");
    println!("  --except CMD,..      Exclude commands from alias generation");
    println!("  --completions SHELL  Print shell completion script for SHELL (bash|zsh|fish|ash)");
    #[cfg(feature = "embed-configs")]
    println!("  --flush-cache        Flush and rebuild cache directory (embed-configs only)");
    println!("  --help, -h           Show this help message");
    println!("  --version, -v        Show installed rgrc version and exit");
    println!();
    println!("Examples:");
    println!("  rgrc ping -c 4 google.com");
    println!("  rgrc --color=off ls -la");
    println!("  rgrc --aliases");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_success() {
        // Test successful parsing with --color=value format
        let result = parse_args_helper(vec!["--color=on", "echo", "hello"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::On);
        assert_eq!(args.command, vec!["echo", "hello"]);
        assert!(!args.show_aliases);
        assert!(!args.show_all_aliases);
        assert!(args.except_aliases.is_empty());

        // Test --color value format
        let result = parse_args_helper(vec!["--color", "off", "ping", "-c", "1"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::Off);
        assert_eq!(args.command, vec!["ping", "-c", "1"]);

        // Test --aliases flag
        let result = parse_args_helper(vec!["--aliases"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::Auto); // default
        assert!(args.command.is_empty());
        assert!(args.show_aliases);
        assert!(!args.show_all_aliases);
        assert!(args.except_aliases.is_empty());

        // Test --all-aliases flag
        let result = parse_args_helper(vec!["--all-aliases"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert!(!args.show_aliases);
        assert!(args.show_all_aliases);

        // Test --except flag
        let result = parse_args_helper(vec!["--except", "cmd1,cmd2", "--aliases"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.except_aliases, vec!["cmd1", "cmd2"]);

        // Test --flush-cache flag
        let result = parse_args_helper(vec!["--flush-cache"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert!(args.flush_cache);
        assert!(args.command.is_empty());

        // Test mixed valid args
        let result = parse_args_helper(vec!["--color=auto", "--except", "badcmd", "ls", "-la"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::Auto);
        assert_eq!(args.command, vec!["ls", "-la"]);
        assert!(!args.show_aliases);
        assert!(!args.show_all_aliases);
        assert_eq!(args.except_aliases, vec!["badcmd"]);

        // Test unknown flag (should be treated as command)
        let result = parse_args_helper(vec!["--unknown-flag", "echo", "test"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.command, vec!["--unknown-flag", "echo", "test"]);
        assert!(!args.flush_cache); // default should be false
        // Test --version and -v
        let result = parse_args_helper(vec!["--version"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert!(args.show_version);

        let result = parse_args_helper(vec!["-v"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert!(args.show_version);

        // Test --completions with space-separated value
        let result = parse_args_helper(vec!["--completions", "bash"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.show_completions, Some("bash".to_string()));

        // Test --completions with equals sign (--completions=SHELL)
        let result = parse_args_helper(vec!["--completions=zsh"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.show_completions, Some("zsh".to_string()));

        // Test --completions=fish
        let result = parse_args_helper(vec!["--completions=fish"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.show_completions, Some("fish".to_string()));

        // Test --completions=ash
        let result = parse_args_helper(vec!["--completions=ash"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.show_completions, Some("ash".to_string()));

        // Test --color with space-separated value
        let result = parse_args_helper(vec!["--color", "on", "ls"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::On);

        // Test --color with equals sign (--color=value)
        let result = parse_args_helper(vec!["--color=off", "ls"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::Off);

        // Test --color=auto
        let result = parse_args_helper(vec!["--color=auto", "df"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.color, ColorMode::Auto);

        // Test --except with space-separated value
        let result = parse_args_helper(vec!["--except", "ls,df", "--all-aliases"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.except_aliases, vec!["ls", "df"]);

        // Test --except with equals sign (--except=value)
        let result = parse_args_helper(vec!["--except=ls,df,ps", "--all-aliases"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.except_aliases, vec!["ls", "df", "ps"]);
    }

    #[test]
    fn test_parse_args_errors() {
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

        // Test no command specified (when not using aliases flags)
        let result = parse_args_helper(vec!["--color=on"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No command specified"));

        // Missing value for --completions should be an error
        let result = parse_args_helper(vec!["--completions"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Missing value for --completions")
        );

        // Empty value for --completions= should be an error
        let result = parse_args_helper(vec!["--completions="]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Missing value for --completions")
        );

        // Empty value for --color= should be an error
        let result = parse_args_helper(vec!["--color=", "ls"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing value for --color"));

        // Empty value for --except= should be an error
        let result = parse_args_helper(vec!["--except=", "--all-aliases"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing value for --except"));
    }

    // Helper function to test parse_args without std::env::args dependency
    fn parse_args_helper(args: Vec<&str>) -> Result<Args, String> {
        // Convert Vec<&str> to Vec<String> to match parse_args_impl signature
        let args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
        parse_args_impl(args)
    }

    #[test]
    fn completion_scripts_present_for_supported_shells() {
        assert!(get_completion_script("bash").is_some());
        assert!(get_completion_script("zsh").is_some());
        assert!(get_completion_script("fish").is_some());
        assert!(get_completion_script("ash").is_some());
        assert!(get_completion_script("unknown").is_none());
    }
}
