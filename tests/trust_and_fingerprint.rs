use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};

#[test]
fn fingerprint_is_stable_across_aliases_and_changes_with_canonical_path() {
    let workspace = TempDir::new().unwrap();
    workspace
        .child("myapp/.jaofolder")
        .touch()
        .unwrap();
    workspace
        .child("myapp/backend/.jaofolder")
        .touch()
        .unwrap();
    workspace
        .child(format!("myapp/backend/scripts/build.{}", script_extension()))
        .write_str(&script_contents("echo fingerprint"))
        .unwrap();
    workspace
        .child(format!("other/scripts/build.{}", script_extension()))
        .write_str(&script_contents("echo fingerprint"))
        .unwrap();

    let root_fp = fingerprint_output(workspace.path(), None, &["myapp", "backend", "build"]);
    let myapp_fp = fingerprint_output(
        workspace
            .child("myapp")
            .path(),
        None,
        &["backend", "build"],
    );
    let backend_fp = fingerprint_output(
        workspace
            .child("myapp/backend")
            .path(),
        None,
        &["build"],
    );
    let other_fp = fingerprint_output(workspace.path(), None, &["build"]);

    assert_eq!(root_fp, myapp_fp);
    assert_eq!(myapp_fp, backend_fp);
    assert_ne!(root_fp, other_fp);
}

#[test]
fn ci_run_requires_matching_fingerprint() {
    let workspace = TempDir::new().unwrap();
    let script_name = "ci-fingerprint-only";
    workspace
        .child(format!("scripts/{script_name}.{}", script_extension()))
        .write_str(&script_contents("echo ci-run"))
        .unwrap();

    let fingerprint = fingerprint_output(workspace.path(), None, &[script_name]);

    let output = command_for(workspace.path(), None)
        .args(["--ci", "--require-fingerprint", fingerprint.trim(), script_name])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output)
            .unwrap()
            .trim_end(),
        "ci-run"
    );

    command_for(workspace.path(), None)
        .args([
            "--ci",
            "--require-fingerprint",
            "0000000000000000000000000000000000000000000000000000000000000000",
            script_name,
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("fingerprint mismatch"));
}

#[test]
fn noninteractive_unknown_trust_fails() {
    let workspace = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let script_name = "unknown-trust-only";
    workspace
        .child(format!("scripts/{script_name}.{}", script_extension()))
        .write_str(&script_contents("echo trust"))
        .unwrap();

    command_for(workspace.path(), Some(home.path()))
        .arg(script_name)
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown script trust requires interactive confirmation"));
}

#[test]
fn trusted_manifest_allows_run_and_reports_modified_after_change() {
    let workspace = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let script_name = "manifest-trust-only";
    let script = workspace.child(format!("scripts/{script_name}.{}", script_extension()));
    script
        .write_str(&script_contents("echo trusted"))
        .unwrap();

    let fingerprint = fingerprint_output(workspace.path(), Some(home.path()), &[script_name]);
    write_trust_manifest(home.path(), script.path(), fingerprint.trim());

    let output = command_for(workspace.path(), Some(home.path()))
        .arg(script_name)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output)
            .unwrap()
            .trim_end(),
        "trusted"
    );

    let trusted_list = list_output(workspace.path(), Some(home.path()));
    assert!(trusted_list.contains("trusted"));
    assert!(trusted_list.contains(script_name));
    assert!(
        trusted_list.contains(
            script
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        )
    );

    script
        .write_str(&script_contents("echo modified"))
        .unwrap();

    let modified_list = list_output(workspace.path(), Some(home.path()));
    assert!(modified_list.contains("modified"));
    assert!(
        modified_list.contains(
            script
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        )
    );

    command_for(workspace.path(), Some(home.path()))
        .arg(script_name)
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown script trust requires interactive confirmation"));
}

fn list_output(cwd: &std::path::Path, home: Option<&std::path::Path>) -> String {
    let output = command_for(cwd, home)
        .arg("--list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn fingerprint_output(cwd: &std::path::Path, home: Option<&std::path::Path>, parts: &[&str]) -> String {
    let mut command = command_for(cwd, home);
    command
        .arg("--fingerprint")
        .args(parts);
    let output = command
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn command_for(cwd: &std::path::Path, home: Option<&std::path::Path>) -> Command {
    let mut command = Command::cargo_bin("jao").unwrap();
    command.current_dir(cwd);
    if let Some(home) = home {
        command.env("HOME", home);
        command.env("USERPROFILE", home);
    }
    command
}

fn write_trust_manifest(home: &std::path::Path, script_path: &std::path::Path, fingerprint: &str) {
    let canonical_path = std::fs::canonicalize(script_path).unwrap();
    let jao_dir = home.join(".jao");
    std::fs::create_dir_all(&jao_dir).unwrap();
    std::fs::write(
        jao_dir.join("jaotrusted.toml"),
        format!("'{}' = {{ fingerprint = '{}' }}\n", canonical_path.display(), fingerprint),
    )
    .unwrap();
}

fn script_extension() -> &'static str {
    #[cfg(windows)]
    return "bat";
    #[cfg(unix)]
    return "sh";
}

fn script_contents(body: &str) -> String {
    #[cfg(windows)]
    {
        format!("@echo off\r\n{body}\r\n")
    }
    #[cfg(unix)]
    {
        format!("#!/bin/sh\n{body}\n")
    }
}
