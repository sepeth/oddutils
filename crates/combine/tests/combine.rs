use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn combine_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_combine"))
}

#[test]
fn and_outputs_file1_lines_present_in_file2() {
    let temp = TestDir::new();
    let a = temp.file("a", "one\ntwo\ntwo\nthree\n");
    let b = temp.file("b", "two\nfour\n");

    let output = run_combine(&[&a, "and", &b], b"");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"two\ntwo\n");
}

#[test]
fn not_outputs_file1_lines_absent_from_file2() {
    let temp = TestDir::new();
    let a = temp.file("a", "one\ntwo\nthree\n");
    let b = temp.file("b", "two\n");

    let output = run_combine(&[&a, "not", &b], b"");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"one\nthree\n");
}

#[test]
fn or_concatenates_file1_then_file2() {
    let temp = TestDir::new();
    let a = temp.file("a", "one\n");
    let b = temp.file("b", "two\n");

    let output = run_combine(&[&a, "or", &b], b"");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"one\ntwo\n");
}

#[test]
fn xor_outputs_lines_not_in_both() {
    let temp = TestDir::new();
    let a = temp.file("a", "one\ntwo\n");
    let b = temp.file("b", "two\nthree\n");

    let output = run_combine(&[&a, "xor", &b], b"");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"one\nthree\n");
}

#[test]
fn reads_one_file_from_stdin() {
    let temp = TestDir::new();
    let b = temp.file("b", "two\n");

    let output = run_combine(&["-", "and", &b], b"one\ntwo\n");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"two\n");
}

#[test]
fn preserves_carriage_returns_when_splitting_lines() {
    let temp = TestDir::new();
    let a = temp.file("a", "two\r\n");
    let b = temp.file("b", "two\n");

    let output = run_combine(&[&a, "and", &b], b"");

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
}

fn run_combine(args: &[&str], stdin: &[u8]) -> std::process::Output {
    let mut command = Command::new(combine_bin());
    for arg in args {
        command.arg(arg);
    }
    let mut child = command
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
        for attempt in 0..1000_u32 {
            let path = std::env::temp_dir().join(format!(
                "oddutils-combine-test-{}-{stamp}-{attempt}",
                std::process::id()
            ));
            if fs::create_dir(&path).is_ok() {
                return Self { path };
            }
        }
        panic!("failed to create unique test directory");
    }

    fn file(&self, name: &str, contents: &str) -> String {
        let path = self.path.join(name);
        fs::write(&path, contents).unwrap();
        path.display().to_string()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
