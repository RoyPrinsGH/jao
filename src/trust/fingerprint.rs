use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::JaoResult;

pub fn fingerprint_file(path: impl AsRef<Path>) -> JaoResult<(PathBuf, String)> {
    let canonical_path = std::fs::canonicalize(path)?;
    let file_contents = std::fs::read(&canonical_path)?;

    let mut hasher = Sha256::new();

    hasher.update(canonical_path.to_string_lossy().as_bytes());
    hasher.update([0]);
    hasher.update(file_contents);

    Ok((canonical_path, format!("{:x}", hasher.finalize())))
}
