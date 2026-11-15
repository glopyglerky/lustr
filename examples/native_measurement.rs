//! Run inside native_qualification's neutral-path, bounded CPU sandbox.
//! Process-cold means disabled compiler caches, not flushed OS page caches.
use serde_json::{Value, json};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

#[allow(dead_code)]
#[path = "../src/owned_child.rs"]
mod owned_child;

const ENGINES: [&str; 4] = ["original-lust", "matched-lust", "lustr", "stock-rust"];
const SAMPLES: usize = 9;

fn compiler(engine: &str) -> &'static str {
    match engine {
        "original-lust" => "/lust/bin/lust",
        "matched-lust" => "/matched/bin/lust",
        "lustr" => "/package/bin/lustr",
        "stock-rust" => "/package/bin/rustc-full",
        _ => unreachable!(),
    }
}

fn digest(path: &Path) -> String {
    let output = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}

struct Campaign {
    root: PathBuf,
    raw: File,
    events: Vec<Value>,
    serial: usize,
}

impl Campaign {
    fn event(&mut self, value: Value) {
        writeln!(self.raw, "{value}").unwrap();
        self.raw.flush().unwrap();
        self.events.push(value);
    }

    fn compile(
        &mut self,
        case: &str,
        source: &Path,
        engine: &str,
        mode: &str,
        phase: &str,
        sample: usize,
    ) -> (Output, f64, PathBuf, Vec<Value>) {
        let dir = self.root.join(case);
        fs::create_dir_all(&dir).unwrap();
        let artifact = dir.join(engine);
        if artifact.exists() {
            fs::remove_file(&artifact).unwrap();
        }
        let trace = dir.join(format!("{engine}.trace"));
        let before = fs::metadata(&trace).map_or(0, |m| m.len()) as usize;
        let mut command = Command::new(compiler(engine));
        if engine == "stock-rust" {
            command
                .args([
                    "--edition=2024",
                    "-Copt-level=0",
                    "-Cdebuginfo=0",
                    "-Cpanic=unwind",
                    "-Coverflow-checks=on",
                    "-Cdebug-assertions=on",
                ])
                .arg(source);
        } else {
            command.arg("build").arg(source);
        }
        command
            .arg("-o")
            .arg(&artifact)
            .env("LUST_CACHE_DIR", dir.join(format!("{engine}-{mode}-cache")))
            .env("LUST_DAEMON_SOCKET", dir.join(format!("{engine}-{mode}.sock")))
            .env("LUST_DAEMON_IDLE", "3")
            .env("LUSTR_TRACE_PATH", &trace)
            .env("RUSTC", "/package/bin/rustc-full")
            .env_remove("LUST_LIBRARY")
            .env_remove("LUST_STD_SRC")
            .env_remove("LUST_NO_DAEMON")
            .env_remove("LUST_NO_CACHE");
        if mode != "daemon" {
            command.env("LUST_NO_DAEMON", "1");
        }
        if mode == "cold" {
            command.env("LUST_NO_CACHE", "1");
        }
        let argv: Vec<_> = std::iter::once(command.get_program().to_owned()).chain(command.get_args().map(OsString::from)).collect();
        let source_hash = digest(source);
        let start = Instant::now();
        let output = owned_child::output(&mut command).unwrap();
        let elapsed = start.elapsed().as_secs_f64();
        let traces = fs::read(&trace).unwrap_or_default();
        let traces: Vec<Value> =
            traces.get(before..).unwrap_or_default().split(|b| *b == b'\n').filter_map(|line| serde_json::from_slice(line).ok()).collect();
        let label = format!("{:05}-{case}-{engine}", self.serial);
        self.serial += 1;
        fs::write(self.root.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(self.root.join(format!("{label}.stderr")), &output.stderr).unwrap();
        self.event(json!({"kind":"compile", "case":case,"engine":engine,"mode":mode,"phase":phase,"sample":sample,"source":source,"source_sha256":source_hash,"argv":argv,"seconds":elapsed,"exit":output.status.code(),"success":output.status.success(),"artifact_exists":artifact.is_file(),"trace":traces,"log_prefix":label}));
        (output, elapsed, artifact, traces)
    }

    fn execute(&mut self, case: &str, engine: &str, artifact: &Path, phase: &str, sample: usize, compile_seconds: f64) -> Output {
        let start = Instant::now();
        let output = owned_child::output(
            Command::new(artifact)
                .current_dir(&self.root)
                .env_remove("LUSTR_ABSENT_VARIABLE")
                .env("LUSTR_NON_UNICODE", OsString::from_vec(vec![b'a', 0xff, b'z'])),
        )
        .unwrap();
        let elapsed = start.elapsed().as_secs_f64();
        let label = format!("{:05}-{case}-{engine}-run", self.serial);
        self.serial += 1;
        fs::write(self.root.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(self.root.join(format!("{label}.stderr")), &output.stderr).unwrap();
        self.event(json!({"kind":"run","case":case,"engine":engine,"phase":phase,"sample":sample,"seconds":elapsed,"compile_plus_run_seconds":compile_seconds+elapsed,"exit":output.status.code(),"success":output.status.success(),"log_prefix":label}));
        output
    }

    fn check_run(&mut self, case: &str, engine: &str, artifact: &Path, phase: &str, sample: usize, seconds: f64, oracle: &Output) {
        let output = self.execute(case, engine, artifact, phase, sample, seconds);
        assert!(output.status.success(), "{case}/{engine}: runtime failed; not retried");
        assert_eq!(output.stdout, oracle.stdout, "{case}/{engine}: stdout differs");
        assert_eq!(output.stderr, oracle.stderr, "{case}/{engine}: stderr differs");
    }
}

fn edit_source(kind: &str, index: usize) -> String {
    let signature = if kind == "signature-edit" && index % 2 == 1 { "u32" } else { "u64" };
    let field = if kind == "type-edit" && index % 2 == 1 { "u32" } else { "u64" };
    let body = if kind == "body-edit" { 1664525 + index as u64 } else { 1664525 };
    let amount = if kind == "macro-edit" { 1013904223 + index as u64 } else { 1013904223 };
    let invalid = if kind == "invalid-edit" { "let _: u32 = \"invalid\";" } else { "" };
    format!(
        r#"
use std::ffi::OsString;
use std::path::PathBuf;
macro_rules! amount {{ () => {{ {amount}u64 }}; }}
struct Salt {{ value: {field} }}
fn step(value: u64, salt: {signature}) -> u64 {{ value.wrapping_mul({body}).wrapping_add(salt as u64).wrapping_add(amount!()) }}
fn main() {{
    let mut name = OsString::from("application");
    name.reserve(19);
    let path = PathBuf::from(&name);
    let salt = Salt {{ value: 17 }};
    let mut total = salt.value as u64;
    for i in 0..100 {{ total = step(total, i); }}
    {invalid}
    println!("{{}}:{{}}:{{}}", path.display(), std::env::consts::EXE_SUFFIX, total);
}}
// sample {index}
"#
    )
}

fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("unique output directory"));
    fs::create_dir(&root).unwrap();
    let mut campaign = Campaign { raw: File::create(root.join("raw.jsonl")).unwrap(), root, events: Vec::new(), serial: 0 };
    let mut identities = serde_json::Map::new();
    for engine in ENGINES {
        identities.insert(engine.into(), json!({"path":compiler(engine),"sha256":digest(Path::new(compiler(engine)))}));
    }
    for file in [
        "/package/lib/liblustr_native_core.so",
        "/package/lib/lustr-native.id",
        "/package/lib/lustr/std/src/platform.rs",
        "/package/lib/lustr/std/src/env.rs",
        "/package/lib/lustr/std/src/path_from.rs",
        "/package/lib/lustr/std/src/os_string.rs",
    ] {
        identities.insert(file.into(), json!(digest(Path::new(file))));
    }
    let version = Command::new("/package/bin/rustc-full").arg("-vV").output().unwrap();
    let cpu = fs::read_to_string("/proc/cpuinfo").unwrap();
    let cpu = cpu.lines().find(|l| l.starts_with("model name")).unwrap_or("unknown");
    campaign.event(json!({"kind":"identities","compiler_inputs":identities,"stock_version":String::from_utf8(version.stdout).unwrap(),"cpu":cpu,"samples":SAMPLES,"notes":["taskset CPU 10-13; shared host, not isolated CPU reservation","cold disables compiler caches, OS pages not flushed","all paths neutral; no global toolchain or configuration changed","upstream Lust exact pin; matched producer uses official nightly 2026-11-09; original producer differs","single-file dev opt0/debug0/unwind/overflow+debug assertions on","times include owned foreground process cleanup; runtime measured separately","unchanged-request is compiler-request reuse, not Cargo no-op"]}));
    let cases = [
        ("vec-oracle", "/lustr/compiler/native/tests/vec/oracle.rs", false),
        ("panic-recovery", "/lustr/compiler/native/tests/compat/recovered_panic.rs", false),
        ("go-flags", "/lustr/compiler/native/tests/compat/go_flags_types.rs", false),
        ("env-consts", "/lustr/tests/fixtures/native-env-consts.rs", true),
        ("path-from", "/lustr/tests/fixtures/native-path-from.rs", true),
        ("os-capacity", "/lustr/tests/fixtures/native-os-string-capacity.rs", true),
        ("path-capacity", "/lustr/tests/fixtures/native-path-capacity.rs", true),
    ];
    for (case, path, expanded) in cases {
        let source = Path::new(path);
        let (reference, _, oracle, _) = campaign.compile(case, source, "stock-rust", "cold", "admission", 0);
        assert!(reference.status.success());
        let oracle = campaign.execute(case, "stock-rust", &oracle, "admission", 0, 0.0);
        assert!(oracle.status.success());
        let mut admitted = vec!["stock-rust"];
        for engine in ["original-lust", "matched-lust", "lustr"] {
            let (output, seconds, artifact, traces) = campaign.compile(case, source, engine, "cold", "admission", 0);
            if !output.status.success() {
                continue;
            }
            if engine == "lustr" && traces.last().and_then(|t| t["engine"].as_str()) != Some(if expanded { "lustr-native" } else { "lust" })
            {
                continue;
            }
            campaign.check_run(case, engine, &artifact, "admission", 0, seconds, &oracle);
            admitted.push(engine);
        }
        assert!(admitted.contains(&"lustr"), "{case} did not qualify as native");
        campaign.event(json!({"kind":"admission","case":case,"expanded":expanded,"admitted_engines":admitted}));
        println!("Admitted {case}: {admitted:?}");
        for mode in ["cold", "cache", "daemon"] {
            for engine in &admitted {
                let (output, seconds, artifact, _) = campaign.compile(case, source, engine, mode, "prime", 0);
                assert!(output.status.success());
                campaign.check_run(case, engine, &artifact, "prime", 0, seconds, &oracle);
            }
            for sample in 0..SAMPLES {
                for offset in 0..admitted.len() {
                    let engine = admitted[(sample + offset) % admitted.len()];
                    let (output, seconds, artifact, traces) = campaign.compile(case, source, engine, mode, "measurement", sample);
                    assert!(output.status.success());
                    if engine == "lustr" {
                        assert_eq!(traces.last().unwrap()["engine"], if expanded { "lustr-native" } else { "lust" });
                    }
                    campaign.check_run(case, engine, &artifact, "measurement", sample, seconds, &oracle);
                }
            }
        }
    }
    for kind in ["body-edit", "signature-edit", "type-edit", "macro-edit", "invalid-edit"] {
        let dir = campaign.root.join(kind);
        fs::create_dir(&dir).unwrap();
        let source = dir.join("main.rs");
        fs::write(&source, edit_source("base", 0)).unwrap();
        for engine in ["lustr", "stock-rust"] {
            let (output, _, _, _) = campaign.compile(kind, &source, engine, "daemon", "prime", 0);
            assert!(output.status.success());
        }
        for sample in 1..=SAMPLES {
            let text = edit_source(kind, sample);
            fs::write(dir.join(format!("sample-{sample}.rs")), &text).unwrap();
            fs::write(&source, &text).unwrap();
            let order = if sample % 2 == 0 { ["lustr", "stock-rust"] } else { ["stock-rust", "lustr"] };
            let mut results = Vec::new();
            for engine in order {
                let (output, seconds, artifact, traces) = campaign.compile(kind, &source, engine, "daemon", "measurement", sample);
                if kind == "invalid-edit" {
                    assert_eq!(output.status.code(), Some(1));
                    assert!(!artifact.exists());
                    if engine == "lustr" {
                        assert!(traces.iter().any(|t| t["engine"] == "lustr-native-rejected"));
                    }
                } else {
                    assert!(output.status.success());
                    if engine == "lustr" {
                        assert_eq!(traces.last().unwrap()["engine"], "lustr-native");
                    }
                    results.push((engine, seconds, artifact));
                }
            }
            if kind != "invalid-edit" {
                let reference = results.iter().find(|(engine, _, _)| *engine == "stock-rust").unwrap();
                let oracle = campaign.execute(kind, reference.0, &reference.2, "measurement", sample, reference.1);
                assert!(oracle.status.success());
                let native = results.iter().find(|(engine, _, _)| *engine == "lustr").unwrap();
                campaign.check_run(kind, native.0, &native.2, "measurement", sample, native.1, &oracle);
            }
        }
        println!("Measured {kind}");
    }
    let mut groups: std::collections::BTreeMap<String, Vec<f64>> = std::collections::BTreeMap::new();
    for event in &campaign.events {
        if event["phase"] == "measurement" {
            let key = format!(
                "{}/{}/{}/{}",
                event["case"].as_str().unwrap(),
                event["engine"].as_str().unwrap(),
                event["mode"].as_str().unwrap_or("runtime"),
                event["kind"].as_str().unwrap()
            );
            groups.entry(key).or_default().push(event["seconds"].as_f64().unwrap());
        }
    }
    let summary: Vec<_> = groups.into_iter().map(|(key, mut values)| {
        values.sort_by(f64::total_cmp);
        json!({"key":key,"samples":values.len(),"median_seconds":values[values.len()/2],"max_seconds":values[values.len()-1],"raw_seconds":values})
    }).collect();
    fs::write(campaign.root.join("summary.json"), serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    campaign.raw.sync_all().unwrap();
    println!("Retained {} events", campaign.events.len());
}
