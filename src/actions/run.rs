use crate::config::JaoContext;
use crate::errors::{JaoError, JaoResult};
use crate::trust::{self, ScriptTrustState};

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_script(script_path: impl AsRef<Path>, context: &mut JaoContext) -> JaoResult<()> {
    let script_path = script_path.as_ref();
    let canonical_path = std::fs::canonicalize(script_path)?;

    let trust_state = trust::get_script_trust(&canonical_path, &context.trusted_manifest)?;

    if trust_state != ScriptTrustState::Trusted {
        if !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
            return Err(JaoError::UnknownScriptNonInteractive {
                path: canonical_path,
            });
        }

        match trust_state {
            ScriptTrustState::Unknown => {
                eprintln!(
                    "warning: script trust is unknown: {}",
                    canonical_path.display()
                );
                eprint!("trust and run? [y/N]: ");
            }
            ScriptTrustState::Modified => {
                eprintln!(
                    "warning: script has been modified since last trust: {}",
                    canonical_path.display()
                );
                eprint!("re-trust and run? [y/N]: ");
            }
            ScriptTrustState::Trusted => {}
        }

        io::stderr().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let answer = answer.trim().to_ascii_lowercase();

        if answer == "y" || answer == "yes" {
            trust::write_script_trust_record(&canonical_path, context)?;
        } else {
            return Err(JaoError::ScriptNotTrusted {
                path: canonical_path,
            });
        }
    }

    let script_dir = script_path
        .parent()
        .ok_or_else(|| JaoError::ScriptHasNoParent {
            path: script_path.to_path_buf(),
        })?;

    let script_file = script_path
        .file_name()
        .ok_or_else(|| JaoError::ScriptHasNoFileName {
            path: script_path.to_path_buf(),
        })?;

    let status = if cfg!(windows) {
        Command::new("cmd")
            .arg("/C")
            .arg(script_file)
            .current_dir(script_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    } else {
        Command::new("bash")
            .arg(script_file)
            .current_dir(script_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(JaoError::ScriptFailed { status })
    }
}
