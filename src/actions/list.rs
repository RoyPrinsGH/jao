use std::io::{self, Write};
use std::path::Path;

use crate::error::JaoResult;
use crate::script_discovery;
#[cfg(feature = "trust-manifest")]
use crate::trust;
#[cfg(feature = "trust-manifest")]
use crate::trust::models::TrustedManifest;

#[cfg(feature = "trust-manifest")]
pub fn list_scripts_with_trust_status(root: impl AsRef<Path>, manifest: &TrustedManifest) -> JaoResult<()> {
    let mut out = io::stdout().lock();
    for script_path in script_discovery::enumerate_scripts_in(root) {
        let trust = trust::manifest::get_script_trust(&script_path, manifest)?;
        writeln!(out, "{trust} {}", script_path.display())?;
    }
    Ok(())
}

pub fn list_scripts(root: impl AsRef<Path>) -> JaoResult<()> {
    let mut out = io::stdout().lock();
    for script_path in script_discovery::enumerate_scripts_in(root) {
        writeln!(out, "{}", script_path.display())?;
    }
    Ok(())
}
