use clap::{Parser, error::ErrorKind};
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;

use crate::errors::JaoError;

mod actions;
mod config;
mod errors;
mod help_screen;
mod script_discovery;
mod trust;

#[derive(Debug, Parser)]
#[command(name = "jao")]
#[command(version)]
#[command(about = "Discover and run workspace scripts")]
#[command(arg_required_else_help = true)]
struct Cli {
    /// CI mode: non-interactive and no config/manifest initialization.
    #[arg(long)]
    ci: bool,

    /// Resolve script command and print SHA-256 of canonical path + file contents.
    #[arg(long, value_name = "SCRIPT_COMMAND", num_args = 1..)]
    #[arg(conflicts_with_all = ["list", "script_command"])]
    fingerprint: Option<Vec<String>>,

    /// Required script fingerprint for CI run mode.
    #[arg(long, value_name = "FINGERPRINT")]
    #[arg(requires_all = ["ci", "script_command"])]
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
    match Cli::try_parse() {
        Ok(cli) => {
            if let Err(err) = run_cli(cli) {
                if let JaoError::Io(io_err) = &err
                    && io_err.kind() == IoErrorKind::BrokenPipe
                {
                    std::process::exit(0);
                }

                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                help_screen::print_help();
            }
            _ => {
                let _ = err.print();
                std::process::exit(err.exit_code());
            }
        },
    };
}

fn run_cli(cli: Cli) -> errors::JaoResult<()> {
    let cwd = std::env::current_dir()?;

    enum CommandAction {
        List {
            ci: bool,
        },
        Fingerprint(PathBuf),
        Run {
            script_path: PathBuf,
            ci: bool,
            required_fingerprint: Option<String>,
        },
        NoOp,
    }

    let resolve = |parts: Vec<String>| {
        script_discovery::resolve_script(&cwd, &parts).ok_or_else(|| JaoError::ScriptNotFound {
            script_name: parts.join("."),
        })
    };

    let action = match (cli.list, cli.fingerprint, cli.script_command) {
        (true, _, _) => CommandAction::List { ci: cli.ci },
        (false, Some(parts), _) => CommandAction::Fingerprint(resolve(parts)?),
        (false, None, parts) if !parts.is_empty() => CommandAction::Run {
            script_path: resolve(parts)?,
            ci: cli.ci,
            required_fingerprint: cli.require_fingerprint,
        },
        _ => CommandAction::NoOp,
    };

    match action {
        CommandAction::List { ci: true } => actions::list_script_paths_in(&cwd)?,
        CommandAction::List { ci: false } => {
            let context = config::load_or_init()?;
            actions::list_scripts_in(&cwd, &context)?;
        }
        CommandAction::Fingerprint(script_path) => actions::fingerprint_script(script_path)?,
        CommandAction::Run {
            script_path,
            ci: true,
            required_fingerprint: Some(required_fingerprint),
        } => actions::run_script_ci(script_path, &required_fingerprint)?,
        CommandAction::Run { ci: true, .. } => return Err(JaoError::CiRunRequiresFingerprint),
        CommandAction::Run {
            script_path,
            ci: false,
            ..
        } => {
            let mut context = config::load_or_init()?;
            actions::run_script(script_path, &mut context)?;
        }
        CommandAction::NoOp => {}
    }

    Ok(())
}
