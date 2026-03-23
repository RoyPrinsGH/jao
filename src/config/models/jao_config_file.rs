use std::collections::BTreeMap;
#[cfg(feature = "trust-manifest")]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::{CURRENT_CONFIG_VERSION, default_config_version};

#[cfg(feature = "trust-manifest")]
const DEFAULT_TRUSTFILE_NAME: &str = "jaotrusted.toml";

#[cfg(feature = "trust-manifest")]
fn default_trustfile() -> std::path::PathBuf {
    use std::path::PathBuf;
    PathBuf::from(DEFAULT_TRUSTFILE_NAME)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaoConfigFile {
    #[serde(default = "default_config_version")]
    pub(crate) version: u32,

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
