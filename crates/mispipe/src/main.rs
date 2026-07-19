use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Command, ExitCode, Stdio};
use std::thread;

use oddutils_core::process::status_code;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("mispipe: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("mispipe: {error}");
            eprintln!("usage: mispipe command1 command2");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    first: OsString,
    second: OsString,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let first = args
            .next()
            .ok_or_else(|| "missing first command".to_string())?;
        let second = args
            .next()
            .ok_or_else(|| "missing second command".to_string())?;
        if args.next().is_some() {
            return Err("expected exactly two commands".to_string());
        }
        Ok(Self { first, second })
    }
}

fn run(config: &Config) -> io::Result<u8> {
    let mut second = Command::new("sh")
        .arg("-c")
        .arg(&config.second)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let second_stdin = second
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("failed to open second command stdin"))?;

    let mut first = Command::new("sh")
        .arg("-c")
        .arg(&config.first)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let first_stdout = first
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("failed to capture first command stdout"))?;

    let copier = thread::spawn(move || {
        let mut input = first_stdout;
        let mut output = second_stdin;
        match io::copy(&mut input, &mut output) {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(error) => Err(error),
        }?;
        output.flush()
    });

    let first_status = first.wait()?;
    copier
        .join()
        .map_err(|_| io::Error::other("pipe copier panicked"))??;
    let _ = second.wait()?;

    Ok(status_code(first_status))
}
