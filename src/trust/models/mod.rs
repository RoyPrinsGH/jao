//! Data models used by the trust-manifest feature.
//!
//! The trust manifest is persisted as TOML and maps canonical script paths to
//! trusted fingerprint records.

#[cfg(feature = "trust-manifest")]
mod script_trust_state;
#[cfg(feature = "trust-manifest")]
mod trusted_file_record;
#[cfg(feature = "trust-manifest")]
mod trusted_manifest;

#[cfg(feature = "trust-manifest")]
pub use script_trust_state::ScriptTrustState;
#[cfg(feature = "trust-manifest")]
pub use trusted_file_record::TrustedFileRecord;
#[cfg(feature = "trust-manifest")]
pub use trusted_manifest::TrustedManifest;
