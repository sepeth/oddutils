use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    match Config::parse(env::args_os()) {
        Ok(config) => match run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("zrun: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("zrun: {error}");
            eprintln!("Usage: zrun <command> <args>");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    program: OsString,
    args: Vec<OsString>,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let argv0 = args.next().unwrap_or_else(|| OsString::from("zrun"));
        let invoked = Path::new(&argv0)
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("zrun");

        if let Some(program) = invoked.strip_prefix('z')
            && program != "run"
            && !program.is_empty()
        {
            let rest = args.collect::<Vec<_>>();
            if rest.is_empty() {
                return Err(format!("missing arguments for z{program}"));
            }
            return Ok(Self {
                program: OsString::from(program),
                args: rest,
            });
        }

        let program = args.next().ok_or_else(|| "missing command".to_string())?;
        let rest = args.collect::<Vec<_>>();
        if rest.is_empty() {
            return Err("missing command arguments".to_string());
        }
        Ok(Self {
            program,
            args: rest,
        })
    }
}

fn run(config: &Config) -> io::Result<u8> {
    let mut temps = Vec::new();
    let mut preprocessors = Vec::new();
    let mut args = Vec::new();

    for arg in &config.args {
        if let Some(kind) = Compression::from_path(Path::new(arg)) {
            let temp = TempPath::new(Path::new(arg))?;
            let child = spawn_decompressor(kind, Path::new(arg), temp.path())?;
            args.push(temp.path().as_os_str().to_owned());
            preprocessors.push(Preprocessor {
                input: PathBuf::from(arg),
                child,
            });
            temps.push(temp);
        } else {
            args.push(arg.clone());
        }
    }

    for mut preprocessor in preprocessors {
        let status = preprocessor.child.wait()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "preprocessing for {} terminated with code {}",
                preprocessor.input.display(),
                status.code().unwrap_or(1)
            )));
        }
    }

    let status = Command::new(&config.program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .unwrap_or(1))
}

fn spawn_decompressor(kind: Compression, input: &Path, output: &Path) -> io::Result<Child> {
    let file = OpenOptions::new().write(true).truncate(true).open(output)?;
    Command::new(kind.program())
        .args(kind.args())
        .arg(input)
        .stdout(Stdio::from(file.try_clone()?))
        .spawn()
}

struct Preprocessor {
    input: PathBuf,
    child: Child,
}

#[derive(Debug, Clone, Copy)]
enum Compression {
    Gzip,
    Bzip2,
    Xz,
    Lzop,
    Lzma,
    Zstd,
}

impl Compression {
    fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(OsStr::to_str) {
            Some("gz" | "Z") => Some(Self::Gzip),
            Some("bz2") => Some(Self::Bzip2),
            Some("xz") => Some(Self::Xz),
            Some("lzo") => Some(Self::Lzop),
            Some("lzma") => Some(Self::Lzma),
            Some("zst") => Some(Self::Zstd),
            _ => None,
        }
    }

    fn program(self) -> &'static str {
        match self {
            Self::Gzip => "gzip",
            Self::Bzip2 => "bzip2",
            Self::Xz => "xz",
            Self::Lzop => "lzop",
            Self::Lzma => "lzma",
            Self::Zstd => "zstd",
        }
    }

    fn args(self) -> &'static [&'static str] {
        match self {
            Self::Gzip | Self::Bzip2 | Self::Xz | Self::Lzop | Self::Lzma | Self::Zstd => {
                &["-d", "-c"]
            }
        }
    }
}

struct TempPath {
    path: PathBuf,
}

impl TempPath {
    fn new(source: &Path) -> io::Result<Self> {
        let dir = env::temp_dir();
        let pid = std::process::id();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let suffix = source
            .file_stem()
            .and_then(OsStr::to_str)
            .map_or(String::new(), |stem| format!("-{stem}"));

        for attempt in 0..1000_u32 {
            let path = dir.join(format!("oddutils-zrun-{pid}-{stamp}-{attempt}{suffix}"));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a unique temporary file",
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
