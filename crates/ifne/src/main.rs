use std::env;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};

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
    let mut input = Vec::new();
    io::stdin().lock().read_to_end(&mut input)?;

    if input.is_empty() && !config.run_if_empty {
        return Ok(0);
    }

    if !input.is_empty() && config.run_if_empty {
        io::stdout().lock().write_all(&input)?;
        return Ok(0);
    }

    run_command(&config.command, &input)
}

fn run_command(command: &[OsString], input: &[u8]) -> io::Result<u8> {
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        match stdin.write_all(input) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
            Err(error) => return Err(error),
        }
    }

    let status = child.wait()?;
    if let Some(code) = status.code() {
        Ok(code.try_into().unwrap_or(1))
    } else {
        Ok(1)
    }
}
