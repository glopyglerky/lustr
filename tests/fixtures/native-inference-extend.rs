use std::ffi::OsString;

fn arguments(jobs: usize) -> Vec<OsString> {
    let mut args = Vec::new();
    args.extend([OsString::from("run"), OsString::from("cargo")]);
    args.extend(["-Zunstable-options".into(), format!("--jobs-frontend={jobs}").into()]);
    args
}

fn main() {
    let args = arguments(3);
    assert_eq!(args.len(), 4);
    assert_eq!(args[0], OsString::from("run"));
    assert_eq!(args[2], OsString::from("-Zunstable-options"));
    assert_eq!(args[3], OsString::from("--jobs-frontend=3"));
    println!("inferred arguments match");
}
