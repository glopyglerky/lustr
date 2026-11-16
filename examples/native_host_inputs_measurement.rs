//! Fresh native host-input proof cost and a separate PathBuf language workload,
//! fixed successful direct-native control, and stock oracle. No imported timings.
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

fn owned_platform_workload(phase: &str, n: usize) -> String {
    let previous = previous::workload(phase, n, true);
    let old = "let mut xs=Vec::new(); xs.extend([\"flag\".into(), format!(\"value={n}\").into()]); xs";
    let owned = r#"let source=OsString::from("roundtrip");
 let inferred=source.into(); let path:std::path::PathBuf=inferred;
 let inferred_back=path.into(); let restored:OsString=inferred_back;
 assert_eq!(restored,OsString::from("roundtrip"));
 let mut xs=Vec::new(); xs.extend([std::path::PathBuf::from("flag").into(),std::path::PathBuf::from(format!("value={n}")).into()]); xs"#;
    assert!(previous.contains(old));
    previous.replace(old, owned)
}

fn borrowed_path_workload(phase: &str, n: usize) -> String {
    let previous = previous::workload(phase, n, true)
        .replace("std::ffi::OsString", "std::path::PathBuf").replace("OsString", "PathBuf");
    let old = "let mut xs=Vec::new(); xs.extend([\"flag\".into(), format!(\"value={n}\").into()]); xs";
    let borrowed = r#"let original=PathBuf::from("borrowed-native-path");
 let source=original.as_path(); let inferred=source.into(); let checked_target:PathBuf=inferred;
 assert_eq!(checked_target,original);
 let a=PathBuf::from("flag"); let b=PathBuf::from(format!("value={n}"));
 let source_a=a.as_path(); let source_b=b.as_path();
 let mut xs=Vec::new(); xs.extend([source_a.into(),source_b.into()]); xs"#;
    assert!(previous.contains(old));
    previous.replace(old, borrowed)
}

fn borrowed_os_workload(phase: &str, n: usize) -> String {
    let previous = borrowed_path_workload(phase, n);
    previous.replace("use std::path::PathBuf;", "use std::path::PathBuf; use std::ffi::OsString;")
        .replace("let original=PathBuf::from(\"borrowed-native-path\");", "let original=OsString::from(\"borrowed-native-os\");")
        .replace("original.as_path()", "original.as_os_str()")
        .replace("assert_eq!(checked_target,original);", "assert_eq!(checked_target,PathBuf::from(original));")
        .replace("let a=PathBuf::from", "let a=OsString::from")
        .replace("let b=PathBuf::from", "let b=OsString::from")
        .replace("a.as_path()", "a.as_os_str()")
        .replace("b.as_path()", "b.as_os_str()")
}

fn slice_array_workload(phase: &str, n: usize) -> String {
    let previous = borrowed_os_workload(phase, n);
    let marker = "assert_eq!(checked_target,PathBuf::from(original));";
    assert!(previous.contains(marker));
    previous.replace(marker, r#"assert_eq!(checked_target,PathBuf::from(original));
 use std::os::unix::ffi::OsStrExt;
 assert_eq!(checked_target.as_os_str().as_bytes(), b"borrowed-native-os");
 assert_eq!(b"borrowed-native-os",checked_target.as_os_str().as_bytes());
 assert_ne!(checked_target.as_os_str().as_bytes(), b"other");"#)
}

fn slice_array_place_workload(phase: &str, n: usize) -> String {
    slice_array_workload(phase, n)
        .replace("assert_eq!(checked_target.as_os_str().as_bytes(), b\"borrowed-native-os\");", "let expected=b\"borrowed-native-os\"; let other=b\"other\"; assert_eq!(checked_target.as_os_str().as_bytes(), expected);")
        .replace("assert_eq!(b\"borrowed-native-os\",checked_target.as_os_str().as_bytes());", "assert_eq!(expected,checked_target.as_os_str().as_bytes());")
        .replace("assert_ne!(checked_target.as_os_str().as_bytes(), b\"other\");", "assert_ne!(checked_target.as_os_str().as_bytes(), other);")
}

fn long_assert_workload(phase: &str, n: usize) -> String {
    let previous = slice_array_place_workload(phase, n);
    let witness = r#"let long_drops=Rc::new(Cell::new(0));
 let failed=catch_unwind(AssertUnwindSafe(||long_assert_failure(long_drops.clone())));
 assert!(failed.is_err(),"long assertion must unwind"); assert_eq!(long_drops.get(),1);"#;
    let payload = if phase == "macro-edit" { n } else { 1 };
    format!("{}\nfn long_assert_failure(drops:Rc<Cell<usize>>){{assert!(catch_unwind(AssertUnwindSafe(||{{let _guard=Guard(drops.clone());println!(\"long payload {{}}\",{payload}u8);}})).is_err());}}\n",
        previous.replace("let args=arguments(3);", &format!("{witness}\n let args=arguments(3);")))
}

fn parenthesized_assert_workload(phase: &str, n: usize) -> String {
    let source=long_assert_workload(phase,n);
    let marker="\nfn long_assert_failure(drops:Rc<Cell<usize>>){";
    let (prior, function)=source.split_once(marker).expect("one appended assertion function");
    assert!(!function.contains(marker));
    let open="assert!(catch_unwind(AssertUnwindSafe(||{";
    let close="})).is_err());}";
    assert_eq!(function.matches(open).count(),1);
    assert_eq!(function.matches(close).count(),1);
    format!("{prior}{marker}{}",function
        .replace(open,"assert!(((catch_unwind(AssertUnwindSafe(((||{")
        .replace(close,"})))).is_err())));}"))
}

fn binary_assert_workload(phase: &str, n: usize) -> String {
    let source=parenthesized_assert_workload(phase,n);
    let marker="\nfn long_assert_failure(drops:Rc<Cell<usize>>){";
    let (prior,function)=source.split_once(marker).expect("one appended assertion function");
    assert!(!function.contains(marker));
    let suffix=");}\n";
    let condition=function.strip_suffix(suffix).expect("final appended assertion terminator");
    format!("{prior}{marker}{condition} && true{suffix}")
}

fn struct_assert_workload(phase: &str, n: usize) -> String {
    let source=binary_assert_workload(phase,n);
    assert_eq!(source.matches(" && true);}").count(),1);
    format!("{}\nstruct AssertionHolder{{value:bool}}\n", source.replace(" && true);}"," && (AssertionHolder { value: true }).value);}"))
}

fn tuple_assert_workload(phase: &str, n: usize) -> String {
    let source=struct_assert_workload(phase,n);
    assert_eq!(source.matches("(AssertionHolder { value: true }).value").count(),1);
    format!("{}\nstruct AssertionTuple(bool,u8);\n",source.replace("(AssertionHolder { value: true }).value","(AssertionTuple { 0: true, 1: 2u8 }).0"))
}

fn tuple_literal_assert_workload(phase: &str, n: usize) -> String {
    let source=tuple_assert_workload(phase,n);
    let marker="(AssertionTuple { 0: true, 1: 2u8 }).0";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"((true,2u8)).0")
}

fn array_literal_assert_workload(phase: &str, n: usize) -> String {
    let source=tuple_literal_assert_workload(phase,n);
    let marker="((true,2u8)).0";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"([true,false])[0]")
}

fn array_repeat_assert_workload(phase: &str, n: usize) -> String {
    let source=array_literal_assert_workload(phase,n);
    let marker="([true,false])[0]";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"([true;2])[0]")
}

fn unsafe_local_const_fn_workload(phase: &str, n: usize) -> String {
    let source=unsafe_nested_fn_assert_workload(phase,n);
    let marker="{fn safe_value()->bool{true}";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"{const fn safe_value()->bool{true}")
}

fn unsafe_nested_fn_assert_workload(phase: &str, n: usize) -> String {
    let source=unsafe_call_block_assert_workload(phase,n);
    let marker="fn long_assert_failure(drops:Rc<Cell<usize>>){";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"fn long_assert_failure(drops:Rc<Cell<usize>>){fn safe_value()->bool{true}")
}

fn unsafe_call_block_assert_workload(phase: &str, n: usize) -> String {
    let source=unsafe_block_assert_workload(phase,n);
    let marker="unsafe {true}";
    assert_eq!(source.matches(marker).count(),1);
    format!("{}\nfn safe_value()->bool{{true}}\n",source.replace(marker,"unsafe {safe_value()}"))
}

fn unsafe_let_block_assert_workload(phase: &str, n: usize) -> String {
    let source=unsafe_block_assert_workload(phase,n);
    let marker="unsafe {true}";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"unsafe {let x=true;x}")
}

fn unsafe_block_assert_workload(phase: &str, n: usize) -> String {
    let source=array_repeat_assert_workload(phase,n);
    let marker="([true;2])[0]";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"([unsafe {true};2])[0]")
}

fn const_block_assert_workload(phase: &str, n: usize) -> String {
    let source=array_repeat_assert_workload(phase,n);
    let marker="([true;2])[0]";
    assert_eq!(source.matches(marker).count(),1);
    source.replace(marker,"([const {true};2])[0]")
}

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
    let replay_compare = std::env::args().nth(2).as_deref() == Some("--replay-compare");
    let owned_platform = std::env::args().nth(2).as_deref() == Some("--owned-platform");
    let borrowed_path = std::env::args().nth(2).as_deref() == Some("--borrowed-path");
    let borrowed_os = std::env::args().nth(2).as_deref() == Some("--borrowed-os");
    let slice_array = std::env::args().nth(2).as_deref() == Some("--slice-array");
    let slice_array_places = std::env::args().nth(2).as_deref() == Some("--slice-array-places");
    let unsafe_local_const_fn = std::env::args().nth(2).as_deref() == Some("--unsafe-local-const-fn");
    let unsafe_nested_fn_assert = unsafe_local_const_fn || std::env::args().nth(2).as_deref() == Some("--unsafe-nested-fn-assert");
    let unsafe_call_block_assert = unsafe_nested_fn_assert || std::env::args().nth(2).as_deref() == Some("--unsafe-call-block-assert");
    let unsafe_let_block_assert = unsafe_call_block_assert || std::env::args().nth(2).as_deref() == Some("--unsafe-let-block-assert");
    let unsafe_block_assert = unsafe_let_block_assert || std::env::args().nth(2).as_deref() == Some("--unsafe-block-assert");
    let const_block_assert = unsafe_block_assert || std::env::args().nth(2).as_deref() == Some("--const-block-assert");
    let array_repeat_assert = const_block_assert || std::env::args().nth(2).as_deref() == Some("--array-repeat-assert");
    let array_literal_assert = array_repeat_assert || std::env::args().nth(2).as_deref() == Some("--array-literal-assert");
    let tuple_literal_assert = array_literal_assert || std::env::args().nth(2).as_deref() == Some("--tuple-literal-assert");
    let tuple_assert = tuple_literal_assert || std::env::args().nth(2).as_deref() == Some("--tuple-assert");
    let struct_assert = tuple_assert || std::env::args().nth(2).as_deref() == Some("--struct-assert");
    let binary_assert = struct_assert || std::env::args().nth(2).as_deref() == Some("--binary-assert");
    let parenthesized_assert = binary_assert || std::env::args().nth(2).as_deref() == Some("--parenthesized-assert");
    let long_assert = parenthesized_assert || std::env::args().nth(2).as_deref() == Some("--long-assert");
    let pathbuf = long_assert || slice_array_places || replay_compare || owned_platform || borrowed_path || borrowed_os || slice_array || std::env::args().nth(2).as_deref() == Some("--pathbuf");
    let direct_library = if replay_compare { "/new/liblustr_native_core.so".into() } else if pathbuf { std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap() } else { "/inference/liblustr_native_core.so".into() };
    let retained_library = std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap();
    let observer = std::env::var_os("LUSTR_TCC_INPUT_OBSERVER").unwrap();
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("sources")).unwrap();
    let mut c = Campaign { raw: File::create(root.join("raw.jsonl")).unwrap(), root, events: Vec::new(), seq: 0 };
    let mut identities = serde_json::Map::new();
    let harness = std::env::current_exe().unwrap();
    identities.insert(harness.to_str().unwrap().into(), json!(hash(&harness)));
    for p in [
        "/current/native-retained-probe",
        "/inference/native-phase-probe",
        "/tcc/lib/libtcc.a",
        "/tcc/bin/tcc",
        "/core/bin/rustc",
        "/core/bin/cargo",
        "/lustr/examples/native_host_inputs_measurement.rs",
        "/lustr/examples/native_inference_measurement.rs",
        "/lustr/tests/native_reuse.rs",
    ] {
        identities.insert(p.into(), json!(hash(Path::new(p))));
    }
    for p in [&retained_library, &direct_library, &observer, &"/new/liblustr_tcc_inputs.so".into()] {
        identities.insert(p.to_str().unwrap().into(), json!(hash(Path::new(p))));
    }
    let ns = Path::new(&retained_library).with_file_name("lustr-native.id");
    identities.insert(ns.to_str().unwrap().into(), json!(hash(&ns)));
    let direct_ns = Path::new(&direct_library).with_file_name("lustr-native.id");
    identities.insert(direct_ns.to_str().unwrap().into(), json!(hash(&direct_ns)));
    c.event(json!({"kind":"identities","files":identities,"samples":9,"cpu":"10-13","jobs":2,"compiler":if unsafe_local_const_fn { "owned native local const-fn header and scope integration: SAME successful57 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting56 or distinct sources" } else if unsafe_nested_fn_assert { "owned native function statement items and lexical scopes: SAME successful56 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting55 or distinct sources" } else if unsafe_call_block_assert { "owned original-context safe Rust call checking: SAME successful55 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting54 or distinct-source timings" } else if unsafe_let_block_assert { "owned lexical boolean unsafe-body checking: SAME successful54 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting53 or distinct-source timings" } else if unsafe_block_assert { "owned original-token unsafe-block frontend: SAME successful53 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting52 or distinct-source timings" } else if const_block_assert { "original-AST const blocks: SAME successful52 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting51 or distinct-source timings" } else if array_repeat_assert { "original-AST array repeats: SAME successful51 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting50 or distinct-source timings" } else if array_literal_assert { "original-AST array literals: SAME successful50 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting49 or distinct-source timings" } else if tuple_literal_assert { "original-AST tuple literals: SAME successful49 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting48 or distinct-source timings" } else if tuple_assert { "original-AST tuple fields: SAME successful48 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting47 or distinct-source timings" } else if struct_assert { "original-AST struct expressions: SAME successful47 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting46 or distinct-source timings" } else if binary_assert { "original-AST binary expressions: SAME successful46 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting45 or distinct-source timings" } else if parenthesized_assert { "original-token nested parentheses: SAME successful45 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting44 or distinct-source timings" } else if long_assert { "original-AST long assertion text: SAME successful44 direct/retained and stock; optimized opt3/CGU1/ThinLTO; no comparison with rejecting43 or earlier distinct sources" } else if slice_array_places { "checked shared slice/array PLACE assertion expansion: SAME successful candidate direct and retained plus stock; optimized opt3/CGU1/ThinLTO; no comparison to rejecting42 or distinct-source earlier timings" } else if slice_array { "checked rvalue shared slice/array assertion expansion: SAME successful candidate direct and retained plus stock; optimized opt3/CGU1/ThinLTO; no comparison to rejecting prior component" } else if borrowed_os { "shared OsStr expansion: SAME successful candidate direct and retained plus stock; optimized opt3/CGU1/ThinLTO; no comparison to rejecting prior component" } else if borrowed_path { "shared Path expansion: SAME successful candidate direct and retained plus stock; optimized opt3/CGU1/ThinLTO; no comparison to rejecting prior component" } else if owned_platform { "owned platform expansion: SAME successful candidate direct and retained plus stock; optimized opt3/CGU1/ThinLTO; no comparison to rejecting prior component" } else if replay_compare { "fixed37 direct and retained vs candidate38 retained two-unit replay and stock; optimized opt3/CGU1/ThinLTO; primary body-edit median >=10% reduction" } else if pathbuf { "new PathBuf component retained vs SAME fixed successful direct component; optimized opt3/CGU1/ThinLTO" } else { "component36 retained vs fixed successful component35 direct; both optimized opt3/CGU1/ThinLTO" },"application":"opt0/debug0/unwind/checks-on","cache":"new bounded process per sample; actual host-file/C-unit replay proof on misses; retained response artifact proof within process; prior direct is one-shot; native disk caches disabled BEFORE Check; stock Cargo target and build fresh per sample and retained for no-op; OS pages untouched","limits":"pinned Linux GNU x86_64 compiler file-input scope, stable files DURING request; runtime-loader closure, product lifecycle, native rlib, general Cargo and full Rust remain open; compile-plus-runtime is not Cargo test"}));
    for case in ["mixed", "coordinator"].into_iter().filter(|case| !pathbuf || *case == "mixed") {
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
            let init = Instant::now();
            let mut baseline = replay_compare.then(|| support::Probe::new_with_library(
                &scope.join("baseline37-probe"), None,
                &[("LUSTR_TCC_INPUT_OBSERVER", "/new/liblustr_tcc_inputs.so")], Path::new("/new/liblustr_native_core.so")));
            let baseline_startup = init.elapsed().as_secs_f64() * 1000.;
            let phases = if case == "coordinator" { &PHASES[..3] } else { PHASES };
            for (phase_index, &phase) in phases.iter().enumerate() {
                if case == "mixed" && !matches!(phase, "repeat" | "noop") {
                    let mut source_text = if unsafe_local_const_fn { unsafe_local_const_fn_workload(phase, 2 * sample + 1) } else if unsafe_nested_fn_assert { unsafe_nested_fn_assert_workload(phase, 2 * sample + 1) } else if unsafe_call_block_assert { unsafe_call_block_assert_workload(phase, 2 * sample + 1) } else if unsafe_let_block_assert { unsafe_let_block_assert_workload(phase, 2 * sample + 1) } else if unsafe_block_assert { unsafe_block_assert_workload(phase, 2 * sample + 1) } else if const_block_assert { const_block_assert_workload(phase, 2 * sample + 1) } else if array_repeat_assert { array_repeat_assert_workload(phase, 2 * sample + 1) } else if array_literal_assert { array_literal_assert_workload(phase, 2 * sample + 1) } else if tuple_literal_assert { tuple_literal_assert_workload(phase, 2 * sample + 1) } else if tuple_assert { tuple_assert_workload(phase, 2 * sample + 1) } else if struct_assert { struct_assert_workload(phase, 2 * sample + 1) } else if binary_assert { binary_assert_workload(phase, 2 * sample + 1) } else if parenthesized_assert { parenthesized_assert_workload(phase, 2 * sample + 1) } else if long_assert { long_assert_workload(phase, 2 * sample + 1) } else if slice_array_places { slice_array_place_workload(phase, 2 * sample + 1) } else if slice_array { slice_array_workload(phase, 2 * sample + 1) } else if borrowed_os { borrowed_os_workload(phase, 2 * sample + 1) } else if borrowed_path { borrowed_path_workload(phase, 2 * sample + 1) } else if owned_platform { owned_platform_workload(phase, 2 * sample + 1) } else { previous::workload(phase, 2 * sample + 1, true) };
                    if pathbuf && !owned_platform && !borrowed_path && !borrowed_os && !slice_array && !slice_array_places && !long_assert {
                        // Generate the separate PathBuf workload; this is not
                        // an adapter or a rewrite performed by the compiler.
                        source_text = source_text.replace("std::ffi::OsString", "std::path::PathBuf").replace("OsString", "PathBuf");
                    }
                    fs::write(&source, source_text).unwrap();
                }
                let source_hash = hash(&source);
                let snapshot = c.root.join("sources").join(format!("{case}-{sample}-{phase}.rs"));
                fs::copy(&source, &snapshot).unwrap();
                c.event(json!({"kind":"source","case":case,"sample":sample,"phase":phase,"path":source,"snapshot":snapshot,"sha256":source_hash}));
                let engines = if replay_compare {
                    let mut arms = vec!["direct37", "baseline37-retained", "candidate38-retained", "stock"];
                    arms.rotate_left((sample + phase_index) % 4);
                    arms
                } else {
                    let mut arms = if unsafe_local_const_fn { vec!["constfn57-direct", "constfn57-retained", "stock"] } else if unsafe_nested_fn_assert { vec!["item56-direct", "item56-retained", "stock"] } else if unsafe_call_block_assert { vec!["call55-direct", "call55-retained", "stock"] } else if unsafe_let_block_assert { vec!["unsafe54-direct", "unsafe54-retained", "stock"] } else if unsafe_block_assert { vec!["unsafe53-direct", "unsafe53-retained", "stock"] } else if const_block_assert { vec!["const52-direct", "const52-retained", "stock"] } else if array_repeat_assert { vec!["repeat51-direct", "repeat51-retained", "stock"] } else if array_literal_assert { vec!["array50-direct", "array50-retained", "stock"] } else if tuple_literal_assert { vec!["tuple49-direct", "tuple49-retained", "stock"] } else if tuple_assert { vec!["tuple48-direct", "tuple48-retained", "stock"] } else if struct_assert { vec!["struct47-direct", "struct47-retained", "stock"] } else if binary_assert { vec!["binary46-direct", "binary46-retained", "stock"] } else if parenthesized_assert { vec!["paren45-direct", "paren45-retained", "stock"] } else if long_assert { vec!["text44-direct", "text44-retained", "stock"] } else if slice_array_places { vec!["place43-direct", "place43-retained", "stock"] } else if slice_array { vec!["assert42-direct", "assert42-retained", "stock"] } else if borrowed_os { vec!["borrowed41-direct", "borrowed41-retained", "stock"] } else if borrowed_path { vec!["borrowed40-direct", "borrowed40-retained", "stock"] } else if owned_platform { vec!["owned39-direct", "owned39-retained", "stock"] } else { vec!["prior-direct", "retained-abi", "stock"] };
                    arms.rotate_left((sample + phase_index) % 3);
                    arms
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
                    let (code, ms) = if matches!(engine, "retained-abi" | "baseline37-retained" | "candidate38-retained" | "owned39-retained" | "borrowed40-retained" | "borrowed41-retained" | "assert42-retained" | "place43-retained" | "text44-retained" | "paren45-retained" | "binary46-retained" | "struct47-retained" | "tuple48-retained" | "tuple49-retained" | "array50-retained" | "repeat51-retained" | "const52-retained" | "unsafe53-retained" | "unsafe54-retained" | "call55-retained" | "item56-retained" | "constfn57-retained") {
                        let start = Instant::now();
                        let (active, start_cost) = if engine == "baseline37-retained" { (baseline.as_mut().unwrap(), baseline_startup) } else { (&mut probe, startup) };
                        let a = active.request(&args, &[]);
                        let ms = start.elapsed().as_secs_f64() * 1000. + if phase == "cold" { start_cost } else { 0. };
                        let expected_hit = matches!(phase, "repeat" | "noop");
                        assert_eq!(a.hit(), expected_hit, "{case}/{phase}: {}", String::from_utf8_lossy(&a.err));
                        c.event(json!({"kind":"compile","case":case,"sample":sample,"phase":phase,"engine":engine,"ms":ms,"abi_call_ms":a.ns as f64/1e6,"code":a.code,"hit":a.hit(),"args":args}));
                        (Some(a.code), ms)
                    } else {
                        let mut command = if matches!(engine, "prior-direct" | "direct37" | "owned39-direct" | "borrowed40-direct" | "borrowed41-direct" | "assert42-direct" | "place43-direct" | "text44-direct" | "paren45-direct" | "binary46-direct" | "struct47-direct" | "tuple48-direct" | "tuple49-direct" | "array50-direct" | "repeat51-direct" | "const52-direct" | "unsafe53-direct" | "unsafe54-direct" | "call55-direct" | "item56-direct" | "constfn57-direct") {
                            let mut x = Command::new("/inference/native-phase-probe");
                            x.arg(&direct_library).args(&args);
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
            if let Some(baseline) = baseline { baseline.finish(); }
        }
    }
    let mut rows = Vec::new();
    for case in ["mixed", "coordinator"] {
        for phase in PHASES {
            for engine in ["prior-direct", "retained-abi", "direct37", "baseline37-retained", "candidate38-retained", "owned39-direct", "owned39-retained", "borrowed40-direct", "borrowed40-retained", "borrowed41-direct", "borrowed41-retained", "assert42-direct", "assert42-retained", "place43-direct", "place43-retained", "text44-direct", "text44-retained", "paren45-direct", "paren45-retained", "binary46-direct", "binary46-retained", "struct47-direct", "struct47-retained", "tuple48-direct", "tuple48-retained", "tuple49-direct", "tuple49-retained", "array50-direct", "array50-retained", "repeat51-direct", "repeat51-retained", "const52-direct", "const52-retained", "unsafe53-direct", "unsafe53-retained", "unsafe54-direct", "unsafe54-retained", "call55-direct", "call55-retained", "item56-direct", "item56-retained", "constfn57-direct", "constfn57-retained", "stock"] {
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
                    rows.push(json!({"case":case,"phase":phase,"engine":engine,"kind":kind,"median_ms":times[4],"max_ms":times[8],"samples_ms":times}));
                }
            }
        }
    }
    if array_repeat_assert {
        assert_eq!(rows.len(), 51);
        for phase in PHASES {
            for engine in if unsafe_local_const_fn { ["constfn57-direct", "constfn57-retained", "stock"] } else if unsafe_nested_fn_assert { ["item56-direct", "item56-retained", "stock"] } else if unsafe_call_block_assert { ["call55-direct", "call55-retained", "stock"] } else if unsafe_let_block_assert { ["unsafe54-direct", "unsafe54-retained", "stock"] } else if unsafe_block_assert { ["unsafe53-direct", "unsafe53-retained", "stock"] } else if const_block_assert { ["const52-direct", "const52-retained", "stock"] } else { ["repeat51-direct", "repeat51-retained", "stock"] } {
                for kind in ["compile", "compile-plus-runtime-suite"] {
                    let events: Vec<_> = c.events.iter().filter(|v| v["case"] == "mixed"
                        && v["phase"] == *phase && v["engine"] == engine && v["kind"] == kind).collect();
                    if *phase == "rejection" && kind == "compile-plus-runtime-suite" {
                        assert!(events.is_empty());
                    } else {
                        assert_eq!(events.len(), 9);
                        let samples: std::collections::BTreeSet<_> = events.iter()
                            .map(|v| v["sample"].as_u64().unwrap()).collect();
                        assert_eq!(samples, (0..9).collect());
                    }
                }
            }
        }
    }
    fs::write(c.root.join("summary.json"), serde_json::to_vec_pretty(&json!({"rows":rows,"events":c.events.len()})).unwrap()).unwrap();
    println!("Successful fresh nine-sample host-proof/direct/stock campaign (PathBuf={pathbuf}, owned-platform={owned_platform}, borrowed-path={borrowed_path}, borrowed-os={borrowed_os}, slice-array={slice_array}, slice-array-places={slice_array_places}): {} raw events", c.events.len());
}
