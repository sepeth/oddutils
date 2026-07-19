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
        self.rename_into_place(dest)
    }

    /// Rename the temporary file to `dest` and disable drop cleanup.
    ///
    /// Unlike [`Self::persist`], this keeps the temporary file available if the
    /// rename fails, so callers can fall back to copying its contents.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be synced or renamed into place.
    ///
    /// # Panics
    ///
    /// Panics only if called after a successful previous rename.
    pub fn rename_into_place(&mut self, dest: impl AsRef<Path>) -> io::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::TempFile;
    use std::fs;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn failed_rename_keeps_temp_file_available() {
        let dir = TestDir::new();
        let target = dir.path().join("target-dir");
        fs::create_dir(&target).unwrap();

        let mut temp = TempFile::in_dir("oddutils-test", dir.path()).unwrap();
        temp.file_mut().write_all(b"content").unwrap();

        let error = temp.rename_into_place(&target).unwrap_err();
        assert!(error.kind() != std::io::ErrorKind::NotFound);

        temp.file_mut().seek(SeekFrom::Start(0)).unwrap();
        let mut contents = String::new();
        temp.file_mut().read_to_string(&mut contents).unwrap();

        assert_eq!(contents, "content");
        assert!(temp.path().exists());
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
            let path = std::env::temp_dir().join(format!("oddutils-core-test-{stamp}"));
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
}
