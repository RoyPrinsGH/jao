use clap::Parser;
use std::path::PathBuf;

mod actions;
mod errors;
mod script_discovery;

#[derive(Debug, Parser)]
#[command(name = "jao")]
#[command(about = "A tiny modern CLI example", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[arg(long, value_name = "FILE")]
    #[arg(conflicts_with_all = ["list", "script_command"])]
    fingerprint: Option<PathBuf>,

    #[arg(long, conflicts_with = "script_command")]
    list: bool,

    #[arg(value_name = "SCRIPT_COMMAND", num_args = 1..)]
    script_command: Vec<String>,
}

fn main() {
    if let Err(err) = run_cli() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_cli() -> errors::ActionResult<()> {
    let cli = Cli::parse();

    if let Some(file_path) = cli.fingerprint {
        let hash = actions::fingerprint(file_path)?;
        println!("{hash}");
    }

    if cli.list {
        let cwd = std::env::current_dir()?;
        for script_path in actions::list_scripts(cwd) {
            println!("{}", script_path.display());
        }
    }

    if !cli.script_command.is_empty() {
        let cwd = std::env::current_dir()?;
        actions::run_script(&cli.script_command, cwd)?;
    }

    Ok(())
}
