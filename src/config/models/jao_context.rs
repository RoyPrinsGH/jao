use super::JaoConfig;
#[cfg(feature = "trust-manifest")]
use crate::trust::models::TrustedManifest;

#[derive(Debug, Clone)]
pub struct JaoContext {
    // For now unused if config feature is on but trust-manifest feature is off,
    // since trust-manifest is the only thing that uses the config
    #[allow(dead_code)]
    pub config: JaoConfig,
    #[cfg(feature = "trust-manifest")]
    pub trusted_manifest: TrustedManifest,
}
