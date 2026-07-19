use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ifne_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ifne"))
}

#[test]
fn skips_command_when_stdin_is_empty() {
    let output = run_ifne(&["sh", "-c", "printf ran"], b"");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn runs_command_when_stdin_is_not_empty() {
    let output = run_ifne(&["cat"], b"hello\n");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello\n");
}

#[test]
fn reverse_runs_command_when_stdin_is_empty() {
    let output = run_ifne(&["-n", "sh", "-c", "printf empty"], b"");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"empty");
}

#[test]
fn reverse_passes_through_nonempty_stdin() {
    let output = run_ifne(&["-n", "sh", "-c", "printf should-not-run"], b"hello\n");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello\n");
}

#[test]
fn returns_child_exit_status() {
    let output = run_ifne(&["sh", "-c", "exit 9"], b"x");

    assert_eq!(output.status.code(), Some(9));
}

fn run_ifne(args: &[&str], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(ifne_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}
