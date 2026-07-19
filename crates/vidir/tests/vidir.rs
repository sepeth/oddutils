use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn vidir_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vidir"))
}

#[test]
fn renames_file_from_editor_change() {
    let temp = TestDir::new();
    let old = temp.path().join("old.txt");
    let new = temp.path().join("new.txt");
    fs::write(&old, "data").unwrap();
    let editor = temp.editor(&format!(
        "#!/bin/sh\nsed 's#{}#{}#' \"$1\" > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"\n",
        old.display(),
        new.display()
    ));

    let output = run_vidir(&editor, &[old.as_path()], b"");

    assert!(output.status.success(), "{output:?}");
    assert!(!old.exists());
    assert_eq!(fs::read_to_string(new).unwrap(), "data");
}

#[test]
fn deletes_removed_lines() {
    let temp = TestDir::new();
    let file = temp.path().join("delete.txt");
    fs::write(&file, "data").unwrap();
    let editor = temp.editor("#!/bin/sh\n: > \"$1\"\n");

    let output = run_vidir(&editor, &[file.as_path()], b"");

    assert!(output.status.success(), "{output:?}");
    assert!(!file.exists());
}

#[test]
fn deletes_item_with_empty_edited_name() {
    let temp = TestDir::new();
    let file = temp.path().join("delete-empty-name.txt");
    fs::write(&file, "data").unwrap();
    let editor = temp.editor("#!/bin/sh\ncut -f1 \"$1\" > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"\n");

    let output = run_vidir(&editor, &[file.as_path()], b"");

    assert!(output.status.success(), "{output:?}");
    assert!(!file.exists());
}

#[test]
fn reads_file_list_from_stdin() {
    let temp = TestDir::new();
    let old = temp.path().join("stdin-old");
    let new = temp.path().join("stdin-new");
    fs::write(&old, "data").unwrap();
    let editor = temp.editor(&format!(
        "#!/bin/sh\nsed 's#{}#{}#' \"$1\" > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"\n",
        old.display(),
        new.display()
    ));

    let output = run_vidir(
        &editor,
        &[Path::new("-")],
        format!("{}\n", old.display()).as_bytes(),
    );

    assert!(output.status.success(), "{output:?}");
    assert!(!old.exists());
    assert_eq!(fs::read_to_string(new).unwrap(), "data");
}

fn run_vidir(editor: &Path, args: &[&Path], stdin: &[u8]) -> std::process::Output {
    let mut child = Command::new(vidir_bin())
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
                "oddutils-vidir-test-{}-{stamp}-{attempt}",
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

    fn editor(&self, contents: &str) -> PathBuf {
        let editor = self.path.join("editor.sh");
        fs::write(&editor, contents).unwrap();
        make_executable(&editor);
        editor
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
