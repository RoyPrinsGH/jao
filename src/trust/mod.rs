//! Trust and fingerprinting support for `jao`.
//!
//! This module contains the pieces used to decide whether a script can be run
//! interactively and how that trust state is persisted.
//!
//! The two core concepts are:
//!
//! - [`fingerprint`]: computes a stable SHA-256 digest from a script's
//!   canonical path and contents
//! - [`models::TrustedManifest`]: stores the last trusted fingerprint per
//!   canonical script path
//!
//! In normal CLI usage these details are mostly internal, but documenting them
//! makes the trust behavior auditable and predictable.

pub mod fingerprint;
#[cfg(feature = "trust-manifest")]
pub mod manifest;
pub mod models;
#[cfg(feature = "trust-manifest")]
pub(crate) mod persistence;
