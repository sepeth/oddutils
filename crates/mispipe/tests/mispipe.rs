use std::path::PathBuf;
use std::process::Command;

fn mispipe_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mispipe"))
}

#[test]
fn pipes_first_command_into_second() {
    let output = Command::new(mispipe_bin())
        .arg("printf hello")
        .arg("tr a-z A-Z")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"HELLO");
}

#[test]
fn returns_first_command_status_not_second() {
    let output = Command::new(mispipe_bin())
        .arg("printf hello; exit 7")
        .arg("cat; exit 0")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"hello");
}

#[test]
fn ignores_second_command_status() {
    let output = Command::new(mispipe_bin())
        .arg("printf hello; exit 0")
        .arg("cat; exit 9")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello");
}

#[test]
fn rejects_wrong_argument_count() {
    let output = Command::new(mispipe_bin()).arg("true").output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing second command"));
}
