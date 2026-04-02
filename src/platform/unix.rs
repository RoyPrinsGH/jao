use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::JaoResult;

/// Returns true when the file has at least one executable mode bit set.
///
/// This checks Unix permission bits (`0o111`) from metadata.
pub(crate) fn is_executable(path: &Path) -> JaoResult<bool> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata
        .permissions()
        .mode()
        & 0o111
        != 0)
}

/// Parses a shebang line and returns interpreter + arguments when present.
///
/// The first line must begin with `#!` and contain at least one token.
/// Returns `Ok(None)` when no valid shebang is present.
pub(crate) fn parse_shebang(path: &Path) -> JaoResult<Option<(String, Vec<String>)>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    if !first_line.starts_with("#!") {
        return Ok(None);
    }

    let shebang = first_line[2..].trim();
    if shebang.is_empty() {
        return Ok(None);
    }

    let mut parts = shebang.split_whitespace();
    let Some(interpreter) = parts.next() else {
        return Ok(None);
    };

    Ok(Some((
        interpreter.to_string(),
        parts
            .map(ToString::to_string)
            .collect(),
    )))
}
