use std::ffi::OsString;

fn variables() -> Vec<(OsString, OsString)> {
    [("CARGO", "cargo"), ("RUSTC", "rustc")]
        .into_iter()
        .map(|(variable, executable)| (variable.into(), OsString::from(executable)))
        .collect()
}

fn main() {
    let variables = variables();
    assert_eq!(variables[0].0, OsString::from("CARGO"));
    assert_eq!(variables[1].1, OsString::from("rustc"));
    println!("inferred environment matches");
}
