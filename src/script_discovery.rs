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

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use ignore::{DirEntry, Walk, WalkBuilder};

use crate::platform::osstr;
use crate::{JaoError, JaoResult};

const FOLDER_MARKER_FILE: &str = ".jaofolder";
const IGNORE_FILE: &str = ".jaoignore";

/// Script path plus parsed command parts discovered during workspace walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveredScript<'a> {
    /// Path to the discovered script file.
    pub(crate) path: &'a Path,
    /// Command parts derived from `.jaofolder` ancestors and script stem.
    pub(crate) parts: ScriptParts<'a>,
}

/// Callback flow-control for discovery iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiscoveryFlow {
    /// Continue scanning the directory walk for additional scripts.
    ContinueSearching,
    /// Stop scanning immediately and return early from discovery.
    StopSearching,
}

/// Walks scripts under `root` and invokes `script_handler` for each discovered script.
///
/// Discovery behavior:
///
/// - Applies standard ignore filtering via `ignore::WalkBuilder`
/// - Honors recursive `.jaoignore` files
/// - Only yields files with platform-supported script extensions
/// - Builds command parts from `.jaofolder` path markers plus script stem
///
/// Return value semantics:
///
/// - `Ok(true)`: traversal stopped early because handler returned
///   [`DiscoveryFlow::StopSearching`]
/// - `Ok(false)`: traversal reached the end naturally
pub(crate) fn for_each_discovered_script(
    root: impl AsRef<Path>,
    mut script_handler: impl for<'a> FnMut(DiscoveredScript<'a>) -> JaoResult<DiscoveryFlow>,
) -> JaoResult<bool> {
    for entry in build_walk_dir(&root) {
        let entry = entry?;

        if !is_script(&entry) {
            continue;
        }

        let Some(script) = into_discovered_script(&root, entry.path()) else {
            continue;
        };

        match script_handler(script)? {
            DiscoveryFlow::StopSearching => return Ok(true),
            DiscoveryFlow::ContinueSearching => continue,
        }
    }

    Ok(false)
}

fn build_walk_dir(root: impl AsRef<Path>) -> Walk {
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

fn into_discovered_script<'a>(root: impl AsRef<Path>, script_path: &'a Path) -> Option<DiscoveredScript<'a>> {
    let script_path_parts = ScriptParts::from_script_stem(script_path.file_stem()?);

    let command_parts = if let Some(parent) = script_path.parent()
        && let Some(marked_folder_parts) = get_marked_folder_parts(root, parent)
    {
        marked_folder_parts.concat(script_path_parts)
    } else {
        script_path_parts
    };

    Some(DiscoveredScript {
        path: script_path,
        parts: command_parts,
    })
}

fn get_marked_folder_parts<'a>(from: impl AsRef<Path>, to: &'a Path) -> Option<ScriptParts<'a>> {
    let from = from.as_ref();

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
            parts.push(directory_name);
        }
    }

    parts.reverse();

    Some(ScriptParts { parts })
}

/// Resolves a command-part list to a script path.
///
/// The input parts are joined with `.` and matched against the command name
/// derived from `.jaofolder` ancestor directories plus the script file stem.
///
/// Matching is case-insensitive on Windows and case-sensitive on Unix-like
/// systems.
///
/// Returns [`JaoError::ScriptNotFound`] when no discovered script matches.
pub(crate) fn resolve_script(root: impl AsRef<Path>, parts: Vec<&OsStr>) -> JaoResult<PathBuf> {
    let requested_parts = ScriptParts::from(parts);
    let mut resolved_path = None;

    let script_found = for_each_discovered_script(root, |script| {
        if script
            .parts
            .matches_exactly(&requested_parts)
        {
            resolved_path = Some(
                script
                    .path
                    .to_path_buf(),
            );
            Ok(DiscoveryFlow::StopSearching)
        } else {
            Ok(DiscoveryFlow::ContinueSearching)
        }
    })?;

    if script_found && let Some(path) = resolved_path {
        return Ok(path);
    }

    Err(JaoError::ScriptNotFound {
        script_name: requested_parts
            .display()
            .to_string_lossy()
            .into_owned(),
    })
}

fn is_command_name_match(discovered_command_name: &OsStr, script_name: &OsStr) -> bool {
    if cfg!(windows) {
        discovered_command_name.eq_ignore_ascii_case(script_name)
    } else {
        discovered_command_name == script_name
    }
}

/// Borrowed command-part collection with prefix and exact-match helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptParts<'a> {
    parts: Vec<&'a OsStr>,
}

impl<'a> From<Vec<&'a OsStr>> for ScriptParts<'a> {
    fn from(parts: Vec<&'a OsStr>) -> Self {
        Self { parts }
    }
}

#[rustfmt::skip]
impl<'a> ScriptParts<'a> {
    /// Creates an empty command-part collection.
    ///
    /// Used while incrementally building completion context from already-typed
    /// command words.
    pub(crate) fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Builds command parts from a script stem by splitting on ASCII `.`.
    ///
    /// For example, `build.docker.local` becomes `build`, `docker`, `local`.
    pub(crate) fn from_script_stem(stem: &'a OsStr) -> Self {
        Self { parts: osstr::split_on_dot(stem) }
    }

    /// Appends a command part.
    ///
    /// This does not normalize or validate the input part.
    pub(crate) fn push(&mut self, part: &'a OsStr) {
        self.parts.push(part);
    }

    /// Returns true when `input_parts` matches in content and length.
    ///
    /// This is an exact match operation (all parts and length must match).
    pub(crate) fn matches_exactly(&self, input_parts: &ScriptParts<'_>) -> bool {
        self.parts.len() == input_parts.parts.len() 
            && self.matches_prior(input_parts)
    }

    /// Returns the next command part when `partial_parts` is a matching prefix.
    ///
    /// This powers dynamic completion by exposing the next segment after the
    /// already-typed command prefix.
    pub(crate) fn try_get_next_part_after(&self, partial_parts: &ScriptParts<'_>) -> Option<&OsStr> {
        if self.parts.len() <= partial_parts.parts.len() 
            || !self.matches_prior(partial_parts) {
            None
        }
        else {
            self.parts.get(partial_parts.parts.len()).copied()
        }
    }

    /// Joins command parts with spaces for display output.
    ///
    /// Intended for human-facing output such as `--list` and error messages.
    pub(crate) fn display(&self) -> OsString {
        self.parts.join(OsStr::new(" "))
    }

    fn matches_prior(&self, input_parts: &ScriptParts<'_>) -> bool {
        self.parts
            .iter()
            .copied()
            .take(input_parts.parts.len())
            .zip(
                input_parts
                    .parts
                    .iter()
                    .copied(),
            )
            .all(|(discovered_command_part, input_part)| is_command_name_match(discovered_command_part, input_part))
    }

    fn concat(mut self, other: Self) -> Self {
        self.parts.extend(other.parts);
        self
    }
}
