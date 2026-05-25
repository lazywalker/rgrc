//! # utils.rs - Utility functions for rgrc
//!
//! This module contains various utility functions used throughout the rgrc application.

/// Update the process title so that the wrapped command name is visible in
/// `ps`, `top`, `tmux` and similar tools.
///
/// On Linux this does two things:
/// 1. Writes to `/proc/self/comm` – changes the short name (max 15 bytes).
/// 2. Overwrites the original `argv` area with the new title – changes what
///    appears in `/proc/self/cmdline` and therefore what tmux shows in its
///    status line.
///
/// If either step fails the error is silently ignored since this is a
/// cosmetic improvement, not a critical operation.
///
/// On non-Linux platforms this is a no-op.
pub fn set_process_title(title: &str) {
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;

        // 1. Update /proc/self/comm (short name, max 15 bytes)
        let truncated = if title.len() > 15 {
            &title[..15]
        } else {
            title
        };
        if let Ok(mut f) = std::fs::File::create("/proc/self/comm") {
            let _ = f.write_all(truncated.as_bytes());
        }

        // 2. Overwrite the argv area so /proc/self/cmdline reflects the new title.
        //    This is what tmux reads to determine the window name.
        //
        //    On Linux the original argv strings are laid out contiguously in memory:
        //        argv0\0argv1\0argv2\0...
        //    The total size equals the length of /proc/self/cmdline.
        //
        //    We locate argv[0]'s start using glibc's `__progname_full`, which
        //    points somewhere inside argv[0] (at the basename). By scanning
        //    backwards from it for a NUL byte we find the true start of argv[0].
        unsafe {
            // Get total argv buffer size from /proc/self/cmdline
            let cmdline = match std::fs::read("/proc/self/cmdline") {
                Ok(c) => c,
                Err(_) => return,
            };
            let total_size = cmdline.len();
            if total_size == 0 {
                return;
            }

            // __progname_full points into argv[0] at the basename
            unsafe extern "C" {
                static __progname_full: *mut std::os::raw::c_char;
            }

            let p = &__progname_full;
            if p.is_null() || (*p).is_null() {
                return;
            }

            // Scan backwards from __progname_full to find the start of argv[0].
            // argv[0] is the very first string, so the byte just before it
            // is either the start of the mapped region or uninitialized memory
            // that should not be '\0' from our string. We look for the first
            // NUL byte going backwards — but argv[0] starts right after the
            // preceding NUL (or at the very beginning of the stack args area).
            let mut start = *p;
            loop {
                if start.is_null() {
                    break;
                }
                let prev = start.sub(1);
                if *prev == 0 {
                    // prev points to a NUL byte, so start is the beginning
                    // of an argv string. For argv[0] this is the very first
                    // string, so we've found it.
                    break;
                }
                start = prev;
            }

            // Overwrite the entire argv string area with the new title + NUL padding
            let dst = start as *mut u8;
            let new_bytes = title.as_bytes();
            let copy_len = new_bytes.len().min(total_size);
            std::ptr::copy_nonoverlapping(new_bytes.as_ptr(), dst, copy_len);
            // Fill the remainder with NUL bytes
            if total_size > copy_len {
                std::ptr::write_bytes(dst.add(copy_len), 0u8, total_size - copy_len);
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = title;
    }
}

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
    "diskutil",
    "dnf",
    "docker",
    "du",
    "kdig",
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
    "podman",
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
    "journalctl",
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
pub const PSEUDO_NO_COLOR: &[&str] = &["ls"];

/// Check whether a pseudo_command should be excluded from colorization.
///
/// Returns `true` if:
/// 1. The command is just the command name alone (e.g., "ls")
/// 2. The command is followed by any non-flag arguments (e.g., "ls /home", "ls ~", "ls .")
///
/// Returns `false` if the first argument starts with `-` (indicating flags like "-l", "--long")
///
/// # Examples
///
/// ```ignore
/// assert!(pseudo_command_excluded("ls"));        // command alone
/// assert!(pseudo_command_excluded("ls /home"));  // followed by path
/// assert!(pseudo_command_excluded("ls ~"));      // followed by path
/// assert!(pseudo_command_excluded("ls ."));      // followed by path
/// assert!(pseudo_command_excluded("ls somefile")); // followed by filename
/// assert!(!pseudo_command_excluded("ls -l"));    // followed by flag
/// assert!(!pseudo_command_excluded("ls --long")); // followed by flag
/// assert!(!pseudo_command_excluded("ls -l /home")); // followed by flag (even if path after)
/// ```
pub fn pseudo_command_excluded(pseudo_command: &str) -> bool {
    if pseudo_command.is_empty() {
        return false;
    }

    // Split into parts
    let parts: Vec<&str> = pseudo_command.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    // Check if the command is in the excluded list
    if !PSEUDO_NO_COLOR.contains(&parts[0]) {
        return false;
    }

    // If it's just the command alone, exclude it
    if parts.len() == 1 {
        return true;
    }

    // If there's a next part, check if it starts with '-' (indicating a flag)
    // If it does NOT start with '-', then it's a path/argument, so exclude it
    !parts[1].starts_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        // The available system commands vary between platforms and CI images.
        // Instead of asserting that a specific utility always exists, test that
        // at least one commonly-present executable is found. This keeps the test
        // robust on Linux, macOS, and Windows runners.

        let candidates_unix = ["sh", "bash", "ls", "true", "false", "echo"];
        let candidates_windows = ["cmd.exe", "powershell.exe", "where.exe"];

        let found_on_unix = candidates_unix.iter().any(|c| command_exists(c));
        let found_on_windows = candidates_windows.iter().any(|c| command_exists(c));

        // We expect at least one of these platform-typical commands to be present
        // on the current host running the tests.
        assert!(
            found_on_unix || found_on_windows,
            "expected at least one standard command to be present on PATH (checked: sh,bash,ls,true,false,echo,cmd.exe,powershell.exe,where.exe)"
        );

        // Test non-existing command
        assert!(
            !command_exists("nonexistent_command_xyz123"),
            "nonexistent command should not exist"
        );

        // On Unix-like systems, many CI images provide /bin/echo or /usr/bin/echo.
        // Make this an optional check only on Unix targets.
        if cfg!(unix) {
            assert!(
                command_exists("/bin/echo") || command_exists("/usr/bin/echo"),
                "echo should exist in standard locations on Unix hosts"
            );
        }

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
        // Journalctl support added
        assert!(should_use_colorization_for_command_supported("journalctl"));

        // Test unsupported commands
        assert!(!should_use_colorization_for_command_supported(
            "unknown_command"
        ));
        assert!(!should_use_colorization_for_command_supported(""));
    }

    #[test]
    fn test_pseudo_command_excluded() {
        // Command alone should be excluded
        assert!(pseudo_command_excluded("ls"));

        // Command with path arguments should be excluded
        assert!(pseudo_command_excluded("ls ~"));
        assert!(pseudo_command_excluded("ls ~/"));
        assert!(pseudo_command_excluded("ls /home"));
        assert!(pseudo_command_excluded("ls ."));
        assert!(pseudo_command_excluded("ls ./"));
        assert!(pseudo_command_excluded("ls .."));
        assert!(pseudo_command_excluded("ls /"));

        // Command with filename/non-flag arguments should be excluded
        assert!(pseudo_command_excluded("ls somefile"));
        assert!(pseudo_command_excluded("ls file.txt"));

        // Command with flags should NOT be excluded
        assert!(!pseudo_command_excluded("ls -l"));
        assert!(!pseudo_command_excluded("ls -l /home"));
        assert!(!pseudo_command_excluded("ls -la"));
        assert!(!pseudo_command_excluded("ls --long"));
        assert!(!pseudo_command_excluded("ls --long /home"));

        // Other commands should not be excluded
        assert!(!pseudo_command_excluded("df"));
        assert!(!pseudo_command_excluded("df /home"));

        // Empty string should not be excluded
        assert!(!pseudo_command_excluded(""));
    }

    #[test]
    fn test_set_process_title() {
        // On Linux, verify that writing to /proc/self/comm works
        #[cfg(target_os = "linux")]
        {
            set_process_title("test_cmd");
            let comm = std::fs::read_to_string("/proc/self/comm")
                .expect("should be able to read /proc/self/comm");
            assert!(
                comm.trim() == "test_cmd",
                "expected 'test_cmd', got '{}'",
                comm.trim()
            );

            // Test truncation: names longer than 15 chars should be cut
            let long_name = "this_is_a_very_long_command_name";
            set_process_title(long_name);
            let comm = std::fs::read_to_string("/proc/self/comm")
                .expect("should be able to read /proc/self/comm");
            assert_eq!(comm.trim().len(), 15);
            assert!(comm.trim().starts_with("this_is_a_very_"));

            // Restore process name
            set_process_title("test_set_proces");
        }

        // On non-Linux, just ensure it doesn't panic
        #[cfg(not(target_os = "linux"))]
        {
            set_process_title("test_cmd");
        }
    }
}
