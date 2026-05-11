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

/// Resolved script plus trailing invocation arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedInvocation<'a> {
    /// Path to the resolved script file.
    pub(crate) script_path: PathBuf,
    /// Trailing arguments that should be forwarded to the script.
    pub(crate) arguments: Vec<&'a OsStr>,
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

/// Resolves an invocation to the longest matching script name prefix.
///
/// The input words are matched against the command name derived from
/// `.jaofolder` ancestor directories plus the script file stem.
///
/// When the invocation contains additional trailing words after the longest
/// matching script name, they are returned as script arguments.
///
/// Matching is case-insensitive on Windows and case-sensitive on Unix-like
/// systems.
///
/// Returns [`JaoError::ScriptNotFound`] when no discovered script matches.
pub(crate) fn resolve_script_invocation<'a>(root: impl AsRef<Path>, words: Vec<&'a OsStr>) -> JaoResult<ResolvedInvocation<'a>> {
    let requested_parts = ScriptParts::from(words.clone());
    let mut best_match = None;

    for_each_discovered_script(root, |script| {
        if script
            .parts
            .matches_exactly(&requested_parts)
        {
            best_match = Some((
                script
                    .parts
                    .len(),
                script
                    .path
                    .to_path_buf(),
            ));
            return Ok(DiscoveryFlow::StopSearching);
        }

        if script
            .parts
            .is_prefix_of(&requested_parts)
        {
            let candidate = (
                script
                    .parts
                    .len(),
                script
                    .path
                    .to_path_buf(),
            );

            if best_match
                .as_ref()
                .is_none_or(|(matched_len, _)| candidate.0 > *matched_len)
            {
                best_match = Some(candidate);
            }
        }

        Ok(DiscoveryFlow::ContinueSearching)
    })?;

    if let Some((matched_len, script_path)) = best_match {
        return Ok(ResolvedInvocation {
            script_path,
            arguments: words
                .into_iter()
                .skip(matched_len)
                .collect(),
        });
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
            && self.is_prefix_of(input_parts)
    }

    /// Returns the next command part when `partial_parts` is a matching prefix.
    ///
    /// This powers dynamic completion by exposing the next segment after the
    /// already-typed command prefix.
    pub(crate) fn try_get_next_part_after(&self, partial_parts: &ScriptParts<'_>) -> Option<&OsStr> {
        if self.parts.len() <= partial_parts.parts.len() 
            || !partial_parts.is_prefix_of(self) {
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

    fn is_prefix_of(&self, other: &ScriptParts<'_>) -> bool {
        self.parts.len() <= other.parts.len()
            && self.parts
                .iter()
                .copied()
                .zip(
                    other
                        .parts
                        .iter()
                        .copied(),
                )
                .all(|(discovered_command_part, input_part)| is_command_name_match(discovered_command_part, input_part))
    }

    fn len(&self) -> usize {
        self.parts.len()
    }

    fn concat(mut self, other: Self) -> Self {
        self.parts.extend(other.parts);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::ScriptParts;

    #[test]
    fn script_stem_splits_into_command_parts() {
        assert_eq!(
            ScriptParts::from_script_stem(OsStr::new("build.docker.local")),
            ScriptParts::from(vec![OsStr::new("build"), OsStr::new("docker"), OsStr::new("local"),])
        );
    }

    #[test]
    fn exact_match_requires_same_length() {
        let discovered = ScriptParts::from(vec![OsStr::new("build"), OsStr::new("local")]);
        let prefix_only = ScriptParts::from(vec![OsStr::new("build")]);

        assert!(!discovered.matches_exactly(&prefix_only));
    }

    #[test]
    fn next_part_is_returned_for_matching_prefix() {
        let discovered = ScriptParts::from(vec![OsStr::new("db"), OsStr::new("reset"), OsStr::new("local")]);
        let partial = ScriptParts::from(vec![OsStr::new("db")]);

        assert_eq!(discovered.try_get_next_part_after(&partial), Some(OsStr::new("reset")));
    }

    #[test]
    fn next_part_is_none_for_non_matching_prefix() {
        let discovered = ScriptParts::from(vec![OsStr::new("db"), OsStr::new("reset")]);
        let partial = ScriptParts::from(vec![OsStr::new("build")]);

        assert_eq!(discovered.try_get_next_part_after(&partial), None);
    }

    #[test]
    fn display_joins_parts_with_spaces() {
        let parts = ScriptParts::from(vec![OsStr::new("myapp"), OsStr::new("backend"), OsStr::new("build")]);

        assert_eq!(parts.display(), OsStr::new("myapp backend build"));
    }

    #[test]
    fn shorter_command_is_prefix_of_longer_invocation() {
        let command = ScriptParts::from(vec![OsStr::new("build")]);
        let invocation = ScriptParts::from(vec![OsStr::new("build"), OsStr::new("local")]);

        assert!(command.is_prefix_of(&invocation));
    }
}
