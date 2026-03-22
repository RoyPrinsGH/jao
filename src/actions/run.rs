use crate::errors::{ActionError, ActionResult};
use crate::script_discovery;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run_script(parts: &[String], root: PathBuf) -> ActionResult<()> {
    let script_name = parts.join(".");

    let script_path =
        script_discovery::find_script_by_name(&root, &script_name).ok_or_else(|| {
            ActionError::ScriptNotFound {
                script_name: script_name.clone(),
            }
        })?;

    let script_dir = script_path
        .parent()
        .ok_or_else(|| ActionError::ScriptHasNoParent {
            path: script_path.clone(),
        })?;

    let script_file = script_path
        .file_name()
        .ok_or_else(|| ActionError::ScriptHasNoFileName {
            path: script_path.clone(),
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
        Err(ActionError::ScriptFailed { status })
    }
}
