use std::ffi::OsString;
use std::path::PathBuf;

fn value() -> OsString {
    let source = OsString::from("owned-native-roundtrip");
    let inferred = source.into();
    let checked_target: PathBuf = inferred;
    let inferred_back = checked_target.into();
    let checked_back: OsString = inferred_back;
    checked_back
}

fn main() {
    println!("{}", value().to_string_lossy());
}
