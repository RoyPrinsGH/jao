//! Script discovery and command resolution.
//!
//! `jao` resolves commands from two sources:
//!
//! - script file stems, where dots split command parts
//! - ancestor directories marked with a `.jaofolder` file
//!
//! A command like `jao myapp backend build` can therefore resolve to a script
//! at `myapp/backend/scripts/build.sh` when both `myapp/` and `backend/`
//! contain `.jaofolder`, while `scripts/` remains invisible because it is not
//! marked.
//!
//! Discovery is platform-aware:
//!
//! - Unix-like systems look for `.sh`
//! - Windows looks for `.bat`
//!
//! Resolution searches recursively from the chosen root directory and returns
//! the first matching script yielded by the directory walk.

use std::ffi::OsStr;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use ignore::{DirEntry, Walk, WalkBuilder};

use crate::{JaoError, JaoResult};

const FOLDER_MARKER_FILE: &str = ".jaofolder";
const IGNORE_FILE: &str = ".jaoignore";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveredScript<'a> {
    pub(crate) path: &'a Path,
    pub(crate) command_parts: Vec<&'a str>,
}

impl<'a> DiscoveredScript<'a> {
    pub(crate) fn make_command_display(&self) -> String {
        self.command_parts
            .join(" ")
    }
    pub(crate) fn matches_parts(&self, parts: &[String]) -> bool {
        self.command_parts
            .len()
            == parts.len()
            && command_parts_match(
                self.command_parts
                    .iter()
                    .copied(),
                parts
                    .iter()
                    .map(String::as_str),
            )
    }
}

pub(crate) fn for_each_discovered_script<B>(
    root: impl AsRef<Path>,
    mut f: impl for<'a> FnMut(DiscoveredScript<'a>) -> JaoResult<ControlFlow<B>>,
) -> JaoResult<ControlFlow<B>> {
    let root = root.as_ref();

    for entry in build_walk_dir(root) {
        let entry = entry?;

        if !is_script(&entry) {
            continue;
        }

        let Some(script) = into_discovered_script(root, entry.path()) else {
            continue;
        };

        if let ControlFlow::Break(value) = f(script)? {
            return Ok(ControlFlow::Break(value));
        }
    }

    Ok(ControlFlow::Continue(()))
}

fn build_walk_dir(root: &Path) -> Walk {
    WalkBuilder::new(root)
        .standard_filters(true)
        .add_custom_ignore_filename(IGNORE_FILE)
        .build()
}

fn is_script(dir_entry: &DirEntry) -> bool {
    dir_entry
        .file_type()
        .is_some_and(|file_type| file_type.is_file())
        && Path::new(dir_entry.file_name())
            .extension()
            .is_some_and(is_supported_script_extension)
}

fn is_supported_script_extension(ext: &OsStr) -> bool {
    #[cfg(windows)]
    return ext.eq_ignore_ascii_case("bat");
    #[cfg(unix)]
    return ext.eq_ignore_ascii_case("sh");
}

fn into_discovered_script<'a>(root: &Path, script_path: &'a Path) -> Option<DiscoveredScript<'a>> {
    let script_path_parts = script_path
        .file_stem()?
        .to_str()?
        .split('.')
        .collect();

    let command_parts = if let Some(parent) = script_path.parent()
        && let Some(marked_folder_parts) = get_marked_folder_parts(root, parent)
    {
        vec![marked_folder_parts, script_path_parts].concat()
    } else {
        script_path_parts
    };

    Some(DiscoveredScript {
        path: script_path,
        command_parts,
    })
}

fn get_marked_folder_parts<'a>(from: &Path, to: &'a Path) -> Option<Vec<&'a str>> {
    if !to.starts_with(from) {
        return None;
    }

    let mut parts = Vec::new();

    for ancestor in to.ancestors() {
        if *ancestor == *from {
            // We don't want to have to type root if we're in the root,
            // so skip it if FOLDER_MARKER_FILE is present here
            break;
        }

        // .file_name() returns directory name in case of directory
        if ancestor
            .join(FOLDER_MARKER_FILE)
            .is_file()
            && let Some(directory_name) = ancestor.file_name()
        {
            parts.push(directory_name.to_str()?);
        }
    }

    parts.reverse();

    Some(parts)
}

/// Resolves a command-part list to a script path.
///
/// The input parts are joined with `.` and matched against the command name
/// derived from `.jaofolder` ancestor directories plus the script file stem.
pub(crate) fn resolve_script(root: impl AsRef<Path>, parts: &[String]) -> JaoResult<PathBuf> {
    if let ControlFlow::Break(path) = for_each_discovered_script(root, |script| {
        if script.matches_parts(parts) {
            Ok(ControlFlow::Break(
                script
                    .path
                    .to_path_buf(),
            ))
        } else {
            Ok(ControlFlow::Continue(()))
        }
    })? {
        return Ok(path);
    }

    Err(JaoError::ScriptNotFound {
        script_name: parts.join(" "),
    })
}

pub(crate) fn command_parts_match<'a>(
    discovered_command_parts: impl IntoIterator<Item = &'a str>,
    input_parts: impl IntoIterator<Item = &'a str>,
) -> bool {
    discovered_command_parts
        .into_iter()
        .zip(input_parts)
        .all(|(discovered_command_part, input_part)| is_command_name_match(discovered_command_part, input_part))
}

pub(crate) fn command_part_has_prefix(discovered_command_name: &str, input_prefix: &str) -> bool {
    if cfg!(windows) {
        discovered_command_name
            .get(..input_prefix.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(input_prefix))
    } else {
        discovered_command_name.starts_with(input_prefix)
    }
}

fn is_command_name_match(discovered_command_name: &str, script_name: &str) -> bool {
    if cfg!(windows) {
        discovered_command_name.eq_ignore_ascii_case(script_name)
    } else {
        discovered_command_name == script_name
    }
}
