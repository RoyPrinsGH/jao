//! `jao` is a small CLI for discovering and running workspace scripts.
//!
//! It is meant for repositories that already have shell or batch scripts and
//! want a thin command layer on top, without adopting a bigger task runner.
//!
//! # What it does
//!
//! - Recursively discovers `.sh` scripts on Unix-like systems and `.bat`
//!   scripts on Windows
//! - Resolves a command like `jao db reset local` to a script selected by
//!   `.jaofolder` path markers plus the script file stem
//! - Respects `.gitignore` during discovery
//! - Honors recursive `.jaoignore` files to skip ignored scripts and directories
//! - Runs the script from the script's own directory
//! - Supports SHA-256 fingerprint checks for CI-safe execution
//! - Optionally keeps a local trust manifest for interactive runs
//! - Prints shell completion scripts for Bash and Zsh
//!
//! # Practical examples
//!
//! ```text
//! # basic use
//! jao --list
//! jao check
//! jao test integration
//! jao db reset local
//! ```
//!
//! ```text
//! # .jaofolder in a multi-project repo
//! jao apps frontend dev
//! jao apps backend build
//! ```
//!
//! ```text
//! # fingerprinting in CI
//! jao --fingerprint db reset local
//! jao --ci --require-fingerprint <FINGERPRINT> db reset local
//! ```
//!
//! ```text
//! # shell completion
//! source <(jao --completions bash)
//! jao m<TAB>    -> myapp
//! jao myapp <TAB> -> backend frontend
//! ```
//!
//! # Completions
//!
//! `jao` supports two completion surfaces:
//!
//! - static completion script output via `jao --completions <bash|zsh>`
//! - dynamic candidate generation via the hidden `jao __complete` protocol
//!
//! The generated shell scripts call:
//!
//! ```text
//! jao __complete --index <CURRENT_WORD_INDEX> -- <WORDS_AFTER_JAO...>
//! ```
//!
//! The internal protocol then returns one completion candidate per line.
//! Suggestions are context-aware and include:
//!
//! - top-level options (`--help`, `--list`, etc.)
//! - shell names after `--completions`
//! - script command parts discovered from the current working directory
//!
//! Dynamic script-part completion respects the same discovery rules as command
//! execution, including `.jaofolder` path markers and recursive `.jaoignore`.
//!
//! # `.jaofolder` and `.jaoignore`
//!
//! `.jaofolder` files mark directories that should appear in the command name.
//! If `apps/`, `frontend/`, and `backend/` contain `.jaofolder`, then scripts
//! with the same stem can stay distinct without forcing long commands everywhere.
//!
//! `.jaoignore` files work recursively like `.gitignore` and can hide
//! throwaway or internal-only scripts from discovery.
//!
//! # Trust behavior
//!
//! In the default build, `jao` keeps a trust manifest under `~/.jao/`.
//!
//! - Unknown scripts prompt before first run
//! - Modified scripts prompt again
//! - `--ci` disables prompting
//! - CI runs require `--require-fingerprint`
//!
//! If the crate is built without the `trust-manifest` feature, interactive
//! trust is disabled and runs require an explicit fingerprint.
//!
//! # Fingerprints and trust manifests
//!
//! `jao` fingerprints a script by hashing two things together:
//!
//! - the script's canonical path
//! - the script file contents
//!
//! This means moving a script to a different real path changes the
//! fingerprint, even if the bytes are identical. That is intentional: trust is
//! attached to the exact file at the exact resolved location.
//!
//! When the `trust-manifest` feature is enabled, trusted scripts are stored in
//! a local trust manifest keyed by canonical path. Each entry records the last
//! trusted fingerprint for that script. If the current fingerprint differs from
//! the stored one, `jao` treats the script as modified and asks for trust again
//! in interactive mode.
//!
//! # Features
//!
//! - `trust-manifest` (default): Enables local trust tracking for interactive
//!   runs
//! - `config`: Enables config file support used by `trust-manifest`
//!
//! See the repository README for a fuller overview and examples aimed at end
//! users.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind as IoErrorKind;

use clap::builder::OsStringValueParser;
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::actions::{CompletionRequest, Shell};
use crate::error::{JaoError, JaoResult};

mod actions;
mod error;
mod platform;
mod script_discovery;
mod trust;

#[cfg(feature = "config")]
// Currently, the config is only used for trust-manifest.
// hence config specific calls are only called in trust manifest code, causing
// dead code if only the config feature is on. So we allow it.
#[cfg_attr(not(feature = "trust-manifest"), allow(dead_code))]
mod config;

#[cfg(feature = "config")]
// See above
#[cfg_attr(not(feature = "trust-manifest"), allow(dead_code))]
mod storage;

#[doc(hidden)]
fn main() {
    __exit(__main())
}

const GENERATE_COMPLETION_SPECIAL_ARG: &'static str = "__complete";

#[doc(hidden)]
fn __main() -> JaoResult<()> {
    let raw_args = std::env::args_os().collect::<Vec<OsString>>();

    if raw_args
        .get(1)
        .is_some_and(|arg| arg == GENERATE_COMPLETION_SPECIAL_ARG)
    {
        // .skip(2) to skip `jao __complete`
        let complete_args = parse_internal_completion_args(
            raw_args
                .iter()
                .skip(2)
                .map(AsRef::as_ref),
        )?;

        // Jao resolution happens in working directory
        let root = std::env::current_dir()?;

        return actions::complete(root, complete_args);
    }

    let matches = clap_command().try_get_matches_from(&raw_args)?;

    let context = CliContext::from(&matches);

    match CliAction::try_from(&matches)? {
        CliAction::Help => actions::print_help(),
        CliAction::PrintCompletionsForShell(shell) => actions::print_shell_completion(shell),
        #[cfg(not(feature = "trust-manifest"))]
        CliAction::List => {
            let root = std::env::current_dir()?;
            actions::list_scripts(root)
        }
        #[cfg(feature = "trust-manifest")]
        CliAction::List => {
            let root = std::env::current_dir()?;
            let config = config::load_or_init()?;
            let trusted_manifest = trust::manifest::load_or_init(&config)?;
            actions::list_scripts_with_trust_status(root, &trusted_manifest)
        }
        CliAction::Fingerprint { parts } => {
            let root = std::env::current_dir()?;
            let script_path = script_discovery::resolve_script(root, parts)?;
            actions::fingerprint_script(script_path)
        }
        CliAction::RunFingerprinted { parts, required_fingerprint } => {
            let root = std::env::current_dir()?;
            let script_path = script_discovery::resolve_script(root, parts)?;
            actions::run_script_with_fingerprint(script_path, required_fingerprint)
        }
        CliAction::RunUntrusted { .. } if context.ci => Err(JaoError::CiRunRequiresFingerprint),
        #[cfg(not(feature = "trust-manifest"))]
        CliAction::RunUntrusted { .. } => Err(JaoError::RunWithoutTrustManifestRequiresFingerprint),
        #[cfg(feature = "trust-manifest")]
        CliAction::RunUntrusted { parts } => {
            let root = std::env::current_dir()?;
            let script_path = script_discovery::resolve_script(root, parts)?;
            let config = config::load_or_init()?;
            let mut trusted_manifest = trust::manifest::load_or_init(&config)?;
            actions::run_script_with_trust(script_path, &config, &mut trusted_manifest)
        }
    }
}

fn __exit(final_result: JaoResult<()>) -> ! {
    match &final_result {
        Err(JaoError::Clap(clap_err)) => {
            clap_err
                .print()
                .unwrap();
        }
        Err(error) => eprintln!("error: {error}"),
        _ => (),
    }

    let exit_code = match final_result {
        Ok(_) => 0,
        // not our fault
        Err(JaoError::Io(io_err)) if io_err.kind() == IoErrorKind::BrokenPipe => 0,
        Err(JaoError::InvalidArguments(_)) | Err(JaoError::Clap(_)) => 2,
        Err(_) => 1,
    };

    std::process::exit(exit_code)
}

fn parse_internal_completion_args<'a>(mut remaining_args: impl Iterator<Item = &'a OsStr>) -> JaoResult<CompletionRequest<'a>> {
    if remaining_args.next() != Some(OsStr::new("--index")) {
        return Err(JaoError::InvalidArguments("missing --index arg"));
    }

    let index_to_complete = if let Some(index_as_str) = remaining_args.next() {
        index_as_str
            .to_string_lossy()
            .parse::<usize>()
            .map_err(|_| JaoError::InvalidArguments("given index is not a valid number"))?
    } else {
        return Err(JaoError::InvalidArguments("missing index"));
    };

    if remaining_args.next() != Some(OsStr::new("--")) {
        return Err(JaoError::InvalidArguments("missing -- after index"));
    }

    let completion_args = CompletionRequest {
        index_to_complete,
        given_arguments: remaining_args.collect(),
    };

    return Ok(completion_args);
}

#[derive(Debug, Clone, Copy)]
struct CliContext {
    ci: bool,
}

impl From<&ArgMatches> for CliContext {
    fn from(matches: &ArgMatches) -> Self {
        Self { ci: matches.get_flag("ci") }
    }
}

#[derive(Debug)]
enum CliAction<'a> {
    Help,
    PrintCompletionsForShell(Shell),
    List,
    Fingerprint {
        parts: Vec<&'a OsStr>,
    },
    RunFingerprinted {
        parts: Vec<&'a OsStr>,
        required_fingerprint: &'a OsStr,
    },
    RunUntrusted {
        #[cfg_attr(not(feature = "trust-manifest"), allow(dead_code))]
        parts: Vec<&'a OsStr>,
    },
}

impl<'a> TryFrom<&'a ArgMatches> for CliAction<'a> {
    type Error = JaoError;

    fn try_from(matches: &'a ArgMatches) -> Result<Self, Self::Error> {
        if let Some(shell_str) = matches
            .get_raw("completions")
            .and_then(|mut values| values.next())
        {
            let shell = Shell::try_from(shell_str)?;
            return Ok(CliAction::PrintCompletionsForShell(shell));
        };

        if matches.get_flag("list") {
            return Ok(CliAction::List);
        }

        if let Some(parts) = matches.get_raw("fingerprint") {
            return Ok(CliAction::Fingerprint { parts: parts.collect() });
        }

        match (
            matches
                .get_raw("require_fingerprint")
                .and_then(|mut values| values.next()),
            matches.get_raw("script_command"),
        ) {
            (Some(required_fingerprint), Some(parts)) => Ok(CliAction::RunFingerprinted {
                parts: parts.collect(),
                required_fingerprint,
            }),
            (None, Some(parts)) => Ok(CliAction::RunUntrusted { parts: parts.collect() }),
            (None, None) => Ok(CliAction::Help),
            (Some(_), None) => Err(JaoError::InvalidArguments("missing script command after --require-fingerprint")),
        }
    }
}

fn clap_command() -> Command {
    Command::new("jao")
        .version(env!("CARGO_PKG_VERSION"))
        .disable_help_subcommand(true)
        .arg(
            Arg::new("ci")
                .long("ci")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list")
                .long("list")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["completions", "script_command", "fingerprint", "require_fingerprint"]),
        )
        .arg(
            Arg::new("fingerprint")
                .long("fingerprint")
                .num_args(1..)
                .value_parser(OsStringValueParser::new())
                .conflicts_with_all(["list", "completions", "script_command", "require_fingerprint"]),
        )
        .arg(
            Arg::new("require_fingerprint")
                .long("require-fingerprint")
                .value_parser(OsStringValueParser::new())
                .conflicts_with_all(["list", "fingerprint", "completions"]),
        )
        .arg(
            Arg::new("completions")
                .long("completions")
                .value_parser(OsStringValueParser::new())
                .conflicts_with_all(["ci", "fingerprint", "require_fingerprint", "list", "script_command"]),
        )
        .arg(
            Arg::new("script_command")
                .num_args(1..)
                .trailing_var_arg(true)
                .value_parser(OsStringValueParser::new()),
        )
}
