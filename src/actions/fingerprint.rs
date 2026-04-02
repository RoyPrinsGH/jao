use std::io::{self, Write};
use std::path::Path;

use crate::{JaoResult, trust};

/// Computes and prints the trust fingerprint for `script_path`.
///
/// The printed value is the same digest used by trust-manifest comparisons and
/// `--require-fingerprint` verification.
///
/// Output is a single lowercase hexadecimal SHA-256 string followed by `\n`.
pub(crate) fn fingerprint_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let (_, record) = trust::create_trust_record(script_path)?;
    let mut out = io::stdout().lock();
    writeln!(out, "{}", record.fingerprint)?;
    Ok(())
}
