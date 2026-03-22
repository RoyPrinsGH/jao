use clap::{Parser, error::ErrorKind};
use std::path::PathBuf;

use crate::errors::JaoError;

mod actions;
mod config;
mod errors;
mod help_screen;
mod script_discovery;

#[derive(Debug, Parser)]
#[command(name = "jao")]
#[command(version)]
#[command(about = "Discover and run workspace scripts")]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Resolve script command and print SHA-256 of canonical path + file contents.
    #[arg(long, value_name = "SCRIPT_COMMAND", num_args = 1..)]
    #[arg(conflicts_with_all = ["list", "script_command"])]
    fingerprint: Option<Vec<String>>,

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
    let _config = config::load_or_init()?;

    let cwd = std::env::current_dir()?;

    enum CommandAction {
        List,
        Fingerprint(PathBuf),
        Run(PathBuf),
        NoOp,
    }

    let resolve = |parts: Vec<String>| {
        script_discovery::resolve_script(&cwd, &parts).ok_or_else(|| JaoError::ScriptNotFound {
            script_name: parts.join("."),
        })
    };

    let action = match (cli.list, cli.fingerprint, cli.script_command) {
        (true, _, _) => CommandAction::List,
        (false, Some(parts), _) => CommandAction::Fingerprint(resolve(parts)?),
        (false, None, parts) if !parts.is_empty() => CommandAction::Run(resolve(parts)?),
        _ => CommandAction::NoOp,
    };

    match action {
        CommandAction::List => actions::list_scripts_in(&cwd)?,
        CommandAction::Fingerprint(script_path) => actions::fingerprint_script(script_path)?,
        CommandAction::Run(script_path) => actions::run_script(script_path)?,
        CommandAction::NoOp => {}
    }

    Ok(())
}
