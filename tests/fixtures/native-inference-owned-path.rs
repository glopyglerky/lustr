use std::ffi::OsString;
use std::path::PathBuf;

fn value() -> PathBuf {
    let source = OsString::from("owned-native-path");
    let inferred = source.into();
    let checked_target: PathBuf = inferred;
    checked_target
}

fn main() {
    let path = value();
    println!("{}", path.display());
}
