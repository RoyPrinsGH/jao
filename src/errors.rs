use std::path::PathBuf;
use std::process::ExitStatus;
use thiserror::Error;

pub type JaoResult<T> = Result<T, JaoError>;

#[derive(Debug, Error)]
pub enum JaoError {
    #[error("unable to determine user storage directory")]
    StorageDirUnavailable,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("invalid trustfile path: {path}")]
    InvalidTrustfilePath { path: PathBuf },

    #[error("script {script_name} not found")]
    ScriptNotFound { script_name: String },

    #[error("script {path} has no parent directory")]
    ScriptHasNoParent { path: PathBuf },

    #[error("script {path} has no file name")]
    ScriptHasNoFileName { path: PathBuf },

    #[error("script exited with status {status}")]
    ScriptFailed { status: ExitStatus },

    #[error("unknown script trust requires interactive confirmation: {path}")]
    UnknownScriptNonInteractive { path: PathBuf },

    #[error("script was not trusted by user: {path}")]
    ScriptNotTrusted { path: PathBuf },
}
