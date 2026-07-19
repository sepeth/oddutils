use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn sponge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sponge"))
}

#[test]
fn writes_stdin_to_file_after_reading() {
    let temp = TestDir::new();
    let file = temp.path().join("values.txt");
    fs::write(&file, "3\n1\n2\n").unwrap();

    let input = fs::read_to_string(&file).unwrap();
    run_sponge(&[], Some(&file), input.as_bytes());

    assert_eq!(fs::read_to_string(file).unwrap(), "3\n1\n2\n");
}

#[test]
fn writes_to_stdout_without_file() {
    let output = Command::new(sponge_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child.stdin.as_mut().unwrap().write_all(b"hello\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello\n");
}

#[test]
fn append_keeps_original_content_before_stdin() {
    let temp = TestDir::new();
    let file = temp.path().join("log.txt");
    fs::write(&file, "old\n").unwrap();

    run_sponge(&["-a"], Some(&file), b"new\n");

    assert_eq!(fs::read_to_string(file).unwrap(), "old\nnew\n");
}

#[test]
fn handles_binary_stdin() {
    let temp = TestDir::new();
    let file = temp.path().join("blob");

    run_sponge(&[], Some(&file), b"\0\xffoddutils\n");

    assert_eq!(fs::read(file).unwrap(), b"\0\xffoddutils\n");
}

#[test]
fn rejects_multiple_output_files() {
    let output = Command::new(sponge_bin())
        .arg("one")
        .arg("two")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("expected at most one output file"));
}

fn run_sponge(args: &[&str], file: Option<&Path>, stdin: &[u8]) {
    let mut command = Command::new(sponge_bin());
    command.args(args);
    if let Some(file) = file {
        command.arg(file);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "{output:?}");
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
        let path = std::env::temp_dir().join(format!("oddutils-test-{stamp}"));
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
