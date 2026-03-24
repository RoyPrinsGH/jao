//! Data models used by the trust-manifest feature.
//!
//! The trust manifest is persisted as TOML and maps canonical script paths to
//! trusted fingerprint records.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// The result of comparing a script's current fingerprint to the trust
/// manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptTrustState {
    /// The current fingerprint matches the stored trust record.
    Trusted,
    /// No trust record exists for this script path.
    Unknown,
    /// A trust record exists, but the current fingerprint no longer matches.
    Modified,
}

impl fmt::Display for ScriptTrustState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ScriptTrustState::Trusted => "trusted",
            ScriptTrustState::Unknown => "unknown",
            ScriptTrustState::Modified => "modified",
        };

        f.write_str(label)
    }
}

/// The persisted trust record for a single script file.
///
/// The record currently stores only the last trusted fingerprint, but it is a
/// separate struct so the manifest format can grow later without changing the
/// top-level map shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct TrustedFileRecord {
    /// SHA-256 fingerprint of the trusted script at the time it was approved.
    pub(crate) fingerprint: String,
}

/// On-disk trust manifest for `jao`.
///
/// This manifest is stored as TOML and keyed by the script's canonical path as
/// a string. Each value records the fingerprint that was trusted for that path.
///
/// Conceptually it looks like:
///
/// ```text
/// "/abs/path/to/scripts/check.sh" = { fingerprint = "..." }
/// "/abs/path/to/scripts/deploy.api.prod.sh" = { fingerprint = "..." }
/// ```
///
/// `jao` compares the current fingerprint for a resolved script against the
/// stored entry:
///
/// - no entry: the script is `unknown`
/// - matching entry: the script is `trusted`
/// - differing entry: the script is `modified`
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct TrustedManifest {
    /// Mapping from canonical script path to the last trusted fingerprint
    /// record for that path.
    #[serde(flatten)]
    pub(crate) scripts: BTreeMap<String, TrustedFileRecord>,
}
