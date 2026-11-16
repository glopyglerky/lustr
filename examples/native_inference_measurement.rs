//! Equivalent-source native inference measurements, including unsuccessful
//! native baselines as coverage evidence only. Run in the bounded sandbox.
use serde_json::{Value, json};
use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}, process::{Command, Output}, time::Instant};
#[allow(dead_code)]
#[path = "../src/owned_child.rs"]
mod owned_child;

const SAMPLES: usize = 9;
const PRODUCT: &str = "/build/candidate/bin/lustr";

fn hash(path: &Path) -> String {
    let out = Command::new("sha256sum").arg("--").arg(path).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()[..64].to_owned()
}

struct Campaign { root: PathBuf, raw: File, events: Vec<Value>, serial: usize }
impl Campaign {
    fn event(&mut self, value: Value) {
        writeln!(self.raw, "{value}").unwrap(); self.raw.flush().unwrap(); self.events.push(value);
    }
    fn run(&mut self, kind: &str, case: &str, phase: &str, engine: &str, sample: usize, c: &mut Command) -> (Output, f64) {
        let label=format!("{:05}-{case}-{phase}-{engine}-{kind}", self.serial); self.serial+=1;
        let argv=format!("{c:?}");
        let start=Instant::now(); let out=owned_child::output(c).unwrap(); let seconds=start.elapsed().as_secs_f64();
        fs::write(self.root.join(format!("{label}.stdout")), &out.stdout).unwrap();
        fs::write(self.root.join(format!("{label}.stderr")), &out.stderr).unwrap();
        self.event(json!({"kind":kind,"case":case,"phase":phase,"engine":engine,"sample":sample,"seconds":seconds,
            "exit":out.status.code(),"success":out.status.success(),"command":argv,"logs":label}));
        (out,seconds)
    }
    fn compile(&mut self, case: &str, phase: &str, engine: &str, sample: usize, source: &Path, manifest: Option<&Path>) -> (Output,f64,PathBuf) {
        let scope=self.root.join(format!("{case}-{sample}")); fs::create_dir_all(&scope).unwrap();
        let artifact=scope.join(engine);
        let trace=scope.join(format!("{engine}.trace"));
        let old=fs::metadata(&trace).map_or(0,|m|m.len()) as usize;
        let mut c;
        if engine=="stock" {
            if let Some(manifest)=manifest {
                c=Command::new("/core/bin/cargo");
                c.args(["build","-vv","--offline","--locked","--jobs","2","--bin","cargo-lust","--manifest-path"]).arg(manifest)
                    .args(["--config","profile.dev.debug=0","--config","profile.dev.opt-level=0","--config","profile.dev.panic=\"unwind\"",
                        "--config","profile.dev.overflow-checks=true","--config","profile.dev.debug-assertions=true"])
                    .arg("--config").arg(format!("build.build-dir={:?}",scope.join("cargo-build")))
                    .env("CARGO_TARGET_DIR",scope.join("cargo-target")).env("RUSTC","/core/bin/rustc");
            } else {
                c=Command::new("/core/bin/rustc");
                c.arg(source).args(["--edition=2024","-Copt-level=0","-Cdebuginfo=0","-Cpanic=unwind","-Coverflow-checks=on","-Cdebug-assertions=on"])
                    .arg("-o").arg(&artifact);
            }
        } else if matches!(engine,"inference-abi"|"prior-inference-abi") {
            c=Command::new("/build/native-phase-probe");
            let library=if engine=="prior-inference-abi" {std::env::var_os("LUSTR_PHASE_PRIOR_LIBRARY").unwrap()}
                else {"/build/target-qualified/release/liblustr_native_core.so".into()};
            c.arg(library).arg("build").arg(source).arg("-o").arg(&artifact);
        } else {
            let program=if engine=="prior-product" {std::env::var_os("LUSTR_PHASE_PRIOR_PRODUCT").unwrap()}
                else if engine=="product" {PRODUCT.into()} else {"/matched/bin/lust".into()};
            c=Command::new(program);
            c.arg("build").arg(source).arg("-o").arg(&artifact);
            if let Some(manifest)=manifest { c.arg("--manifest-path").arg(manifest).args(["--bin","cargo-lust","--offline","--locked"]); }
        }
        c.env("LUST_NO_DAEMON","1").env_remove("LUST_NO_CACHE").env_remove("LUST_LIBRARY").env_remove("LUST_STD_SRC")
            .env("LUST_CACHE_DIR",scope.join(format!("{engine}-cache"))).env("LUSTR_TRACE_PATH",&trace);
        if phase=="cold" {c.env("LUST_NO_CACHE","1");}
        if matches!(engine,"inference-abi"|"prior-inference-abi") {c.env("LUST_LIBRARY","/lustr/library/lustr");}
        self.event(json!({"kind":"source","case":case,"phase":phase,"sample":sample,"engine":engine,"path":source,"sha256":hash(source)}));
        let (out,seconds)=self.run("compile",case,phase,engine,sample,&mut c);
        let trace_bytes=fs::read(&trace).unwrap_or_default();
        let added=String::from_utf8_lossy(trace_bytes.get(old..).unwrap_or_default()).to_string();
        self.event(json!({"kind":"routing","case":case,"phase":phase,"sample":sample,"engine":engine,"trace":added}));
        if out.status.success() && matches!(engine,"product"|"prior-product") {
            assert!(!added.contains("full-rust-core"),"native benchmark entered compatibility compiler: {added}");
            assert!(added.contains("\"engine\":\"lust\"") || added.contains("\"engine\":\"lustr-native\""));
        }
        let artifact=if engine=="stock" && manifest.is_some() {scope.join("cargo-target/debug/cargo-lust")} else {artifact};
        (out,seconds,artifact)
    }
    fn execute(&mut self,case:&str,phase:&str,engine:&str,sample:usize,artifact:&Path,coordinator:bool,compile:f64) -> Vec<(Option<i32>,Vec<u8>,Vec<u8>)> {
        let cases=if coordinator {vec![vec!["--help"],vec!["--version"],vec!["plan","--frontend-jobs=3","rustc","--offline"],
            vec!["plan","--bluefe-root","/package","build","--offline"],vec!["--frontend-jobs=0","rustc"]]} else {vec![vec![]]};
        let mut seconds=compile; let mut results=Vec::new();
        for args in cases {
            let (out,t)=self.run("runtime",case,phase,engine,sample,Command::new(artifact).args(args));
            seconds+=t; results.push((out.status.code(),out.stdout,out.stderr));
        }
        self.event(json!({"kind":"compile-plus-runtime-suite","case":case,"phase":phase,"engine":engine,"sample":sample,"seconds":seconds}));
        results
    }
}

pub(crate) fn workload(phase:&str,n:usize,inference:bool)->String {
    let body=if phase=="body-edit" {17+n} else {17};
    let signature=if phase=="signature-edit" && n%2==1 {"u32"} else {"u64"};
    let field=if phase=="type-edit" && n%2==1 {"u32"} else {"u64"};
    let amount=if phase=="macro-edit" {41+n} else {41};
    let into=if inference {"let mut xs=Vec::new(); xs.extend([\"flag\".into(), format!(\"value={n}\").into()]); xs"}
        else {"vec![OsString::from(\"flag\"), OsString::from(format!(\"value={n}\"))]"};
    let invalid=if phase=="rejection" {"let _:u32=\"invalid\";"} else {""};
    format!(r#"use std::ffi::OsString;
use std::cell::Cell; use std::rc::Rc; use std::panic::{{catch_unwind, AssertUnwindSafe}};
macro_rules! amount {{ () => {{ {amount}u64 }}; }}
struct Salt {{ value:{field} }}
struct Guard(Rc<Cell<usize>>); impl Drop for Guard {{ fn drop(&mut self) {{ self.0.set(self.0.get()+1); }} }}
fn arguments(n:usize)->Vec<OsString> {{ {into} }}
fn step(value:u64,salt:{signature})->u64 {{ value.wrapping_mul({body}).wrapping_add(salt as u64).wrapping_add(amount!()) }}
fn main() {{ std::panic::set_hook(Box::new(|_|{{}})); let drops=Rc::new(Cell::new(0));
 let salt=Salt{{value:7}}; let mut total=salt.value as u64;
 for i in 0..100 {{ total=step(total,i); }}
 let args=arguments(3); assert_eq!(args.len(),2);
 assert!(catch_unwind(AssertUnwindSafe(||{{let _g=Guard(drops.clone()); panic!("witness");}})).is_err(),"must unwind");
 assert_eq!(drops.get(),1); {invalid} println!("{{args:?}}:{{total}}:{{}}",drops.get());
}}
"#)
}

fn main() {
    let root=PathBuf::from(std::env::args_os().nth(1).expect("new output directory"));
    let mode=std::env::args().nth(2).unwrap_or_default();
    if mode=="--summarize-only" {
        let events:Vec<Value>=fs::read_to_string(root.join("raw.jsonl")).unwrap().lines().map(|s|serde_json::from_str(s).unwrap()).collect();
        summarize(&root,&events); return;
    }
    fs::create_dir(&root).unwrap();
    let mut c=Campaign{raw:File::create(root.join("raw.jsonl")).unwrap(),root,events:Vec::new(),serial:0};
    let mut identities=serde_json::Map::new();
    let paired=std::env::var_os("LUSTR_PHASE_PRIOR_PRODUCT").is_some() && std::env::var_os("LUSTR_PHASE_PRIOR_LIBRARY").is_some();
    if paired {
        for name in ["LUSTR_PHASE_PRIOR_PRODUCT","LUSTR_PHASE_PRIOR_LIBRARY"] {
            let path=PathBuf::from(std::env::var_os(name).unwrap());
            identities.insert(path.to_str().unwrap().into(),json!(hash(&path)));
        }
    }
    for p in [PRODUCT,"/build/candidate/bin/lustr-native-worker","/build/candidate/lib/liblustr_inference_core.so",
        "/build/candidate/lib/liblustr_native_core.so","/build/candidate/lib/native-family.sha256","/build/candidate/lib/lustr-native.id",
        "/core/bin/rustc","/core/bin/cargo","/matched/bin/lust","/matched/bin/lust-core"] {
        identities.insert(p.into(),json!(hash(Path::new(p))));
    }
    c.event(json!({"kind":"identity","files":identities,"samples":SAMPLES,"cpu":"10-13","cargo_jobs":2,
        "profile":"opt0/debug0/unwind/overflow-checks-on/debug-assertions-on; Cargo dev flags logged with -vv",
        "cache":"cold disables native caches and fresh Cargo target+build; OS pages untouched; repeat/noop reuse target; no native daemon",
        "limits":"whole-target native compilation, no Rust rlib interoperability; compile-plus-runtime-suite is not cargo test"}));
    for case in ["subset-control","inference"].into_iter().filter(|_|mode!="--coordinator-only") {
        let dir=c.root.join(case); fs::create_dir(&dir).unwrap(); let source=dir.join("main.rs");
        for sample in 0..SAMPLES {
            for phase in ["cold","repeat","noop","body-edit","signature-edit","type-edit","macro-edit","rejection","recovery"] {
                if !matches!(phase,"repeat"|"noop") {fs::write(&source,workload(phase,sample,case=="inference")).unwrap();}
        let engines=if case=="subset-control" {vec!["matched","product","stock"]}
            else if paired {vec!["prior-product","product","prior-inference-abi","inference-abi","stock"]}
            else {vec!["product","inference-abi","stock"]};
                let engines:Vec<_>=if sample%2==0 {engines} else {engines.into_iter().rev().collect()};
                let mut expected=None;
                for engine in engines {
                    let (out,t,artifact)=c.compile(case,phase,engine,sample,&source,None);
                    if phase=="rejection" {assert!(!out.status.success());continue;}
                    assert!(out.status.success(),"{case}/{phase}/{engine} failed, no retry: {}",String::from_utf8_lossy(&out.stderr));
                    assert!(fs::read(&artifact).unwrap().starts_with(b"\x7fELF"));
                    let results=c.execute(case,phase,engine,sample,&artifact,false,t);
                    assert_eq!(results[0].0,Some(0));
                    if let Some(expected)=&expected {assert_eq!(&results,expected);} else {expected=Some(results);}
                }
            }
        }
    }
    // Only selected manifest/lock and Rust sources, never the user's bench WIP.
    let dir=c.root.join("coordinator"); fs::create_dir(&dir).unwrap(); fs::create_dir(dir.join("src")).unwrap();
    for name in ["Cargo.toml","Cargo.lock"] {fs::copy(Path::new("/lustr/crates/lustr").join(name),dir.join(name)).unwrap();}
    fn copy_sources(source:&Path,dest:&Path) {
        for e in fs::read_dir(source).unwrap() {let e=e.unwrap();let target=dest.join(e.file_name());let kind=e.file_type().unwrap();
            if kind.is_dir() {fs::create_dir(&target).unwrap();copy_sources(&e.path(),&target);}
            else {assert!(kind.is_file());fs::copy(e.path(),target).unwrap();}
        }
    }
    copy_sources(Path::new("/lustr/crates/lustr/src"),&dir.join("src"));
    fn record_sources(c:&mut Campaign,dir:&Path) {
        for e in fs::read_dir(dir).unwrap() {let e=e.unwrap();if e.file_type().unwrap().is_dir() {record_sources(c,&e.path());}
            else {c.event(json!({"kind":"source-input","case":"coordinator","path":e.path(),"sha256":hash(&e.path())}));}
        }
    }
    record_sources(&mut c,&dir);
    let source=dir.join("src/main.rs"); let manifest=dir.join("Cargo.toml");
    for sample in 0..SAMPLES {
        for phase in ["cold","repeat","noop"] {
            let mut engines=if paired {vec!["prior-product","product","stock"]} else {vec!["product","stock"]};
            if sample%2!=0 {engines.reverse();} let mut expected=None;
            for engine in engines {
                let (out,t,artifact)=c.compile("coordinator",phase,engine,sample,&source,Some(&manifest));
                assert!(out.status.success(),"coordinator/{engine} failed, no retry: {}",String::from_utf8_lossy(&out.stderr));
                assert!(fs::read(&artifact).unwrap().starts_with(b"\x7fELF"));
                let results=c.execute("coordinator",phase,engine,sample,&artifact,true,t);
                if let Some(expected)=&expected {assert_eq!(&results,expected);} else {expected=Some(results);}
            }
        }
    }
    summarize(&c.root,&c.events);
    println!("{} raw events; raw/summary retained; all actual runtime comparisons passed",c.events.len());
}

fn summarize(root:&Path,events:&[Value]) {
    let mut groups=std::collections::BTreeMap::<(String,String,String,String),Vec<f64>>::new();
    for e in events {if matches!(e["kind"].as_str(),Some("compile"|"compile-plus-runtime-suite")) {
        groups.entry((e["kind"].as_str().unwrap().into(),e["case"].as_str().unwrap().into(),e["phase"].as_str().unwrap().into(),e["engine"].as_str().unwrap().into()))
            .or_default().push(e["seconds"].as_f64().unwrap());
    }}
    let rows:Vec<_>=groups.into_iter().map(|((kind,case,phase,engine),mut s)|{s.sort_by(f64::total_cmp);json!({"kind":kind,"case":case,"phase":phase,"engine":engine,"n":s.len(),"median_ms":s[s.len()/2]*1000.,"p90_ms":s[(s.len()*9/10).min(s.len()-1)]*1000.,"samples_seconds":s})}).collect();
    fs::write(root.join("summary.json"),serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
}
