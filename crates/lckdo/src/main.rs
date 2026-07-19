use std::env;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const EX_TEMPFAIL: u8 = 75;
const EX_USAGE: u8 = 64;
const EX_CANTCREAT: u8 = 73;

const LOCK_SH: i32 = 1;
const LOCK_EX: i32 = 2;
const LOCK_NB: i32 = 4;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(Action::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(Action::Run(config)) => match run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("lckdo: {error}");
                ExitCode::from(EX_CANTCREAT)
            }
        },
        Err(error) => {
            eprintln!("lckdo: {error}");
            print_usage();
            ExitCode::from(EX_USAGE)
        }
    }
}

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Config {
    create: bool,
    quiet: bool,
    shared: bool,
    test: bool,
    wait: WaitMode,
    lockfile: OsString,
    command: Vec<OsString>,
}

#[derive(Debug)]
enum Action {
    Help,
    Run(Config),
}

#[derive(Debug, Clone, Copy)]
enum WaitMode {
    NoWait,
    Forever,
    Seconds(u64),
}

impl Config {
    fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Action, String> {
        let mut create = true;
        let mut quiet = false;
        let mut shared = false;
        let mut test = false;
        let mut wait = WaitMode::NoWait;
        let mut rest = Vec::new();

        while let Some(arg) = args.next() {
            let option_text = arg.to_string_lossy();
            match option_text.as_ref() {
                "-h" | "--help" => return Ok(Action::Help),
                "-w" => wait = WaitMode::Forever,
                "-W" => {
                    let seconds = args
                        .next()
                        .ok_or_else(|| "-W requires seconds".to_string())?
                        .to_string_lossy()
                        .parse::<u64>()
                        .map_err(|_| "invalid wait time".to_string())?;
                    if seconds == 0 {
                        return Err("invalid wait time".to_string());
                    }
                    wait = WaitMode::Seconds(seconds);
                }
                "-n" => create = false,
                "-q" => quiet = true,
                "-s" => shared = true,
                "-x" => shared = false,
                "-t" => {
                    test = true;
                    create = false;
                }
                "-e" => {}
                "-E" => {
                    let _ = args
                        .next()
                        .ok_or_else(|| "-E requires a file descriptor".to_string())?;
                }
                value if value.starts_with('-') => return Err(format!("unknown option {value}")),
                _ => {
                    rest.push(arg);
                    rest.extend(args);
                    break;
                }
            }
        }

        if rest.is_empty() || (!test && rest.len() < 2) {
            return Err("too few arguments given".to_string());
        }
        let lockfile = rest.remove(0);
        Ok(Action::Run(Self {
            create,
            quiet,
            shared,
            test,
            wait,
            lockfile,
            command: rest,
        }))
    }
}

fn run(config: &Config) -> io::Result<u8> {
    let Some(file) = open_lockfile(config)? else {
        if config.test {
            if !config.quiet {
                println!(
                    "lockfile `{}` is not locked",
                    config.lockfile.to_string_lossy()
                );
            }
            return Ok(0);
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "lockfile does not exist",
        ));
    };
    let lock_op = if config.shared { LOCK_SH } else { LOCK_EX };
    let locked = acquire_lock(&file, lock_op, config.wait)?;

    if config.test {
        if locked {
            if !config.quiet {
                println!(
                    "lockfile `{}` is not locked",
                    config.lockfile.to_string_lossy()
                );
            }
            return Ok(0);
        }
        if config.quiet {
            println!("locked");
        } else {
            println!("lockfile `{}` is locked", config.lockfile.to_string_lossy());
        }
        return Ok(EX_TEMPFAIL);
    }

    if !locked {
        if !config.quiet {
            eprintln!(
                "lckdo: lockfile `{}` is already locked",
                config.lockfile.to_string_lossy()
            );
        }
        return Ok(EX_TEMPFAIL);
    }

    let status = Command::new(&config.command[0])
        .args(&config.command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .unwrap_or(1))
}

fn open_lockfile(config: &Config) -> io::Result<Option<File>> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if config.create {
        options.create(true);
    }
    match options.open(&config.lockfile) {
        Ok(file) => Ok(Some(file)),
        Err(error) if config.test && error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn acquire_lock(file: &File, lock_op: i32, wait: WaitMode) -> io::Result<bool> {
    match wait {
        WaitMode::Forever => flock_call(file, lock_op).map(|()| true),
        WaitMode::NoWait => try_lock(file, lock_op),
        WaitMode::Seconds(seconds) => {
            let deadline = Instant::now() + Duration::from_secs(seconds);
            loop {
                if try_lock(file, lock_op)? {
                    return Ok(true);
                }
                if Instant::now() >= deadline {
                    return Ok(false);
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn try_lock(file: &File, lock_op: i32) -> io::Result<bool> {
    match flock_call(file, lock_op | LOCK_NB) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(false),
        Err(error) => Err(error),
    }
}

fn flock_call(file: &File, operation: i32) -> io::Result<()> {
    // SAFETY: `file.as_raw_fd()` is a valid open file descriptor and `operation`
    // is one of the platform flock operation bitmasks.
    let result = unsafe { flock(file.as_raw_fd(), operation) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn print_usage() {
    eprintln!("Usage: lckdo [options] lockfile program [arguments]");
    eprintln!("  -w       wait for lock");
    eprintln!("  -W sec   wait up to sec seconds for lock");
    eprintln!("  -n       do not create lock file");
    eprintln!("  -q       quiet lock failure");
    eprintln!("  -s       shared lock");
    eprintln!("  -x       exclusive lock");
    eprintln!("  -t       test lock existence");
}

unsafe extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}
