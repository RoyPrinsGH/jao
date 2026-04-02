/// Cross-platform `OsStr` helper functions.
pub(crate) mod osstr;

#[cfg(unix)]
/// Unix-specific platform helpers used by script execution.
pub(crate) mod unix;
