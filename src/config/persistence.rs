//! Persistence helpers for `jao` configuration files.

use std::fs;
use std::path::Path;

use crate::JaoResult;
use crate::config::models::JaoConfigFile;

/// Serializes and writes a config file to disk.
pub(super) fn write_config(path: &Path, config: &JaoConfigFile) -> JaoResult<()> {
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

/// Reads and deserializes a config file from disk.
pub(super) fn parse_config(path: &Path) -> JaoResult<JaoConfigFile> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}
