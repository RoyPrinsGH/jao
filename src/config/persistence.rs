use std::fs;
use std::path::Path;

use crate::config::JaoConfigFile;
use crate::errors::JaoResult;

pub(super) fn write_config(path: &Path, config: &JaoConfigFile) -> JaoResult<()> {
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

pub(super) fn parse_config(path: &Path) -> JaoResult<JaoConfigFile> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}
