use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

use oddutils_core::editor::{attach_tty, editor_command};
use oddutils_core::temp::TempPath;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("vipe: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("vipe: {error}");
            eprintln!("Usage: vipe [--suffix=extension]");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    suffix: String,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut suffix = String::new();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            let text = arg.to_string_lossy();
            if text == "--help" || text == "-h" {
                return Err("usage requested".to_string());
            }
            if text == "--suffix" {
                let value = args
                    .next()
                    .ok_or_else(|| "--suffix requires a value".to_string())?;
                suffix = normalize_suffix(&value.to_string_lossy());
                continue;
            }
            if let Some(value) = text.strip_prefix("--suffix=") {
                suffix = normalize_suffix(value);
                continue;
            }
            return Err(format!("unknown argument '{text}'"));
        }

        Ok(Self { suffix })
    }
}

fn run(config: &Config) -> io::Result<()> {
    let temp = TempPath::in_default_dir("oddutils-vipe", &config.suffix)?;
    if !stdin_is_tty() {
        let mut file = OpenOptions::new().write(true).open(temp.path())?;
        io::copy(&mut io::stdin().lock(), &mut file)?;
        file.flush()?;
    }

    let editor = editor_command();
    let mut command = Command::new(&editor[0]);
    command.args(&editor[1..]).arg(temp.path());
    attach_tty(&mut command);
    let status = command.status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "{} exited nonzero, aborting",
            editor
                .iter()
                .map(|part| part.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        )));
    }

    let mut file = OpenOptions::new().read(true).open(temp.path())?;
    io::copy(&mut file, &mut io::stdout().lock())?;
    Ok(())
}

fn normalize_suffix(suffix: &str) -> String {
    if suffix.is_empty() || suffix.starts_with('.') {
        suffix.to_string()
    } else {
        format!(".{suffix}")
    }
}

fn stdin_is_tty() -> bool {
    // SAFETY: `isatty` only inspects the supplied file descriptor.
    unsafe { isatty(0) == 1 }
}

unsafe extern "C" {
    fn isatty(fd: i32) -> i32;
}
