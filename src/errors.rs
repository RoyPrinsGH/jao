use std::path::PathBuf;
use std::process::ExitStatus;
use thiserror::Error;

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Debug, Error)]
pub enum ActionError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("script {script_name} not found")]
    ScriptNotFound { script_name: String },

    #[error("script {path} has no parent directory")]
    ScriptHasNoParent { path: PathBuf },

    #[error("script {path} has no file name")]
    ScriptHasNoFileName { path: PathBuf },

    #[error("script exited with status {status}")]
    ScriptFailed { status: ExitStatus },
}
