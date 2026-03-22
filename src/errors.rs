use std::path::PathBuf;
use std::process::ExitStatus;
use thiserror::Error;

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Debug, Error)]
pub enum ActionError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("script not found: {script_name}")]
    ScriptNotFound { script_name: String },

    #[error("script has no parent directory: {path}")]
    ScriptHasNoParent { path: PathBuf },

    #[error("script has no file name: {path}")]
    ScriptHasNoFileName { path: PathBuf },

    #[error("script exited with status: {status}")]
    ScriptFailed { status: ExitStatus },
}
