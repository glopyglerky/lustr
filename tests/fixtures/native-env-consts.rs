use std::env::consts::{ARCH, DLL_EXTENSION, DLL_PREFIX, DLL_SUFFIX, EXE_EXTENSION, EXE_SUFFIX, FAMILY, OS};

fn main() {
    let mut values = Vec::new();
    for value in [ARCH, OS, FAMILY, EXE_SUFFIX, EXE_EXTENSION, DLL_PREFIX, DLL_SUFFIX, DLL_EXTENSION] {
        values.push(value);
    }
    println!("{:?}", values);
    let missing = std::env::var("LUSTR_ABSENT_VARIABLE").unwrap_err();
    assert_eq!(missing, std::env::VarError::NotPresent);
    println!("{} / {:?}", missing, missing.clone());
    println!("{:?}", std::env::var("LUSTR_NON_UNICODE"));
    println!("{}", std::env::var("LUSTR_NON_UNICODE").unwrap_err());
    std::panic::set_hook(Box::new(|_| {}));
    let invalid_vars = std::panic::catch_unwind(|| for _ in std::env::vars() {});
    println!("invalid-vars={}", invalid_vars.is_err());
    let path = std::env::current_dir().unwrap().join(format!("native-program{EXE_SUFFIX}"));
    println!("{}", path.file_name().unwrap().to_str().unwrap());
    let mut total: usize = 0;
    for (key, value) in std::env::vars_os() {
        if key == "LUSTR_NON_UNICODE" {
            total += value.len();
        }
    }
    println!("{total}");
}
