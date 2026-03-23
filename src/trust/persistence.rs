use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::error::{JaoError, JaoResult};
use crate::trust::models::TrustedManifest;

pub(crate) fn load_or_init_trusted_manifest(trustfile_path: impl AsRef<Path>) -> JaoResult<TrustedManifest> {
    let trustfile_path = trustfile_path.as_ref();

    if let Some(parent) = trustfile_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if trustfile_path.exists() {
        parse_manifest(trustfile_path)
    } else {
        let manifest = TrustedManifest::default();
        write_manifest(trustfile_path, &manifest)?;
        Ok(manifest)
    }
}

pub(crate) fn write_manifest(path: &Path, manifest: &TrustedManifest) -> JaoResult<()> {
    let content = toml::to_string_pretty(manifest)?;
    fs::write(path, content)?;
    Ok(())
}

pub(crate) fn normalize_trustfile_path(storage_dir: impl AsRef<Path>, configured_path: PathBuf) -> JaoResult<PathBuf> {
    let storage_dir = fs::canonicalize(storage_dir)?;

    if configured_path.components().any(|component| matches!(component, Component::ParentDir)) {
        return Err(JaoError::InvalidTrustfilePath { path: configured_path });
    }

    let candidate = if configured_path.is_absolute() {
        configured_path
    } else {
        storage_dir.join(configured_path)
    };

    let parent = candidate
        .parent()
        .ok_or_else(|| JaoError::InvalidTrustfilePath { path: candidate.clone() })?;

    fs::create_dir_all(parent)?;

    let canonical_parent = fs::canonicalize(parent)?;
    if !canonical_parent.starts_with(&storage_dir) {
        return Err(JaoError::InvalidTrustfilePath { path: candidate });
    }

    let file_name = candidate
        .file_name()
        .ok_or_else(|| JaoError::InvalidTrustfilePath { path: candidate.clone() })?;

    Ok(canonical_parent.join(file_name))
}

fn parse_manifest(path: &Path) -> JaoResult<TrustedManifest> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}
