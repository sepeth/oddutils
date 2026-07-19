use std::path::PathBuf;
use std::process::Command;

fn chronic_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_chronic"))
}

#[test]
fn suppresses_successful_output() {
    let output = Command::new(chronic_bin())
        .arg("sh")
        .arg("-c")
        .arg("printf out; printf err >&2")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn replays_output_and_exit_code_on_failure() {
    let output = Command::new(chronic_bin())
        .arg("sh")
        .arg("-c")
        .arg("printf out; printf err >&2; exit 7")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"out");
    assert_eq!(output.stderr, b"err");
}

#[test]
fn stderr_trigger_exits_two_when_command_succeeds() {
    let output = Command::new(chronic_bin())
        .arg("-e")
        .arg("sh")
        .arg("-c")
        .arg("printf err >&2")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"err");
}

#[test]
fn verbose_adds_labels_and_retval() {
    let output = Command::new(chronic_bin())
        .arg("-v")
        .arg("sh")
        .arg("-c")
        .arg("printf out; printf err >&2; exit 3")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout.contains("STDOUT:\nout"));
    assert!(stdout.contains("STDERR:"));
    assert!(stdout.contains("RETVAL: 3"));
    assert_eq!(output.stderr, b"err");
}
