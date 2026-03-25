use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::Path;

#[cfg(feature = "trust-manifest")]
use crate::trust;
#[cfg(feature = "trust-manifest")]
use crate::trust::models::TrustedManifest;
use crate::{JaoResult, script_discovery};

#[cfg(feature = "trust-manifest")]
pub(crate) fn list_scripts_with_trust_status(root: impl AsRef<Path>, manifest: &TrustedManifest) -> JaoResult<()> {
    let mut out = io::stdout().lock();

    let _ = script_discovery::for_each_discovered_script(&root, |script| {
        let script_path = script.path;
        let trust = trust::manifest::get_script_trust(&script_path, manifest)?;
        writeln!(out, "{trust} {} -> {}", script.make_command_display(), script_path.display())?;
        Ok(ControlFlow::<()>::Continue(()))
    })?;

    Ok(())
}

pub(crate) fn list_scripts(root: impl AsRef<Path>) -> JaoResult<()> {
    let mut out = io::stdout().lock();

    let _ = script_discovery::for_each_discovered_script(&root, |script| {
        writeln!(out, "{} -> {}", script.make_command_display(), script.path.display())?;
        Ok(ControlFlow::<()>::Continue(()))
    })?;

    Ok(())
}
