use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn isutf8_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_isutf8"))
}

#[test]
fn accepts_valid_stdin() {
    let output = run_isutf8(&[], "hello\n".as_bytes());

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn rejects_invalid_stdin() {
    let output = run_isutf8(&[], b"hello\xff\n");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("(standard input): line 1"));
}

#[test]
fn quiet_suppresses_output() {
    let output = run_isutf8(&["-q"], b"\xff");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn list_prints_invalid_file_names() {
    let temp = TestDir::new();
    let invalid = temp.path().join("bad");
    fs::write(&invalid, b"\xff").unwrap();

    let output = Command::new(isutf8_bin())
        .arg("-l")
        .arg(&invalid)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", invalid.display())
    );
}

#[test]
fn invert_lists_valid_file_names() {
    let temp = TestDir::new();
    let valid = temp.path().join("good");
    fs::write(&valid, "ok\n").unwrap();

    let output = Command::new(isutf8_bin())
        .arg("-i")
        .arg(&valid)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", valid.display())
    );
}

fn run_isutf8(args: &[&str], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(isutf8_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("oddutils-isutf8-test-{stamp}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
