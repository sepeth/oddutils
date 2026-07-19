use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn lckdo_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lckdo"))
}

#[test]
fn runs_command_with_lock() {
    let temp = TestDir::new();
    let lock = temp.path().join("lock");

    let output = Command::new(lckdo_bin())
        .arg(&lock)
        .arg("sh")
        .arg("-c")
        .arg("printf ok")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"ok");
}

#[test]
fn returns_child_status() {
    let temp = TestDir::new();
    let lock = temp.path().join("lock");

    let output = Command::new(lckdo_bin())
        .arg(&lock)
        .arg("sh")
        .arg("-c")
        .arg("exit 7")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn direct_exec_returns_command_status() {
    let temp = TestDir::new();
    let lock = temp.path().join("lock");

    let output = Command::new(lckdo_bin())
        .arg("-e")
        .arg(&lock)
        .arg("sh")
        .arg("-c")
        .arg("exit 6")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(6));
}

#[test]
fn direct_exec_keeps_requested_fd_open() {
    let temp = TestDir::new();
    let lock = temp.path().join("lock");

    let output = Command::new(lckdo_bin())
        .arg("-E")
        .arg("9")
        .arg(&lock)
        .arg("sh")
        .arg("-c")
        .arg("test -e /dev/fd/9 || test -e /proc/self/fd/9")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
}

#[test]
fn test_mode_reports_unlocked_file() {
    let temp = TestDir::new();
    let lock = temp.path().join("lock");

    let output = Command::new(lckdo_bin())
        .arg("-t")
        .arg(&lock)
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("not locked")
    );
}

#[test]
fn no_create_fails_for_missing_lockfile() {
    let temp = TestDir::new();
    let lock = temp.path().join("missing");

    let output = Command::new(lckdo_bin())
        .arg("-n")
        .arg(&lock)
        .arg("true")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(73));
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
                "oddutils-lckdo-test-{}-{stamp}-{attempt}",
                std::process::id()
            ));
            if fs::create_dir(&path).is_ok() {
                return Self { path };
            }
        }
        panic!("failed to create unique test directory");
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
