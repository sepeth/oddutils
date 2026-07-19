//! Temporary file support for commands that must fully consume input before
//! touching their output path.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A temporary file removed on drop unless it has been persisted.
#[derive(Debug)]
pub struct TempFile {
    path: Option<PathBuf>,
    file: File,
}

impl TempFile {
    /// Create a temporary file in `$TMPDIR`, or `/tmp` if `TMPDIR` is unset.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be accessed or a
    /// unique file cannot be created.
    pub fn in_default_dir(prefix: &str) -> io::Result<Self> {
        let dir = std::env::var_os("TMPDIR").map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
        Self::in_dir(prefix, dir)
    }

    /// Create a temporary file in `dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if `dir` cannot be accessed or a unique file cannot be
    /// created after several attempts.
    pub fn in_dir(prefix: &str, dir: impl AsRef<Path>) -> io::Result<Self> {
        let dir = dir.as_ref();
        let pid = std::process::id();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        for attempt in 0..1000_u32 {
            let path = dir.join(format!("{prefix}.{pid}.{stamp}.{attempt}.tmp"));
            match OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    return Ok(Self {
                        path: Some(path),
                        file,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a unique temporary file",
        ))
    }

    /// Return the temporary path.
    ///
    /// # Panics
    ///
    /// Panics only if called after a successful [`Self::persist`], which
    /// consumes `self` and is therefore impossible for normal callers.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("temporary path is available until persist succeeds")
    }

    /// Return a mutable handle to the file.
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Rename the temporary file to `dest` and disable drop cleanup.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be synced or renamed into place.
    ///
    /// # Panics
    ///
    /// Panics only if the temporary path has already been removed from this
    /// value, which cannot happen through the public API before `self` is
    /// consumed.
    pub fn persist(mut self, dest: impl AsRef<Path>) -> io::Result<()> {
        self.file.sync_all()?;
        let path = self
            .path
            .as_ref()
            .expect("temporary path is available until persist succeeds");
        fs::rename(path, dest).inspect(|()| {
            self.path = None;
        })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}
