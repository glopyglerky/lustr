use crate::ffi::{OsStr, OsString};
use crate::path::{Path, PathBuf};

// The baseline already implements owned OsString, String, &str, &String and
// &Path conversions. Supply the missing borrowed OS-string/path conversions
// through the same standard From interface, without an inherent replacement.
impl<'a> From<&'a OsStr> for PathBuf {
    fn from(source: &'a OsStr) -> PathBuf {
        PathBuf::from_os_string(source.to_os_string())
    }
}

impl<'a> From<&'a OsString> for PathBuf {
    fn from(source: &'a OsString) -> PathBuf {
        PathBuf::from_os_string(source.clone())
    }
}

impl<'a> From<&'a PathBuf> for PathBuf {
    fn from(source: &'a PathBuf) -> PathBuf {
        Path::to_path_buf(source.as_path())
    }
}
