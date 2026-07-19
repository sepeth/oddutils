use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(Action::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(Action::Run(config)) => match run(&config) {
            Ok(valid) => {
                if valid {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(error) => {
                eprintln!("isutf8: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("isutf8: {error}");
            eprintln!("Usage: isutf8 [OPTION]... [FILE]...");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Config {
    quiet: bool,
    list_only: bool,
    invert: bool,
    verbose: bool,
    files: Vec<PathBuf>,
}

#[derive(Debug)]
enum Action {
    Help,
    Run(Config),
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
        let mut quiet = false;
        let mut list_only = false;
        let mut invert = false;
        let mut verbose = false;
        let mut files = Vec::new();
        let mut parsing_options = true;

        for arg in args {
            if parsing_options {
                match arg.to_string_lossy().as_ref() {
                    "-h" | "--help" => return Ok(Action::Help),
                    "-q" | "--quiet" => {
                        quiet = true;
                        continue;
                    }
                    "-l" | "--list" | "--list-only" => {
                        list_only = true;
                        continue;
                    }
                    "-i" | "--invert" => {
                        invert = true;
                        continue;
                    }
                    "-v" | "--verbose" => {
                        verbose = true;
                        continue;
                    }
                    "--" => {
                        parsing_options = false;
                        continue;
                    }
                    text if text.starts_with('-') && text.len() > 1 => {
                        for option in text[1..].chars() {
                            match option {
                                'q' => quiet = true,
                                'l' => list_only = true,
                                'i' => invert = true,
                                'v' => verbose = true,
                                'h' => return Ok(Action::Help),
                                _ => return Err(format!("unknown option '-{option}'")),
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            parsing_options = false;
            files.push(PathBuf::from(arg));
        }

        Ok(Action::Run(Self {
            quiet,
            list_only,
            invert,
            verbose,
            files,
        }))
    }
}

fn run(config: &Config) -> io::Result<bool> {
    let mut all_valid = true;
    if config.files.is_empty() {
        let mut input = Vec::new();
        io::stdin().lock().read_to_end(&mut input)?;
        all_valid &= check_input(config, "(standard input)", &input)?;
    } else {
        for path in &config.files {
            match fs::read(path) {
                Ok(input) => {
                    all_valid &= check_input(config, &path.display().to_string(), &input)?;
                }
                Err(error) => {
                    all_valid = false;
                    if !config.quiet {
                        eprintln!("{}: {error}", path.display());
                    }
                }
            }
        }
    }
    Ok(all_valid)
}

fn check_input(config: &Config, label: &str, input: &[u8]) -> io::Result<bool> {
    match std::str::from_utf8(input) {
        Ok(_) => {
            if config.invert && !config.quiet {
                println!("{label}");
            }
            Ok(true)
        }
        Err(error) => {
            if !config.quiet && !config.invert {
                if config.list_only {
                    println!("{label}");
                } else {
                    let invalid_at = error.valid_up_to();
                    let (line, column) = line_column(input, invalid_at);
                    println!(
                        "{label}: line {line}, char {column}, byte {invalid_at}: invalid UTF-8"
                    );
                    if config.verbose {
                        print_context(input, invalid_at)?;
                    }
                }
            }
            Ok(false)
        }
    }
}

fn line_column(input: &[u8], offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut line_start = 0;
    for (index, byte) in input.iter().copied().enumerate().take(offset) {
        if byte == b'\n' {
            line += 1;
            line_start = index + 1;
        }
    }
    (line, offset - line_start + 1)
}

fn print_context(input: &[u8], offset: usize) -> io::Result<()> {
    let start = offset.saturating_sub(8);
    let end = input.len().min(offset + 8);
    let window = &input[start..end];
    let mut stdout = io::stdout().lock();

    for byte in window {
        write!(stdout, "{byte:02X} ")?;
    }
    writeln!(stdout)?;
    for byte in window {
        let printable = if (b' '..=b'~').contains(byte) {
            char::from(*byte)
        } else {
            '.'
        };
        write!(stdout, "{printable}")?;
    }
    writeln!(stdout)?;
    writeln!(stdout, "{:>width$}^", "", width = offset - start)
}

fn print_usage() {
    println!("Usage: isutf8 [OPTION]... [FILE]...");
    println!("Check whether input files are valid UTF-8.");
    println!("  -h, --help       display this help text and exit");
    println!("  -q, --quiet      suppress all normal output");
    println!("  -l, --list       print only names of FILEs containing invalid UTF-8");
    println!("  -i, --invert     list valid UTF-8 files instead of invalid ones");
    println!("  -v, --verbose    print detailed error context");
}
