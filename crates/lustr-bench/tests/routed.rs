//! Retired engine selection must fail before invoking any supplied tool.
#![forbid(unsafe_code)]
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn scratch() -> Scratch {
    let root = std::env::temp_dir().join(format!(
        "lustr retired acquisition {}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    Scratch(root)
}

#[test]
fn retired_routed_engines_never_invoke_the_supplied_compiler() {
    use std::os::unix::fs::PermissionsExt;
    let root = scratch();
    let tool = root.0.join("sentinel");
    let marker = root.0.join("invoked");
    fs::write(&tool, format!("#!/bin/sh\nprintf invoked > '{}'\nexit 0\n", marker.display())).unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
    for engine in ["auto", "rustc", "cranelift"] {
        let result = Command::new(env!("CARGO_BIN_EXE_lustr-bench-gate"))
            .arg("tests-via")
            .arg(&tool)
            .arg("--bluefe-root")
            .arg(&root.0)
            .args(["--engine", engine, "test", "--locked", "--offline"])
            .env("CARGO", &tool)
            .env("RUSTC", &tool)
            .env("RUSTDOC", &tool)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(42));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("acquisition is disabled"));
        assert!(!marker.exists(), "retired {engine} route invoked a tool");
    }
}

#[test]
fn retired_mixed_campaign_cannot_create_output_or_query_a_tool() {
    let root = scratch();
    let output = root.0.join("output");
    let result = Command::new(env!("CARGO_BIN_EXE_lustr-mixed-bench"))
        .arg(&root.0)
        .arg(root.0.join("not-a-compiler"))
        .arg(&root.0)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("acquisition is disabled"));
    assert!(result.stdout.is_empty());
    assert!(!output.exists());
}

#[test]
fn retired_state_policy_cannot_be_acquired_through_the_general_runner() {
    let root = scratch();
    let output = root.0.join("output");
    let spec = root.0.join("spec.json");
    let mut value: serde_json::Value = serde_json::from_str(include_str!("../sample-spec.json")).unwrap();
    value["native_state"] =
        json!({"kind":"owned-lustr-mixed-v1","toolchain":root.0,"component_manifest":"missing","build_receipt":"missing"});
    fs::write(&spec, serde_json::to_vec(&value).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_lustr-bench")).arg(&spec).arg(&output).output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("acquisition is disabled"));
    assert!(!output.exists());
}
