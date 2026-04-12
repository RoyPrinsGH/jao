use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::JaoResult;

/// Returns true when the file has at least one executable mode bit set.
///
/// This checks Unix permission bits (`0o111`) from metadata.
pub(crate) fn is_executable(path: &Path) -> JaoResult<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata
        .permissions()
        .mode()
        & 0o111
        != 0)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Shebang {
    pub(crate) interpreter: String,
    pub(crate) argument: Option<String>,
}

/// Parses a shebang line and returns interpreter + arguments when present.
///
/// The first line must begin with `#!` and contain an interpreter path.
/// Any text after the interpreter is preserved as a single raw argument,
/// matching Unix shebang execution semantics.
/// Returns `Ok(None)` when no valid shebang is present.
pub(crate) fn parse_shebang(path: &Path) -> JaoResult<Option<Shebang>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    Ok(parse_shebang_line(&first_line))
}

// Shebang syntax is #![INTERPRETER] [ARG: optional, can contain whitespace!]
fn parse_shebang_line(first_line: &str) -> Option<Shebang> {
    if !first_line.starts_with("#!") {
        return None;
    }

    let shebang_data = first_line[2..]
        .trim_end_matches(['\n', '\r'])
        .trim_start();

    if shebang_data.is_empty() {
        return None;
    }

    let first_whitespace_ix = shebang_data
        .find(char::is_whitespace)
        .unwrap_or(shebang_data.len());

    let interpreter = shebang_data[..first_whitespace_ix].to_string();
    if interpreter.is_empty() {
        return None;
    }

    let remainder = shebang_data[first_whitespace_ix..].trim_start();
    let argument = remainder
        .is_empty()
        .then(|| String::from(remainder));

    Some(Shebang { interpreter, argument })
}

#[cfg(test)]
mod tests {
    use super::{Shebang, parse_shebang_line};

    #[test]
    fn parses_interpreter_without_arguments() {
        assert_eq!(
            parse_shebang_line("#!/bin/sh\n"),
            Some(Shebang {
                interpreter: "/bin/sh".to_string(),
                argument: None,
            })
        );
    }

    #[test]
    fn preserves_single_raw_argument_for_standard_shebangs() {
        assert_eq!(
            parse_shebang_line("#!/bin/bash -eu\n"),
            Some(Shebang {
                interpreter: "/bin/bash".to_string(),
                argument: Some("-eu".to_string()),
            })
        );
    }

    #[test]
    fn preserves_env_split_string_as_one_argument() {
        assert_eq!(
            parse_shebang_line("#!/usr/bin/env -S python3 -c 'print(1)'\n"),
            Some(Shebang {
                interpreter: "/usr/bin/env".to_string(),
                argument: Some("-S python3 -c 'print(1)'".to_string()),
            })
        );
    }

    #[test]
    fn trims_leading_whitespace_after_shebang_marker() {
        assert_eq!(
            parse_shebang_line("#!   /usr/bin/env python3\r\n"),
            Some(Shebang {
                interpreter: "/usr/bin/env".to_string(),
                argument: Some("python3".to_string()),
            })
        );
    }

    #[test]
    fn rejects_empty_shebangs() {
        assert_eq!(parse_shebang_line("#!   \n"), None);
    }
}
