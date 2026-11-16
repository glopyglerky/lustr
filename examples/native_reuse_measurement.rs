//! Paired successful native ABI, retained ABI and real stock measurements.
//! The retained ABI is an experiment; this is not product/host-proof admission.
#[allow(dead_code)]
#[path = "native_inference_measurement.rs"]
mod previous;
#[allow(dead_code, unused_imports)]
#[path = "../tests/native_reuse.rs"]
mod support;
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Instant,
};
const PHASES: &[&str] = &["cold", "repeat", "noop", "body-edit", "signature-edit", "type-edit", "macro-edit", "rejection", "recovery"];

struct Campaign {
    root: PathBuf,
    raw: File,
    events: Vec<Value>,
    seq: usize,
}
impl Campaign {
    fn event(&mut self, v: Value) {
        writeln!(self.raw, "{v}").unwrap();
        self.raw.flush().unwrap();
        self.events.push(v);
    }
    fn run(&mut self, kind: &str, case: &str, phase: &str, engine: &str, sample: usize, c: &mut Command) -> (Output, f64) {
        self.seq += 1;
        let stem = self.root.join(format!("{:05}-{case}-{sample}-{phase}-{engine}-{kind}", self.seq));
        let start = Instant::now();
        let o = c.output().unwrap();
        let ms = start.elapsed().as_secs_f64() * 1000.;
        fs::write(stem.with_extension("stdout"), &o.stdout).unwrap();
        fs::write(stem.with_extension("stderr"), &o.stderr).unwrap();
        self.event(json!({"kind":kind,"case":case,"phase":phase,"engine":engine,"sample":sample,"ms":ms,"code":o.status.code(),"command":format!("{c:?}"),"log":stem}));
        (o, ms)
    }
    fn execute(
        &mut self,
        case: &str,
        phase: &str,
        engine: &str,
        sample: usize,
        out: &Path,
        compile: f64,
    ) -> Vec<(Option<i32>, Vec<u8>, Vec<u8>)> {
        let cli = if case == "coordinator" {
            vec![
                vec!["--help"],
                vec!["--version"],
                vec!["plan", "--frontend-jobs=3", "rustc", "--offline"],
                vec!["plan", "--bluefe-root", "/package", "build", "--offline"],
                vec!["--frontend-jobs=0", "rustc"],
            ]
        } else {
            vec![vec![]]
        };
        let mut total = compile;
        let mut values = Vec::new();
        for (i,args) in cli.into_iter().enumerate() {
            let (o, t) = self.run("runtime", case, phase, engine, sample, Command::new(out).args(args));
            if case != "coordinator" || i < 2 || i == 4 {
                assert_eq!(o.status.code(), Some(if case == "coordinator" && i == 4 { 2 } else { 0 }), "runtime {case}/{engine}/{phase}");
            }
            total += t;
            values.push((o.status.code(), o.stdout, o.stderr));
        }
        self.event(json!({"kind":"compile-plus-runtime-suite","case":case,"phase":phase,"engine":engine,"sample":sample,"ms":total}));
        values
    }
}
fn hash(path: &Path) -> String {
    let o = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(o.status.success());
    String::from_utf8(o.stdout).unwrap()[..64].into()
}
fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("NEW OUTPUT"));
    fs::create_dir(&root).unwrap();
    let mut c = Campaign { raw: File::create(root.join("raw.jsonl")).unwrap(), root, events: Vec::new(), seq: 0 };
    let mut identities = serde_json::Map::new();
    let harness = std::env::current_exe().unwrap();
    identities.insert(harness.to_str().unwrap().into(), json!(hash(&harness)));
    for p in [
        "/build/target-qualified/release/liblustr_native_core.so",
        "/inference/liblustr_native_core.so",
        "/build/native-retained-probe",
        "/inference/native-phase-probe",
        "/core/bin/rustc",
        "/core/bin/cargo",
    ] {
        identities.insert(p.into(), json!(hash(Path::new(p))));
    }
    c.event(json!({"kind":"identities","files":identities,"samples":9,"cpu":"10-13","jobs":2,"compiler":"optimized opt3/CGU1/ThinLTO","application":"opt0/debug0/unwind/checks-on","cache":"new bounded process per sample; retained response artifact proof within process; prior direct is one-shot; native disk caches disabled; stock Cargo target and build fresh per sample and retained for no-op; OS pages untouched","limits":"retained ABI is not product integration, native rlib or general Cargo; platform installation assumed stable, complete host-linker input proof remains open"}));
    let coordinator_only = std::env::args().nth(2).as_deref() == Some("--coordinator-only");
    for case in ["mixed", "coordinator"].into_iter().filter(|case| !coordinator_only || *case == "coordinator") {
        for sample in 0..9 {
            let scope = c.root.join(format!("{case}-{sample}"));
            fs::create_dir(&scope).unwrap();
            let project = scope.join("project");
            fs::create_dir(&project).unwrap();
            let (source, manifest) = if case == "coordinator" {
                for n in ["Cargo.toml", "Cargo.lock", "src"] {
                    support::copy_tree(&Path::new("/lustr/crates/lustr").join(n), &project.join(n));
                }
                (project.join("src/main.rs"), Some(project.join("Cargo.toml")))
            } else {
                (project.join("main.rs"), None)
            };
            let init = Instant::now();
            let mut probe = support::Probe::new(&scope.join("retained-probe"), None);
            let startup = init.elapsed().as_secs_f64() * 1000.;
            let phases = if case == "coordinator" { &PHASES[..3] } else { PHASES };
            for &phase in phases {
                if case == "mixed" && !matches!(phase, "repeat" | "noop") {
                    fs::write(&source, previous::workload(phase, 2 * sample + 1, true)).unwrap();
                }
                let source_hash = hash(&source);
                c.event(json!({"kind":"source","case":case,"sample":sample,"phase":phase,"path":source,"sha256":source_hash}));
                let engines = if sample % 2 == 0 {
                    vec!["prior-direct", "retained-abi", "stock"]
                } else {
                    vec!["stock", "retained-abi", "prior-direct"]
                };
                let mut reference = None;
                for engine in engines {
                    let out = if engine == "stock" && case == "coordinator" {
                        scope.join("stock-target/debug/cargo-lust")
                    } else {
                        scope.join(engine)
                    };
                    let mut args = support::args(&source, &out);
                    if let Some(m) = &manifest {
                        args.extend([
                            "--manifest-path".into(),
                            m.to_str().unwrap().into(),
                            "--bin".into(),
                            "cargo-lust".into(),
                            "--offline".into(),
                            "--locked".into(),
                        ]);
                    }
                    let (code, ms) = if engine == "retained-abi" {
                        let start = Instant::now();
                        let a = probe.request(&args, &[]);
                        let ms = start.elapsed().as_secs_f64() * 1000. + if phase == "cold" { startup } else { 0. };
                        let expected_hit = matches!(phase, "repeat" | "noop");
                        assert_eq!(a.hit(), expected_hit, "{case}/{phase}: {}", String::from_utf8_lossy(&a.err));
                        c.event(json!({"kind":"compile","case":case,"sample":sample,"phase":phase,"engine":engine,"ms":ms,"abi_call_ms":a.ns as f64/1e6,"code":a.code,"hit":a.hit(),"args":args}));
                        (Some(a.code), ms)
                    } else {
                        let mut command = if engine == "prior-direct" {
                            let mut x = Command::new("/inference/native-phase-probe");
                            x.arg("/inference/liblustr_native_core.so").args(&args);
                            x
                        } else if let Some(m) = &manifest {
                            let mut x = Command::new("/core/bin/cargo");
                            x.args(["build", "-vv", "--config"])
                                .arg(format!("build.build-dir={:?}", scope.join("stock-build")))
                                .args([
                                    "--config",
                                    "profile.dev.debug=0",
                                    "--config",
                                    "profile.dev.opt-level=0",
                                    "--config",
                                    "profile.dev.panic=\"unwind\"",
                                    "--config",
                                    "profile.dev.overflow-checks=true",
                                    "--config",
                                    "profile.dev.debug-assertions=true",
                                    "--offline",
                                    "--locked",
                                    "--jobs",
                                    "2",
                                    "--manifest-path",
                                ])
                                .arg(m)
                                .args(["--bin", "cargo-lust"])
                                .env("CARGO_TARGET_DIR", scope.join("stock-target"));
                            x
                        } else {
                            let mut x = Command::new("/core/bin/rustc");
                            x.arg(&source)
                                .args([
                                    "--edition=2024",
                                    "-Copt-level=0",
                                    "-Cdebuginfo=0",
                                    "-Cpanic=unwind",
                                    "-Coverflow-checks=on",
                                    "-Cdebug-assertions=on",
                                ])
                                .arg("-o")
                                .arg(&out);
                            x
                        };
                        command.env("LUST_NO_CACHE", "1").env("LUST_NO_DAEMON", "1");
                        let (o, t) = c.run("compile", case, phase, engine, sample, &mut command);
                        (o.status.code(), t)
                    };
                    if phase == "rejection" {
                        assert_eq!(code, Some(1), "{case}/{engine}/{phase}");
                        continue;
                    }
                    assert_eq!(code, Some(0), "{case}/{engine}/{phase}");
                    assert!(fs::read(&out).unwrap().starts_with(b"\x7fELF"));
                    let values = c.execute(case, phase, engine, sample, &out, ms);
                    if let Some(expected) = &reference {
                        assert_eq!(&values, expected, "{case}/{phase}/{engine}");
                    } else {
                        reference = Some(values);
                    }
                }
            }
            probe.finish();
        }
    }
    if coordinator_only {
        for line in fs::read_to_string("/build/measure2/raw.jsonl").unwrap().lines() {
            let mut event: Value = serde_json::from_str(line).unwrap();
            if event["kind"] == "identities" || event["case"] == "mixed" {
                event["imported_from"] = json!("measure2: fully settled mixed scope, before coordinator harness assertion");
                c.event(event);
            }
        }
    }
    let mut rows = Vec::new();
    for case in ["mixed", "coordinator"] {
        for phase in PHASES {
            for engine in ["prior-direct", "retained-abi", "stock"] {
                for kind in ["compile", "compile-plus-runtime-suite"] {
                    let mut times: Vec<f64> = c
                        .events
                        .iter()
                        .filter(|v| v["case"] == case && v["phase"] == *phase && v["engine"] == engine && v["kind"] == kind)
                        .map(|v| v["ms"].as_f64().unwrap())
                        .collect();
                    if times.is_empty() {
                        continue;
                    }
                    times.sort_by(f64::total_cmp);
                    assert_eq!(times.len(), 9);
                    rows.push(json!({"case":case,"phase":phase,"engine":engine,"kind":kind,"median_ms":times[4],"samples_ms":times}));
                }
            }
        }
    }
    fs::write(c.root.join("summary.json"), serde_json::to_vec_pretty(&json!({"rows":rows,"events":c.events.len()})).unwrap()).unwrap();
    println!("Successful nine-sample retained/prior/stock campaign: {} raw events", c.events.len());
}
