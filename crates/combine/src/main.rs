use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    match Config::parse(env::args_os().skip(1)) {
        Ok(config) => match run(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("combine: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("combine: {error}");
            eprintln!("Usage: combine file1 OP file2");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
struct Config {
    file1: OsString,
    op: Operation,
    file2: OsString,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    And,
    Not,
    Or,
    Xor,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args.into_iter().collect::<Vec<_>>();
        if args.len() >= 4 && args.get(3).is_some_and(|arg| arg == "_") {
            args.remove(3);
        }
        if args.len() != 3 {
            return Err("expected file1 OP file2".to_string());
        }
        let op = match args[1].to_string_lossy().to_lowercase().as_str() {
            "and" => Operation::And,
            "not" => Operation::Not,
            "or" => Operation::Or,
            "xor" => Operation::Xor,
            op => return Err(format!("unknown operation, {op}")),
        };

        Ok(Self {
            file1: args.remove(0),
            op,
            file2: args.remove(1),
        })
    }
}

fn run(config: &Config) -> io::Result<()> {
    if config.file1 == "-" && config.file2 == "-" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "only one input may be '-'",
        ));
    }

    let file1 = read_lines(&config.file1)?;
    let file2 = read_lines(&config.file2)?;
    let output = match config.op {
        Operation::And => compare_and(&file1, &file2),
        Operation::Not => compare_not(&file1, &file2),
        Operation::Or => compare_or(&file1, &file2),
        Operation::Xor => compare_xor(&file1, &file2),
    };

    let mut stdout = io::stdout().lock();
    for line in output {
        writeln!(stdout, "{line}")?;
    }
    Ok(())
}

fn read_lines(path: &OsString) -> io::Result<Vec<String>> {
    let mut contents = String::new();
    if path == "-" {
        io::stdin().lock().read_to_string(&mut contents)?;
    } else {
        contents = fs::read_to_string(Path::new(path))?;
    }
    Ok(contents.lines().map(ToOwned::to_owned).collect())
}

fn compare_or(file1: &[String], file2: &[String]) -> Vec<String> {
    file1.iter().chain(file2).cloned().collect()
}

fn compare_and(file1: &[String], file2: &[String]) -> Vec<String> {
    let seen = counts(file2);
    file1
        .iter()
        .filter(|line| seen.contains_key(*line))
        .cloned()
        .collect()
}

fn compare_not(file1: &[String], file2: &[String]) -> Vec<String> {
    let seen = counts(file2);
    file1
        .iter()
        .filter(|line| !seen.contains_key(*line))
        .cloned()
        .collect()
}

fn compare_xor(file1: &[String], file2: &[String]) -> Vec<String> {
    let mut seen2 = file2
        .iter()
        .map(|line| (line.clone(), true))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();

    for line in file1 {
        if let Some(value) = seen2.get_mut(line) {
            *value = false;
        } else {
            output.push(line.clone());
        }
    }

    for line in file2 {
        if seen2.get(line).copied().unwrap_or(false) {
            output.push(line.clone());
        }
    }

    output
}

fn counts(lines: &[String]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for line in lines {
        *counts.entry(line.clone()).or_insert(0) += 1;
    }
    counts
}
