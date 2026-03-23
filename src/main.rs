use std::io::ErrorKind as IoErrorKind;
use std::path::Path;

use clap::Parser;
use clap::error::ErrorKind;

use crate::errors::{JaoError, JaoResult};

mod actions;
mod errors;
mod help_screen;
mod script_discovery;
mod trust;

#[cfg(feature = "config")]
mod config;

#[derive(Debug, Parser)]
#[command(name = "jao")]
#[command(version)]
#[command(about = "Discover and run workspace scripts")]
#[command(arg_required_else_help = true)]
struct CliArgs {
    /// CI mode: non-interactive and no config/manifest initialization.
    #[arg(long)]
    ci: bool,

    /// Resolve script command and print SHA-256 of canonical path + file contents.
    #[arg(long, value_name = "SCRIPT_COMMAND", num_args = 1..)]
    #[arg(conflicts_with_all = ["list", "script_command"])]
    fingerprint: Option<Vec<String>>,

    /// Required script fingerprint for CI run mode.
    #[arg(long, value_name = "FINGERPRINT")]
    #[cfg_attr(feature = "trust-manifest", arg(requires_all = ["ci", "script_command"]))]
    #[cfg_attr(not(feature = "trust-manifest"), arg(requires = "script_command"))]
    #[arg(conflicts_with_all = ["list", "fingerprint"])]
    require_fingerprint: Option<String>,

    /// List discovered scripts for this OS
    #[arg(long, conflicts_with = "script_command")]
    list: bool,

    /// Script command, e.g. 'deploy api prod'
    #[arg(value_name = "SCRIPT_COMMAND", num_args = 1..)]
    script_command: Vec<String>,
}

fn main() {
    match CliArgs::try_parse() {
        Ok(cli_args) => {
            let cli_result = run_cli(cli_args);
            handle_cli_result_and_exit(cli_result)
        }
        Err(clap_err) => handle_clap_parse_result_and_exit(clap_err),
    };
}

type ScriptCommandParts = Vec<String>;

enum CommandAction {
    List { ci: bool },
    Fingerprint { parts: ScriptCommandParts },
    Run { parts: ScriptCommandParts, ci: bool, required_fingerprint: Option<String> },
    NoOp,
}

impl From<CliArgs> for CommandAction {
    fn from(cli: CliArgs) -> Self {
        match (cli.list, cli.fingerprint, cli.script_command) {
            (true, _, _) => CommandAction::List { ci: cli.ci },
            (false, Some(parts), _) => CommandAction::Fingerprint { parts },
            (false, None, parts) if !parts.is_empty() => CommandAction::Run {
                parts,
                ci: cli.ci,
                required_fingerprint: cli.require_fingerprint,
            },
            _ => CommandAction::NoOp,
        }
    }
}
fn handle_clap_parse_result_and_exit(clap_err: clap::Error) -> ! {
    match clap_err.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            help_screen::print_help();
            std::process::exit(0)
        }
        _ => {
            clap_err.print().expect("Error writing error");
            std::process::exit(clap_err.exit_code());
        }
    }
}

fn handle_cli_result_and_exit(result: JaoResult<()>) -> ! {
    match result {
        Err(JaoError::Io(io_err)) if io_err.kind() == IoErrorKind::BrokenPipe => std::process::exit(0),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1)
        }
        Ok(()) => std::process::exit(0),
    }
}

fn run_cli(cli_args: CliArgs) -> JaoResult<()> {
    // jao resolves scripts from the jao invocation dir
    let root = std::env::current_dir()?;

    run_command(cli_args, root)
}

fn run_command(command: impl Into<CommandAction>, root: impl AsRef<Path>) -> JaoResult<()> {
    match command.into() {
        CommandAction::List { ci: true } => actions::list_scripts(root),
        CommandAction::List { ci: false } => {
            #[cfg(feature = "trust-manifest")]
            {
                let context = config::load_or_init()?;
                actions::list_scripts_with_trust_status(root, &context.trusted_manifest)
            }
            #[cfg(not(feature = "trust-manifest"))]
            {
                actions::list_scripts(root)
            }
        }
        CommandAction::Fingerprint { parts } => {
            let script_path = script_discovery::resolve_script(root, &parts)?;
            actions::fingerprint_script(script_path)
        }
        CommandAction::Run {
            parts,
            ci: true,
            required_fingerprint: Some(required_fingerprint),
        } => {
            let script_path = script_discovery::resolve_script(root, &parts)?;
            actions::run_script_with_fingerprint(script_path, &required_fingerprint)
        }
        CommandAction::Run { ci: true, .. } => return Err(JaoError::CiRunRequiresFingerprint),
        CommandAction::Run {
            parts,
            ci: false,
            required_fingerprint: Some(required_fingerprint),
        } => {
            let script_path = script_discovery::resolve_script(root, &parts)?;
            actions::run_script_with_fingerprint(script_path, &required_fingerprint)
        }
        CommandAction::Run { parts, ci: false, .. } => {
            #[cfg(feature = "trust-manifest")]
            {
                let script_path = script_discovery::resolve_script(root, &parts)?;
                let mut context = config::load_or_init()?;
                actions::run_script_with_trust(script_path, &mut context)
            }
            #[cfg(not(feature = "trust-manifest"))]
            {
                let _ = parts;
                Err(JaoError::RunWithoutTrustManifestRequiresFingerprint)
            }
        }
        CommandAction::NoOp => Ok(()),
    }
}
