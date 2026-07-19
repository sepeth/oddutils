use std::collections::{BTreeMap, HashMap};
use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(success) => {
                if success {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(error) => {
                eprintln!("vidir: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("vidir: {error}");
            eprintln!("Usage: vidir [--verbose] [directory|file|-]...");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    verbose: bool,
    items: Vec<OsString>,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut verbose = false;
        let mut items = Vec::new();
        let mut parsing_options = true;

        for arg in args {
            if parsing_options {
                match arg.to_string_lossy().as_ref() {
                    "-v" | "--verbose" => {
                        verbose = true;
                        continue;
                    }
                    "--" => {
                        parsing_options = false;
                        continue;
                    }
                    text if text.starts_with('-') && text != "-" => {
                        return Err(format!("unknown option '{text}'"));
                    }
                    _ => {}
                }
            }
            parsing_options = false;
            items.push(arg);
        }

        if items.is_empty() {
            items.push(OsString::from("."));
        }

        Ok(Self { verbose, items })
    }
}

fn run(config: &Config) -> io::Result<bool> {
    let entries = collect_entries(&config.items)?;
    reject_control_chars(&entries)?;
    let temp = TempPath::new()?;
    write_edit_file(temp.path(), &entries)?;

    let editor = editor_command();
    let status = Command::new(&editor[0])
        .args(&editor[1..])
        .arg(temp.path())
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "{} exited nonzero, aborting",
            editor
                .iter()
                .map(|part| part.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        )));
    }

    apply_edit_file(config.verbose, temp.path(), &entries)
}

fn collect_entries(items: &[OsString]) -> io::Result<BTreeMap<usize, PathBuf>> {
    let mut paths = Vec::new();
    for item in items {
        if item == "-" {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                paths.push(PathBuf::from(line?));
            }
        } else {
            let path = PathBuf::from(item);
            if path.is_dir() {
                let mut children = fs::read_dir(&path)?
                    .map(|entry| entry.map(|entry| entry.path()))
                    .collect::<io::Result<Vec<_>>>()?;
                children.sort();
                paths.extend(children);
            } else {
                paths.push(path);
            }
        }
    }

    let mut entries = BTreeMap::new();
    for path in paths {
        if is_dot_entry(&path) {
            continue;
        }
        let number = entries.len() + 1;
        entries.insert(number, path);
    }
    Ok(entries)
}

fn reject_control_chars(entries: &BTreeMap<usize, PathBuf>) -> io::Result<()> {
    for path in entries.values() {
        if path.to_string_lossy().chars().any(char::is_control) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "control characters in filenames are not supported",
            ));
        }
    }
    Ok(())
}

fn is_dot_entry(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "." || name == "..")
}

fn write_edit_file(path: &Path, entries: &BTreeMap<usize, PathBuf>) -> io::Result<()> {
    let width = entries.len().max(1).to_string().len();
    let mut file = OpenOptions::new().write(true).open(path)?;
    for (number, item) in entries {
        writeln!(file, "{number:0width$}\t{}", item.display())?;
    }
    file.flush()
}

fn apply_edit_file(
    verbose: bool,
    path: &Path,
    original: &BTreeMap<usize, PathBuf>,
) -> io::Result<bool> {
    let mut edited = String::new();
    OpenOptions::new()
        .read(true)
        .open(path)?
        .read_to_string(&mut edited)?;

    let mut remaining = original.clone();
    let mut current_by_number = original.clone();
    let mut success = true;

    for line in edited.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let (number, new_path) = parse_line(line)?;
        let Some(src) = current_by_number.get(&number).cloned() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown item number {number}"),
            ));
        };

        remaining.remove(&number);
        if new_path == src || new_path.as_os_str().is_empty() {
            continue;
        }

        if !src.exists() && src.symlink_metadata().is_err() {
            eprintln!("vidir: {} does not exist", src.display());
            success = false;
            continue;
        }

        if new_path.exists() || new_path.symlink_metadata().is_ok() {
            let temporary = conflict_path(&new_path);
            if let Err(error) = fs::rename(&new_path, &temporary) {
                eprintln!(
                    "vidir: failed to rename {} to {}: {error}",
                    new_path.display(),
                    temporary.display()
                );
                success = false;
                continue;
            }
            if verbose {
                println!("'{}' -> '{}'", new_path.display(), temporary.display());
            }
            for value in current_by_number.values_mut() {
                if *value == new_path {
                    value.clone_from(&temporary);
                }
            }
        }

        if let Some(parent) = new_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        match fs::rename(&src, &new_path) {
            Ok(()) => {
                if verbose {
                    println!("'{}' => '{}'", src.display(), new_path.display());
                }
                update_children(&mut current_by_number, &src, &new_path);
            }
            Err(error) => {
                eprintln!(
                    "vidir: failed to rename {} to {}: {error}",
                    src.display(),
                    new_path.display()
                );
                success = false;
            }
        }
    }

    let mut remaining_paths = remaining.into_values().collect::<Vec<_>>();
    remaining_paths.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for item in remaining_paths {
        if remove_path(&item).is_err() {
            eprintln!("vidir: failed to remove {}", item.display());
            success = false;
        } else if verbose {
            println!("removed '{}'", item.display());
        }
    }

    Ok(success)
}

fn parse_line(line: &str) -> io::Result<(usize, PathBuf)> {
    let Some((number, name)) = line.split_once(char::is_whitespace) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unable to parse line \"{line}\", aborting"),
        ));
    };
    let number = number.parse::<usize>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unable to parse item number in \"{line}\": {error}"),
        )
    })?;
    Ok((number, PathBuf::from(name.trim_start())))
}

fn conflict_path(path: &Path) -> PathBuf {
    let mut suffix = 0_u32;
    loop {
        let candidate = if suffix == 0 {
            PathBuf::from(format!("{}~", path.display()))
        } else {
            PathBuf::from(format!("{}~{suffix}", path.display()))
        };
        if !candidate.exists() && candidate.symlink_metadata().is_err() {
            return candidate;
        }
        suffix += 1;
    }
}

fn update_children(paths: &mut BTreeMap<usize, PathBuf>, old: &Path, new: &Path) {
    let replacements = paths
        .iter()
        .filter_map(|(number, path)| {
            path.strip_prefix(old)
                .ok()
                .map(|suffix| (*number, new.join(suffix)))
        })
        .collect::<HashMap<_, _>>();
    for (number, path) in replacements {
        paths.insert(number, path);
    }
}

fn remove_path(path: &Path) -> io::Result<()> {
    if path.is_dir() && !path.symlink_metadata()?.file_type().is_symlink() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    }
}

fn editor_command() -> Vec<OsString> {
    if let Some(editor) = env::var_os("VISUAL").or_else(|| env::var_os("EDITOR")) {
        return editor
            .to_string_lossy()
            .split_whitespace()
            .map(OsString::from)
            .collect();
    }
    vec![OsString::from("vi")]
}

struct TempPath {
    path: PathBuf,
}

impl TempPath {
    fn new() -> io::Result<Self> {
        let dir = env::temp_dir();
        let pid = std::process::id();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        for attempt in 0..1000_u32 {
            let path = dir.join(format!("oddutils-vidir-{pid}-{stamp}-{attempt}"));
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
