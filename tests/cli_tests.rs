#[cfg(any(all(
    any(target_arch = "x86_64"),
    any(
        all(target_os = "linux", any(target_env = "gnu", target_env = "musl")),
        target_os = "macos"
    )
),))]
mod test {

    use assert_fs::TempDir;
    use predicates::prelude::*;

    #[test]
    fn prints_help() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.arg("--help");
        cmd.assert().success().stdout(
            predicate::str::contains("Usage: rgrc").or(predicate::str::contains("Options:")),
        );
    }

    #[test]
    fn prints_version() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn completions_bash_succeeds() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.args(["--completions", "bash"]);
        cmd.assert().success().stdout(
            predicate::str::contains("_rgrc_completions").or(predicate::str::contains("compdef")),
        );
    }

    #[test]
    fn all_aliases_print() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.arg("--all-aliases");
        let assert = cmd.assert().success();
        // Expect some alias output; most installations will include common commands like 'ls'
        assert.stdout(predicate::str::contains("alias "));
    }

    #[test]
    fn no_command_shows_help_and_exits_nonzero() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.assert()
            .failure()
            .stdout(predicate::str::contains("Usage: rgrc"));
    }

    #[test]
    fn unknown_command_returns_127() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.arg("this-command-should-not-exist-xyz");
        cmd.assert().failure().stderr(
            predicate::str::contains("command not found").or(predicate::str::contains("not found")),
        );
    }

    #[test]
    fn flush_cache_with_embed_configs_works_in_temp_home() {
        // set HOME to a tempdir so cache creation doesn't touch the real user directory
        let td = TempDir::new().unwrap();
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.env("HOME", td.path()).arg("--flush-cache");

        // If embed-configs is enabled this should attempt to flush/rebuild and print success
        cmd.assert().success().stdout(
            predicate::str::contains("Cache rebuild successful")
                .or(predicate::str::contains("Flushing and rebuilding cache")),
        );
    }

    #[test]
    fn main_prints_version_and_exits() {
        // `cargo` provides CARGO_BIN_EXE_<name> env var to integration tests and
        // points to the built binary for this package. Use it to run the binary.
        let bin = env!("CARGO_BIN_EXE_rgrc");

        // Test --version
        let out = std::process::Command::new(bin)
            .arg("--version")
            .output()
            .expect("failed to run binary with --version");

        assert!(out.status.success(), "binary did not exit successfully");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let expected = format!("rgrc {}", env!("CARGO_PKG_VERSION"));
        assert_eq!(stdout.trim(), expected);

        // Test -v shortcut
        let out2 = std::process::Command::new(env!("CARGO_BIN_EXE_rgrc"))
            .arg("-v")
            .output()
            .expect("failed to run binary with -v");

        assert!(out2.status.success(), "binary did not exit successfully");
        let stdout2 = String::from_utf8_lossy(&out2.stdout);
        assert_eq!(stdout2.trim(), expected);
    }

    #[test]
    fn unsupported_completions_shell_fails() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.args(["--completions", "nope"]);
        cmd.assert().failure().stderr(predicate::str::contains(
            "Unsupported shell for completions",
        ));
    }

    #[test]
    fn invalid_color_arg_causes_error_exit() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.args(["--color=invalid", "echo"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Invalid color mode"));
    }

    #[test]
    fn piped_child_command_outputs_same_text() {
        // With piped stdout the binary should spawn the child and forward output
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.args(["echo", "hello-from-child"]);
        let assert = cmd.assert().success();
        assert.stdout(predicate::str::contains("hello-from-child"));
    }

    #[test]
    fn all_aliases_with_except_filters_out_names() {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rgrc");
        cmd.args(["--all-aliases", "--except", "ls,grep"]);
        cmd.assert()
            .success()
            // Ensure excluded names are not printed
            .stdout(predicate::str::contains("alias ls='").not())
            .stdout(predicate::str::contains("alias grep='").not())
            // And we still expect some alias lines in the output
            .stdout(predicate::str::contains("alias "));
    }
}
