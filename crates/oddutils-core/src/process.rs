//! Process status helpers.

use std::process::ExitStatus;

use std::os::unix::process::ExitStatusExt;

/// Convert a child process status into the byte-sized exit code oddutils should
/// return to its caller.
#[must_use]
pub fn status_code(status: ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return u8::try_from(code).unwrap_or(1);
    }

    status
        .signal()
        .and_then(|signal| u8::try_from(128 + signal).ok())
        .unwrap_or(1)
}
