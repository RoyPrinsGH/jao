use super::JaoConfig;
#[cfg(feature = "trust-manifest")]
use crate::trust::models::TrustedManifest;

#[derive(Debug, Clone)]
pub struct JaoContext {
    pub config: JaoConfig,
    #[cfg(feature = "trust-manifest")]
    pub trusted_manifest: TrustedManifest,
}
