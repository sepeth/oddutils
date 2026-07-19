use std::path::PathBuf;
use std::process::Command;

fn parallel_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_parallel"))
}

#[test]
fn runs_command_for_each_argument() {
    let output = Command::new(parallel_bin())
        .arg("-j")
        .arg("1")
        .arg("printf")
        .arg("%s\\n")
        .arg("--")
        .arg("a")
        .arg("b")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"a\nb\n");
}

#[test]
fn replacement_mode_substitutes_braces() {
    let output = Command::new(parallel_bin())
        .arg("-j")
        .arg("1")
        .arg("-i")
        .arg("printf")
        .arg("x{}\\n")
        .arg("--")
        .arg("a")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"xa\n");
}

#[test]
fn runs_raw_commands_after_separator() {
    let output = Command::new(parallel_bin())
        .arg("-j")
        .arg("1")
        .arg("--")
        .arg("printf one")
        .arg("printf two")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"onetwo");
}

#[test]
fn returns_ored_exit_statuses() {
    let output = Command::new(parallel_bin())
        .arg("-j")
        .arg("1")
        .arg("--")
        .arg("exit 1")
        .arg("exit 2")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn groups_arguments_with_n() {
    let output = Command::new(parallel_bin())
        .arg("-j")
        .arg("1")
        .arg("-n")
        .arg("2")
        .arg("printf")
        .arg("%s-%s\\n")
        .arg("--")
        .arg("a")
        .arg("b")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"a-b\n");
}
