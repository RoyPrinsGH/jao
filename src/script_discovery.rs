//! Script discovery and command resolution.
//!
//! `jao` treats script names as dotted command identifiers. A command like
//! `jao deploy api prod` is resolved to the base name `deploy.api.prod`, then
//! matched against runnable script files under the current workspace.
//!
//! Discovery is platform-aware:
//!
//! - Unix-like systems look for `.sh`
//! - Windows looks for `.bat`
//!
//! Resolution searches recursively from the chosen root directory and returns
//! the first matching script yielded by the directory walk.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use jwalk::WalkDir;

use crate::{JaoError, JaoResult};

/// Recursively enumerates runnable scripts below `root`.
///
/// Only files with the platform-supported script extension are returned:
///
/// - `.sh` on Unix-like systems
/// - `.bat` on Windows
pub(crate) fn enumerate_scripts_in(root: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let ext = Path::new(entry.file_name()).extension()?.to_str()?;
            if is_supported_script_extension(ext) { Some(entry.path()) } else { None }
        })
}

fn is_supported_script_extension(ext: &str) -> bool {
    #[cfg(windows)]
    return ext.eq_ignore_ascii_case("bat");
    #[cfg(unix)]
    return ext.eq_ignore_ascii_case("sh");
}

/// Resolves a command-part list to a script path.
///
/// The input parts are joined with `.` to form the script base name. For
/// example, `["deploy", "api", "prod"]` resolves to `deploy.api.prod`.
///
/// Matching is done against the discovered script file stem using
/// platform-specific comparison rules:
///
/// - case-sensitive on Unix-like systems
/// - case-insensitive on Windows
pub(crate) fn resolve_script(root: impl AsRef<Path>, parts: &[String]) -> JaoResult<PathBuf> {
    let script_name = parts.join(".");
    enumerate_scripts_in(root)
        .find(|path| path.file_stem().is_some_and(|file_stem| is_script_name_match(file_stem, &script_name)))
        .ok_or(JaoError::ScriptNotFound { script_name })
}

fn is_script_name_match(file_stem: &OsStr, script_name: &str) -> bool {
    let Some(file_stem) = file_stem.to_str() else {
        return false;
    };

    if cfg!(windows) {
        file_stem.eq_ignore_ascii_case(script_name)
    } else {
        file_stem == script_name
    }
}
