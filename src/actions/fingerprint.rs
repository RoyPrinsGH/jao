use crate::errors::ActionResult;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub fn fingerprint(path: PathBuf) -> ActionResult<String> {
    let canonical_path = fs::canonicalize(&path)?;
    let file_contents = fs::read(&canonical_path)?;

    let mut hasher = Sha256::new();

    hasher.update(canonical_path.to_string_lossy().as_bytes());
    hasher.update([0]);
    hasher.update(file_contents);

    Ok(format!("{:x}", hasher.finalize()))
}
