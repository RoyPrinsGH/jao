use std::path::{Path, PathBuf};

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};

#[test]
fn jaofolder_commands_change_with_current_directory() {
    let workspace = TempDir::new().unwrap();
    workspace
        .child(Path::new("myapp").join(".jaofolder"))
        .touch()
        .unwrap();
    workspace
        .child(
            Path::new("myapp")
                .join("backend")
                .join(".jaofolder"),
        )
        .touch()
        .unwrap();
    workspace
        .child(script_rel_path(&["myapp", "backend", "scripts"], "build"))
        .write_str(script_contents())
        .unwrap();

    let root_output = list_output(workspace.path());
    let myapp_output = list_output(
        workspace
            .child(Path::new("myapp"))
            .path(),
    );
    let backend_output = list_output(
        workspace
            .child(Path::new("myapp").join("backend"))
            .path(),
    );

    assert_list_line_matches(
        root_output,
        "myapp backend build",
        workspace
            .child(script_rel_path(&["myapp", "backend", "scripts"], "build"))
            .path(),
    );
    assert_list_line_matches(
        myapp_output,
        "backend build",
        workspace
            .child(script_rel_path(&["myapp", "backend", "scripts"], "build"))
            .path(),
    );
    assert_list_line_matches(
        backend_output,
        "build",
        workspace
            .child(script_rel_path(&["myapp", "backend", "scripts"], "build"))
            .path(),
    );

    let root_fingerprint = fingerprint_output(workspace.path(), &["myapp", "backend", "build"]);
    let myapp_fingerprint = fingerprint_output(
        workspace
            .child(Path::new("myapp"))
            .path(),
        &["backend", "build"],
    );
    let backend_fingerprint = fingerprint_output(
        workspace
            .child(Path::new("myapp").join("backend"))
            .path(),
        &["build"],
    );

    assert_eq!(root_fingerprint, myapp_fingerprint);
    assert_eq!(myapp_fingerprint, backend_fingerprint);
}

#[test]
fn recursive_jaoignore_hides_nested_matches() {
    let workspace = TempDir::new().unwrap();
    workspace
        .child(".jaofolder")
        .touch()
        .unwrap();
    workspace
        .child(".jaoignore")
        .write_str(&format!("ignored/\nskip-me.{}\n", script_extension()))
        .unwrap();
    workspace
        .child(Path::new("myapp").join(".jaofolder"))
        .touch()
        .unwrap();
    workspace
        .child(
            Path::new("myapp")
                .join("backend")
                .join(".jaofolder"),
        )
        .touch()
        .unwrap();
    workspace
        .child(
            Path::new("myapp")
                .join("backend")
                .join(".jaoignore"),
        )
        .write_str(format!("build.{}\n", script_extension()).as_str())
        .unwrap();
    workspace
        .child(script_rel_path(&["myapp", "backend", "scripts"], "build"))
        .write_str(script_contents())
        .unwrap();
    workspace
        .child(script_rel_path(&["myapp", "backend", "scripts"], "keep"))
        .write_str(script_contents())
        .unwrap();
    workspace
        .child(script_rel_path(&["ignored", "scripts"], "nope"))
        .write_str(script_contents())
        .unwrap();
    workspace
        .child(script_rel_path(&[], "skip-me"))
        .write_str(script_contents())
        .unwrap();

    let output = list_output(workspace.path());
    assert_list_line_matches(
        output,
        "myapp backend keep",
        workspace
            .child(script_rel_path(&["myapp", "backend", "scripts"], "keep"))
            .path(),
    );

    fingerprint_output(workspace.path(), &["myapp", "backend", "keep"]);

    command_for(workspace.path())
        .args(["--fingerprint", "myapp", "backend", "build"])
        .assert()
        .failure()
        .stderr("error: script myapp backend build not found\n");
}

fn list_output(cwd: &std::path::Path) -> String {
    let output = command_for(cwd)
        .args(["--ci", "--list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn fingerprint_output(cwd: &std::path::Path, parts: &[&str]) -> String {
    let mut command = command_for(cwd);
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

fn assert_list_line_matches(output: String, command: &str, path: &std::path::Path) {
    let trimmed = output.trim_end_matches(['\n', '\r']);
    let (actual_prefix, actual_path) = trimmed
        .rsplit_once(" \t\t ")
        .expect("list output should contain a path column");

    #[cfg(feature = "trust-manifest")]
    assert_eq!(actual_prefix, format!("unknown \t {command}"));

    #[cfg(not(feature = "trust-manifest"))]
    assert_eq!(actual_prefix, command);

    let expected_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let actual_path = std::fs::canonicalize(actual_path).unwrap_or_else(|_| PathBuf::from(actual_path));

    assert_eq!(actual_path, expected_path);
}

fn script_rel_path(directories: &[&str], stem: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for directory in directories {
        path.push(directory);
    }
    path.push(format!("{stem}.{}", script_extension()));
    path
}

fn command_for(cwd: &std::path::Path) -> Command {
    let mut command = Command::cargo_bin("jao").unwrap();
    command.current_dir(cwd);
    command
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
