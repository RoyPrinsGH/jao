//! Config loading and initialization.
//!
//! This module reads `config.toml`, and applies defaults and normalization.

use std::collections::BTreeMap;
#[cfg(feature = "trust-manifest")]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{JaoResult, storage};

#[cfg(feature = "trust-manifest")]
const DEFAULT_TRUSTFILE_NAME: &str = "jaotrusted.toml";
const CONFIG_FILE_LOCATION: &str = "config.toml";
const CURRENT_CONFIG_VERSION: u32 = 1;

/// Loads the user's config from disk, creating it with defaults if needed.
///
/// Behavior summary:
///
/// - reads `~/.jao/config.toml` when present
/// - creates a default file when absent
/// - normalizes persisted version to [`CURRENT_CONFIG_VERSION`]
pub(crate) fn load_or_init() -> JaoResult<JaoConfig> {
    let mut config_file = match storage::load_from_storage(CONFIG_FILE_LOCATION)? {
        Some(config) => config,
        None => {
            let default_config = JaoConfigFile::default();
            storage::write_to_storage(CONFIG_FILE_LOCATION, &default_config)?;
            default_config
        }
    };

    // Apply version normalization if needed (e.g., bump version or migrate fields)
    if config_file.version != CURRENT_CONFIG_VERSION {
        config_file.version = CURRENT_CONFIG_VERSION;
        storage::write_to_storage(CONFIG_FILE_LOCATION, &config_file)?;
    }

    Ok(JaoConfig::from(config_file))
}

/// Normalized runtime configuration used by `jao`.
///
/// This is derived from [`JaoConfigFile`] after defaults and path normalization
/// have been applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JaoConfig {
    /// Absolute normalized path to the trust manifest file.
    ///
    /// This path is interpreted relative to the storage root when configured as
    /// a relative path in the on-disk config.
    #[cfg(feature = "trust-manifest")]
    pub(crate) trustfile: PathBuf,
}

impl From<JaoConfigFile> for JaoConfig {
    #[allow(unused_variables)]
    fn from(config: JaoConfigFile) -> Self {
        Self {
            #[cfg(feature = "trust-manifest")]
            trustfile: config.trustfile,
        }
    }
}

/// On-disk config file format stored under `~/.jao/config.toml`.
///
/// Unknown fields are preserved so the config can grow over time without
/// causing older tooling to discard newer keys when rewriting the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JaoConfigFile {
    /// Config file format version.
    ///
    /// Used for lightweight in-place normalization and forward compatibility.
    #[serde(default = "default_config_version")]
    pub(crate) version: u32,

    /// Configured trust manifest location.
    #[cfg(feature = "trust-manifest")]
    #[serde(default = "default_trustfile")]
    pub(crate) trustfile: PathBuf,

    /// Unknown keys preserved during read/write roundtrips.
    ///
    /// This avoids losing newer config fields when older builds rewrite the
    /// file.
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, toml::Value>,
}

impl Default for JaoConfigFile {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            #[cfg(feature = "trust-manifest")]
            trustfile: default_trustfile(),
            extra: BTreeMap::new(),
        }
    }
}

#[cfg(feature = "trust-manifest")]
fn default_trustfile() -> PathBuf {
    PathBuf::from(DEFAULT_TRUSTFILE_NAME)
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}
