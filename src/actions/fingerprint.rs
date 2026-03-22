use crate::errors::JaoResult;
use crate::trust;
use std::io::{self, Write};
use std::path::Path;

pub fn fingerprint_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let (_, fingerprint) = trust::fingerprint_file(script_path)?;
    let mut out = io::stdout().lock();

    writeln!(out, "{fingerprint}")?;

    Ok(())
}
