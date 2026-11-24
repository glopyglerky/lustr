use std::ffi::OsString;
use std::path::PathBuf;

fn value() -> PathBuf {
    let original = OsString::from("borrowed-native-os");
    let source = original.as_os_str();
    let inferred = source.into();
    let checked_target: PathBuf = inferred;
    checked_target
}

fn main() {
    println!("{}", value().display());
}
