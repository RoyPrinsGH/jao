#[cfg(feature = "trust-manifest")]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::JaoConfigFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaoConfig {
    #[cfg(feature = "trust-manifest")]
    pub trustfile: PathBuf,
}

impl From<JaoConfigFile> for JaoConfig {
    fn from(_config: JaoConfigFile) -> Self {
        Self {
            #[cfg(feature = "trust-manifest")]
            trustfile: _config.trustfile,
        }
    }
}
