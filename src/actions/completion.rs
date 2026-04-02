//! Completion support for both shell integration and the internal completion protocol.
//!
//! `jao --completions <shell>` prints a static shell script that wires Bash or Zsh
//! into the hidden `jao __complete` subcommand. That internal command returns one
//! completion candidate per line, based on the current working directory and the
//! partially typed command words supplied by the shell.

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::Path;

use crate::platform::osstr::starts_with;
use crate::script_discovery::{DiscoveryFlow, ScriptParts};
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

/// Supported shell script outputs for command completion integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Shell {
    /// Emit Bash completion script source.
    Bash,
    /// Emit Zsh completion script source.
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
///
/// This structure intentionally keeps values as [`OsStr`] references to avoid
/// lossy or unnecessary UTF-8 conversion during matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompletionRequest<'a> {
    /// Index of the argument currently being completed.
    pub(crate) index_to_complete: usize,
    /// Raw arguments after `jao __complete --index <N> --`.
    pub(crate) given_arguments: Vec<&'a OsStr>,
}

/// Prints the shell integration script for the requested shell.
///
/// This writes the static completion script to stdout.
///
/// The script installs shell-side glue that calls back into
/// `jao __complete` for dynamic command-part suggestions.
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
///
/// Completion sources include:
///
/// - static top-level options
/// - shell names for `--completions`
/// - dynamic script-part suggestions based on discovery
pub(crate) fn complete(root: impl AsRef<Path>, request: CompletionRequest<'_>) -> JaoResult<()> {
    let completions = match build_completion_context(&request.given_arguments, request.index_to_complete) {
        CompletionContext::Options { word_being_typed: prefix } => complete_options(prefix),
        CompletionContext::Shells { word_being_typed: prefix } => complete_shells(prefix),
        CompletionContext::Scripts {
            prior_parts,
            word_being_typed,
        } => complete_script_parts(root, prior_parts, word_being_typed),
        CompletionContext::None => Vec::new(),
    };

    let mut out = io::stdout().lock();

    for completion in completions {
        #[rustfmt::skip]
        writeln!(out, "{}", completion.display())?;
    }

    Ok(())
}

#[rustfmt::skip]
fn complete_script_parts(root: impl AsRef<Path>, prior_parts: ScriptParts, word_being_typed: &OsStr) -> Vec<OsString> {
    let mut suggested_completions = BTreeSet::new();

    _ = script_discovery::for_each_discovered_script(root, |script| {
        if let Some(candidate) = script.parts.try_get_next_part_after(&prior_parts)
            && starts_with(candidate, word_being_typed)
        {
            suggested_completions.insert(candidate.to_os_string());
        }

        Ok(DiscoveryFlow::ContinueSearching)
    });

    suggested_completions
        .into_iter()
        .collect()
}

fn complete_options(prefix: &OsStr) -> Vec<OsString> {
    STATIC_OPTIONS
        .iter()
        .copied()
        .map(OsStr::new)
        .filter(|option| starts_with(option, prefix))
        .map(OsStr::to_os_string)
        .collect()
}

fn complete_shells(prefix: &OsStr) -> Vec<OsString> {
    ["bash", "zsh"]
        .into_iter()
        .map(OsStr::new)
        .filter(|shell| starts_with(shell, prefix))
        .map(OsStr::to_os_string)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompletionContext<'a> {
    Options { word_being_typed: &'a OsStr },
    Shells { word_being_typed: &'a OsStr },
    Scripts { word_being_typed: &'a OsStr, prior_parts: ScriptParts<'a> },
    None,
}

fn build_completion_context<'a>(words: &[&'a OsStr], current_index: usize) -> CompletionContext<'a> {
    let mut mode = ParseMode::TopLevel;
    let mut prior_parts = ScriptParts::new();
    let mut expects_require_fingerprint_value = false;
    let mut expects_shell_value = false;

    for word in words
        .iter()
        .take(current_index)
    {
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
            ParseMode::TopLevel => {
                if *word == OsStr::new("--ci") {
                    //
                } else if *word == OsStr::new("--fingerprint") {
                    mode = ParseMode::ScriptParts;
                } else if *word == OsStr::new("--require-fingerprint") {
                    expects_require_fingerprint_value = true;
                } else if *word == OsStr::new("--completions") {
                    expects_shell_value = true;
                } else if *word == OsStr::new("--list") || *word == OsStr::new("--help") || *word == OsStr::new("--version") {
                    return CompletionContext::None;
                } else if starts_with(word, OsStr::new("-")) {
                    //
                } else {
                    mode = ParseMode::ScriptParts;
                    prior_parts.push(*word);
                }
            }
            ParseMode::ScriptParts => prior_parts.push(*word),
        }
    }

    if expects_require_fingerprint_value {
        return CompletionContext::None;
    }

    let word_being_typed = words
        .get(current_index)
        .copied()
        .unwrap_or_else(|| OsStr::new(""));

    if expects_shell_value {
        return CompletionContext::Shells { word_being_typed };
    }

    match mode {
        ParseMode::TopLevel if starts_with(word_being_typed, OsStr::new("-")) => CompletionContext::Options { word_being_typed },
        ParseMode::TopLevel | ParseMode::ScriptParts => CompletionContext::Scripts {
            prior_parts,
            word_being_typed,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    TopLevel,
    ScriptParts,
}
