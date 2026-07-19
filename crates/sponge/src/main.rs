use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use oddutils_core::temp::TempFile;
use oddutils_core::unix::{output_metadata, set_mode};

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
        replacement.persist(output)?;
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

fn print_usage() {
    println!("sponge [-a] [file]");
    println!("  soak up standard input and write it to file, or stdout if no file is given");
}
