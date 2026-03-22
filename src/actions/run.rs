use crate::errors::{JaoError, JaoResult};

use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let script_path = script_path.as_ref();

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
