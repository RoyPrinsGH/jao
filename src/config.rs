use crate::errors::{JaoError, JaoResult};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "config.toml";
const DEFAULT_TRUSTFILE_NAME: &str = "jaotrusted.toml";
const CURRENT_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaoConfig {
    pub trustfile: PathBuf,
}

impl From<JaoConfigFile> for JaoConfig {
    fn from(config: JaoConfigFile) -> Self {
        Self {
            trustfile: config.trustfile,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JaoConfigFile {
    #[serde(default = "default_config_version")]
    version: u32,

    #[serde(default = "default_trustfile")]
    trustfile: PathBuf,

    // Preserve unknown fields so extending config does not break older tooling.
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}

impl Default for JaoConfigFile {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            trustfile: default_trustfile(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedScript {
    pub canonical_path: PathBuf,
    pub fingerprint: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrustedManifest {
    #[serde(flatten)]
    pub scripts: BTreeMap<String, TrustedScript>,
}

pub fn load_or_init() -> JaoResult<JaoConfig> {
    let storage_dir = home_dir()
        .ok_or(JaoError::StorageDirUnavailable)?
        .join(".jao");

    fs::create_dir_all(&storage_dir)?;

    let config_path = storage_dir.join(CONFIG_FILE_NAME);

    let mut config_file = if config_path.exists() {
        parse_config(&config_path)?
    } else {
        let default_config = JaoConfigFile::default();
        write_config(&config_path, &default_config)?;
        default_config
    };

    config_file.trustfile = normalize_trustfile_path(&storage_dir, config_file.trustfile)?;

    if config_file.version != CURRENT_CONFIG_VERSION {
        config_file.version = CURRENT_CONFIG_VERSION;
        write_config(&config_path, &config_file)?;
    }

    let config: JaoConfig = config_file.into();

    let _ = load_or_init_trusted_manifest(&config)?;

    Ok(config)
}

pub fn load_or_init_trusted_manifest(config: &JaoConfig) -> JaoResult<TrustedManifest> {
    let trustfile_path = &config.trustfile;

    if let Some(parent) = trustfile_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if trustfile_path.exists() {
        parse_manifest(trustfile_path)
    } else {
        let manifest = TrustedManifest::default();
        write_manifest(trustfile_path, &manifest)?;
        Ok(manifest)
    }
}

fn write_config(path: &Path, config: &JaoConfigFile) -> JaoResult<()> {
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn parse_config(path: &Path) -> JaoResult<JaoConfigFile> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

fn default_trustfile() -> PathBuf {
    PathBuf::from(DEFAULT_TRUSTFILE_NAME)
}

fn parse_manifest(path: &Path) -> JaoResult<TrustedManifest> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

fn write_manifest(path: &Path, manifest: &TrustedManifest) -> JaoResult<()> {
    let content = toml::to_string_pretty(manifest)?;
    fs::write(path, content)?;
    Ok(())
}

fn normalize_trustfile_path(storage_dir: &Path, configured_path: PathBuf) -> JaoResult<PathBuf> {
    let storage_dir = fs::canonicalize(storage_dir)?;

    if configured_path.is_absolute() {
        let parent = configured_path
            .parent()
            .ok_or_else(|| JaoError::InvalidTrustfilePath {
                path: configured_path.clone(),
            })
            .and_then(|path| fs::canonicalize(path).map_err(JaoError::Io))?;

        if !parent.starts_with(&storage_dir) {
            return Err(JaoError::InvalidTrustfilePath {
                path: configured_path,
            });
        }

        return Ok(configured_path);
    }

    if configured_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(JaoError::InvalidTrustfilePath {
            path: configured_path,
        });
    }

    Ok(storage_dir.join(configured_path))
}
