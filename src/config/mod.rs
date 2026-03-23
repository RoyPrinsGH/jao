use std::fs;

use home::home_dir;

use crate::config;
use crate::config::models::{JaoConfig, JaoConfigFile, JaoContext};
use crate::errors::{JaoError, JaoResult};
#[cfg(feature = "trust-manifest")]
use crate::trust;

pub mod models;

mod persistence;

const CONFIG_FILE_NAME: &str = "config.toml";
const CURRENT_CONFIG_VERSION: u32 = 1;

pub fn load_or_init() -> JaoResult<JaoContext> {
    let storage_dir = home_dir().ok_or(JaoError::StorageDirUnavailable)?.join(".jao");

    fs::create_dir_all(&storage_dir)?;

    let config_path = storage_dir.join(CONFIG_FILE_NAME);

    let mut config_file = if config_path.exists() {
        config::persistence::parse_config(&config_path)?
    } else {
        let default_config = JaoConfigFile::default();
        config::persistence::write_config(&config_path, &default_config)?;
        default_config
    };

    #[cfg(feature = "trust-manifest")]
    let trustfile_changed = {
        let normalized_trustfile = trust::persistence::normalize_trustfile_path(&storage_dir, config_file.trustfile.clone())?;
        let trustfile_changed = normalized_trustfile != config_file.trustfile;
        config_file.trustfile = normalized_trustfile;
        trustfile_changed
    };

    #[cfg(not(feature = "trust-manifest"))]
    let trustfile_changed = false;

    if trustfile_changed || config_file.version != CURRENT_CONFIG_VERSION {
        config_file.version = CURRENT_CONFIG_VERSION;
        config::persistence::write_config(&config_path, &config_file)?;
    }

    let config: JaoConfig = config_file.into();

    #[cfg(feature = "trust-manifest")]
    let trusted_manifest = trust::persistence::load_or_init_trusted_manifest(&config.trustfile)?;

    Ok(JaoContext {
        config,
        #[cfg(feature = "trust-manifest")]
        trusted_manifest,
    })
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}
