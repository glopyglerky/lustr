//! Run each subreaper lifecycle in its own process, never reap the test harness's
//! unrelated children. These ownership tests do not stand in for native gates.
#![forbid(unsafe_code)]
#[path = "../src/daemons.rs"]
mod daemons;
#[path = "../src/native_state.rs"]
mod native_state;

use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
type R<T> = Result<T, Box<dyn std::error::Error>>;
fn hash(path: &Path) -> R<String> {
    Ok(Sha256::digest(fs::read(path)?).iter().map(|b| format!("{b:02x}")).collect())
}
fn temporary() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "lustr-state-test-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

#[test]
fn owned_lifecycle_rejects_failed_shutdown_and_escaping_paths() {
    for case in ["warm-cold", "failed-shutdown", "escaping-state", "escaping-target"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "native_state_child", "--nocapture"])
            .env("LUSTR_NATIVE_STATE_TEST_CASE", case)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{case}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[ignore = "isolated child helper; the parent test supplies an owned case"]
fn native_state_child() {
    let case = std::env::var("LUSTR_NATIVE_STATE_TEST_CASE").expect("parent must select the isolated case");
    let root = temporary();
    let targets = root.join("targets");
    let mut state = native_state::Runtime::new(&targets).unwrap();
    let target = targets.join("candidate");
    fs::create_dir(&target).unwrap();
    let first = state.prepare("candidate", &target, 0).unwrap();
    for directory in ["tmp", "xdg", "mixed-cache", "native-cache"] {
        fs::write(target.join(directory).join("retained"), b"owned state").unwrap();
    }
    match case.as_str() {
        "warm-cold" => {
            assert_eq!(first, state.prepare("candidate", &target, 1).unwrap());
            for directory in ["tmp", "xdg", "mixed-cache", "native-cache"] {
                assert_eq!(fs::read(target.join(directory).join("retained")).unwrap(), b"owned state");
            }
            let _child = Command::new("/bin/sh").args(["-c", "exit 0"]).spawn().unwrap();
            state.reset(&target, "cold-timed").unwrap();
            assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
            let next = state.prepare("candidate", &target, 2).unwrap();
            assert_ne!(first[std::ffi::OsStr::new("LUST_DAEMON_SOCKET")], next[std::ffi::OsStr::new("LUST_DAEMON_SOCKET")]);
            state.finish().unwrap();
            assert_eq!(state.receipts[2]["event"], "shutdown");
            assert_eq!(state.receipts[2]["observed_successful_exits"], 1);
            assert_eq!(state.receipts[3]["event"], "cleared");
            assert_eq!(state.receipts.last().unwrap()["reason"], "final");
        }
        "failed-shutdown" => {
            let _child = Command::new("/bin/sh").args(["-c", "exit 3"]).spawn().unwrap();
            assert!(state.reset(&target, "cold-timed").is_err());
            assert!(target.join("mixed-cache/retained").is_file());
            assert_eq!(state.receipts.last().unwrap()["status"], "failed");
            assert!(!state.receipts.iter().any(|r| r["event"] == "cleared"));
        }
        "escaping-state" => {
            let outside = root.join("outside");
            fs::create_dir(&outside).unwrap();
            fs::remove_dir_all(target.join("native-cache")).unwrap();
            symlink(&outside, target.join("native-cache")).unwrap();
            assert!(state.prepare("candidate", &target, 1).is_err());
            assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
            state.finish().unwrap();
        }
        "escaping-target" => {
            let outside = root.join("outside");
            fs::rename(&target, &outside).unwrap();
            symlink(&outside, &target).unwrap();
            assert!(state.reset(&target, "cold-timed").is_err());
            assert!(outside.join("mixed-cache/retained").is_file());
            state.finish().unwrap();
        }
        _ => panic!("unknown lifecycle test"),
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn component_receipts_bind_actual_optimized_binaries() {
    let root = temporary();
    let toolchain = root.join("toolchain");
    fs::create_dir_all(toolchain.join("bin")).unwrap();
    let manifest = root.join("COMPONENTS.txt");
    let receipt = root.join("NATIVE-BUILD-RECEIPT.txt");
    let mut inputs = vec![manifest.clone(), receipt.clone()];
    let mut text = "Native build=0123456789abcdef\nCargo artifact profile=release\n".to_owned();
    for name in ["lustr", "rustc-full", "lust", "lust-core", "cargo-lustr"] {
        let path = toolchain.join("bin").join(name);
        fs::write(&path, name).unwrap();
        text.push_str(&format!("{}  {}\n", hash(&path).unwrap(), path.display()));
        inputs.push(path);
    }
    fs::write(&manifest, &text).unwrap();
    fs::write(&receipt, "cargo:rustc-env=LUST_BUILD_HASH=0123456789abcdef\n").unwrap();
    let policy = native_state::Policy {
        kind: "owned-lustr-mixed-v1".into(),
        toolchain: toolchain.clone(),
        component_manifest: manifest.clone(),
        build_receipt: receipt.clone(),
    };
    policy.validate(&inputs).unwrap();
    assert!(policy.validate(&inputs[1..]).is_err());
    fs::write(&manifest, text.replace("profile=release", "profile=debug")).unwrap();
    assert!(policy.validate(&inputs).is_err());
    fs::write(&manifest, &text).unwrap();
    fs::write(&receipt, "cargo:rustc-env=LUST_BUILD_HASH=fedcba9876543210\n").unwrap();
    assert!(policy.validate(&inputs).is_err());
    fs::write(&receipt, "cargo:rustc-env=LUST_BUILD_HASH=0123456789abcdef\n").unwrap();
    fs::write(toolchain.join("bin/lust-core"), b"changed").unwrap();
    assert!(policy.validate(&inputs).is_err());
    // The opt-in path reserves both retained state and daemon/cache disable flags.
    assert!(native_state::OWNED_KEYS.contains(&"LUST_NO_DAEMON"));
    fs::remove_dir_all(root).unwrap();
}
