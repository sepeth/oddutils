use std::env;
use std::ffi::{CStr, CString, OsString};
use std::os::raw::{c_char, c_int};
use std::process::{Command, ExitCode};

const LC_ALL: c_int = 0;

fn main() -> ExitCode {
    let _ = set_locale("");

    match Config::parse(env::args_os().skip(1)) {
        Ok(Action::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(Action::Run(config)) => {
            if run(&config) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("errno: {error}");
            eprintln!("Usage: errno [-lsS] [--list] [--search] [--search-all-locales] [keyword]");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    mode: Mode,
    args: Vec<String>,
}

#[derive(Debug)]
enum Action {
    Help,
    Run(Config),
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Lookup,
    List,
    Search,
    SearchAllLocales,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
        let mut mode = Mode::Lookup;
        let mut values = Vec::new();
        let mut parsing_options = true;

        for arg in args {
            if parsing_options {
                match arg.to_string_lossy().as_ref() {
                    "-h" | "--help" => return Ok(Action::Help),
                    "-l" | "--list" => {
                        mode = Mode::List;
                        continue;
                    }
                    "-s" | "--search" => {
                        mode = Mode::Search;
                        continue;
                    }
                    "-S" | "--search-all-locales" => {
                        mode = Mode::SearchAllLocales;
                        continue;
                    }
                    "--" => {
                        parsing_options = false;
                        continue;
                    }
                    text if text.starts_with('-') && text.len() > 1 => {
                        for option in text[1..].chars() {
                            match option {
                                'l' => mode = Mode::List,
                                's' => mode = Mode::Search,
                                'S' => mode = Mode::SearchAllLocales,
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
            values.push(arg.to_string_lossy().into_owned());
        }

        Ok(Action::Run(Self { mode, args: values }))
    }
}

fn run(config: &Config) -> bool {
    match config.mode {
        Mode::Lookup => lookup(&config.args),
        Mode::List => {
            for errno in ERRNOS {
                report(*errno);
            }
            true
        }
        Mode::Search => {
            search(&config.args);
            true
        }
        Mode::SearchAllLocales => search_all_locales(&config.args),
    }
}

fn lookup(args: &[String]) -> bool {
    let mut ok = true;
    for arg in args {
        if arg.starts_with('E') || arg.starts_with('e') {
            if let Some(errno) = by_name(arg) {
                report(errno);
            } else {
                ok = false;
            }
        } else if let Ok(code) = arg.parse::<i32>() {
            if let Some(errno) = by_code(code) {
                report(errno);
            } else {
                ok = false;
            }
        } else {
            eprintln!("ERROR: Not understood: {arg}");
            ok = false;
        }
    }
    ok
}

fn search(words: &[String]) {
    let words = words
        .iter()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>();
    for errno in ERRNOS {
        let description = description(errno.code).to_lowercase();
        if words.iter().all(|word| description.contains(word)) {
            report(*errno);
        }
    }
}

fn search_all_locales(words: &[String]) -> bool {
    let output = match Command::new("locale").arg("-a").output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("ERROR: Can't execute locale -a: {error}");
            return false;
        }
    };
    if !output.status.success() {
        eprintln!("ERROR: locale -a failed");
        return false;
    }

    for locale in String::from_utf8_lossy(&output.stdout).lines() {
        let _ = set_locale(locale);
        search(words);
    }
    true
}

fn report(errno: Errno) {
    println!("{} {} {}", errno.name, errno.code, description(errno.code));
}

fn description(code: i32) -> String {
    // SAFETY: strerror returns a pointer to a NUL-terminated static buffer for
    // the supplied errno value.
    let ptr = unsafe { strerror(code) };
    if ptr.is_null() {
        return "Unknown error".to_string();
    }
    // SAFETY: `ptr` is expected to point to a NUL-terminated C string.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

fn set_locale(locale: &str) -> bool {
    let Ok(locale) = CString::new(locale) else {
        return false;
    };
    // SAFETY: `locale` is a NUL-terminated C string and this command is
    // single-threaded while changing the process locale.
    !unsafe { setlocale(LC_ALL, locale.as_ptr()) }.is_null()
}

fn by_name(name: &str) -> Option<Errno> {
    ERRNOS
        .iter()
        .copied()
        .find(|errno| errno.name.eq_ignore_ascii_case(name))
}

fn by_code(code: i32) -> Option<Errno> {
    ERRNOS.iter().copied().find(|errno| errno.code == code)
}

fn print_usage() {
    println!("Usage: errno [-lsS] [--list] [--search] [--search-all-locales] [keyword]");
}

#[derive(Debug, Clone, Copy)]
struct Errno {
    name: &'static str,
    code: i32,
}

const ERRNOS: &[Errno] = &[
    Errno {
        name: "EPERM",
        code: 1,
    },
    Errno {
        name: "ENOENT",
        code: 2,
    },
    Errno {
        name: "ESRCH",
        code: 3,
    },
    Errno {
        name: "EINTR",
        code: 4,
    },
    Errno {
        name: "EIO",
        code: 5,
    },
    Errno {
        name: "ENXIO",
        code: 6,
    },
    Errno {
        name: "E2BIG",
        code: 7,
    },
    Errno {
        name: "ENOEXEC",
        code: 8,
    },
    Errno {
        name: "EBADF",
        code: 9,
    },
    Errno {
        name: "ECHILD",
        code: 10,
    },
    Errno {
        name: "EDEADLK",
        code: 11,
    },
    Errno {
        name: "ENOMEM",
        code: 12,
    },
    Errno {
        name: "EACCES",
        code: 13,
    },
    Errno {
        name: "EFAULT",
        code: 14,
    },
    Errno {
        name: "ENOTBLK",
        code: 15,
    },
    Errno {
        name: "EBUSY",
        code: 16,
    },
    Errno {
        name: "EEXIST",
        code: 17,
    },
    Errno {
        name: "EXDEV",
        code: 18,
    },
    Errno {
        name: "ENODEV",
        code: 19,
    },
    Errno {
        name: "ENOTDIR",
        code: 20,
    },
    Errno {
        name: "EISDIR",
        code: 21,
    },
    Errno {
        name: "EINVAL",
        code: 22,
    },
    Errno {
        name: "ENFILE",
        code: 23,
    },
    Errno {
        name: "EMFILE",
        code: 24,
    },
    Errno {
        name: "ENOTTY",
        code: 25,
    },
    Errno {
        name: "ETXTBSY",
        code: 26,
    },
    Errno {
        name: "EFBIG",
        code: 27,
    },
    Errno {
        name: "ENOSPC",
        code: 28,
    },
    Errno {
        name: "ESPIPE",
        code: 29,
    },
    Errno {
        name: "EROFS",
        code: 30,
    },
    Errno {
        name: "EMLINK",
        code: 31,
    },
    Errno {
        name: "EPIPE",
        code: 32,
    },
    Errno {
        name: "EDOM",
        code: 33,
    },
    Errno {
        name: "ERANGE",
        code: 34,
    },
    Errno {
        name: "EAGAIN",
        code: 35,
    },
    Errno {
        name: "EINPROGRESS",
        code: 36,
    },
    Errno {
        name: "EALREADY",
        code: 37,
    },
    Errno {
        name: "ENOTSOCK",
        code: 38,
    },
    Errno {
        name: "EDESTADDRREQ",
        code: 39,
    },
    Errno {
        name: "EMSGSIZE",
        code: 40,
    },
    Errno {
        name: "EPROTOTYPE",
        code: 41,
    },
    Errno {
        name: "ENOPROTOOPT",
        code: 42,
    },
    Errno {
        name: "EPROTONOSUPPORT",
        code: 43,
    },
    Errno {
        name: "ESOCKTNOSUPPORT",
        code: 44,
    },
    Errno {
        name: "ENOTSUP",
        code: 45,
    },
    Errno {
        name: "EPFNOSUPPORT",
        code: 46,
    },
    Errno {
        name: "EAFNOSUPPORT",
        code: 47,
    },
    Errno {
        name: "EADDRINUSE",
        code: 48,
    },
    Errno {
        name: "EADDRNOTAVAIL",
        code: 49,
    },
    Errno {
        name: "ENETDOWN",
        code: 50,
    },
    Errno {
        name: "ENETUNREACH",
        code: 51,
    },
    Errno {
        name: "ENETRESET",
        code: 52,
    },
    Errno {
        name: "ECONNABORTED",
        code: 53,
    },
    Errno {
        name: "ECONNRESET",
        code: 54,
    },
    Errno {
        name: "ENOBUFS",
        code: 55,
    },
    Errno {
        name: "EISCONN",
        code: 56,
    },
    Errno {
        name: "ENOTCONN",
        code: 57,
    },
    Errno {
        name: "ESHUTDOWN",
        code: 58,
    },
    Errno {
        name: "ETOOMANYREFS",
        code: 59,
    },
    Errno {
        name: "ETIMEDOUT",
        code: 60,
    },
    Errno {
        name: "ECONNREFUSED",
        code: 61,
    },
    Errno {
        name: "ELOOP",
        code: 62,
    },
    Errno {
        name: "ENAMETOOLONG",
        code: 63,
    },
    Errno {
        name: "EHOSTDOWN",
        code: 64,
    },
    Errno {
        name: "EHOSTUNREACH",
        code: 65,
    },
    Errno {
        name: "ENOTEMPTY",
        code: 66,
    },
    Errno {
        name: "EPROCLIM",
        code: 67,
    },
    Errno {
        name: "EUSERS",
        code: 68,
    },
    Errno {
        name: "EDQUOT",
        code: 69,
    },
    Errno {
        name: "ESTALE",
        code: 70,
    },
    Errno {
        name: "EREMOTE",
        code: 71,
    },
    Errno {
        name: "EBADRPC",
        code: 72,
    },
    Errno {
        name: "ERPCMISMATCH",
        code: 73,
    },
    Errno {
        name: "EPROGUNAVAIL",
        code: 74,
    },
    Errno {
        name: "EPROGMISMATCH",
        code: 75,
    },
    Errno {
        name: "EPROCUNAVAIL",
        code: 76,
    },
    Errno {
        name: "ENOLCK",
        code: 77,
    },
    Errno {
        name: "ENOSYS",
        code: 78,
    },
    Errno {
        name: "EFTYPE",
        code: 79,
    },
    Errno {
        name: "EAUTH",
        code: 80,
    },
    Errno {
        name: "ENEEDAUTH",
        code: 81,
    },
    Errno {
        name: "EPWROFF",
        code: 82,
    },
    Errno {
        name: "EDEVERR",
        code: 83,
    },
    Errno {
        name: "EOVERFLOW",
        code: 84,
    },
    Errno {
        name: "EBADEXEC",
        code: 85,
    },
    Errno {
        name: "EBADARCH",
        code: 86,
    },
    Errno {
        name: "ESHLIBVERS",
        code: 87,
    },
    Errno {
        name: "EBADMACHO",
        code: 88,
    },
    Errno {
        name: "ECANCELED",
        code: 89,
    },
    Errno {
        name: "EIDRM",
        code: 90,
    },
    Errno {
        name: "ENOMSG",
        code: 91,
    },
    Errno {
        name: "EILSEQ",
        code: 92,
    },
    Errno {
        name: "ENOATTR",
        code: 93,
    },
    Errno {
        name: "EBADMSG",
        code: 94,
    },
    Errno {
        name: "EMULTIHOP",
        code: 95,
    },
    Errno {
        name: "ENODATA",
        code: 96,
    },
    Errno {
        name: "ENOLINK",
        code: 97,
    },
    Errno {
        name: "ENOSR",
        code: 98,
    },
    Errno {
        name: "ENOSTR",
        code: 99,
    },
    Errno {
        name: "EPROTO",
        code: 100,
    },
    Errno {
        name: "ETIME",
        code: 101,
    },
    Errno {
        name: "EOPNOTSUPP",
        code: 102,
    },
    Errno {
        name: "ENOPOLICY",
        code: 103,
    },
    Errno {
        name: "ENOTRECOVERABLE",
        code: 104,
    },
    Errno {
        name: "EOWNERDEAD",
        code: 105,
    },
    Errno {
        name: "EQFULL",
        code: 106,
    },
];

unsafe extern "C" {
    fn strerror(errnum: c_int) -> *mut c_char;
    fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char;
}
