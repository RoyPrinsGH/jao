use serde::{Deserialize, Serialize};

/// The persisted trust record for a single script file.
///
/// The record currently stores only the last trusted fingerprint, but it is a
/// separate struct so the manifest format can grow later without changing the
/// top-level map shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustedFileRecord {
    /// SHA-256 fingerprint of the trusted script at the time it was approved.
    pub fingerprint: String,
}
