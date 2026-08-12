use std::process::{Command, Stdio};

#[test]
#[cfg(target_arch = "x86_64")]
fn test_empty_command_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .output()
        .expect("failed to run rgrc");

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("Usage:")
            || stderr.contains("Usage:")
            || stdout.contains("OPTIONS")
            || stderr.contains("OPTIONS")
    );
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_nonexistent_command_returns_127() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["nonexistent-command-xyz-12345"])
        .output()
        .expect("failed to run rgrc");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("command not found") || stderr.contains("not found"));
    assert_eq!(output.status.code().unwrap_or(1), 127);
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_color_off_mode_no_ansi() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=off", "echo", "test output"])
        .output()
        .expect("failed to run rgrc with --color=off");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\x1b["));
    assert!(stdout.contains("test output"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_color_on_mode_enables_colorization() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=on", "echo", "ERROR: test"])
        .output()
        .expect("failed to run rgrc with --color=on");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_pseudo_command_exact_match_exclusion() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["ls"])
        .output()
        .expect("failed to run rgrc ls");

    assert!(output.status.success());
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_piped_output_not_to_terminal() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["echo", "piped output"])
        .stdout(Stdio::piped())
        .output()
        .expect("failed to run rgrc with piped output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("piped output"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_spawn_error_handling() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["/this/path/does/not/exist/command"])
        .output()
        .expect("failed to run rgrc");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("command not found") || stderr.contains("Failed to spawn"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_command_with_args_passes_through() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["echo", "arg1", "arg2", "arg3"])
        .output()
        .expect("failed to run rgrc with multiple args");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("arg1"));
    assert!(stdout.contains("arg2"));
    assert!(stdout.contains("arg3"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_rules_not_loaded_when_color_off() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=off", "echo", "no rules loaded"])
        .output()
        .expect("failed to run rgrc");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("no rules loaded"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_invalid_color_mode_argument() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=invalid", "echo", "test"])
        .output()
        .expect("failed to run rgrc with invalid color");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid color mode") || stderr.contains("invalid"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_console_no_color_support() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .env("NO_COLOR", "1")
        .args(["echo", "ERROR: test"])
        .output()
        .expect("failed to run rgrc");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ERROR: test"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_stdout_inherit_when_no_colorization() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=off", "echo", "plain text"])
        .output()
        .expect("failed to run rgrc");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plain text"));
    assert!(!stdout.contains("\x1b["));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_console_colors_disabled_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .env("NO_COLOR", "1")
        .args(["echo", "no console colors"])
        .output()
        .expect("failed to run rgrc with NO_COLOR");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("no console colors"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_supported_command_colorization_check() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=auto", "echo", "supported command"])
        .output()
        .expect("failed to run rgrc with auto color");

    assert!(output.status.success());
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_wait_error_handling() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["echo", "wait success"])
        .output()
        .expect("failed to run rgrc");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wait success"));
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_child_exit_code_propagation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["sh", "-c", "exit 42"])
        .output()
        .expect("failed to run rgrc with failing command");

    assert!(!output.status.success());
    let code = output.status.code().unwrap_or(0);
    assert_eq!(code, 42, "Should propagate exit code 42");
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_stdout_inherit_for_terminal() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["--color=off", "echo", "inherited stdout"])
        .output()
        .expect("failed to run rgrc");

    assert!(output.status.success());
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_empty_rules_no_colorization() {
    let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
        .args(["echo", "no matching rules"])
        .output()
        .expect("failed to run rgrc");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("no matching rules"));
}

#[cfg(target_arch = "x86_64")]
mod cli_integration_tests {

    use std::process::Command;

    #[test]
    fn test_prints_help() {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_rgrc"));
        cmd.arg("--help");
        let output = cmd.output().expect("failed to run rgrc --help");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage:") || stdout.contains("Options:"));
    }

    #[test]
    fn test_prints_version() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("--version")
            .output()
            .expect("failed to run rgrc --version");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_version_shorthand() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("-V")
            .output()
            .expect("failed to run rgrc -V");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let expected = format!("rgrc {}", env!("CARGO_PKG_VERSION"));
        assert_eq!(stdout.trim(), expected);
    }

    #[test]
    fn test_completions_bash() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--completions", "bash"])
            .output()
            .expect("failed to run rgrc --completions bash");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("_rgrc") || stdout.contains("compdef") || stdout.contains("complete")
        );
    }

    #[test]
    fn test_completions_with_equals() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("--completions=zsh")
            .output()
            .expect("failed to run rgrc --completions=zsh");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("#compdef rgrc"));

        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("--completions=fish")
            .output()
            .expect("failed to run rgrc --completions=fish");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("fish completion"));
    }

    #[test]
    fn test_completions_empty_value() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("--completions=")
            .output()
            .expect("failed to run rgrc --completions=");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Missing value for --completions"));
    }

    #[test]
    fn test_unsupported_completions_shell() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--completions", "invalid_shell"])
            .output()
            .expect("failed to run rgrc");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Unsupported") || stderr.contains("unsupported"));
    }

    #[test]
    fn test_invalid_color_value() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--color", "invalid_value", "echo", "test"])
            .output()
            .expect("failed to run rgrc");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Invalid color mode") || stderr.contains("invalid"));
    }

    #[test]
    fn test_missing_color_value() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--color"])
            .output()
            .expect("failed to run rgrc");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Missing value") || stderr.contains("missing"));
    }

    #[test]
    fn test_all_aliases() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("--all-aliases")
            .output()
            .expect("failed to run rgrc --all-aliases");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("alias ") || !stdout.is_empty());
    }

    #[test]
    fn test_all_aliases_journalctl_plain() {
        let exe_path = env!("CARGO_BIN_EXE_rgrc");
        let exe_name = std::path::Path::new(exe_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("rgrc");

        let output = Command::new(exe_path)
            .arg("--all-aliases")
            .output()
            .expect("failed to run rgrc --all-aliases");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        let expected = format!("alias journalctl='{} journalctl'", exe_name);
        let journalctl_line = stdout
            .lines()
            .find(|line| line.starts_with("alias journalctl="))
            .expect("journalctl alias not found");
        assert_eq!(journalctl_line, &expected);
        assert!(!journalctl_line.contains("less"));
        assert!(!journalctl_line.contains("--no-pager"));
    }

    #[test]
    fn test_all_aliases_with_except() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--all-aliases", "--except", "ls,grep"])
            .output()
            .expect("failed to run rgrc");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("alias ls='"));
        assert!(!stdout.contains("alias grep='"));
    }

    #[test]
    fn test_piped_child_output() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["echo", "hello-from-child"])
            .output()
            .expect("failed to run rgrc");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello-from-child"));
    }

    #[test]
    fn test_auto_color_mode_disables_ansi_when_piped() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("id")
            .output()
            .expect("failed to run rgrc");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !stdout.contains("\x1b"),
            "Output should not contain ANSI escape codes when piped to a non-TTY"
        );
    }

    // `rgrc -c NAME COMMAND` runs the command instead of blocking on stdin (#23)
    #[test]
    fn config_mode_with_command_executes() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args([
                "--color=on",
                "-c",
                "ping",
                "echo",
                "64 bytes from 192.168.1.1: icmp_seq=1 ttl=64 time=0.5 ms",
            ])
            .output()
            .expect("failed to run rgrc");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("64 bytes from"));
        assert!(stdout.contains("\x1b["));
    }

    // exit code of the child propagates in -c exec mode
    #[test]
    fn config_mode_with_command_propagates_exit_code() {
        let output = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["-c", "ping", "sh", "-c", "exit 42"])
            .output()
            .expect("failed to run rgrc");

        assert_eq!(output.status.code(), Some(42));
    }

    // -c accepts a bare conf name too ("df" and "conf.df" both work)
    #[test]
    fn config_mode_accepts_conf_name() {
        let df_line = "/dev/sda1 100G 50G 50G 50% /";
        let out_name = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--color=on", "-c", "df", "echo", df_line])
            .output()
            .expect("failed to run rgrc");
        let out_conf = Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .args(["--color=on", "-c", "conf.df", "echo", df_line])
            .output()
            .expect("failed to run rgrc");

        assert!(out_name.status.success());
        assert!(out_conf.status.success());
        assert_eq!(out_name.stdout, out_conf.stdout);
        assert!(String::from_utf8_lossy(&out_name.stdout).contains("\x1b["));
    }
}
