use std::io::{self, Write};
use std::path::Path;

use crate::script_discovery::DiscoveryFlow;
#[cfg(feature = "trust-manifest")]
use crate::trust;
#[cfg(feature = "trust-manifest")]
use crate::trust::manifest::TrustedManifest;
use crate::{JaoResult, script_discovery};

/// Lists discovered scripts with trust-state labels.
///
/// Output format: `<trust>\t<command>\t\t<resolved_path>`.
///
/// `trust` values map to [`crate::trust::manifest::ScriptTrustState`] display labels.
#[cfg(feature = "trust-manifest")]
pub(crate) fn list_scripts_with_trust_status(root: impl AsRef<Path>, manifest: &TrustedManifest) -> JaoResult<()> {
    let mut out = io::stdout().lock();

    script_discovery::for_each_discovered_script(&root, |script| {
        let trust = trust::manifest::determine_script_trust_state(script.path, manifest)?;

        #[rustfmt::skip]
        writeln!(
            out,"{trust} \t {} \t\t {}",
            script.parts.display().to_string_lossy(),
            script.path.display()
        )?;

        Ok(DiscoveryFlow::ContinueSearching)
    })?;

    Ok(())
}

/// Lists discovered scripts without trust-state labels.
///
/// Output format: `<command>\t\t<resolved_path>`.
///
/// This variant is compiled when trust-manifest support is disabled.
#[cfg(not(feature = "trust-manifest"))]
pub(crate) fn list_scripts(root: impl AsRef<Path>) -> JaoResult<()> {
    let mut out = io::stdout().lock();

    script_discovery::for_each_discovered_script(&root, |script| {
        #[rustfmt::skip]
        writeln!(
            out, "{} \t\t {}",
            script.parts.display().to_string_lossy(),
            script.path.display()
        )?;

        Ok(DiscoveryFlow::ContinueSearching)
    })?;

    Ok(())
}
