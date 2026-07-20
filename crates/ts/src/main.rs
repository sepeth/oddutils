use std::env;
use std::ffi::{CStr, CString, OsString};
use std::io::{self, BufRead, Write};
use std::os::raw::c_char;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(Action::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(Action::Run(config)) => match run(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("ts: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("ts: {error}");
            eprintln!("usage: ts [-r] [-i | -s] [-m] [format]");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Clone)]
struct Config {
    mode: Mode,
    format: String,
    monotonic: bool,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Absolute,
    Incremental,
    SinceStart,
}

#[derive(Debug)]
enum Action {
    Help,
    Run(Config),
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
        let mut mode = Mode::Absolute;
        let mut monotonic = false;
        let mut relative = false;
        let mut positional = false;
        let mut format = None;

        for arg in args {
            if !positional && (arg == "-h" || arg == "--help") {
                return Ok(Action::Help);
            }
            if !positional && arg == "--" {
                positional = true;
                continue;
            }
            if !positional && arg == "-r" {
                relative = true;
                continue;
            }
            if !positional && arg == "-i" {
                mode = Mode::Incremental;
                continue;
            }
            if !positional && arg == "-s" {
                mode = Mode::SinceStart;
                continue;
            }
            if !positional && arg == "-m" {
                monotonic = true;
                continue;
            }
            if !positional && arg.to_string_lossy().starts_with('-') {
                return Err(format!("unknown option '{}'", arg.to_string_lossy()));
            }
            if format.replace(arg.to_string_lossy().into_owned()).is_some() {
                return Err("expected at most one format".to_string());
            }
        }

        if relative {
            return Err("-r timestamp conversion is not implemented yet".to_string());
        }

        let format = format.unwrap_or_else(|| match mode {
            Mode::Absolute => "%b %d %H:%M:%S".to_string(),
            Mode::Incremental | Mode::SinceStart => "%H:%M:%S".to_string(),
        });

        Ok(Action::Run(Self {
            mode,
            format,
            monotonic,
        }))
    }
}

fn run(config: &Config) -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut line = Vec::new();
    let mut clock = Clock::new();

    while input.read_until(b'\n', &mut line)? != 0 {
        let timestamp = clock.timestamp(config)?;
        output.write_all(timestamp.as_bytes())?;
        output.write_all(b" ")?;
        output.write_all(&line)?;
        output.flush()?;
        line.clear();
    }

    output.flush()
}

struct Clock {
    start_wall: UnixTime,
    start: Instant,
    last: Instant,
}

impl Clock {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            start_wall: UnixTime::now(),
            start: now,
            last: now,
        }
    }

    fn timestamp(&mut self, config: &Config) -> io::Result<String> {
        let stamp = match config.mode {
            Mode::Absolute if config.monotonic => {
                let elapsed = self.start.elapsed();
                self.start_wall.add_duration(elapsed)
            }
            Mode::Absolute => UnixTime::now(),
            Mode::Incremental => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last);
                self.last = now;
                UnixTime::from_duration(elapsed)
            }
            Mode::SinceStart => UnixTime::from_duration(self.start.elapsed()),
        };
        let utc = matches!(config.mode, Mode::Incremental | Mode::SinceStart);
        format_time(&config.format, stamp, utc)
    }
}

#[derive(Debug, Clone, Copy)]
struct UnixTime {
    seconds: i64,
    microseconds: u32,
}

impl UnixTime {
    fn now() -> Self {
        let mut tv = libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        // SAFETY: `tv` points to valid writable memory and no timezone pointer
        // is supplied.
        unsafe {
            libc::gettimeofday(&raw mut tv, std::ptr::null_mut());
        }
        Self {
            seconds: time_t_to_i64(tv.tv_sec),
            microseconds: tv.tv_usec.try_into().unwrap_or(0),
        }
    }

    fn from_duration(duration: std::time::Duration) -> Self {
        Self {
            seconds: duration.as_secs().try_into().unwrap_or(i64::MAX),
            microseconds: duration.subsec_micros(),
        }
    }

    fn add_duration(self, duration: std::time::Duration) -> Self {
        let mut seconds = self
            .seconds
            .saturating_add(duration.as_secs().try_into().unwrap_or(i64::MAX));
        let mut microseconds = self.microseconds + duration.subsec_micros();
        if microseconds >= 1_000_000 {
            seconds = seconds.saturating_add(1);
            microseconds -= 1_000_000;
        }
        Self {
            seconds,
            microseconds,
        }
    }
}

#[allow(clippy::useless_conversion)]
fn time_t_to_i64(value: libc::time_t) -> i64 {
    value.try_into().unwrap_or_else(|_| {
        if value.is_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    })
}

fn format_time(format: &str, stamp: UnixTime, utc: bool) -> io::Result<String> {
    if format.is_empty() {
        return Ok(String::new());
    }

    let expanded = expand_subseconds(format, stamp.microseconds);
    let c_format = CString::new(expanded)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "format contains NUL byte"))?;
    let mut time = libc::time_t::try_from(stamp.seconds).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "timestamp is outside platform time_t range",
        )
    })?;
    let mut tm = std::mem::MaybeUninit::<libc::tm>::uninit();

    let tm_ptr = if utc {
        // SAFETY: Pointers refer to initialized local variables.
        unsafe { libc::gmtime_r(&raw mut time, tm.as_mut_ptr()) }
    } else {
        // SAFETY: Pointers refer to initialized local variables.
        unsafe { libc::localtime_r(&raw mut time, tm.as_mut_ptr()) }
    };
    if tm_ptr.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: localtime_r/gmtime_r returned non-null, so `tm` has been initialized.
    let tm = unsafe { tm.assume_init() };

    let mut buffer = vec![0_u8; 256];
    loop {
        // SAFETY: `buffer` is writable, `c_format` is NUL-terminated, and
        // `tm` was initialized by localtime_r/gmtime_r.
        let written = unsafe {
            libc::strftime(
                buffer.as_mut_ptr().cast::<c_char>(),
                buffer.len(),
                c_format.as_ptr(),
                &raw const tm,
            )
        };
        if written == 0 {
            if buffer.len() >= 16 * 1024 {
                return Err(io::Error::other("formatted timestamp is too long"));
            }
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }

        // SAFETY: strftime wrote a NUL-terminated string into `buffer`.
        let text = unsafe { CStr::from_ptr(buffer.as_ptr().cast::<c_char>()) };
        return Ok(text.to_string_lossy().into_owned());
    }
}

fn expand_subseconds(format: &str, microseconds: u32) -> String {
    let micros = format!("{microseconds:06}");
    format
        .replace("%.S", &format!("%S.{micros}"))
        .replace("%.s", &format!("%s.{micros}"))
        .replace("%.T", &format!("%T.{micros}"))
}

fn print_usage() {
    println!("ts [-r] [-i | -s] [-m] [format]");
    println!("  add a timestamp to the beginning of each input line");
}
