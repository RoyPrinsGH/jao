//! Completion support for both shell integration and the internal completion protocol.
//!
//! `jao --completions <shell>` prints a static shell script that wires Bash or Zsh
//! into the hidden `jao __complete` subcommand. That internal command returns one
//! completion candidate per line, based on the current working directory and the
//! partially typed command words supplied by the shell.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::Path;

use crate::{JaoError, JaoResult, script_discovery};

const STATIC_OPTIONS: &[&str] = &[
    "--help",
    "--version",
    "--list",
    "--ci",
    "--fingerprint",
    "--require-fingerprint",
    "--completions",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Shell {
    Bash,
    Zsh,
}

impl TryFrom<&OsStr> for Shell {
    type Error = JaoError;

    fn try_from(shell_str: &OsStr) -> Result<Self, Self::Error> {
        if shell_str == OsStr::new("bash") {
            return Ok(Shell::Bash);
        }

        if shell_str == OsStr::new("zsh") {
            return Ok(Shell::Zsh);
        }

        return Err(JaoError::InvalidArguments("Unknown shell type passed"));
    }
}

/// Parsed arguments for the hidden `jao __complete` protocol.
///
/// `words` contains the command words after `jao`, and `current_index` points to
/// the word the shell is trying to complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompletionRequest<'a> {
    pub(crate) index_to_complete: usize,
    pub(crate) given_arguments: Vec<&'a OsStr>,
}

/// Prints the shell integration script for the requested shell.
pub(crate) fn print_shell_completion(shell: Shell) -> JaoResult<()> {
    let script = match shell {
        Shell::Bash => include_str!("completion_scripts/jao.bash"),
        Shell::Zsh => include_str!("completion_scripts/jao.zsh"),
    };

    let mut out = io::stdout().lock();
    out.write_all(script.as_bytes())?;
    Ok(())
}

/// Executes the hidden completion protocol and writes candidates to stdout.
///
/// Each candidate is emitted on its own line so the shell integration scripts can
/// consume the output without extra parsing rules.
pub(crate) fn complete(root: impl AsRef<Path>, request: CompletionRequest<'_>) -> JaoResult<()> {
    let completions = complete_request(root, &request)?;
    let mut out = io::stdout().lock();

    for completion in completions {
        writeln!(out, "{completion}")?;
    }

    Ok(())
}

fn complete_request(root: impl AsRef<Path>, args: &CompletionRequest<'_>) -> JaoResult<Vec<String>> {
    match completion_context(&args.given_arguments, args.index_to_complete) {
        CompletionContext::Options { prefix } => Ok(complete_options(&prefix)),
        CompletionContext::Shells { prefix } => Ok(complete_shells(&prefix)),
        CompletionContext::Scripts { prior_parts, current_prefix } => complete_script_parts(root, &prior_parts, &current_prefix),
        CompletionContext::None => Ok(Vec::new()),
    }
}

#[rustfmt::skip]
fn complete_script_parts(root: impl AsRef<Path>, prior_parts: &[String], current_prefix: &str) -> JaoResult<Vec<String>> {
    let mut completions = BTreeSet::new();
    let next_index = prior_parts.len();

    let _ = script_discovery::for_each_discovered_script(root, |script| {
        if script.command_parts.len() <= next_index
            || !script_discovery::command_parts_match(
                script.command_parts.iter().copied(),
                prior_parts.iter().map(String::as_str)
            )
        {
            return Ok(std::ops::ControlFlow::<()>::Continue(()));
        }

        let candidate = script.command_parts[next_index];
        if script_discovery::command_part_has_prefix(candidate, current_prefix) {
            completions.insert(candidate.to_string());
        }

        Ok(std::ops::ControlFlow::<()>::Continue(()))
    })?;

    Ok(completions.into_iter().collect())
}

fn complete_options(prefix: &str) -> Vec<String> {
    STATIC_OPTIONS
        .iter()
        .copied()
        .filter(|option| option.starts_with(prefix))
        .map(str::to_string)
        .collect()
}

fn complete_shells(prefix: &str) -> Vec<String> {
    ["bash", "zsh"]
        .into_iter()
        .filter(|shell| shell.starts_with(prefix))
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompletionContext {
    Options { prefix: String },
    Shells { prefix: String },
    Scripts { prior_parts: Vec<String>, current_prefix: String },
    None,
}

fn completion_context(words: &[&OsStr], current_index: usize) -> CompletionContext {
    let mut mode = ParseMode::TopLevel;
    let mut prior_script_parts = Vec::new();
    let mut expects_require_fingerprint_value = false;
    let mut expects_shell_value = false;

    for word in words
        .iter()
        .take(current_index)
    {
        let str_word = word.to_string_lossy();

        if expects_require_fingerprint_value {
            expects_require_fingerprint_value = false;
            continue;
        }

        if expects_shell_value {
            // `--completions` consumes exactly one value, so any later position is
            // outside the supported completion surface.
            return CompletionContext::None;
        }

        match mode {
            ParseMode::TopLevel => match str_word.as_ref() {
                "--ci" => {}
                "--fingerprint" => mode = ParseMode::ScriptParts,
                "--require-fingerprint" => expects_require_fingerprint_value = true,
                "--completions" => expects_shell_value = true,
                "--list" | "--help" | "--version" => return CompletionContext::None,
                _ if str_word.starts_with('-') => {}
                _ => {
                    mode = ParseMode::ScriptParts;
                    prior_script_parts.push(str_word.into_owned());
                }
            },
            ParseMode::ScriptParts => prior_script_parts.push(str_word.into_owned()),
        }
    }

    if expects_require_fingerprint_value {
        return CompletionContext::None;
    }

    let current = words
        .get(current_index)
        .map_or_else(String::new, |word| {
            word.to_string_lossy()
                .into_owned()
        });

    if expects_shell_value {
        return CompletionContext::Shells { prefix: current };
    }

    match mode {
        ParseMode::TopLevel if current.starts_with('-') => CompletionContext::Options { prefix: current },
        // Once a positional script part appears, all following words are treated
        // as command parts rather than top-level options.
        ParseMode::TopLevel => CompletionContext::Scripts {
            prior_parts: Vec::new(),
            current_prefix: current,
        },
        ParseMode::ScriptParts => CompletionContext::Scripts {
            prior_parts: prior_script_parts,
            current_prefix: current,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    TopLevel,
    ScriptParts,
}
