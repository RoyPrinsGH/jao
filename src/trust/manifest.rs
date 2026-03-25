use std::path::Path;

use crate::JaoResult;
use crate::config::models::JaoContext;
use crate::trust::models::{ScriptTrustState, TrustedFileRecord, TrustedManifest};
use crate::trust::{fingerprint, persistence};

pub(crate) fn get_script_trust(script_path: impl AsRef<Path>, manifest: &TrustedManifest) -> JaoResult<ScriptTrustState> {
    let (canonical_path, record) = build_trusted_record_for_file(script_path.as_ref())?;
    let key = canonical_path.to_string_lossy();

    match manifest.scripts.get(key.as_ref()) {
        None => Ok(ScriptTrustState::Unknown),
        Some(entry) if *entry == record => Ok(ScriptTrustState::Trusted),
        Some(_) => Ok(ScriptTrustState::Modified),
    }
}

pub(crate) fn write_script_trust_record(script_path: impl AsRef<Path>, context: &mut JaoContext) -> JaoResult<()> {
    let (canonical_path, record) = build_trusted_record_for_file(script_path.as_ref())?;
    let key = canonical_path.to_string_lossy().into_owned();
    context.trusted_manifest.scripts.insert(key, record);
    persistence::write_manifest(&context.config.trustfile, &context.trusted_manifest)
}

fn build_trusted_record_for_file(path: &Path) -> JaoResult<(std::path::PathBuf, TrustedFileRecord)> {
    let (canonical_path, fingerprint) = fingerprint::fingerprint_file(path)?;
    Ok((canonical_path, TrustedFileRecord { fingerprint }))
}
