use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{JaoError, JaoResult};

/// Serializes and writes `contents` to the `~/.jao` storage area.
///
/// `file_path` can be relative to the storage root or an absolute path inside
/// that root. Paths outside the storage root are rejected.
pub(crate) fn write_to_storage<C>(file_path: impl AsRef<Path>, contents: &C) -> JaoResult<()>
where
    C: Serialize,
{
    let canonical_file_path = canonicalise_within(get_or_init_jao_storage_dir()?, file_path)?;
    let content = toml::to_string_pretty(contents)?;
    fs::write(canonical_file_path, content)?;
    Ok(())
}

/// Loads and deserializes a value from the `~/.jao` storage area.
///
/// Returns `Ok(None)` when the resolved file does not exist. Paths outside the
/// storage root are rejected.
pub(crate) fn load_from_storage<C>(file_path: impl AsRef<Path>) -> JaoResult<Option<C>>
where
    C: for<'a> Deserialize<'a>,
{
    let canonical_file_path = canonicalise_within(get_or_init_jao_storage_dir()?, file_path)?;
    if !canonical_file_path.exists() {
        Ok(None)
    } else {
        Ok(Some(toml::from_str(&fs::read_to_string(canonical_file_path)?)?))
    }
}

fn get_or_init_jao_storage_dir() -> JaoResult<PathBuf> {
    let jao_storage_dir = home::home_dir()
        .ok_or(JaoError::StorageDirUnavailable)?
        .join(".jao");

    fs::create_dir_all(&jao_storage_dir)?;

    Ok(jao_storage_dir)
}

fn canonicalise_within(safe_root: impl AsRef<Path>, path_to_canonicalise: impl AsRef<Path>) -> JaoResult<PathBuf> {
    let canonical_root = fs::canonicalize(safe_root)?;

    let path_to_canonicalise = path_to_canonicalise.as_ref();

    let path_to_canonicalise = if path_to_canonicalise.is_absolute() {
        path_to_canonicalise.to_path_buf()
    } else {
        canonical_root.join(path_to_canonicalise)
    };

    let canonical_file_path = match fs::canonicalize(&path_to_canonicalise) {
        Ok(canonical) => canonical,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let Some(parent) = path_to_canonicalise.parent() else {
                return Err(JaoError::InvalidStoragePath { path: path_to_canonicalise });
            };

            let canonical_parent = fs::canonicalize(parent)?;
            let Some(file_name) = path_to_canonicalise.file_name() else {
                return Err(JaoError::InvalidStoragePath { path: path_to_canonicalise });
            };

            canonical_parent.join(file_name)
        }
        Err(error) => return Err(error.into()),
    };

    if !canonical_file_path.starts_with(&canonical_root) {
        return Err(JaoError::InvalidStoragePath { path: path_to_canonicalise });
    }

    Ok(canonical_file_path)
}
