use std::env;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, Command, ExitCode, Stdio};

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("pee: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("pee: {error}");
            eprintln!(
                "usage: pee [--[no-]ignore-sigpipe] [--[no-]ignore-write-errors] [command ...]"
            );
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    ignore_sigpipe: bool,
    ignore_write_errors: bool,
    commands: Vec<OsString>,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut ignore_sigpipe = true;
        let mut ignore_write_errors = true;
        let mut commands = Vec::new();
        let mut parsing_options = true;

        for arg in args {
            if parsing_options {
                match arg.to_string_lossy().as_ref() {
                    "--ignore-sigpipe" => {
                        ignore_sigpipe = true;
                        continue;
                    }
                    "--no-ignore-sigpipe" => {
                        ignore_sigpipe = false;
                        continue;
                    }
                    "--ignore-write-errors" => {
                        ignore_write_errors = true;
                        continue;
                    }
                    "--no-ignore-write-errors" => {
                        ignore_write_errors = false;
                        continue;
                    }
                    "--" => {
                        parsing_options = false;
                        continue;
                    }
                    text if text.starts_with('-') => {
                        return Err(format!("unknown option '{text}'"));
                    }
                    _ => {}
                }
            }
            parsing_options = false;
            commands.push(arg);
        }

        Ok(Self {
            ignore_sigpipe,
            ignore_write_errors,
            commands,
        })
    }
}

struct PipeCommand {
    label: String,
    child: Child,
    stdin: Option<ChildStdin>,
    inactive: bool,
}

fn run(config: &Config) -> io::Result<u8> {
    configure_sigpipe(config.ignore_sigpipe);

    let mut children = config
        .commands
        .iter()
        .map(spawn_pipe)
        .collect::<io::Result<Vec<_>>>()?;
    let mut input = io::stdin().lock();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        for index in 0..children.len() {
            if children[index].inactive {
                continue;
            }

            let write_result = children[index]
                .stdin
                .as_mut()
                .expect("active child has stdin")
                .write_all(&buffer[..read]);

            if let Err(_error) = write_result {
                children[index].inactive = true;
                children[index].stdin.take();

                if !config.ignore_write_errors {
                    eprintln!("Write error to `{}`", children[index].label);
                    kill_all(&mut children);
                    return Ok(1);
                }
                if children.iter().all(|child| child.inactive) {
                    return Ok(1);
                }
            }
        }
    }

    for child in &mut children {
        child.stdin.take();
    }

    let mut result = 0_u8;
    for mut child in children {
        let status = child.child.wait()?;
        result |= status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1);
    }
    Ok(result)
}

fn spawn_pipe(command: &OsString) -> io::Result<PipeCommand> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("failed to open child stdin"))?;

    Ok(PipeCommand {
        label: command.to_string_lossy().into_owned(),
        child,
        stdin: Some(stdin),
        inactive: false,
    })
}

fn kill_all(children: &mut [PipeCommand]) {
    for child in children {
        let _ = child.child.kill();
        let _ = child.child.wait();
    }
}

fn configure_sigpipe(ignore: bool) {
    #[cfg(unix)]
    {
        const SIGPIPE: i32 = 13;
        const SIG_DFL: usize = 0;
        const SIG_IGN: usize = 1;
        let handler = if ignore { SIG_IGN } else { SIG_DFL };
        // SAFETY: `signal` changes the current process disposition for SIGPIPE.
        unsafe {
            signal(SIGPIPE, handler);
        }
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}
