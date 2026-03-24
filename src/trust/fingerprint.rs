use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::JaoResult;

/// Computes the script fingerprint used by `jao`.
///
/// The fingerprint is the SHA-256 digest of:
///
/// 1. the script's canonical path as bytes
/// 2. a single `0` separator byte
/// 3. the full file contents
///
/// This is intentionally stricter than hashing only the contents. Two copies of
/// the same script in different locations will produce different fingerprints,
/// which lets trust follow the exact resolved script file rather than "any file
/// with these contents".
///
/// The returned tuple contains the canonical path that was hashed and the
/// hexadecimal digest string.
pub(crate) fn fingerprint_file(path: impl AsRef<Path>) -> JaoResult<(PathBuf, String)> {
    let canonical_path = std::fs::canonicalize(path)?;
    let file_contents = std::fs::read(&canonical_path)?;

    let mut hasher = Sha256::new();

    hasher.update(canonical_path.to_string_lossy().as_bytes());
    hasher.update([0]);
    hasher.update(file_contents);

    Ok((canonical_path, format!("{:x}", hasher.finalize())))
}
