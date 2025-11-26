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
