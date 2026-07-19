//! Unix-only filesystem helpers.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

unsafe extern "C" {
    fn umask(mask: u32) -> u32;
}

/// Metadata about an output path before it is modified.
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    pub exists: bool,
    pub regular_file: bool,
    pub mode: u32,
}

/// Inspect an output path without following symlinks.
///
/// # Errors
///
/// Returns an error if metadata lookup fails for a reason other than the path
/// not existing.
pub fn output_metadata(path: &Path) -> io::Result<OutputMetadata> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            Ok(OutputMetadata {
                exists: true,
                regular_file: file_type.is_file() && !file_type.is_symlink(),
                mode: metadata.permissions().mode() & 0o7777,
            })
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(OutputMetadata {
            exists: false,
            regular_file: false,
            mode: default_file_mode(),
        }),
        Err(error) => Err(error),
    }
}

/// Apply Unix mode bits to a path.
///
/// # Errors
///
/// Returns an error if permissions cannot be changed.
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

fn default_file_mode() -> u32 {
    let mask = current_umask();
    0o666 & !mask
}

fn current_umask() -> u32 {
    // SAFETY: umask is process-global, but this command is single-threaded at
    // the point where it is called. The old mask is immediately restored.
    let mask = unsafe { umask(0) };
    // SAFETY: Restores the value just returned by the previous umask call.
    unsafe { umask(mask) };
    mask
}
