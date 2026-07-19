//! Editor selection and terminal attachment helpers.

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

/// Return the editor command from the standard environment variables.
///
/// Selection order matches moreutils: `$VISUAL`, `$EDITOR`, `/usr/bin/editor`
/// when present, then `vi`.
#[must_use]
pub fn editor_command() -> Vec<OsString> {
    if let Some(editor) = env::var_os("VISUAL").or_else(|| env::var_os("EDITOR")) {
        return editor
            .to_string_lossy()
            .split_whitespace()
            .map(OsString::from)
            .collect();
    }
    if Path::new("/usr/bin/editor").is_file() {
        return vec![OsString::from("/usr/bin/editor")];
    }
    vec![OsString::from("vi")]
}

/// Attach a command's stdin/stdout to `/dev/tty` when available.
pub fn attach_tty(command: &mut Command) {
    if let Ok(tty) = File::options().read(true).write(true).open("/dev/tty") {
        if let Ok(stdin) = tty.try_clone() {
            command.stdin(Stdio::from(stdin));
        }
        command.stdout(Stdio::from(tty));
    }
}
