use std::path::Path;

use crate::config::JaoContext;
use crate::errors::JaoResult;
use crate::script_discovery;
use crate::trust;

pub fn list_scripts_in(root: impl AsRef<Path>, context: &JaoContext) -> JaoResult<()> {
    for script_path in script_discovery::enumerate_scripts_in(root) {
        let trust = trust::get_script_trust(&script_path, &context.trusted_manifest)?;
        println!("{trust} {}", script_path.display());
    }

    Ok(())
}
