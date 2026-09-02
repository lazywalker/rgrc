//! Wrapping pipeline: decide whether to colorize, spawn the command,
//! pipe through the colorizer, report an exit code. Nothing here calls
//! exit(); callers turn the returned codes into process exits.

use std::io::{self, BufReader, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};

use crate::ColorMode;
use crate::buffer::LineBufferedWriter;
use crate::colorizer::colorize_regex;
use crate::grc::GrcatConfigEntry;
use crate::utils::pseudo_command_excluded;
use crate::{load_rules_for_command, load_rules_for_config};

/// Exit status to propagate: the child's code, or 128+signal like a shell
/// reports when the child was killed.
pub fn child_exit_code(status: &ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        128 + status.signal().unwrap_or(1)
    }
    #[cfg(not(unix))]
    {
        1
    }
}

// BrokenPipe means the downstream reader went away (`... | head`): exit 0
// quietly instead of surfacing an error
fn fail_code(e: Box<dyn std::error::Error>) -> i32 {
    let broken = e
        .downcast_ref::<std::io::Error>()
        .is_some_and(|io_err| io_err.kind() == io::ErrorKind::BrokenPipe);
    if broken {
        return 0;
    }
    eprintln!("Error: {}", e);
    1
}

// ---- stdin filter mode (`rgrc -c NAME` without a command) ----

pub enum StdinPlan {
    /// no color: copy stdin through
    Passthrough,
    /// colorize stdin with these rules
    Filter(Vec<GrcatConfigEntry>),
    /// no rules resolved: report and copy stdin through, exit 1
    NoRules(String),
}

pub fn stdin_plan(config: &str, color_mode: ColorMode, stdout_is_tty: bool) -> StdinPlan {
    let should_colorize = match color_mode {
        ColorMode::Off => false,
        ColorMode::On => true,
        ColorMode::Auto => stdout_is_tty,
    };
    if !should_colorize {
        return StdinPlan::Passthrough;
    }
    let rules = load_rules_for_config(config);
    if rules.is_empty() {
        return StdinPlan::NoRules(config.to_string());
    }
    StdinPlan::Filter(rules)
}

/// Execute a stdin plan against the real stdin/stdout, returning the exit code.
pub fn filter_stdin_exit(plan: StdinPlan) -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    filter_exit(plan, &mut stdin.lock(), &mut stdout.lock())
}

// streams are injected so tests never touch the process stdin
// (a tty stdin would block io::copy forever)
pub fn filter_exit<R: Read, W: Write>(plan: StdinPlan, reader: &mut R, writer: &mut W) -> i32 {
    match plan {
        StdinPlan::Passthrough => copy_stream(reader, writer, 0, 0),
        StdinPlan::NoRules(name) => {
            eprintln!(
                "Error: Failed to load rules for config '{}': No matching rules found",
                name
            );
            copy_stream(reader, writer, 0, 1)
        }
        StdinPlan::Filter(rules) => colorize_stream(reader, writer, &rules),
    }
}

// copy reader to writer; `ok` on success, `err` on a copy failure
fn copy_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W, ok: i32, err: i32) -> i32 {
    let mut writer = io::BufWriter::new(writer);
    let code = match io::copy(reader, &mut writer) {
        Ok(_) => ok,
        Err(_) => err,
    };
    let _ = writer.flush();
    code
}

fn colorize_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    rules: &[GrcatConfigEntry],
) -> i32 {
    let mut reader = io::BufReader::with_capacity(64 * 1024, reader);
    let mut writer = io::BufWriter::with_capacity(64 * 1024, writer);
    let mut line_writer = LineBufferedWriter::new(&mut writer);
    if let Err(e) = colorize_regex(&mut reader, &mut line_writer, rules) {
        return fail_code(e);
    }
    match writer.flush() {
        Ok(()) => 0,
        Err(e) => fail_code(Box::new(e)),
    }
}

// ---- command wrapping mode ----

/// Color-mode decision before rules are loaded.
pub fn should_colorize(color_mode: ColorMode, stdout_is_tty: bool) -> bool {
    match color_mode {
        ColorMode::Off => false,
        ColorMode::On => true,
        ColorMode::Auto => stdout_is_tty,
    }
}

/// Pseudo-command exclusions apply unless a `-c` config was given explicitly.
pub fn eligible_for_colorize(
    color_mode: ColorMode,
    stdout_is_tty: bool,
    pseudo_command: &str,
    explicit_config: bool,
) -> bool {
    should_colorize(color_mode, stdout_is_tty)
        && (explicit_config || !pseudo_command_excluded(pseudo_command))
}

pub fn rules_for(explicit_config: Option<&str>, pseudo_command: &str) -> Vec<GrcatConfigEntry> {
    match explicit_config {
        Some(name) => load_rules_for_config(name),
        None => load_rules_for_command(pseudo_command),
    }
}

// spawn with the shared not-found / failure reporting; Err carries the code
fn spawn_or_code(cmd: &mut Command, command_name: &str) -> Result<Child, i32> {
    match cmd.spawn() {
        Ok(child) => Ok(child),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("Error: command not found: '{}'", command_name);
            Err(127)
        }
        Err(e) => {
            eprintln!("Failed to spawn '{}': {}", command_name, e);
            Err(1)
        }
    }
}

fn wait_code(child: &mut Child, command_name: &str) -> i32 {
    match child.wait() {
        Ok(status) => child_exit_code(&status),
        Err(e) => {
            eprintln!("Failed while waiting for '{}': {}", command_name, e);
            1
        }
    }
}

/// Run the child with inherited stdio; used when we are not colorizing.
/// Going through a pipe here only risks corrupting binary output
/// (`docker save > file`, see #31) and adds copying overhead for no gain.
pub fn run_passthrough(cmd: &mut Command, command_name: &str) -> i32 {
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    match spawn_or_code(cmd, command_name) {
        Ok(mut child) => wait_code(&mut child, command_name),
        Err(code) => code,
    }
}

/// Run the child with piped stdout and colorize the stream.
pub fn run_colorized(cmd: &mut Command, command_name: &str, rules: &[GrcatConfigEntry]) -> i32 {
    cmd.stdout(Stdio::piped());
    let mut child = match spawn_or_code(cmd, command_name) {
        Ok(c) => c,
        Err(code) => return code,
    };

    let mut stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    let mut reader = BufReader::with_capacity(64 * 1024, &mut stdout);
    let mut writer = io::BufWriter::with_capacity(64 * 1024, io::stdout());
    let mut line_writer = LineBufferedWriter::new(&mut writer);

    if let Err(e) = colorize_regex(&mut reader, &mut line_writer, rules) {
        return fail_code(e);
    }
    if let Err(e) = writer.flush() {
        return fail_code(Box::new(e));
    }
    wait_code(&mut child, command_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdin_plan_modes() {
        assert!(matches!(
            stdin_plan("df", ColorMode::Off, true),
            StdinPlan::Passthrough
        ));
        assert!(matches!(
            stdin_plan("df", ColorMode::Auto, false),
            StdinPlan::Passthrough
        ));

        // a name no mapper resolves yields NoRules
        assert!(matches!(
            stdin_plan("definitely-not-a-config-xyz", ColorMode::On, true),
            StdinPlan::NoRules(_)
        ));

        #[cfg(feature = "embed-configs")]
        assert!(matches!(
            stdin_plan("df", ColorMode::On, true),
            StdinPlan::Filter(_)
        ));
    }

    #[test]
    fn colorize_eligibility() {
        assert!(should_colorize(ColorMode::On, false));
        assert!(!should_colorize(ColorMode::Off, true));
        assert!(should_colorize(ColorMode::Auto, true));
        assert!(!should_colorize(ColorMode::Auto, false));

        // bare `rgrc ls` skips coloring unless a config was given explicitly
        assert!(!eligible_for_colorize(ColorMode::On, true, "ls", false));
        assert!(eligible_for_colorize(ColorMode::On, true, "ls -l", false));
        assert!(eligible_for_colorize(ColorMode::On, true, "ls", true));
        assert!(eligible_for_colorize(ColorMode::On, true, "df -h", false));
        assert!(!eligible_for_colorize(ColorMode::Off, true, "df -h", false));
    }

    #[cfg(unix)]
    fn sh(script: &str) -> Command {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(script);
        cmd
    }

    #[test]
    #[cfg(unix)]
    fn passthrough_exit_codes() {
        assert_eq!(run_passthrough(&mut sh("exit 3"), "sh"), 3);
        assert_eq!(
            run_passthrough(&mut Command::new("definitely-not-a-command-xyz"), "x"),
            127
        );
    }

    #[test]
    #[cfg(unix)]
    fn colorized_exit_codes() {
        #[cfg(feature = "embed-configs")]
        let rules = load_rules_for_config("ping");
        #[cfg(feature = "embed-configs")]
        {
            assert_eq!(run_colorized(&mut sh("exit 0"), "sh", &rules), 0);
            assert_eq!(run_colorized(&mut sh("kill -9 $$"), "sh", &rules), 137);
        }
        assert_eq!(
            run_colorized(&mut Command::new("definitely-not-a-command-xyz"), "x", &[]),
            127
        );
    }

    // streams are injected; touching the process stdin here would hang a
    // terminal `make test` on io::copy
    #[test]
    fn stdin_plan_exit_codes() {
        let mut input = io::Cursor::new(b"line\n".to_vec());
        let mut output = Vec::new();
        assert_eq!(
            filter_exit(StdinPlan::Passthrough, &mut input, &mut output),
            0
        );
        assert_eq!(output, b"line\n");

        let mut input = io::Cursor::new(b"line\n".to_vec());
        let mut output = Vec::new();
        // NoRules copies through too; exit 1 only when the copy itself fails
        assert_eq!(
            filter_exit(
                StdinPlan::NoRules("no-such-config".into()),
                &mut input,
                &mut output
            ),
            0
        );

        let mut input = io::Cursor::new(b"hello\n".to_vec());
        let mut output = Vec::new();
        assert_eq!(
            filter_exit(StdinPlan::Filter(Vec::new()), &mut input, &mut output),
            0
        );
        assert_eq!(output, b"hello\n");
    }
}
