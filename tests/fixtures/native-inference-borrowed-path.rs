use std::path::PathBuf;

fn value() -> PathBuf {
    let original = PathBuf::from("borrowed-native-path");
    let source = original.as_path();
    let inferred = source.into();
    let checked_target: PathBuf = inferred;
    checked_target
}

fn main() {
    println!("{}", value().display());
}
