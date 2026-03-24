use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::TrustedFileRecord;

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
pub struct TrustedManifest {
    /// Mapping from canonical script path to the last trusted fingerprint
    /// record for that path.
    #[serde(flatten)]
    pub scripts: BTreeMap<String, TrustedFileRecord>,
}
