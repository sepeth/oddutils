use std::path::PathBuf;
use std::process::Command;

fn errno_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_errno"))
}

#[test]
fn looks_up_name_case_insensitively() {
    let output = Command::new(errno_bin()).arg("enoent").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.starts_with("ENOENT 2 "));
}

#[test]
fn looks_up_code() {
    let output = Command::new(errno_bin()).arg("13").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.starts_with("EACCES 13 "));
}

#[test]
fn list_includes_common_errors() {
    let output = Command::new(errno_bin()).arg("-l").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("ENOENT 2 "));
    assert!(stdout.contains("EPIPE 32 "));
}

#[test]
fn search_matches_description_words() {
    let output = Command::new(errno_bin())
        .arg("-s")
        .arg("permission")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("EACCES 13 "));
}

#[test]
fn unknown_lookup_fails() {
    let output = Command::new(errno_bin()).arg("ENOPE").output().unwrap();

    assert!(!output.status.success());
}
