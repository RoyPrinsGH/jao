use crate::errors::JaoResult;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn fingerprint_file(path: impl AsRef<Path>) -> JaoResult<String> {
    let canonical_path = fs::canonicalize(path)?;
    let file_contents = fs::read(&canonical_path)?;

    let mut hasher = Sha256::new();

    hasher.update(canonical_path.to_string_lossy().as_bytes());
    hasher.update([0]);
    hasher.update(file_contents);

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn fingerprint_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let fingerprint = fingerprint_file(script_path)?;

    println!("{fingerprint}");

    Ok(())
}
