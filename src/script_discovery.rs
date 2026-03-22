use jwalk::WalkDir;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

pub fn list_scripts(root: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let ext = Path::new(entry.file_name()).extension()?.to_str()?;
            if is_supported_script_extension(ext) {
                Some(entry.path())
            } else {
                None
            }
        })
}

pub fn find_script_by_name(root: &Path, script_name: &str) -> Option<PathBuf> {
    list_scripts(root).find(|path| {
        path.file_stem()
            .is_some_and(|file_stem| is_script_match(file_stem, script_name))
    })
}

pub fn is_supported_script_extension(ext: &str) -> bool {
    if cfg!(windows) {
        ext.eq_ignore_ascii_case("bat")
    } else {
        ext.eq_ignore_ascii_case("sh")
    }
}

fn is_script_match(file_stem: &OsStr, script_name: &str) -> bool {
    let Some(file_stem) = file_stem.to_str() else {
        return false;
    };

    if cfg!(windows) {
        file_stem.eq_ignore_ascii_case(script_name)
    } else {
        file_stem == script_name
    }
}
