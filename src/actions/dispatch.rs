use std::path::Path;

use super::fingerprint::fingerprint_script;
use super::list::list_scripts;
#[cfg(feature = "trust-manifest")]
use super::list::list_scripts_with_trust_status;
use super::run::run_script_with_fingerprint;
#[cfg(feature = "trust-manifest")]
use super::run::run_script_with_trust;
#[cfg(feature = "trust-manifest")]
use crate::config;
use crate::{CliArgs, JaoError, JaoResult, script_discovery};

type ScriptCommandParts = Vec<String>;

pub(crate) enum CommandAction {
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

pub(crate) fn run_jao_action(command: impl Into<CommandAction>, root: impl AsRef<Path>) -> JaoResult<()> {
    match command.into() {
        CommandAction::List { ci: true } => list_scripts(root),
        CommandAction::List { ci: false } => {
            #[cfg(feature = "trust-manifest")]
            {
                let context = config::load_or_init()?;
                list_scripts_with_trust_status(root, &context.trusted_manifest)
            }
            #[cfg(not(feature = "trust-manifest"))]
            {
                list_scripts(root)
            }
        }
        CommandAction::Fingerprint { parts } => {
            let script_path = script_discovery::resolve_script(root, &parts)?;
            fingerprint_script(script_path)
        }
        CommandAction::Run {
            parts,
            ci: true,
            required_fingerprint: Some(required_fingerprint),
        } => {
            let script_path = script_discovery::resolve_script(root, &parts)?;
            run_script_with_fingerprint(script_path, &required_fingerprint)
        }
        CommandAction::Run { ci: true, .. } => Err(JaoError::CiRunRequiresFingerprint),
        CommandAction::Run {
            parts,
            ci: false,
            required_fingerprint: Some(required_fingerprint),
        } => {
            let script_path = script_discovery::resolve_script(root, &parts)?;
            run_script_with_fingerprint(script_path, &required_fingerprint)
        }
        CommandAction::Run { parts, ci: false, .. } => {
            #[cfg(feature = "trust-manifest")]
            {
                let script_path = script_discovery::resolve_script(root, &parts)?;
                let mut context = config::load_or_init()?;
                run_script_with_trust(script_path, &mut context)
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
