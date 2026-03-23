use std::path::PathBuf;
use std::process::ExitStatus;
use thiserror::Error;

pub type JaoResult<T> = Result<T, JaoError>;

#[derive(Debug, Error)]
pub enum JaoError {
    #[cfg(feature = "config")]
    #[error("unable to determine user storage directory")]
    StorageDirUnavailable,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[cfg(feature = "config")]
    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),

    #[cfg(feature = "config")]
    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),

    #[cfg(feature = "config")]
    #[error("invalid trustfile path: {path}")]
    InvalidTrustfilePath { path: PathBuf },

    #[error("script {script_name} not found")]
    ScriptNotFound { script_name: String },

    #[error("script {path} has no parent directory")]
    ScriptHasNoParent { path: PathBuf },

    #[error("script {path} has no file name")]
    ScriptHasNoFileName { path: PathBuf },

    #[error("script is not executable and has no shebang: {path}")]
    ScriptNotExecutableAndNoShebang { path: PathBuf },

    #[error("script exited with status {status}")]
    ScriptFailed { status: ExitStatus },

    #[cfg(feature = "trust-manifest")]
    #[error("unknown script trust requires interactive confirmation: {path}")]
    UnknownScriptNonInteractive { path: PathBuf },

    #[error("--ci run requires --require-fingerprint <FINGERPRINT>")]
    CiRunRequiresFingerprint,

    #[cfg(not(feature = "trust-manifest"))]
    #[error("run requires --require-fingerprint <FINGERPRINT> when built without trust-manifest feature")]
    RunWithoutTrustManifestRequiresFingerprint,

    #[error("invalid --require-fingerprint value (expected 64 hex chars): {fingerprint}")]
    InvalidRequiredFingerprint { fingerprint: String },

    #[error("fingerprint mismatch for {path}: expected {expected}, got {actual}")]
    FingerprintMismatch { path: PathBuf, expected: String, actual: String },

    #[cfg(feature = "trust-manifest")]
    #[error("script was not trusted by user: {path}")]
    ScriptNotTrusted { path: PathBuf },
}
