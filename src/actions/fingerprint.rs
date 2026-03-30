use std::io::{self, Write};
use std::path::Path;

use crate::{JaoResult, trust};

pub(crate) fn fingerprint_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let (_, record) = trust::create_trust_record(script_path)?;
    let mut out = io::stdout().lock();
    writeln!(out, "{}", record.fingerprint)?;
    Ok(())
}
