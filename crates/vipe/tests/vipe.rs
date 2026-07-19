use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn vipe_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vipe"))
}

#[test]
fn outputs_editor_modified_input() {
    let temp = TestDir::new();
    let editor = temp.path().join("editor.sh");
    fs::write(&editor, "#!/bin/sh\nprintf edited > \"$1\"\n").unwrap();
    make_executable(&editor);

    let output = run_vipe(&editor, &[], b"original");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"edited");
}

#[test]
fn suffix_is_added_to_tempfile() {
    let temp = TestDir::new();
    let editor = temp.path().join("editor.sh");
    let seen = temp.path().join("seen");
    fs::write(
        &editor,
        format!(
            "#!/bin/sh\ncase \"$1\" in *.csv) echo ok > {};; esac\n",
            shell_quote(&seen)
        ),
    )
    .unwrap();
    make_executable(&editor);

    let output = run_vipe(&editor, &["--suffix", "csv"], b"");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(fs::read_to_string(seen).unwrap(), "ok\n");
}

#[test]
fn editor_failure_fails_vipe() {
    let temp = TestDir::new();
    let editor = temp.path().join("editor.sh");
    fs::write(&editor, "#!/bin/sh\nexit 7\n").unwrap();
    make_executable(&editor);

    let output = run_vipe(&editor, &[], b"");

    assert!(!output.status.success());
}

fn run_vipe(editor: &Path, args: &[&str], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(vipe_bin())
        .args(args)
        .env("EDITOR", editor)
        .env_remove("VISUAL")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
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
        for attempt in 0..1000_u32 {
            let path = std::env::temp_dir().join(format!(
                "oddutils-vipe-test-{}-{stamp}-{attempt}",
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
