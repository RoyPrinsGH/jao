use crate::script_discovery;
use std::path::PathBuf;

pub fn list_scripts(root: PathBuf) -> impl Iterator<Item = PathBuf> {
    script_discovery::list_scripts(root)
}
