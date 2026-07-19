use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn pee_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pee"))
}

#[test]
fn sends_stdin_to_each_command() {
    let temp = TestDir::new();
    let one = temp.path().join("one");
    let two = temp.path().join("two");

    let output = run_pee(
        &[
            &format!("cat > {}", shell_quote(&one)),
            &format!("tr a-z A-Z > {}", shell_quote(&two)),
        ],
        b"hello\n",
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(fs::read_to_string(one).unwrap(), "hello\n");
    assert_eq!(fs::read_to_string(two).unwrap(), "HELLO\n");
}

#[test]
fn does_not_copy_input_to_stdout_itself() {
    let output = run_pee(&[], b"hello\n");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn returns_bitwise_or_of_child_statuses() {
    let output = run_pee(&["exit 1", "exit 2"], b"x");

    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn can_emit_command_output_to_stdout() {
    let output = run_pee(&["cat"], b"hello\n");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello\n");
}

fn run_pee(args: &[&str], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(pee_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
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
        let path = std::env::temp_dir().join(format!("oddutils-pee-test-{stamp}"));
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
