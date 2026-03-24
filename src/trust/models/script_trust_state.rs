use std::fmt;

/// The result of comparing a script's current fingerprint to the trust
/// manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptTrustState {
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
