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
fn relative_mode_reformats_iso_without_fractional_seconds() {
    let output = run_ts(
        &["-r", "%Y"],
        b"2026-07-20T18:31:19Z first event\n2026-07-20T18:34:49Z second event\n",
    );

    assert_eq!(output.stdout, b"2026 first event\n2026 second event\n");
}

#[test]
fn relative_mode_reformats_moreutils_fractional_iso() {
    let output = run_ts(
        &["-r", "%Y-%m-%d %H:%M:%S"],
        b"2026-07-20T18:31:19.000Z first event\n",
    );
    let text = String::from_utf8(output.stdout).unwrap();

    assert!(text.ends_with(" first event\n"));
    assert!(text.starts_with("2026-07-20 "));
}

#[test]
fn relative_mode_reformats_rfc2822_timestamp() {
    let output = run_ts(&["-r", "%Y"], b"Wed, 02 Jun 2021 06:31:39 GMT mail event\n");

    assert_eq!(output.stdout, b"2021 mail event\n");
}

#[test]
fn relative_mode_describes_old_timestamp() {
    let output = run_ts(&["-r"], b"1970-01-01T00:00:00Z old event\n");
    let text = String::from_utf8(output.stdout).unwrap();

    assert!(text.contains("ago old event\n"));
    assert!(!text.contains("1970-01-01"));
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
