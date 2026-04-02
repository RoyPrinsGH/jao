use std::ffi::OsStr;

/// Returns whether `full` starts with `start` using platform-native OsStr semantics.
///
/// On Unix this is a raw byte-prefix check. On Windows this compares UTF-16
/// code units produced by `OsStrExt::encode_wide`.
pub(crate) fn starts_with(full: &OsStr, start: &OsStr) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        return full
            .as_bytes()
            .starts_with(start.as_bytes());
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        let mut full_units = full.encode_wide();

        for start_unit in start.encode_wide() {
            if full_units.next() != Some(start_unit) {
                return false;
            }
        }

        true
    }
}

/// Splits an OsStr on ASCII dot bytes (`.`) without UTF-8 conversion.
///
/// This keeps each segment borrowed from the input [`OsStr`] and therefore
/// avoids allocation and lossy conversion.
pub(crate) fn split_on_dot<'a>(value: &'a OsStr) -> Vec<&'a OsStr> {
    value
        .as_encoded_bytes()
        .split(|byte| *byte == b'.')
        .map(|part| {
            // SAFETY: `part` is a subslice of `value.as_encoded_bytes()` split only
            // on the ASCII `.` delimiter, which preserves valid OsStr boundaries.
            unsafe { OsStr::from_encoded_bytes_unchecked(part) }
        })
        .collect()
}
