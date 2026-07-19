use std::env;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use oddutils_core::temp::TempFile;
use oddutils_core::unix::{OutputMetadata, output_metadata, set_mode};

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(Action::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(Action::Run(config)) => match run(config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("sponge: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("sponge: {error}");
            eprintln!("Try 'sponge --help' for usage.");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    append: bool,
    output: Option<PathBuf>,
}

#[derive(Debug)]
enum Action {
    Help,
    Run(Config),
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
        let mut append = false;
        let mut output = None;
        let mut positional = false;

        for arg in args {
            if !positional && (arg == "-h" || arg == "--help") {
                return Ok(Action::Help);
            }

            if !positional && arg == "-a" {
                append = true;
                continue;
            }

            if !positional && arg == "--" {
                positional = true;
                continue;
            }

            if !positional && arg.to_string_lossy().starts_with('-') {
                return Err(format!("unknown option '{}'", arg.to_string_lossy()));
            }

            if output.replace(PathBuf::from(&arg)).is_some() {
                return Err("expected at most one output file".to_string());
            }
        }

        Ok(Action::Run(Self { append, output }))
    }
}

fn run(config: Config) -> io::Result<()> {
    let mut input = TempFile::in_default_dir("oddutils-sponge-stdin")?;
    io::copy(&mut io::stdin().lock(), input.file_mut())?;
    input.file_mut().flush()?;
    input.file_mut().seek(SeekFrom::Start(0))?;

    if let Some(output) = config.output {
        write_file(config.append, &mut input, output)
    } else {
        io::copy(input.file_mut(), &mut io::stdout().lock())?;
        Ok(())
    }
}

fn write_file(append: bool, input: &mut TempFile, output: PathBuf) -> io::Result<()> {
    let metadata = output_metadata(&output)?;

    if metadata.regular_file || !metadata.exists {
        let mut replacement = TempFile::in_default_dir("oddutils-sponge-output")?;

        if append && metadata.regular_file {
            let mut current = File::open(&output)?;
            io::copy(&mut current, replacement.file_mut())?;
        }

        input.file_mut().seek(SeekFrom::Start(0))?;
        io::copy(input.file_mut(), replacement.file_mut())?;
        replacement.file_mut().flush()?;
        set_mode(replacement.path(), metadata.mode)?;
        if replacement.rename_into_place(&output).is_err() {
            copy_replacement_fallback(&mut replacement, &output, &metadata)?;
        }
        Ok(())
    } else {
        let mut out = File::create(output)?;
        if append {
            input.file_mut().seek(SeekFrom::Start(0))?;
        }
        io::copy(input.file_mut(), &mut out)?;
        out.flush()
    }
}

fn copy_replacement_fallback(
    replacement: &mut TempFile,
    output: &Path,
    metadata: &OutputMetadata,
) -> io::Result<()> {
    replacement.file_mut().seek(SeekFrom::Start(0))?;
    let mut out = fallback_output(output, metadata)?;
    io::copy(replacement.file_mut(), &mut out)?;
    out.flush()
}

fn fallback_output(output: &Path, metadata: &OutputMetadata) -> io::Result<File> {
    if metadata.exists {
        let out = OpenOptions::new().write(true).open(output)?;
        ensure_same_file(&out, metadata)?;
        out.set_len(0)?;
        Ok(out)
    } else {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(metadata.mode)
            .open(output)
    }
}

fn ensure_same_file(file: &File, metadata: &OutputMetadata) -> io::Result<()> {
    let current = file.metadata()?;
    if Some(current.dev()) == metadata.device && Some(current.ino()) == metadata.inode {
        Ok(())
    } else {
        Err(io::Error::other(
            "output path changed before fallback copy, aborting",
        ))
    }
}

fn print_usage() {
    println!("sponge [-a] [file]");
    println!("  soak up standard input and write it to file, or stdout if no file is given");
}

#[cfg(test)]
mod tests {
    use super::{copy_replacement_fallback, output_metadata};
    use oddutils_core::temp::TempFile;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn fallback_refuses_changed_output_path_before_truncating() {
        let dir = TestDir::new();
        let output = dir.path().join("output");
        let target = dir.path().join("target");
        fs::write(&output, "old").unwrap();
        fs::write(&target, "keep").unwrap();
        let metadata = output_metadata(&output).unwrap();

        fs::remove_file(&output).unwrap();
        symlink(&target, &output).unwrap();

        let mut replacement = TempFile::in_dir("oddutils-sponge-test", dir.path()).unwrap();
        replacement.file_mut().write_all(b"new").unwrap();

        let error = copy_replacement_fallback(&mut replacement, &output, &metadata).unwrap_err();

        assert!(error.to_string().contains("output path changed"));
        assert_eq!(fs::read_to_string(target).unwrap(), "keep");
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
                    "oddutils-sponge-unit-{}-{stamp}-{attempt}",
                    std::process::id()
                ));
                if fs::create_dir(&path).is_ok() {
                    return Self { path };
                }
            }

            panic!("could not create test directory");
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
