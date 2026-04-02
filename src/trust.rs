//! Trust and fingerprinting support for `jao`.
//!
//! This module contains the pieces used to decide whether a script can be run
//! interactively and how that trust state is persisted.
//!
//! The two core concepts are:
//!
//! - [`create_trust_record`]: computes a stable SHA-256 digest from a script's
//!   canonical path and contents
//! - [`manifest::TrustedManifest`]: stores the last trusted fingerprint per
//!   canonical script path
//!
//! In normal CLI usage these details are mostly internal, but documenting them
//! makes the trust behavior auditable and predictable.

use std::path::{Path, PathBuf};

#[cfg(feature = "trust-manifest")]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::JaoResult;

/// The persisted trust record for a single script file.
///
/// The record currently stores only the last trusted fingerprint, but it is a
/// separate struct so the manifest format can grow later without changing the
/// top-level map shape.
#[derive(PartialEq)]
#[cfg_attr(feature = "trust-manifest", derive(Clone, Serialize, Deserialize))]
pub(crate) struct TrustedFileRecord {
    /// SHA-256 fingerprint of the trusted script at the time it was approved.
    pub(crate) fingerprint: String,
}

/// Computes the script fingerprint used by `jao`.
///
/// The fingerprint is the SHA-256 digest of:
///
/// 1. the script's canonical path as bytes
/// 2. a single `0` separator byte
/// 3. the full file contents
///
/// This is intentionally stricter than hashing only the contents. Two copies of
/// the same script in different locations will produce different fingerprints,
/// which lets trust follow the exact resolved script file rather than "any file
/// with these contents".
///
/// The returned tuple contains the canonical path that was hashed and the
/// hexadecimal digest string.
pub(crate) fn create_trust_record(path: impl AsRef<Path>) -> JaoResult<(PathBuf, TrustedFileRecord)> {
    let canonical_path = std::fs::canonicalize(path)?;
    let file_contents = std::fs::read(&canonical_path)?;

    let mut hasher = Sha256::new();

    hasher.update(
        canonical_path
            .to_string_lossy()
            .as_bytes(),
    );
    hasher.update([0]);
    hasher.update(file_contents);

    Ok((
        canonical_path,
        TrustedFileRecord {
            fingerprint: hex::encode(hasher.finalize()),
        },
    ))
}

/// Trust-manifest-specific types and operations.
///
/// This submodule groups trust-state transitions and manifest persistence so
/// the top-level module can keep non-manifest fingerprint helpers separate.
#[cfg(feature = "trust-manifest")]
pub(crate) mod manifest {
    use core::fmt;
    use std::collections::BTreeMap;
    use std::path::Path;

    use serde::{Deserialize, Serialize};

    use crate::config::JaoConfig;
    use crate::trust::{TrustedFileRecord, create_trust_record};
    use crate::{JaoResult, storage};

    /// The result of comparing a script's current fingerprint to the trust
    /// manifest.
    ///
    /// This drives the interactive trust UX:
    ///
    /// - `Trusted`: run immediately
    /// - `Unknown`: ask for trust in interactive mode
    /// - `Modified`: ask to re-trust in interactive mode
    #[derive(Clone, Copy, PartialEq, Eq)]
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
    #[derive(Default, Clone, Serialize, Deserialize)]
    pub(crate) struct TrustedManifest {
        /// Mapping from canonical script path to the last trusted fingerprint
        /// record for that path.
        #[serde(flatten)]
        pub(crate) scripts: BTreeMap<String, TrustedFileRecord>,
    }

    /// Loads the trust manifest from storage, creating a default one when absent.
    ///
    /// The manifest path is read from [`JaoConfig::trustfile`].
    /// When the file does not exist yet, an empty [`TrustedManifest`] is written
    /// to storage first and returned.
    pub(crate) fn load_or_init(config: &JaoConfig) -> JaoResult<TrustedManifest> {
        let trust_manifest = match storage::load_from_storage(&config.trustfile)? {
            Some(config) => config,
            None => {
                let default_trustfile = TrustedManifest::default();
                storage::write_to_storage(&config.trustfile, &default_trustfile)?;
                default_trustfile
            }
        };

        Ok(trust_manifest)
    }

    /// Determines whether a script is currently trusted.
    ///
    /// This computes the script's current trust record using
    /// [`create_trust_record`] and compares it against `manifest` by canonical path.
    ///
    /// Returns:
    ///
    /// - [`ScriptTrustState::Unknown`] when no entry exists
    /// - [`ScriptTrustState::Trusted`] when fingerprint matches exactly
    /// - [`ScriptTrustState::Modified`] when an entry exists but fingerprint differs
    pub(crate) fn determine_script_trust_state(script_path: impl AsRef<Path>, manifest: &TrustedManifest) -> JaoResult<ScriptTrustState> {
        let (canonical_path, record) = create_trust_record(script_path.as_ref())?;

        let key = canonical_path.to_string_lossy();

        match manifest
            .scripts
            .get(key.as_ref())
        {
            None => Ok(ScriptTrustState::Unknown),
            Some(entry) if *entry == record => Ok(ScriptTrustState::Trusted),
            Some(_) => Ok(ScriptTrustState::Modified),
        }
    }

    /// Writes or updates the trust record for `script_path` and persists the manifest.
    ///
    /// This recomputes the current trust record, stores it under the script's
    /// canonical path key, and writes the updated `manifest` to `trustfile_path`.
    ///
    /// Existing entries for the same canonical path are replaced.
    pub(crate) fn write_script_trust_record(
        script_path: impl AsRef<Path>,
        trustfile_path: impl AsRef<Path>,
        manifest: &mut TrustedManifest,
    ) -> JaoResult<()> {
        let (canonical_path, record) = create_trust_record(script_path.as_ref())?;

        let key = canonical_path
            .to_string_lossy()
            .into_owned();

        manifest
            .scripts
            .insert(key, record);

        storage::write_to_storage(&trustfile_path, &manifest)
    }
}
