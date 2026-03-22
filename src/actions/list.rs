use std::path::Path;

use crate::errors::JaoResult;
use crate::script_discovery;

pub fn list_scripts_in(root: impl AsRef<Path>) -> JaoResult<()> {
    for script_path in script_discovery::enumerate_scripts_in(root) {
        println!("{}", script_path.display());
    }

    Ok(())
}
