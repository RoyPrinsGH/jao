use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::fixture::{FileWriteStr, PathChild};
use predicates::str::contains;

#[test]
fn no_args_prints_help() {
    command()
        .assert()
        .success()
        .stdout(contains("USAGE:"))
        .stdout(contains("jao --list"));
}

#[test]
fn version_flag_prints_version() {
    command()
        .arg("--version")
        .assert()
        .failure()
        .code(2)
        .stdout(contains("jao "));
}

#[test]
fn require_fingerprint_without_ci_is_a_parse_error() {
    const CI_ONLY_SCRIPT: &str = "ci-discovery-only";

    let workspace = TempDir::new().unwrap();
    workspace
        .child(format!("scripts/{CI_ONLY_SCRIPT}.{}", script_extension()))
        .write_str(script_contents())
        .unwrap();

    command()
        .current_dir(workspace.path())
        .args(["--require-fingerprint", "deadbeef", CI_ONLY_SCRIPT])
        .assert()
        .failure()
        .code(1)
        .stderr("error: invalid --require-fingerprint value (expected 64 hex chars): deadbeef\n");
}

#[test]
fn invalid_shell_is_a_parse_error() {
    command()
        .args(["--completions", "fish"])
        .assert()
        .failure()
        .code(2)
        .stderr("error: Unknown shell type passed\n");
}

fn script_extension() -> &'static str {
    #[cfg(windows)]
    return "bat";
    #[cfg(unix)]
    return "sh";
}

fn script_contents() -> &'static str {
    #[cfg(windows)]
    return "@echo off\r\n";
    #[cfg(unix)]
    return "#!/bin/sh\n";
}

fn command() -> Command {
    Command::cargo_bin("jao").unwrap()
}
