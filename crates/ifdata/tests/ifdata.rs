use std::path::PathBuf;
use std::process::Command;

fn ifdata_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ifdata"))
}

#[test]
fn reports_loopback_exists() {
    let output = Command::new(ifdata_bin())
        .arg("-pe")
        .arg("lo0")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"yes\n");
}

#[test]
fn existence_mode_fails_for_missing_interface() {
    let output = Command::new(ifdata_bin())
        .arg("-e")
        .arg("oddutils_no_such_if")
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn prints_loopback_address_and_mtu() {
    let output = Command::new(ifdata_bin())
        .arg("-pa")
        .arg("-pm")
        .arg("lo0")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("127.0.0.1\n"));
    assert!(stdout.lines().any(|line| line.parse::<u32>().is_ok()));
}

#[test]
fn print_exists_says_no_for_missing_interface() {
    let output = Command::new(ifdata_bin())
        .arg("-pe")
        .arg("oddutils_no_such_if")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"no\n");
}
