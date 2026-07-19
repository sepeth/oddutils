use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ts_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ts"))
}

#[test]
fn prefixes_each_line_with_custom_format() {
    let output = run_ts(&["literal"], b"one\ntwo\n");

    assert_eq!(output.stdout, b"literal one\nliteral two\n");
}

#[test]
fn accepts_empty_format() {
    let output = run_ts(&[""], b"one\n");

    assert_eq!(output.stdout, b" one\n");
}

#[test]
fn writes_subsecond_incremental_timestamps() {
    let output = run_ts(&["-i", "%.S"], b"one\n");
    let text = String::from_utf8(output.stdout).unwrap();

    assert!(text.starts_with("00."));
    assert!(text.ends_with(" one\n"));
}

#[test]
fn writes_since_start_with_utc_default_shape() {
    let output = run_ts(&["-s"], b"one\n");
    let text = String::from_utf8(output.stdout).unwrap();

    assert!(text.starts_with("00:00:"));
    assert!(text.ends_with(" one\n"));
}

#[test]
fn rejects_relative_mode_until_date_parser_exists() {
    let output = Command::new(ts_bin()).arg("-r").output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not implemented yet"));
}

fn run_ts(args: &[&str], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(ts_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "{output:?}");
    output
}
