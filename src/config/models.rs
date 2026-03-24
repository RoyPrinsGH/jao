//! Data models for `jao` configuration and runtime config state.
//!
//! These types separate the persisted config file shape from the normalized
//! runtime config used by the rest of the application.

use std::collections::BTreeMap;
#[cfg(feature = "trust-manifest")]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[cfg(feature = "trust-manifest")]
use crate::trust::models::TrustedManifest;

#[cfg(feature = "trust-manifest")]
const DEFAULT_TRUSTFILE_NAME: &str = "jaotrusted.toml";

/// Current version of the persisted config format.
pub(crate) const CURRENT_CONFIG_VERSION: u32 = 1;

#[cfg(feature = "trust-manifest")]
fn default_trustfile() -> PathBuf {
    PathBuf::from(DEFAULT_TRUSTFILE_NAME)
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

/// Normalized runtime configuration used by `jao`.
///
/// This is derived from [`JaoConfigFile`] after defaults and path normalization
/// have been applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JaoConfig {
    /// Absolute normalized path to the trust manifest file.
    #[cfg(feature = "trust-manifest")]
    pub(crate) trustfile: PathBuf,
}

impl From<JaoConfigFile> for JaoConfig {
    fn from(_config: JaoConfigFile) -> Self {
        Self {
            #[cfg(feature = "trust-manifest")]
            trustfile: _config.trustfile,
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
    #[serde(default = "default_config_version")]
    pub(crate) version: u32,

    /// Configured trust manifest location.
    #[cfg(feature = "trust-manifest")]
    #[serde(default = "default_trustfile")]
    pub(crate) trustfile: PathBuf,

    // Preserve unknown fields so extending config does not break older tooling.
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

/// Runtime context assembled from config and other loaded state.
///
/// In the default build this includes the loaded trust manifest alongside the
/// normalized configuration.
#[derive(Debug, Clone)]
pub(crate) struct JaoContext {
    // For now unused if config feature is on but trust-manifest feature is off,
    // since trust-manifest is the only thing that uses the config
    #[allow(dead_code)]
    pub(crate) config: JaoConfig,
    #[cfg(feature = "trust-manifest")]
    pub(crate) trusted_manifest: TrustedManifest,
}
