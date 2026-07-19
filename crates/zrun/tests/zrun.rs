use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn zrun_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_zrun"))
}

#[test]
fn decompresses_gzip_argument_before_running_command() {
    let temp = TestDir::new();
    let plain = temp.path().join("data.txt");
    let gz = temp.path().join("data.txt.gz");
    fs::write(&plain, "hello\n").unwrap();
    let gzip = Command::new("gzip").arg("-c").arg(&plain).output().unwrap();
    assert!(gzip.status.success());
    fs::write(&gz, gzip.stdout).unwrap();

    let output = Command::new(zrun_bin())
        .arg("cat")
        .arg(&gz)
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"hello\n");
}

#[test]
fn passes_uncompressed_arguments_through() {
    let temp = TestDir::new();
    let plain = temp.path().join("data.txt");
    fs::write(&plain, "plain\n").unwrap();

    let output = Command::new(zrun_bin())
        .arg("cat")
        .arg(&plain)
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"plain\n");
}

#[test]
fn returns_child_exit_status() {
    let output = Command::new(zrun_bin())
        .arg("sh")
        .arg("-c")
        .arg("exit 7")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
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
                "oddutils-zrun-test-{}-{stamp}-{attempt}",
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
