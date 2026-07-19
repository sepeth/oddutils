use std::env;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};

use oddutils_core::process::status_code;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("ifne: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("ifne: {error}");
            eprintln!("Usage: ifne [-n] command [args]");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    run_if_empty: bool,
    command: Vec<OsString>,
}

impl Config {
    fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self, String> {
        let first = args.next().ok_or_else(|| "missing command".to_string())?;
        let (run_if_empty, command_first) = if first == "-n" {
            (
                true,
                args.next()
                    .ok_or_else(|| "missing command after -n".to_string())?,
            )
        } else {
            (false, first)
        };

        let mut command = vec![command_first];
        command.extend(args);
        Ok(Self {
            run_if_empty,
            command,
        })
    }
}

fn run(config: &Config) -> io::Result<u8> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut first = [0_u8; 8192];
    let first_read = stdin.read(&mut first)?;

    if first_read == 0 && !config.run_if_empty {
        return Ok(0);
    }

    if first_read > 0 && config.run_if_empty {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        stdout.write_all(&first[..first_read])?;
        io::copy(&mut stdin, &mut stdout)?;
        return Ok(0);
    }

    run_command(&config.command, &first[..first_read], &mut stdin)
}

fn run_command(command: &[OsString], first: &[u8], rest: &mut impl Read) -> io::Result<u8> {
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut child_stdin) = child.stdin.take() {
        match child_stdin.write_all(first) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
            Err(error) => return Err(error),
        }
        match io::copy(rest, &mut child_stdin) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
            Err(error) => return Err(error),
        }
    }

    let status = child.wait()?;
    Ok(status_code(status))
}
