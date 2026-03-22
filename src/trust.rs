use sha2::{Digest, Sha256};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::config::{self, JaoContext, TrustedFileRecord, TrustedManifest};
use crate::errors::JaoResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptTrustState {
    Trusted,
    Unknown,
    Modified,
}

impl fmt::Display for ScriptTrustState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ScriptTrustState::Trusted => "trusted",
            ScriptTrustState::Unknown => "unknown",
            ScriptTrustState::Modified => "modified",
        };

        f.write_str(label)
    }
}

pub fn get_script_trust(
    script_path: impl AsRef<Path>,
    manifest: &TrustedManifest,
) -> JaoResult<ScriptTrustState> {
    let (key, record) = build_trusted_record_for_file(script_path.as_ref())?;

    match manifest.scripts.get(&key) {
        None => Ok(ScriptTrustState::Unknown),
        Some(entry) if *entry == record => Ok(ScriptTrustState::Trusted),
        Some(_) => Ok(ScriptTrustState::Modified),
    }
}

pub fn write_script_trust_record(
    script_path: impl AsRef<Path>,
    context: &mut JaoContext,
) -> JaoResult<()> {
    let (key, record) = build_trusted_record_for_file(script_path.as_ref())?;
    context.trusted_manifest.scripts.insert(key, record);
    config::write_manifest(&context.config.trustfile, &context.trusted_manifest)
}

fn build_trusted_record_for_file(path: &Path) -> JaoResult<(String, TrustedFileRecord)> {
    let (canonical_path, fingerprint) = fingerprint_file(path)?;
    let key = canonical_path.to_string_lossy().to_string();
    Ok((key, TrustedFileRecord { fingerprint }))
}

pub fn fingerprint_file(path: impl AsRef<Path>) -> JaoResult<(PathBuf, String)> {
    let canonical_path = std::fs::canonicalize(path)?;
    let file_contents = std::fs::read(&canonical_path)?;

    let mut hasher = Sha256::new();

    hasher.update(canonical_path.to_string_lossy().as_bytes());
    hasher.update([0]);
    hasher.update(file_contents);

    Ok((canonical_path, format!("{:x}", hasher.finalize())))
}
