//! Real retained-ABI/cold-native/Rust-oracle qualification. No fallback.
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

pub(crate) struct Answer {
    pub code: i32,
    pub out: Vec<u8>,
    pub err: Vec<u8>,
    pub ns: u128,
}

#[test]
#[ignore = "requires native qualified local items and cold native/oracle controls"]
fn unsafe_local_const_fn_reuse_tracks_qualifiers_const_checks_and_recovery() {
    let original=include_str!("fixtures/native-inference-unsafe-local-const-fn-original.rs");
    let local="const fn safe_value()->bool{false}";
    platform_inference_edits("unsafe-local-const-fn-items",original,&[
        original.replacen(local,"fn safe_value()->bool{false}",1),
        original.replacen(local,"const fn safe_value()->bool{id(false)} const fn id(x:bool)->bool{x}",1),
        original.replace("safe_value","different_const_item"),
        original.replace("once {{raw braces}} λ","changed {{braces}} αβγ"),
    ]);
    let r=root("unsafe-local-const-fn-rejections");let project=r.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");let out=r.join("program");let request=args(&source,&out);let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for text in[
        original.replacen(local,"const fn safe_value()->bool{println!(\"not const\");false}",1),
        original.replacen(local,"const fn safe_value()->bool{let _owned=String::new();false}",1),
        original.replacen(local,"let captured=false;const fn safe_value()->bool{captured}",1),
        original.replacen(local,"const fn safe_value()->u8{0}",1),
    ]{
        fs::write(&source,text).unwrap();let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        for(label,mut c)in[("cold",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());c.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&r.join(format!("cold-{}",p.seq))));c}),("oracle",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());c.arg(&source).arg("--edition=2024").arg("-o").arg(r.join(format!("oracle-{}",p.seq)));c})]{let result=c.output().unwrap();fs::write(r.join(format!("{label}-{}.stderr",p.seq)),&result.stderr).unwrap();fs::write(r.join(format!("{label}-{}.status",p.seq)),format!("{}\n",result.status)).unwrap();assert_eq!(result.status.code(),Some(1));assert!(!r.join(format!("{label}-{}",p.seq)).exists());}
        fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}

#[test]
#[ignore = "requires native nested-item integration and cold native/oracle controls"]
fn unsafe_nested_fn_reuse_tracks_item_scope_changes_and_recovery() {
    let original=include_str!("fixtures/native-inference-unsafe-nested-fn-assert-original.rs");
    platform_inference_edits("unsafe-nested-fn-items",original,&[
        original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{println!(\"local item\");false}",1),
        original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{id(false)} fn id(x:bool)->bool{x}",1),
        original.replace("safe_value","different_item"),
        original.replace("once {{raw braces}} λ","changed {{braces}} αβγ"),
    ]);
    let r=root("unsafe-nested-fn-rejections");let project=r.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");let out=r.join("program");let request=args(&source,&out);let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for text in[
        original.replacen("fn safe_value()->bool{false}","fn safe_value()->bool{false} fn safe_value()->bool{true}",1),
        original.replacen("fn safe_value()->bool{false}","let captured=false;fn safe_value()->bool{captured}",1),
        original.replacen("fn safe_value()->bool{false}","fn safe_value()->u8{0}",1),
        original.replace("unsafe {safe_value()}","unsafe {let safe_value=false;safe_value()}"),
    ]{
        fs::write(&source,text).unwrap();let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        for(label,mut c)in[("cold",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());c.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&r.join(format!("cold-{}",p.seq))));c}),("oracle",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());c.arg(&source).arg("--edition=2024").arg("-o").arg(r.join(format!("oracle-{}",p.seq)));c})]{let result=c.output().unwrap();fs::write(r.join(format!("{label}-{}.stderr",p.seq)),&result.stderr).unwrap();fs::write(r.join(format!("{label}-{}.status",p.seq)),format!("{}\n",result.status)).unwrap();assert_eq!(result.status.code(),Some(1));assert!(!r.join(format!("{label}-{}",p.seq)).exists());}
        fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}

#[test]
#[ignore = "requires native call/conversion composition and cold native/oracle controls"]
fn unsafe_call_and_delayed_conversion_reuse_rejects_moves_and_recovers() {
    let base=include_str!("fixtures/native-inference-owned-path.rs");
    let original=format!("{}\nfn safe_value()->bool{{true}}\n",base.replace("fn value() -> PathBuf {","fn value() -> PathBuf {let _boolean=unsafe {safe_value()};"));
    platform_inference_edits("unsafe-call-conversion-edits",&original,&[
        original.replace("fn value() -> PathBuf {","fn value() -> PathBuf {fn safe_value()->bool{true}"),
        original.replace("fn value() -> PathBuf {","fn value() -> PathBuf {const fn safe_value()->bool{true}"),
        original.replace("safe_value","other_value"),
        original.replace("unsafe {safe_value()}","unsafe {let x:bool=safe_value();x}"),
        original.replace("owned-native-path","changed-native-path"),
    ]);
    let r=root("unsafe-call-conversion-rejections");let project=r.join("project");fs::create_dir(&project).unwrap();let source=project.join("main.rs");let out=r.join("program");let request=args(&source,&out);let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,&original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for text in [original.replace("let inferred = source.into();","drop(source);let inferred = source.into();"),original.replace("let _boolean=unsafe","let safe_value=false;let _boolean=unsafe")] {
        fs::write(&source,text).unwrap();let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        for(label,mut command)in[("cold",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());c.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&r.join(format!("cold-{}",p.seq))));c}),("oracle",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());c.arg(&source).arg("--edition=2024").arg("-o").arg(r.join(format!("oracle-{}",p.seq)));c})]{let result=command.output().unwrap();fs::write(r.join(format!("{label}-{}.stderr",p.seq)),&result.stderr).unwrap();fs::write(r.join(format!("{label}-{}.status",p.seq)),format!("{}\n",result.status)).unwrap();assert_eq!(result.status.code(),Some(1));assert!(!r.join(format!("{label}-{}",p.seq)).exists());}
        fs::write(&source,&original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}

#[test]
#[ignore = "requires native safe-call target checking and cold native/oracle controls"]
fn unsafe_call_block_reuse_tracks_target_safety_abi_and_recovery() {
    let original=include_str!("fixtures/native-inference-unsafe-call-block-assert-original.rs");
    platform_inference_edits("unsafe-call-block-assert-text",original,&[
        original.replace("unsafe {safe_value()}","unsafe {let x:bool=safe_value();x}"),
        original.replace("unsafe {safe_value()}","unsafe {let x=safe_value();unsafe {x}}"),
        original.replace("fn safe_value()->bool{false}","fn safe_value()->bool{let x=false;x}"),
        original.replace("safe_value","other_value"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
    ]);
    let r=root("unsafe-call-target-edits");let project=r.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");let out=r.join("program");let request=args(&source,&out);let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for text in [
        original.replace("fn safe_value()->bool{false}","unsafe fn safe_value()->bool{false}"),
        original.replace("fn safe_value()->bool{false}","extern \"C\" fn safe_value()->bool{false}"),
        original.replace("fn safe_value()->bool{false}","fn safe_value()->u8{0}"),
        original.replace("unsafe {safe_value()}","unsafe {let safe_value=false;safe_value()}"),
    ] {
        fs::write(&source,text).unwrap();let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        let cold_out=r.join(format!("rejected-{}",p.seq));let cold=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap()).arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&cold_out)).output().unwrap();
        fs::write(r.join(format!("cold-{}.stderr",p.seq)),&cold.stderr).unwrap();assert_eq!(cold.status.code(),Some(1));assert!(!cold_out.exists());
        fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}

#[test]
#[ignore = "requires owned unsafe-body lexical checking and cold native/oracle controls"]
fn unsafe_let_block_assertion_reuse_tracks_binding_edits_and_recovery() {
    let original=include_str!("fixtures/native-inference-unsafe-let-block-assert-original.rs");
    platform_inference_edits("unsafe-let-block-assert-text",original,&[
        original.replace("unsafe {let x=false;x}","unsafe {let x:bool=false;let y=x;y}"),
        original.replace("unsafe {let x=false;x}","unsafe {let x=false;{let x=true;x};x}"),
        original.replace("unsafe {let x=false;x}","unsafe {let x=false;unsafe {let y=x;y}}"),
        original.replace("unsafe {let x=false;x}","unsafe {let x=unsafe {let y=false;y};x}"),
        original.replace("unsafe {let x=false;x}","unsafe {let mut x:bool=unsafe {false};x}"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
    ]);
    let r=root("unsafe-let-block-rejections");let project=r.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");let out=r.join("program");let request=args(&source,&out);
    let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for value in ["unsafe {let x=x;x}","unsafe {{let x=false;x};x}","unsafe {let x:u8=false;x}"] {
        fs::write(&source,original.replace("unsafe {let x=false;x}",value)).unwrap();
        let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        for (label,mut command) in [
            ("cold",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());c.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&r.join(format!("cold-{}",p.seq))));c}),
            ("oracle",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());c.arg(&source).arg("--edition=2024").arg("-o").arg(r.join(format!("oracle-{}",p.seq)));c}),
        ] {
            fs::write(r.join(format!("{label}-{}.command",p.seq)),format!("{command:?}\n")).unwrap();
            let result=command.output().unwrap();fs::write(r.join(format!("{label}-{}.stderr",p.seq)),&result.stderr).unwrap();
            fs::write(r.join(format!("{label}-{}.status",p.seq)),format!("{}\n",result.status)).unwrap();assert_eq!(result.status.code(),Some(1));
            assert!(!r.join(format!("{label}-{}",p.seq)).exists());
        }
        fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}

#[test]
#[ignore = "requires the owned unsafe frontend and cold native/oracle controls"]
fn unsafe_block_assertion_reuse_tracks_original_tokens_types_and_recovery() {
    let original=include_str!("fixtures/native-inference-unsafe-block-assert-original.rs");
    platform_inference_edits("unsafe-block-assert-text",original,&[
        original.replace("unsafe {false}","unsafe {unsafe {false}}"),
        original.replace("unsafe {false}","unsafe {{false}}"),
        original.replace("unsafe {false}","unsafe /*keyword*/ {/*body*/false}"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// unsafe source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
    let r=root("unsafe-block-rejections"); let project=r.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");let out=r.join("program");let request=args(&source,&out);
    let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for value in ["unsafe {1u8}","unsafe {true(())}"] {
        fs::write(&source,format!("{}\nfn safe_value()->bool{{false}}\n",original.replace("unsafe {false}",value))).unwrap();
        let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        let cold=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap()).arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&r.join(format!("rejected-{}",p.seq)))).output().unwrap();
        fs::write(r.join(format!("cold-{}.stderr",p.seq)),&cold.stderr).unwrap();assert_eq!(cold.status.code(),Some(1));
        fs::write(&source,original).unwrap();accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}
impl Answer {
    pub fn hit(&self) -> bool {
        String::from_utf8_lossy(&self.err).contains("verified no-op artifact reuse")
    }
}
pub(crate) struct Probe {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    root: PathBuf,
    seq: usize,
    env: Vec<(OsString, OsString)>,
}
impl Probe {
    pub(crate) fn new(root: &Path, runtime: Option<&Path>) -> Self {
        Self::new_with_env(root, runtime, &[])
    }
    fn new_with_env(root: &Path, runtime: Option<&Path>, extra: &[(&str, &str)]) -> Self {
        Self::new_with_library(root, runtime, extra, Path::new(&std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()))
    }
    pub(crate) fn new_with_library(root: &Path, runtime: Option<&Path>, extra: &[(&str, &str)], library: &Path) -> Self {
        fs::create_dir_all(root).unwrap();
        let mut command = Command::new(std::env::var_os("LUSTR_REUSE_PROBE").unwrap());
        command
            .arg(library)
            .env(
                "LUSTR_INFERENCE_REUSE_ROOTS",
                runtime.map(|r| r.join("lust/library")).unwrap_or_else(|| PathBuf::from("/lustr/compiler/native/library")),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(fs::File::create(root.join("probe.stderr")).unwrap());
        if let Some(r) = runtime {
            command.env("LUST_LIBRARY", r.join("lustr"));
        }
        command.envs(extra.iter().copied());
        fs::write(root.join("probe.command"), format!("{command:?}\n")).unwrap();
        let mut child = command.spawn().unwrap();
        Self {
            input: child.stdin.take(),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            root: root.into(),
            seq: 0,
            env: std::env::vars_os().collect(),
        }
    }
    pub(crate) fn request(&mut self, args: &[String], patch: &[(&str, Option<&str>)]) -> Answer {
        let mut env = self.env.clone();
        for (name, value) in patch {
            env.retain(|(k, _)| k != name);
            if let Some(value) = value {
                env.push(((*name).into(), (*value).into()));
            }
        }
        let flat: Vec<_> = env.iter().flat_map(|(k, v)| [k.to_str().unwrap().to_owned(), v.to_str().unwrap().to_owned()]).collect();
        let input = self.input.as_mut().unwrap();
        for fields in [args, &flat] {
            writeln!(input, "{}", fields.len()).unwrap();
            for value in fields {
                writeln!(input, "{}", value.len()).unwrap();
                input.write_all(value.as_bytes()).unwrap();
            }
        }
        input.flush().unwrap();
        let mut header = String::new();
        assert!(self.output.read_line(&mut header).unwrap() > 0, "probe exited: {:?}", self.child.try_wait());
        let parts: Vec<_> = header.split_whitespace().collect();
        assert_eq!(parts.len(), 4, "{header}");
        let mut out = vec![0; parts[1].parse().unwrap()];
        let mut err = vec![0; parts[2].parse().unwrap()];
        self.output.read_exact(&mut out).unwrap();
        self.output.read_exact(&mut err).unwrap();
        self.seq += 1;
        let stem = self.root.join(format!("request-{:02}", self.seq));
        fs::write(stem.with_extension("request"), format!("{args:?}\npatch {patch:?}\n")).unwrap();
        fs::write(stem.with_extension("stdout"), &out).unwrap();
        fs::write(stem.with_extension("stderr"), &err).unwrap();
        fs::write(stem.with_extension("result"), &header).unwrap();
        Answer { code: parts[0].parse().unwrap(), out, err, ns: parts[3].parse().unwrap() }
    }
    pub(crate) fn finish(mut self) {
        drop(self.input.take());
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        drop(self.input.take());
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
pub(crate) fn args(source: &Path, out: &Path) -> Vec<String> {
    vec!["build".into(), source.to_str().unwrap().into(), "-o".into(), out.to_str().unwrap().into()]
}
fn accepted(a: &Answer, hit: bool) {
    assert_eq!(a.code, 0, "{}", String::from_utf8_lossy(&a.err));
    assert_eq!(a.hit(), hit, "{}", String::from_utf8_lossy(&a.err));
    assert!(a.out.is_empty());
}
fn runtime(path: &Path) -> std::process::Output {
    let o = Command::new(path).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    o
}
fn equivalent(source: &Path, out: &Path, root: &Path, patch: &[(&str, Option<&str>)]) {
    let cold = root.join("cold-native");
    let oracle = root.join("oracle");
    let mut native = Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());
    native.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(source, &cold));
    let mut stock = Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());
    stock
        .arg(source)
        .args(["--edition=2024", "-Copt-level=0", "-Cdebuginfo=0", "-Cpanic=unwind", "-Coverflow-checks=on", "-Cdebug-assertions=on"])
        .arg("-o")
        .arg(&oracle);
    for c in [&mut native, &mut stock] {
        for (k, v) in patch {
            if let Some(v) = v {
                c.env(k, v);
            } else {
                c.env_remove(k);
            }
        }
    }
    for (label, c) in [("cold", &mut native), ("stock", &mut stock)] {
        let o = c.output().unwrap();
        fs::write(root.join(format!("{label}.stderr")), &o.stderr).unwrap();
        assert!(o.status.success(), "{label}: {}", String::from_utf8_lossy(&o.stderr));
    }
    let actual = runtime(out);
    for p in [cold, oracle] {
        let expected = runtime(&p);
        assert_eq!(actual.status.code(), expected.status.code());
        assert_eq!(actual.stdout, expected.stdout);
        assert_eq!(actual.stderr, expected.stderr);
    }
}
fn root(name: &str) -> PathBuf {
    let r = PathBuf::from(std::env::var_os("LUSTR_PHASE_OUTPUT").unwrap()).join(name);
    fs::create_dir_all(r.parent().unwrap()).unwrap();
    fs::create_dir(&r).unwrap();
    r
}

#[test]
#[ignore = "requires the explicitly built PathBuf inference ABI and cold native/oracle controls"]
fn pathbuf_inference_reuse_tracks_target_edits_rejection_and_recovery() {
    let original = include_str!("fixtures/native-inference-pathbuf.rs");
    platform_inference_edits("pathbuf-inference", original, &[
        original.replace("std::path::PathBuf as OwnedPath", "std::ffi::OsString as OwnedPath"),
    ]);
}

#[test]
#[ignore = "requires the explicitly built owned platform inference ABI and cold native/oracle controls"]
fn owned_platform_reuse_tracks_source_and_target_edits_rejection_and_recovery() {
    let original = include_str!("fixtures/native-inference-owned-mixed.rs");
    platform_inference_edits("owned-platform-inference", original, &[
        original.replace("OwnedOs::from(format!(\"owned-{n}\"))", "OwnedPath::from(format!(\"owned-{n}\"))"),
        original.replace("std::path::PathBuf as OwnedPath", "std::ffi::OsString as OwnedPath"),
    ]);
}

#[test]
#[ignore = "requires the explicitly built owned platform inference ABI and cold native/oracle controls"]
fn owned_same_function_reuse_tracks_source_and_target_edits_rejection_and_recovery() {
    let original = include_str!("fixtures/native-inference-owned-same-function.rs");
    platform_inference_edits("owned-same-function", original, &[
        original.replace("let source=OsString::from", "let source=std::path::PathBuf::from"),
        original.replace("let path:std::path::PathBuf", "let path:OsString"),
    ]);
}

#[test]
#[ignore = "requires the explicitly built shared Path inference ABI and cold native/oracle controls"]
fn borrowed_path_reuse_tracks_source_target_edits_rejection_and_recovery() {
    let original = include_str!("fixtures/native-inference-borrowed-mixed.rs");
    platform_inference_edits("borrowed-path-inference", original, &[
        original.replace("let source = owner.as_path();", "let source = \"borrowed-owned\";"),
        original.replace("let copied: OwnedPath = inferred;", "let copied: std::path::PathBuf = inferred;"),
        original.replace("borrowed-owned", "borrowed-edited"),
    ]);
}

fn platform_inference_edits(scope: &str, original: &str, edits: &[String]) {
    let r = root(scope);
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let out = r.join("program");
    fs::write(&source, original).unwrap();
    let request = args(&source, &out);
    let mut p = Probe::new(&r.join("probe"), None);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    equivalent(&source, &out, &r, &[]);
    for changed in edits {
        assert_ne!(original, changed);
        fs::write(&source, changed).unwrap();
        accepted(&p.request(&request, &[]), false);
        accepted(&p.request(&request, &[]), true);
        equivalent(&source, &out, &r, &[]);
    }
    let invalid = original.replace("fn main() {", "fn main() { let _invalid: u32 = \"invalid\";");
    fs::write(&source, invalid).unwrap();
    let rejected = p.request(&request, &[]);
    assert_eq!(rejected.code, 1);
    assert!(!rejected.hit());
    for (label, mut command) in [
        ("cold-invalid", {
            let mut c = Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());
            c.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source, &r.join("cold-invalid")));
            c
        }),
        ("oracle-invalid", {
            let mut c = Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());
            c.arg(&source).arg("--edition=2024").arg("-o").arg(r.join("oracle-invalid"));
            c
        }),
    ] {
        let answer = command.output().unwrap();
        fs::write(r.join(format!("{label}.stderr")), &answer.stderr).unwrap();
        assert_eq!(answer.status.code(), Some(1));
    }
    fs::write(&source, original).unwrap();
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    equivalent(&source, &out, &r, &[]);
    p.finish();
    assert_eq!(fs::read_to_string(&source).unwrap(), original);
}

#[test]
#[ignore = "requires a private writable host-linker overlay and explicitly built native ABI"]
fn host_linker_script_change_cannot_reuse_old_success() {
    let r = root("host-linker-script");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let out = r.join("program");
    fs::write(
        &source,
        "use std::ffi::OsString; fn value()->OsString { let x=\"proof\".into(); x } fn main(){drop(value()); println!(\"host-proof\");}",
    )
    .unwrap();
    let host = PathBuf::from(std::env::var_os("LUSTR_REUSE_HOST_MUTATION").expect("private host-linker overlay"));
    let original = fs::read(&host).unwrap();
    struct Restore(PathBuf, Vec<u8>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = fs::write(&self.0, &self.1);
        }
    }
    let _restore = Restore(host.clone(), original);
    let mut p = Probe::new(&r.join("probe"), None);
    let request = args(&source, &out);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::write(&host, "this is an invalid linker script\n").unwrap();
    let cold = Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap())
        .arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap())
        .args(args(&source, &r.join("cold")))
        .output()
        .unwrap();
    fs::write(r.join("cold-invalid.stderr"), &cold.stderr).unwrap();
    assert!(!cold.status.success(), "invalid host script unexpectedly compiled");
    let warm = p.request(&request, &[]);
    assert!(!warm.hit(), "stale success after host linker-script mutation");
    assert_ne!(warm.code, 0, "invalid host script must reject on a fresh compile");
    p.finish();
}

#[test]
#[ignore = "requires the explicitly built host-observing retained ABI and cold native control"]
fn host_header_reads_outside_search_root_and_new_candidates_invalidate() {
    let r = root("host-headers");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let out = r.join("program");
    fs::write(
        &source,
        "use std::ffi::OsString; fn value()->OsString { let x=\"proof\".into(); x } fn main(){drop(value()); println!(\"header-proof\");}",
    )
    .unwrap();
    let first = r.join("first");
    // This cannot be represented safely by TinyCC's line-oriented -vvv log.
    let second = r.join("second\nheader-directory");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    let external = r.join("external.h");
    fs::write(&external, "/* external header initially valid */\n").unwrap();
    let alias = r.join("external-alias.h");
    std::os::unix::fs::symlink(&external, &alias).unwrap();
    fs::write(second.join("stdint.h"), format!("#include \"{}\"\n#include_next <stdint.h>\n", alias.display())).unwrap();
    let paths = format!("{}:{}", first.display(), second.display());
    let env = [("CPATH", paths.as_str())];
    let mut p = Probe::new_with_env(&r.join("probe"), None, &env);
    let request = args(&source, &out);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    let mut reject = |label: &str| {
        let warm = p.request(&request, &[]);
        assert!(!warm.hit(), "{label}: stale retained artifact");
        assert_ne!(warm.code, 0, "{label}: invalid header compiled");
        let cold = Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap())
            .arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap())
            .args(args(&source, &r.join("cold")))
            .envs(env)
            .output()
            .unwrap();
        fs::write(r.join(format!("{label}-cold.stderr")), &cold.stderr).unwrap();
        assert!(!cold.status.success());
    };
    fs::write(&external, "#error changed external header\n").unwrap();
    reject("external");
    fs::write(&external, "/* external header restored */\n").unwrap();
    drop(reject);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::write(first.join("stdint.h"), "#error newly appearing higher priority header\n").unwrap();
    let warm = p.request(&request, &[]);
    assert!(!warm.hit());
    assert_ne!(warm.code, 0);
    fs::remove_file(first.join("stdint.h")).unwrap();
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    let other = r.join("other-external.h");
    fs::write(&other, "#error retargeted C header alias\n").unwrap();
    fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&other, &alias).unwrap();
    let warm = p.request(&request, &[]);
    assert!(!warm.hit());
    assert_ne!(warm.code, 0);
    p.finish();
}

#[test]
#[ignore = "requires the explicitly built host-observing retained ABI"]
fn token_pasted_native_clock_macros_force_recomputation() {
    let r = root("host-clock");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let out = r.join("program");
    fs::write(
        &source,
        "use std::ffi::OsString; fn value()->OsString { let x=\"proof\".into(); x } fn main(){drop(value()); println!(\"clock-proof\");}",
    )
    .unwrap();
    let headers = r.join("headers");
    fs::create_dir(&headers).unwrap();
    fs::write(headers.join("stdint.h"), "#ifndef LUSTR_CLOCK_HEADER\n#define LUSTR_CLOCK_HEADER\n#define LUSTR_CAT_RAW(a,b) a ## b\n#define LUSTR_CAT(a,b) LUSTR_CAT_RAW(a,b)\nstatic const char *lustr_clock = LUSTR_CAT(__TI,ME__);\n#include_next <stdint.h>\n#endif\n").unwrap();
    let env = [("CPATH", headers.to_str().unwrap())];
    let mut p = Probe::new_with_env(&r.join("probe"), None, &env);
    let request = args(&source, &out);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), false);
    p.finish();
}

#[test]
#[ignore = "requires a private host-linker overlay and the explicitly built native ABI"]
fn selected_shared_library_in_linker_script_is_a_real_input() {
    use std::os::unix::fs::PermissionsExt;
    let r = root("host-selected-library");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let out = r.join("program");
    fs::write(
        &source,
        "use std::ffi::OsString; fn value()->OsString { let x=\"proof\".into(); x } fn main(){drop(value()); println!(\"library-proof\");}",
    )
    .unwrap();
    let host = PathBuf::from(std::env::var_os("LUSTR_REUSE_HOST_MUTATION").unwrap());
    struct Restore(PathBuf, Vec<u8>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = fs::write(&self.0, &self.1);
        }
    }
    let _restore = Restore(host.clone(), fs::read(&host).unwrap());
    let library = r.join("private-libm.so");
    fs::copy("/usr/lib/libm.so.6", &library).unwrap();
    fs::set_permissions(&library, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&host, format!("GROUP ( {} )\n", library.display())).unwrap();
    let mut p = Probe::new(&r.join("probe"), None);
    let request = args(&source, &out);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    equivalent(&source, &out, &r, &[]);
    // Only TinyCC reads this private copy. Its SONAME still loads the original
    // system runtime library; no active mapped library is truncated.
    fs::write(&library, b"invalid native linker input\n").unwrap();
    let warm = p.request(&request, &[]);
    assert!(!warm.hit());
    assert_ne!(warm.code, 0);
    let cold = Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap())
        .arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap())
        .args(args(&source, &r.join("cold-invalid")))
        .output()
        .unwrap();
    fs::write(r.join("cold-invalid.stderr"), &cold.stderr).unwrap();
    assert!(!cold.status.success());
    p.finish();
}
pub(crate) fn copy_tree(from: &Path, to: &Path) {
    let m = fs::symlink_metadata(from).unwrap();
    if m.file_type().is_symlink() {
        std::os::unix::fs::symlink(fs::read_link(from).unwrap(), to).unwrap();
    } else if m.is_dir() {
        fs::create_dir(to).unwrap();
        for e in fs::read_dir(from).unwrap() {
            let e = e.unwrap();
            copy_tree(&e.path(), &to.join(e.file_name()));
        }
    } else {
        fs::copy(from, to).unwrap();
    }
}

#[test]
#[ignore = "requires the explicitly built bounded retained ABI and real native/Rust oracles"]
fn no_op_and_semantic_edits_match_cold_native_and_stock() {
    let r = root("reuse-edits");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let helper = project.join("helper.rs");
    let include = project.join("value.txt");
    let out = r.join("program");
    let text = "use std::ffi::OsString; mod helper; macro_rules! label { () => { \"macro-one\" }; } fn value()->OsString { let x=\"proof\".into(); x } fn main(){drop(value()); println!(\"{} / {} / {} / {}\",helper::value(),include_str!(\"value.txt\"),label!(),option_env!(\"LUSTR_REUSE_ENV\").unwrap_or(\"unset\"));}";
    fs::write(&source, text).unwrap();
    fs::write(&helper, "pub fn value()->&'static str { \"body-one\" }").unwrap();
    fs::write(&include, "include-one").unwrap();
    let patch = [("LUSTR_REUSE_ENV", None)];
    let mut p = Probe::new(&r.join("probe"), None);
    let request = args(&source, &out);
    accepted(&p.request(&request, &patch), false);
    accepted(&p.request(&request, &patch), true);
    equivalent(&source, &out, &r, &patch);
    fs::write(&helper, "pub fn value()->&'static str { \"body-two\" }").unwrap();
    accepted(&p.request(&request, &patch), false);
    accepted(&p.request(&request, &patch), true);
    equivalent(&source, &out, &r, &patch);
    fs::write(&helper, "pub fn value()->String { String::from(\"signature\") }").unwrap();
    accepted(&p.request(&request, &patch), false);
    equivalent(&source, &out, &r, &patch);
    fs::write(&source, text.replace("macro-one", "macro-two")).unwrap();
    accepted(&p.request(&request, &patch), false);
    equivalent(&source, &out, &r, &patch);
    fs::write(&include, "include-two").unwrap();
    accepted(&p.request(&request, &patch), false);
    equivalent(&source, &out, &r, &patch);
    let set = [("LUSTR_REUSE_ENV", Some("present"))];
    accepted(&p.request(&request, &set), false);
    accepted(&p.request(&request, &set), true);
    equivalent(&source, &out, &r, &set);
    accepted(&p.request(&request, &patch), false);
    equivalent(&source, &out, &r, &patch);
    fs::remove_file(&include).unwrap();
    fs::remove_file(&out).unwrap();
    let bad = p.request(&request, &patch);
    assert_eq!(bad.code, 1);
    assert!(!bad.hit());
    assert!(!out.exists());
    fs::write(&include, "recovered").unwrap();
    accepted(&p.request(&request, &patch), false);
    accepted(&p.request(&request, &patch), true);
    fs::create_dir(project.join("helper")).unwrap();
    fs::write(project.join("helper/mod.rs"), "pub fn value()->&'static str { \"ambiguous\" }").unwrap();
    fs::remove_file(&out).unwrap();
    let bad = p.request(&request, &patch);
    assert_eq!(bad.code, 1);
    assert!(!bad.hit());
    assert!(!out.exists());
    fs::remove_dir_all(project.join("helper")).unwrap();
    accepted(&p.request(&request, &patch), false);
    fs::write(&source, text.replace("let x=\"proof\".into(); x", "let x=\"proof\".into(); missing")).unwrap();
    fs::remove_file(&out).unwrap();
    let bad = p.request(&request, &patch);
    assert_eq!(bad.code, 1);
    assert!(!bad.hit());
    assert!(!out.exists());
    fs::write(&source, text).unwrap();
    accepted(&p.request(&request, &patch), false);
    accepted(&p.request(&request, &patch), true);
    equivalent(&source, &out, &r, &patch);
    fs::write(&out, b"tampered").unwrap();
    accepted(&p.request(&request, &patch), false);
    assert!(fs::read(&out).unwrap().starts_with(b"\x7fELF"));
    fs::remove_file(&out).unwrap();
    accepted(&p.request(&request, &patch), false);
    let mut foreign = request.clone();
    foreign.extend(["--target".into(), "aarch64-unknown-linux-gnu".into()]);
    fs::remove_file(&out).unwrap();
    let bad = p.request(&foreign, &patch);
    assert_eq!(bad.code, 2);
    assert!(!bad.hit());
    assert!(!out.exists());
    accepted(&p.request(&request, &patch), false);
    accepted(&p.request(&request, &patch), true);
    p.finish();
}

#[test]
#[ignore = "requires real retained/native/Rust compiler artifacts"]
fn unchanged_full_coordinator_and_cargo_graph_inputs() {
    let r = root("reuse-coordinator");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    for name in ["Cargo.toml", "Cargo.lock", "src"] {
        copy_tree(&Path::new("/lustr/crates/lustr").join(name), &project.join(name));
    }
    let source = project.join("src/main.rs");
    let out = r.join("lustr");
    let mut request = args(&source, &out);
    request.extend([
        "--manifest-path".into(),
        project.join("Cargo.toml").to_str().unwrap().into(),
        "--bin".into(),
        "lustr".into(),
        "--offline".into(),
        "--locked".into(),
    ]);
    let mut p = Probe::new(&r.join("probe"), None);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    let target = r.join("oracle-target");
    let built = Command::new(std::env::var_os("LUSTR_PHASE_CARGO").unwrap())
        .args(["build", "--config"])
        .arg(format!("build.build-dir={:?}", r.join("oracle-build")))
        .args(["--offline", "--locked", "--jobs", "2", "--manifest-path"])
        .arg(project.join("Cargo.toml"))
        .args(["--bin", "lustr"])
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .unwrap();
    fs::write(r.join("oracle-build.stderr"), &built.stderr).unwrap();
    assert!(built.status.success());
    for cli in [
        vec!["--help"],
        vec!["--version"],
        vec!["plan", "--frontend-jobs=3", "rustc", "--offline"],
        vec!["plan", "--bluefe-root", "/package", "build", "--offline"],
        vec!["--frontend-jobs=0", "rustc"],
    ] {
        let actual = Command::new(&out).args(&cli).output().unwrap();
        let expected = Command::new(target.join("debug/lustr")).args(&cli).output().unwrap();
        assert_eq!(actual.status.code(), expected.status.code());
        assert_eq!(actual.stdout, expected.stdout);
        assert_eq!(actual.stderr, expected.stderr);
    }
    fs::create_dir_all(project.join("src/bin")).unwrap();
    fs::write(project.join("src/bin/new.rs"), "fn main(){}\n").unwrap();
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::create_dir(project.join(".cargo")).unwrap();
    fs::write(project.join(".cargo/config.toml"), "[net]\noffline=true\n").unwrap();
    accepted(&p.request(&request, &[]), false);
    let manifest = project.join("Cargo.toml");
    let original = fs::read(&manifest).unwrap();
    fs::write(&manifest, b"[invalid").unwrap();
    fs::remove_file(&out).unwrap();
    let bad = p.request(&request, &[]);
    assert_ne!(bad.code, 0);
    assert!(!bad.hit());
    assert!(!out.exists());
    fs::write(&manifest, original).unwrap();
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    p.finish();
}

#[test]
#[ignore = "requires real retained/native/Rust artifacts"]
fn no_op_preserves_mixed_drop_unwind_and_unknown_reads_recompute() {
    let r = root("reuse-mixed");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    let out = r.join("program");
    fs::copy("/lustr/tests/fixtures/native-inference-mixed.rs", &source).unwrap();
    let mut p = Probe::new(&r.join("probe"), None);
    let request = args(&source, &out);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    equivalent(&source, &out, &r, &[]);
    p.finish();
    let external = r.join("external.txt");
    fs::write(&external, "external-one").unwrap();
    fs::write(&source,format!("use std::ffi::OsString; fn value()->OsString{{let x=\"proof\".into();x}}fn main(){{drop(value());println!(\"{{}}\",include_str!({external:?}));}}" )).unwrap();
    let mut p = Probe::new(&r.join("unknown-probe"), None);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), false);
    fs::write(&external, "external-two").unwrap();
    accepted(&p.request(&request, &[]), false);
    equivalent(&source, &out, &r, &[]);
    p.finish();
}

#[test]
#[ignore = "requires retained/native artifacts and writable copied runtime"]
fn runtime_mutation_retires_the_instance_and_request_limit_is_terminal() {
    let r = root("reuse-runtime");
    let project = r.join("project");
    fs::create_dir(&project).unwrap();
    let source = project.join("main.rs");
    fs::copy("/lustr/tests/fixtures/native-inference-into.rs", &source).unwrap();
    let out = r.join("program");
    let lib = r.join("runtime");
    fs::create_dir(&lib).unwrap();
    fs::create_dir(lib.join("lust")).unwrap();
    copy_tree(Path::new("/lustr/compiler/native/library"), &lib.join("lust/library"));
    copy_tree(Path::new("/lustr/library/lustr"), &lib.join("lustr"));
    let request = args(&source, &out);
    let mut p = Probe::new(&r.join("probe"), Some(&lib));
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::write(lib.join("lustr/new-source.rs"), "// newly appearing runtime input\n").unwrap();
    fs::remove_file(&out).unwrap();
    let bad = p.request(&request, &[]);
    assert_eq!(bad.code, 2);
    assert!(!out.exists());
    fs::remove_file(lib.join("lustr/new-source.rs")).unwrap();
    assert_eq!(p.request(&request, &[]).code, 2);
    p.finish();
    let mut p = Probe::new(&r.join("bound-probe"), None);
    for i in 0..64 {
        accepted(&p.request(&request, &[]), i != 0);
    }
    assert_eq!(p.request(&request, &[]).code, 3);
    p.finish();
}

#[test]
#[ignore = "requires real retained ABI"]
fn retargeted_source_and_output_directory_aliases_recompute() {
    let r = root("reuse-alias");
    let a = r.join("tree-a");
    let b = r.join("tree-b");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    fs::copy("/lustr/tests/fixtures/native-inference-into.rs", a.join("main.rs")).unwrap();
    fs::write(b.join("main.rs"), "fn main(){ missing; }\n").unwrap();
    let alias = r.join("alias");
    std::os::unix::fs::symlink(&a, &alias).unwrap();
    let out = r.join("program");
    let request = args(&alias.join("main.rs"), &out);
    let mut p = Probe::new(&r.join("probe"), None);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&b, &alias).unwrap();
    let rejected = p.request(&request, &[]);
    assert_eq!(rejected.code, 1, "retargeted invalid source reused an old executable");
    assert!(!rejected.hit());
    fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&a, &alias).unwrap();
    accepted(&p.request(&request, &[]), false);
    let oa = r.join("out-a");
    let ob = r.join("out-b");
    fs::create_dir(&oa).unwrap();
    fs::create_dir(&ob).unwrap();
    let link = r.join("out-alias");
    std::os::unix::fs::symlink(&oa, &link).unwrap();
    let request = args(&alias.join("main.rs"), &link.join("program"));
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&ob, &link).unwrap();
    accepted(&p.request(&request, &[]), false);
    assert!(ob.join("program").is_file());
    fs::write(a.join("data-a.txt"), "one").unwrap();
    fs::write(a.join("data-b.txt"), "two").unwrap();
    let external = r.join("external-include-alias");
    std::os::unix::fs::symlink(a.join("data-a.txt"), &external).unwrap();
    fs::write(a.join("main.rs"), format!("use std::ffi::OsString; fn value()->OsString{{let x=\"proof\".into();x}}fn main(){{drop(value());println!(\"{{}}\",include_str!({external:?}));}}" )).unwrap();
    let request = args(&alias.join("main.rs"), &out);
    accepted(&p.request(&request, &[]), false);
    accepted(&p.request(&request, &[]), true);
    fs::remove_file(&external).unwrap();
    std::os::unix::fs::symlink(a.join("data-b.txt"), &external).unwrap();
    accepted(&p.request(&request, &[]), false);
    assert_eq!(runtime(&out).stdout, b"two\n");
    p.finish();
}
#[test]
#[ignore = "requires the explicitly built shared OsStr inference ABI and cold native/oracle controls"]
fn borrowed_os_reuse_tracks_source_target_edits_rejection_and_recovery() {
    let original = include_str!("fixtures/native-inference-borrowed-os-mixed.rs");
    platform_inference_edits("borrowed-os-inference", original, &[
        original.replace("let source = owner.as_os_str();", "let source = \"borrowed-owned\";"),
        original.replace("let copied: OwnedPath = inferred;", "let copied: std::path::PathBuf = inferred;"),
        original.replace("borrowed-owned", "borrowed-edited"),
    ]);
}

#[test]
#[ignore = "requires checked slice/array assertions and cold native/oracle controls"]
fn slice_array_assertion_reuse_tracks_operand_length_type_edits_and_recovery() {
    let original = include_str!("fixtures/native-inference-slice-array-original.rs");
    platform_inference_edits("slice-array-inference", original, &[
        original.replace("borrowed-owned", "borrowed-edited"),
        original.replace("b'x', 0xfe", "b'x', 0xfd, b'z'"),
        original.replace("&[b'r', 0xff, b'w']", "&[b'r', 0xff, b'w'][..]"),
    ]);
    let semantics = include_str!("fixtures/native-inference-slice-array-semantics.rs");
    platform_inference_edits("slice-array-semantics", semantics, &[
        semantics.replace("&[true, false][..], &[true, false]", "&[true, false], &[true, false][..]"),
        semantics.replace("1.0f64", "2.0f32"),
    ]);
}

#[test]
#[ignore = "requires place-preserving assertions and cold native/oracle controls"]
fn slice_array_places_reuse_tracks_place_operand_type_length_and_recovery() {
    let original = include_str!("fixtures/native-inference-slice-array-place-original.rs");
    platform_inference_edits("slice-array-place-inference", original, &[
        original.replace("borrowed-owned", "borrowed-edited"),
        original.replace("b'x', 0xfe", "b'x', 0xfd, b'z'"),
        original.replace("let expected_raw = &[b'r', 0xff, b'w'];", "let expected_raw: &[u8] = &[b'r', 0xff, b'w'];"),
    ]);
    let semantics = include_str!("fixtures/native-inference-slice-array-place-semantics.rs");
    platform_inference_edits("slice-array-place-semantics", semantics, &[
        semantics.replace("let bools = &[true, false];", "let bools = &[false, true];").replace("&[true, false][..], bools", "&[false, true][..], bools"),
        semantics.replace("1.0f64", "2.0f32"),
    ]);
}

#[test]
#[ignore = "requires original-AST assertion text and cold native/oracle controls"]
fn long_assertion_text_reuse_tracks_payload_tokens_positions_rejection_and_recovery() {
    let original=include_str!("fixtures/native-inference-long-assert-payload.rs");
    platform_inference_edits("long-assert-text",original,&[
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        original.replace("println!(\"once {{raw braces}} λ\");","println!(r#\"long raw {{braces}} {value}\"#,value=7u8);"),
        format!("\n\n// source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
    let mixed=include_str!("fixtures/native-inference-long-assert-original.rs");
    platform_inference_edits("long-assert-original",mixed,&[
        mixed.replace("1.0f64","2.0f32"),
        mixed.replace("&[true, false][..], &[true, false]","&[true, false], &[true, false][..]"),
    ]);
}

#[test]
#[ignore = "requires original-token parentheses and cold native/oracle controls"]
fn parenthesized_assertion_reuse_tracks_nesting_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-parenthesized-assert-original.rs");
    platform_inference_edits("parenthesized-assert-text",original,&[
        original.replace("assert!((catch_unwind", "assert!(((catch_unwind").replace("})).is_err()));", "})).is_err())));"),
        original.replace("catch_unwind(AssertUnwindSafe", "((catch_unwind))(AssertUnwindSafe"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}
#[test]
#[ignore = "requires original-AST binary formatting and cold native/oracle controls"]
fn binary_assertion_reuse_tracks_operators_nesting_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-binary-assert-original.rs");
    platform_inference_edits("binary-assert-text",original,&[
        original.replace("&& true","|| false"),
        original.replace("&& true","|| (false || false) && true"),
        original.replace("&& true","|| { let a=12u32; a << 2 >> 1 == 0 }"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}

#[test]
#[ignore = "requires original-AST struct formatting and cold native/oracle controls"]
fn struct_assertion_reuse_tracks_fields_types_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-struct-assert-original.rs");
    platform_inference_edits("struct-assert-text",original,&[
        original.replace("Holder { value: true }","Holder { value: false, }"),
        original.replace("value: true","r#type: true").replace("value: bool","r#type: bool").replace("}).value","}).r#type"),
        original.replace("value: true","value: 1u8 == 1u8"),
        original.replace("value: true","value: 1u8").replace("value: bool","value: u8").replace("}).value","}).value == 1u8"),
        original.replace("Holder { value: true }","Holder { ..Holder { value: true } }"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}

#[test]
#[ignore = "requires original-AST tuple-field formatting and cold native/oracle controls"]
fn tuple_field_assertion_reuse_tracks_indexes_chains_types_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-tuple-assert-original.rs");
    platform_inference_edits("tuple-field-assert-text",original,&[
        original.replace("Pair{0:false,1:2u8}).0","Pair{0:2u8,1:false}).1").replace("struct Pair(bool,u8)","struct Pair(u8,bool)"),
        original.replace("Pair{0:false,1:2u8}).0","Pair{0:false,1:2u8}) /* field */ . /* index */ 0"),
        original.replace("Pair{0:false,1:2u8}).0","Pair{0:false,1:3u16}).0").replace("struct Pair(bool,u8)","struct Pair(bool,u16)"),
        original.replace("Pair{0:false,1:2u8}).0","Nest{0:Pair{0:false,1:2u8}}).0.0")+"\nstruct Nest(Pair);\n",
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}

#[test]
#[ignore = "requires original-AST tuple literals and cold native/oracle runtime controls"]
fn tuple_literal_assertion_reuse_tracks_elements_types_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-tuple-literal-assert-original.rs");
    platform_inference_edits("tuple-literal-assert-text",original,&[
        original.replace("((false,2u8)).0","((false,)).0"),
        original.replace("((false,2u8)).0","((2u8,false)).1"),
        original.replace("((false,2u8)).0","(((false,2u8),)).0.0"),
        original.replace("((false,2u8)).0","((false,2u16,)).0"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// tuple source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}

#[test]
#[ignore = "requires original-AST const blocks and cold native/oracle runtime controls"]
fn const_block_assertion_reuse_tracks_values_bodies_types_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-const-block-assert-original.rs");
    platform_inference_edits("const-block-assert-text",original,&[
        original.replace("const {false}","const {1u8 != 1u8}"),
        original.replace("const {false}","const {let x=false;x}"),
        original.replace("const {false}","const {const {false}}"),
        original.replace("const {false}","const /*keyword*/ {/*body*/false}"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// const source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
    let r=root("const-block-retained-rejections");
    let project=r.join("project");fs::create_dir(&project).unwrap();
    let source=project.join("main.rs");let out=r.join("program");
    let request=args(&source,&out);let mut p=Probe::new(&r.join("probe"),None);
    fs::write(&source,original).unwrap();
    accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
    for (name,value) in [("type","const {1u8}"),("capture","const {drops.get()==0}"),
        ("mutability","const {let x=false;x=true;x}")] {
        fs::write(&source,original.replace("const {false}",value)).unwrap();
        let answer=p.request(&request,&[]);assert_eq!(answer.code,1);assert!(!answer.hit());
        for (label,mut command) in [
            ("cold",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_PROBE").unwrap());c.arg(std::env::var_os("LUSTR_PHASE_LIBRARY").unwrap()).args(args(&source,&r.join(format!("{name}-cold"))));c}),
            ("oracle",{let mut c=Command::new(std::env::var_os("LUSTR_PHASE_ORACLE").unwrap());c.arg(&source).arg("--edition=2024").arg("-o").arg(r.join(format!("{name}-oracle")));c})
        ] {let answer=command.output().unwrap();fs::write(r.join(format!("{name}-{label}.stderr")),&answer.stderr).unwrap();assert_eq!(answer.status.code(),Some(1));}
        fs::write(&source,original).unwrap();
        accepted(&p.request(&request,&[]),false);accepted(&p.request(&request,&[]),true);
        equivalent(&source,&out,&r,&[]);
    }
    p.finish();
}

#[test]
#[ignore = "requires original-AST array repeats and cold native/oracle runtime controls"]
fn array_repeat_assertion_reuse_tracks_value_length_types_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-array-repeat-assert-original.rs");
    platform_inference_edits("array-repeat-assert-text",original,&[
        original.replace("([false;2])[0]","([false;1])[0]"),
        original.replace("([false;2])[0]","([false;1+1])[1]"),
        original.replace("([false;2])[0]","([[false;2];2])[0][0]"),
        original.replace("([false;2])[0]","([0u8;2])[0] == 1u8"),
        original.replace("([false;2])[0]","([/*value*/false;/*length*/{2}])[0]"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// repeat source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}

#[test]
#[ignore = "requires original-AST array literals and cold native/oracle runtime controls"]
fn array_literal_assertion_reuse_tracks_elements_types_tokens_positions_and_recovery() {
    let original=include_str!("fixtures/native-inference-array-literal-assert-original.rs");
    platform_inference_edits("array-literal-assert-text",original,&[
        original.replace("([false,true])[0]","([false])[0]"),
        original.replace("([false,true])[0]","([true,false,])[1]"),
        original.replace("([false,true])[0]","([[false,true], [true,false]])[0][0]"),
        original.replace("([false,true])[0]","([0u8,1u8])[0] == 1u8"),
        original.replace("once {{raw braces}} λ","different {{braces}} αβγ"),
        format!("\n\n// array source positions change\n{original}"),
        original.replace("fn fail(drops: Rc<Cell<usize>>)","fn fail(mut drops: Rc<Cell<usize>>)"),
    ]);
}
