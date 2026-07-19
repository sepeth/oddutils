use std::collections::VecDeque;
use std::env;
use std::ffi::OsString;
use std::process::{Command, ExitCode, Output};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use oddutils_core::process::status_code;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => ExitCode::from(run(&config)),
        Err(error) => {
            eprintln!("parallel: {error}");
            eprintln!("Usage: parallel [OPTIONS] command -- arguments");
            eprintln!("       parallel [OPTIONS] -- commands");
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
struct Config {
    max_jobs: usize,
    max_load: Option<f64>,
    replace: bool,
    args_at_once: usize,
    command: Vec<OsString>,
    arguments: Vec<OsString>,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let mut max_jobs = std::thread::available_parallelism().map_or(1, usize::from);
        let mut max_load = None;
        let mut replace = false;
        let mut args_at_once = 1;
        let mut before_separator = Vec::new();

        while let Some(arg) = args.next() {
            let text = arg.to_string_lossy();
            match text.as_ref() {
                "-h" | "--help" => return Err("help requested".to_string()),
                "-i" => replace = true,
                "-j" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "-j requires a value".to_string())?;
                    max_jobs = value
                        .to_string_lossy()
                        .parse::<usize>()
                        .map_err(|_| "option '-j' is not a number".to_string())?;
                }
                "-n" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "-n requires a value".to_string())?;
                    args_at_once = value
                        .to_string_lossy()
                        .parse::<usize>()
                        .map_err(|_| "option '-n' is not a positive number".to_string())?;
                    if args_at_once == 0 {
                        return Err("option '-n' is not a positive number".to_string());
                    }
                }
                "-l" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "-l requires a value".to_string())?;
                    max_load = Some(
                        value
                            .to_string_lossy()
                            .parse::<f64>()
                            .map_err(|_| "option '-l' is not a number".to_string())?,
                    );
                }
                "--" => {
                    let arguments = args.collect::<Vec<_>>();
                    if arguments.is_empty() {
                        return Err("missing arguments".to_string());
                    }
                    if replace && args_at_once > 1 {
                        return Err("options -i and -n are incompatible".to_string());
                    }
                    if args_at_once > 1 && before_separator.is_empty() {
                        return Err("option -n cannot be used without a command".to_string());
                    }
                    return Ok(Self {
                        max_jobs,
                        max_load,
                        replace,
                        args_at_once,
                        command: before_separator,
                        arguments,
                    });
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown option {value}"));
                }
                _ => before_separator.push(arg),
            }
        }

        Err("missing -- separator".to_string())
    }
}

fn run(config: &Config) -> u8 {
    let jobs = build_jobs(config);
    let mut pending = VecDeque::from(jobs);
    let mut running: Vec<JoinHandle<std::io::Result<Output>>> = Vec::new();
    let mut result = 0_u8;
    let max_jobs = if config.max_jobs == 0 {
        usize::MAX
    } else {
        config.max_jobs
    };

    while !pending.is_empty() || !running.is_empty() {
        while running.len() < max_jobs {
            let Some(job) = pending.pop_front() else {
                break;
            };
            wait_for_load(config.max_load);
            running.push(thread::spawn(move || run_job(job)));
        }

        let handle = running.remove(0);
        match handle.join() {
            Ok(Ok(output)) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                result |= status_code(output.status);
            }
            Ok(Err(error)) => {
                eprintln!("parallel: {error}");
                result |= 1;
            }
            Err(_) => {
                eprintln!("parallel: worker panicked");
                result |= 1;
            }
        }
    }

    result
}

fn wait_for_load(max_load: Option<f64>) {
    let Some(max_load) = max_load else {
        return;
    };
    while load_average() >= max_load {
        thread::sleep(Duration::from_secs(1));
    }
}

fn load_average() -> f64 {
    let mut load = [0.0_f64; 1];
    // SAFETY: `load` points to writable memory for one load-average value.
    let read = unsafe { getloadavg(load.as_mut_ptr(), 1) };
    if read == 1 { load[0] } else { 0.0 }
}

#[derive(Debug)]
enum Job {
    Shell(OsString),
    Command(Vec<OsString>),
}

fn build_jobs(config: &Config) -> Vec<Job> {
    if config.command.is_empty() {
        return config.arguments.iter().cloned().map(Job::Shell).collect();
    }

    config
        .arguments
        .chunks(config.args_at_once)
        .map(|chunk| {
            let mut command = config.command.clone();
            if config.replace {
                let replacement = chunk[0].to_string_lossy();
                for part in &mut command {
                    let replaced = part.to_string_lossy().replace("{}", &replacement);
                    *part = OsString::from(replaced);
                }
            } else {
                command.extend(chunk.iter().cloned());
            }
            Job::Command(command)
        })
        .collect()
}

fn run_job(job: Job) -> std::io::Result<Output> {
    match job {
        Job::Shell(command) => Command::new("sh").arg("-c").arg(command).output(),
        Job::Command(command) => Command::new(&command[0]).args(&command[1..]).output(),
    }
}

unsafe extern "C" {
    fn getloadavg(loadavg: *mut f64, nelem: i32) -> i32;
}
