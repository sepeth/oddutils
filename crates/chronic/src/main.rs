use std::env;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::thread;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("chronic: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("chronic: {error}");
            eprintln!("usage: chronic [-ev] COMMAND...");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    stderr_trigger: bool,
    verbose: bool,
    command: Vec<OsString>,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut stderr_trigger = false;
        let mut verbose = false;
        let mut command = Vec::new();
        let mut parsing_options = true;

        for arg in args {
            if parsing_options && arg == "--" {
                parsing_options = false;
                continue;
            }

            if parsing_options {
                let text = arg.to_string_lossy();
                if text == "-h" || text == "--help" {
                    return Err("usage requested".to_string());
                }
                if text.starts_with('-') && text.len() > 1 {
                    for option in text[1..].chars() {
                        match option {
                            'e' => stderr_trigger = true,
                            'v' => verbose = true,
                            _ => return Err(format!("unknown option '-{option}'")),
                        }
                    }
                    continue;
                }
            }

            parsing_options = false;
            command.push(arg);
        }

        if command.is_empty() {
            return Err("missing command".to_string());
        }

        Ok(Self {
            stderr_trigger,
            verbose,
            command,
        })
    }
}

fn run(config: &Config) -> io::Result<u8> {
    let mut child = Command::new(&config.command[0])
        .args(&config.command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("failed to capture child stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("failed to capture child stderr"))?;

    let out_thread = thread::spawn(move || read_all(stdout));
    let err_thread = thread::spawn(move || read_all(stderr));
    let status = child.wait()?;
    let out = out_thread
        .join()
        .map_err(|_| io::Error::other("stdout reader panicked"))??;
    let err = err_thread
        .join()
        .map_err(|_| io::Error::other("stderr reader panicked"))??;

    if let Some(code) = status.code() {
        if code != 0 {
            show_output(config.verbose, code, &out, &err)?;
            return Ok(code.try_into().unwrap_or(1));
        }
        if config.stderr_trigger && !err.is_empty() {
            show_output(config.verbose, 0, &out, &err)?;
            return Ok(2);
        }
        return Ok(0);
    }

    show_output(config.verbose, 1, &out, &err)?;
    Ok(1)
}

fn read_all(mut reader: impl Read) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn show_output(verbose: bool, code: i32, out: &[u8], err: &[u8]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    if verbose {
        stdout.write_all(b"STDOUT:\n")?;
    }
    stdout.write_all(out)?;
    if verbose {
        stdout.write_all(b"\nSTDERR:\n")?;
        stdout.flush()?;
    }
    stderr.write_all(err)?;
    stderr.flush()?;
    if verbose {
        writeln!(stdout, "\nRETVAL: {code}")?;
    }
    stdout.flush()
}
