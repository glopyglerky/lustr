//! Cook real Cargo metadata and check the native projection epoch history.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const CONSUMER: &str = r#"use schema::{Codec, Family};

fn fold(value: <Codec as Family>::View<'_>) -> u32 {
    *value + 9
}

fn spare(value: u32) -> u32 {
    value & 7
}

fn main() {
    let value = 25u32;
    println!("{} {}", fold(&value), spare(value));
}
"#;

fn new_workspace(path: &Path, referent: Option<&str>, consumer: &str) {
    fs::create_dir_all(path.join("schema/src")).unwrap();
    fs::create_dir(path.join("src")).unwrap();
    fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"projection-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nschema = { path = \"schema\" }\n",
    ).unwrap();
    fs::write(
        path.join("schema/Cargo.toml"),
        "[package]\nname = \"schema\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    ).unwrap();
    fs::write(path.join("schema/src/lib.rs"), provider_source(referent)).unwrap();
    fs::write(path.join("src/main.rs"), consumer).unwrap();
}

fn provider_source(referent: Option<&str>) -> String {
    let mut provider = String::from("#![no_std]\n\npub struct Codec;\npub trait Family {\n    type View<'a> where Self: 'a;\n}\n");
    if let Some(referent) = referent {
        provider.push_str(&format!("impl Family for Codec {{\n    type View<'a> = &'a {referent};\n}}\n"));
    }
    provider
}

fn recorded(command: &mut Command, root: &Path, label: &str, expected: i32) -> Output {
    let output = command.output().unwrap();
    fs::write(root.join(format!("{label}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{label}.stderr")), &output.stderr).unwrap();
    fs::write(root.join(format!("{label}.status")), format!("{}\n", output.status)).unwrap();
    assert_eq!(output.status.code(), Some(expected), "{label}: {}", String::from_utf8_lossy(&output.stderr));
    output
}

fn cargo(cargo: &Path, rustc: &Path, fixture: &Path, action: &str) -> Command {
    let mut command = Command::new(cargo);
    command
        .current_dir(fixture)
        .env("RUSTC", rustc)
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .args([action, "--offline", "--jobs", "2", "--target-dir"])
        .arg(fixture.join("target"))
        .arg("--config")
        .arg(format!("build.build-dir={:?}", fixture.join("intermediates")));
    command
}

fn metadata(directory: &Path, matches: &mut Vec<PathBuf>) {
    for item in fs::read_dir(directory).unwrap() {
        let item = item.unwrap();
        if item.file_type().unwrap().is_dir() {
            metadata(&item.path(), matches);
        } else if item.file_name().to_str().is_some_and(|name| name.starts_with("libschema-") && name.ends_with(".rmeta")) {
            matches.push(item.path());
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [cargo_program, rustc, qualifier, root] = args.as_slice() else {
        panic!("usage: CARGO RUSTC QUALIFIER NEW_OUTPUT");
    };
    let (cargo_program, rustc, qualifier) = (
        cargo_program.canonicalize().unwrap(), rustc.canonicalize().unwrap(), qualifier.canonicalize().unwrap(),
    );
    fs::create_dir(root).unwrap();
    let root = root.canonicalize().unwrap();
    let base = root.join("base");
    new_workspace(&base, Some("u32"), CONSUMER);
    let original = recorded(&mut cargo(&cargo_program, &rustc, &base, "run"), &root, "original", 0);
    assert_eq!(original.stdout, b"34 1\n");
    let snapshot = |label: &str| {
        let mut artifacts = Vec::new();
        metadata(&base.join("intermediates/debug/build/schema"), &mut artifacts);
        assert_eq!(artifacts.len(), 1, "{artifacts:?}");
        let output = root.join(format!("{label}.rmeta"));
        fs::copy(&artifacts[0], &output).unwrap();
        output
    };
    let base_artifact = snapshot("original-provider");
    let provider = base.join("schema/src/lib.rs");
    let edit = |referent, label, diagnostic| {
        fs::write(&provider, provider_source(referent)).unwrap();
        let tested = recorded(&mut cargo(&cargo_program, &rustc, &base, "build"), &root, label, 101);
        assert!(String::from_utf8_lossy(&tested.stderr).contains(diagnostic));
        snapshot(label)
    };
    let changed_artifact = edit(Some("u64"), "original-provider-edit", "E0308");
    let missing_artifact = edit(None, "original-provider-missing", "E0277");
    fs::write(&provider, provider_source(Some("u32"))).unwrap();

    let executable = root.join("consumer");
    let source = base.join("src/main.rs");
    let samples = recorded(
        Command::new(&qualifier)
            .arg("--projection").arg(&source).arg(&base_artifact).arg(&executable).arg(root.join("history"))
            .arg(&base_artifact).arg(changed_artifact).arg(missing_artifact).arg(&base_artifact).arg(&base_artifact),
        &root, "native-history", 0,
    );
    let stderr = String::from_utf8(samples.stderr).unwrap();
    let mut projection_identity = None;
    for (step, status, checked, queries) in [
        (0, 0, 3, 1), (1, 0, 0, 0), (2, 1, 2, 1),
        (3, 1, 0, 1), (4, 0, 2, 1), (5, 0, 0, 0),
    ] {
        let expected = format!("step={step} native status={status} ");
        assert!(stderr.lines().any(|line| line.starts_with(&expected) && line.contains(&format!(" checked={checked} "))));
        let expected = format!("step={step} projection ");
        let projection = stderr.lines().find_map(|line| line.strip_prefix(&expected)).unwrap();
        assert!(projection.contains(&format!("upstream-queries={queries} ")));
        let (identity, _) = projection.split_once("}: ").unwrap();
        assert_eq!(*projection_identity.get_or_insert(identity), identity, "provider edit changed the projection identity");
    }
    assert!(stderr.contains("step=4 native checked bodies=[\"fold\", \"main\"]"));
    assert!(stderr.contains("expected `&u64`, found `&u32`"));
    assert!(stderr.contains("no matching trait implementation"));
    assert_eq!(fs::read_to_string(&source).unwrap(), CONSUMER);
    assert_eq!(recorded(&mut Command::new(&executable), &root, "native-run", 0).stdout, original.stdout);

    let escaped = root.join("escaped");
    let escaping = format!("{CONSUMER}\nfn escape<'a>(value: <Codec as Family>::View<'a>) -> &'static u32 {{ value }}\n");
    new_workspace(&escaped, Some("u32"), &escaping);
    let original_error = recorded(&mut cargo(&cargo_program, &rustc, &escaped, "check"), &root, "original-escape", 101);
    assert!(String::from_utf8_lossy(&original_error.stderr).contains("lifetime may not live long enough"));
    let output = root.join("refused-escape");
    let native_error = recorded(
        Command::new(qualifier).arg("--projection").arg(escaped.join("src/main.rs"))
            .arg(&base_artifact).arg(&output).arg(root.join("escape-state")),
        &root, "native-escape", 1,
    );
    assert!(String::from_utf8_lossy(&native_error.stderr).contains("error[L0733]"));
    assert!(!output.exists());
    println!("retained associated type history and lifetime rejection passed");
}
