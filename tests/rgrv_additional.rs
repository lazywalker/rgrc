use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_rgrv_binary() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test exe name
    path.pop(); // remove deps
    path.push("rgrv");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

// Unexpected line after colours= should produce a FormatError mention
#[test]
fn test_unexpected_line_after_colours() {
    let tmp = TempDir::new().unwrap();
    let conf = tmp.path().join("conf.test");

    let mut f = fs::File::create(&conf).unwrap();
    writeln!(f, "regexp=^test").unwrap();
    writeln!(f, "colours=red").unwrap();
    writeln!(f, "weird=unexpected").unwrap();
    drop(f);

    let output = Command::new(get_rgrv_binary())
        .arg("conf")
        .arg(conf.to_str().unwrap())
        .output()
        .expect("failed to run rgrv");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Unexpected line after colours=") || combined.contains("FormatError")
    );
}

// Unexpected line after regexp: when a following non-colours/non-config line appears
#[test]
fn test_unexpected_line_after_regexp() {
    let tmp = TempDir::new().unwrap();
    let conf = tmp.path().join("conf.test");

    let mut f = fs::File::create(&conf).unwrap();
    writeln!(f, "regexp=^hello").unwrap();
    writeln!(f, "not-a-valid-line").unwrap();
    drop(f);

    let output = Command::new(get_rgrv_binary())
        .arg("conf")
        .arg(conf.to_str().unwrap())
        .output()
        .expect("failed to run rgrv");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(combined.contains("Unexpected line after regexp") || combined.contains("FormatError"));
}
