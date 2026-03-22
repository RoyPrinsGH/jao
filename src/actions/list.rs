use std::io::{self, Write};
use std::path::Path;

use crate::config::JaoContext;
use crate::errors::JaoResult;
use crate::script_discovery;
use crate::trust;

pub fn list_scripts_in(root: impl AsRef<Path>, context: &JaoContext) -> JaoResult<()> {
    let mut out = io::stdout().lock();

    for script_path in script_discovery::enumerate_scripts_in(root) {
        let trust = trust::get_script_trust(&script_path, &context.trusted_manifest)?;
        writeln!(out, "{trust} {}", script_path.display())?;
    }

    Ok(())
}

pub fn list_script_paths_in(root: impl AsRef<Path>) -> JaoResult<()> {
    let mut out = io::stdout().lock();

    for script_path in script_discovery::enumerate_scripts_in(root) {
        writeln!(out, "{}", script_path.display())?;
    }

    Ok(())
}
