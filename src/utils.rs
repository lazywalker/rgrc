//! # utils.rs - Utility functions for rgrc
//!
//! This module contains various utility functions used throughout the rgrc application.

/// Simple command existence check without external dependencies
/// Check whether an executable named `cmd` exists on the user's `PATH`.
///
/// This performs a lightweight search of directories in the `PATH` environment
/// variable and returns `true` if a file with the given name exists in any
/// directory. On Windows, common executable extensions are also considered.
///
/// # Examples
///
/// ```ignore
/// assert!(rgrc::utils::command_exists("ls"));
/// assert!(!rgrc::utils::command_exists("this-command-doesnt-exist-xyz"));
/// ```
pub fn command_exists(cmd: &str) -> bool {
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

/// Curated list of commands that ship with colorization rules.
///
/// This array contains the command identifiers corresponding to files in
/// `share/conf.*` and is used by alias generation and the "Always" color
/// strategy to decide which commands are supported.
///
/// # Example
///
/// ```ignore
/// if rgrc::utils::SUPPORTED_COMMANDS.contains(&"ping") {
///     println!("ping is supported for colorization");
/// }
/// ```
pub const SUPPORTED_COMMANDS: &[&str] = &[
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
    "ls",
    "lsusb",
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
    "go",
    "iostat",
];

/// Check if a command has colorization rules available (used for Always strategy)
/// Return `true` when a command has shipped colorization rules (present in
/// `SUPPORTED_COMMANDS`). This is a simple membership check used by the
/// Always colorization strategy.
///
/// # Examples
///
/// ```ignore
/// assert!(rgrc::utils::should_use_colorization_for_command_supported("ls"));
/// assert!(!rgrc::utils::should_use_colorization_for_command_supported("unknown"));
/// ```
pub fn should_use_colorization_for_command_supported(command: &str) -> bool {
    SUPPORTED_COMMANDS.contains(&command)
}

/// Pseudo-commands (exact match) that should NOT be colorized for explicit checks
/// (e.g. `rgrc ls` should not colorize but `rgrc ls -l` should).
pub const PSEUDO_NO_COLOR: &[&str] = &[
    "ls",
];

/// Check whether an exact pseudo_command should be excluded from colorization.
pub fn pseudo_command_excluded(pseudo_command: &str) -> bool {
    if pseudo_command.is_empty() {
        return false;
    }
    PSEUDO_NO_COLOR.contains(&pseudo_command)
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
        assert!(
            !command_exists("nonexistent_command_xyz123"),
            "nonexistent command should not exist"
        );

        // Test with absolute path (if it exists)
        assert!(
            command_exists("/bin/echo") || command_exists("/usr/bin/echo"),
            "echo should exist in standard locations"
        );

        // Test empty string
        assert!(
            !command_exists(""),
            "empty string should not be a valid command"
        );

        // Test command with spaces (should not exist)
        assert!(
            !command_exists("command with spaces"),
            "commands with spaces should not exist"
        );
    }

    #[test]
    fn test_should_use_colorization_for_command_supported() {
        // Test supported commands
        assert!(should_use_colorization_for_command_supported("ping"));
        assert!(should_use_colorization_for_command_supported("ls"));
        assert!(should_use_colorization_for_command_supported("df"));

        // Test unsupported commands
        assert!(!should_use_colorization_for_command_supported(
            "unknown_command"
        ));
        assert!(!should_use_colorization_for_command_supported(""));
    }

    #[test]
    fn test_pseudo_command_excluded() {
        assert!(pseudo_command_excluded("ls"));
        assert!(!pseudo_command_excluded("ls -l"));
        assert!(!pseudo_command_excluded(""));
    }
}
