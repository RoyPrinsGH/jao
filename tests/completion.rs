use std::path::{Path, PathBuf};

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};
use predicates::str::contains;

#[test]
fn dynamic_completion_follows_jaofolder_and_prefixes() {
    let workspace = TempDir::new().unwrap();
    workspace
        .child(Path::new("myapp").join(".jaofolder"))
        .touch()
        .unwrap();
    workspace
        .child(
            Path::new("myapp")
                .join("frontend")
                .join(".jaofolder"),
        )
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
        .child(script_rel_path(&["myapp", "frontend", "scripts"], "dev"))
        .write_str(script_contents())
        .unwrap();
    workspace
        .child(script_rel_path(&["myapp", "backend", "scripts"], "build"))
        .write_str(script_contents())
        .unwrap();

    assert_eq!(completion_lines(workspace.path(), 0, &["m"]), vec![String::from("myapp")]);
    assert_eq!(
        completion_lines(workspace.path(), 1, &["myapp", ""]),
        vec![String::from("backend"), String::from("frontend")]
    );
    assert_eq!(
        completion_lines(workspace.path(), 2, &["myapp", "backend", "b"]),
        vec![String::from("build")]
    );
    assert_eq!(
        completion_lines(
            workspace
                .child("myapp")
                .path(),
            0,
            &[""]
        ),
        vec![String::from("backend"), String::from("frontend")]
    );
}

#[test]
fn dynamic_completion_respects_recursive_jaoignore() {
    let workspace = TempDir::new().unwrap();
    workspace
        .child(".jaoignore")
        .write_str("scratch/\n")
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
        .write_str(format!("seed.dev.{}\n", script_extension()).as_str())
        .unwrap();
    workspace
        .child(script_rel_path(&["myapp", "backend", "scripts"], "migrate.dev"))
        .write_str(script_contents())
        .unwrap();
    workspace
        .child(script_rel_path(&["myapp", "backend", "scripts"], "seed.dev"))
        .write_str(script_contents())
        .unwrap();
    workspace
        .child(script_rel_path(&["scratch", "scripts"], "one-off-fix"))
        .write_str(script_contents())
        .unwrap();

    let root_completions = completion_lines(workspace.path(), 0, &[""]);
    assert_eq!(root_completions, vec![String::from("myapp")]);

    let backend_completions = completion_lines(workspace.path(), 2, &["myapp", "backend", ""]);
    assert_eq!(backend_completions, vec![String::from("migrate")]);
}

#[test]
fn dynamic_completion_supports_options_and_fingerprint_context() {
    let workspace = TempDir::new().unwrap();
    let script_name = "completion-fingerprint-only";
    workspace
        .child(script_rel_path(&["scripts"], script_name))
        .write_str(script_contents())
        .unwrap();

    assert_eq!(
        completion_lines(workspace.path(), 0, &["--c"]),
        vec![String::from("--ci"), String::from("--completions")]
    );
    assert_eq!(completion_lines(workspace.path(), 1, &["--completions", "b"]), vec![String::from("bash")]);
    assert_eq!(
        completion_lines(workspace.path(), 1, &["--fingerprint", ""]),
        vec![script_name.to_string()]
    );
    assert!(completion_lines(workspace.path(), 2, &["--ci", "--require-fingerprint", "deadbeef"],).is_empty());
    assert_eq!(
        completion_lines(workspace.path(), 3, &["--ci", "--require-fingerprint", "deadbeef", ""],),
        vec![script_name.to_string()]
    );
}

#[test]
fn completion_scripts_invoke_internal_protocol() {
    let workspace = TempDir::new().unwrap();

    command_for(workspace.path())
        .args(["--completions", "bash"])
        .assert()
        .success()
        .stdout(contains("jao __complete --index"));

    command_for(workspace.path())
        .args(["--completions", "zsh"])
        .assert()
        .success()
        .stdout(contains("jao __complete --index"));
}

fn completion_lines(cwd: &Path, current_index: usize, words: &[&str]) -> Vec<String> {
    let mut command = command_for(cwd);
    command
        .arg("__complete")
        .arg("--index")
        .arg(current_index.to_string())
        .arg("--")
        .args(words);
    let output = command
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn command_for(cwd: &Path) -> Command {
    let mut command = Command::cargo_bin("jao").unwrap();
    command.current_dir(cwd);
    command
}

fn script_rel_path(directories: &[&str], stem: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for directory in directories {
        path.push(directory);
    }
    path.push(format!("{stem}.{}", script_extension()));
    path
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
