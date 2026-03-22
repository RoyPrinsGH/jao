use crate::errors::JaoResult;
use crate::trust;
use std::path::Path;

pub fn fingerprint_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let (_, fingerprint) = trust::fingerprint_file(script_path)?;

    println!("{fingerprint}");

    Ok(())
}
