//! SDK-bootstrapped observer qualifier. Every application is built natively.
#![allow(dead_code, unused_imports)]
include!("../crates/lustr-native-core/src/lib.rs");
#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

#[derive(Default)]
struct Events {
    id: Option<driver::PhaseId>,
    input: std::sync::Weak<ast::Ast>,
    collected: std::sync::Weak<ast::Ast>,
    resolver: Option<std::sync::Weak<resolve::imports::ResolvedTables>>,
    work: Vec<usize>,
    checked: Vec<usize>,
    lowered: Vec<usize>,
    check_errors: usize,
    borrow_errors: usize,
    complete: bool,
    diagnostics: usize,
    fresh: bool,
    uncalled_return: Option<String>,
    opaque_hidden: usize,
}
#[derive(Default)]
struct Recorder(std::sync::Mutex<Events>);
impl Recorder {
    fn same(events: &Events, phase: driver::Prepared<'_>) {
        assert_eq!(events.id, Some(phase.id));
        assert!(events.input.upgrade().is_some_and(|ast| std::sync::Arc::ptr_eq(&ast, phase.input)));
        assert!(events.collected.upgrade().is_some_and(|ast| std::sync::Arc::ptr_eq(&ast, &phase.program.ast)));
        match (&events.resolver, &phase.program.resolved) {
            (Some(old), Some(now)) => assert!(old.upgrade().is_some_and(|old| std::sync::Arc::ptr_eq(&old, now))),
            (None, None) => {},
            _ => panic!("resolver generation changed during a native phase"),
        }
        assert!(!events.complete);
    }
}
impl driver::PhaseObserver for Recorder {
    fn prepared(&self, phase: driver::Prepared<'_>) {
        let mut events = self.0.lock().unwrap();
        assert!(events.id.is_none());
        events.id = Some(phase.id);
        events.input = std::sync::Arc::downgrade(phase.input);
        events.collected = std::sync::Arc::downgrade(&phase.program.ast);
        events.resolver = phase.program.resolved.as_ref().map(std::sync::Arc::downgrade);
        events.work = phase.work.to_vec();
        events.fresh = check::collect::provenance::current(phase.input, phase.program);
        events.uncalled_return = phase.program.fns.iter().find(|f| phase.program.name(f.name)=="uncalled")
            .map(|f|format!("{:?}",ty::TyIntern::kind(&phase.program.tys,f.ret)));
        events.opaque_hidden = phase.program.gx.opaque_hidden.len();
        assert_eq!(phase.request.session.krate.root, phase.request.file);
        assert!(!phase.map.text(diag::FileId(0)).unwrap_or(phase.source).is_empty());
    }
    fn checked(&self, phase: driver::Prepared<'_>, function: def::LocalDefId, checked: &check::Checked<'_>, diagnostics: &[diag::Diag]) {
        let mut events = self.0.lock().unwrap();
        Self::same(&events, phase);
        assert!(!check::derive_delta::open(), "observer must run outside native derive-read logging");
        check::derive_delta::note_def(phase.program.def_id(function), check::derive_delta::bit::ALL);
        assert!(events.work.contains(&(function.0 as usize)));
        assert!(!events.checked.contains(&(function.0 as usize)));
        // Access the actual live type store, expression and resolution tables.
        assert_eq!(checked.expr_ty.len(), checked.expr_res.len());
        for &ty in &checked.expr_ty { let _ = ty::TyIntern::kind(&checked.tys, ty); }
        assert!(!phase.program.name(phase.program.fns[function.0 as usize].name).is_empty());
        events.checked.push(function.0 as usize);
        events.check_errors += diagnostics.len();
    }
    fn lowered(&self, phase: driver::Prepared<'_>, function: def::LocalDefId, body: &ir::Body<'_>, diagnostics: &[diag::Diag]) {
        let mut events = self.0.lock().unwrap();
        Self::same(&events, phase);
        assert!(!check::derive_delta::open(), "IR observation must not add native derive reads");
        assert!(events.checked.contains(&(function.0 as usize)));
        assert!(!events.lowered.contains(&(function.0 as usize)));
        assert_eq!(body.fn_id, function);
        assert!(!body.blocks.is_empty());
        events.lowered.push(function.0 as usize);
        events.borrow_errors += diagnostics.len();
    }
    fn diagnostics(&self, phase: driver::Prepared<'_>, diagnostics: &[diag::Diag]) {
        let mut events = self.0.lock().unwrap();
        Self::same(&events, phase);
        events.work.sort_unstable(); events.checked.sort_unstable();
        assert_eq!(events.work, events.checked, "one actual Checked callback per work item");
        events.complete = true;
        events.diagnostics = diagnostics.len();
    }
}
fn put(path: &std::path::Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}
fn main() {
    raise_stack_limit();
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).expect("NEW_OUTPUT_DIRECTORY"));
    std::fs::create_dir(&root).unwrap();
    let file = root.join("main.rs");
    let key = driver::SessionKey::synthetic(&file, false);
    let mut observed = driver::Session::new(root.join("objects-observed"), true, key.clone());
    let mut plain = driver::Session::new(root.join("objects-plain"), true, key.clone());
    for session in [&mut observed, &mut plain] {
        session.cache_tried = true;
        session.set_output(driver::OutputKind::Standalone);
    }
    let good = "type Number=i64;\nconst fn uncalled()->i64 {1}\nfn identity<T>(value:T)->T {value}\nfn value()->i64 { identity(31) }\nfn main(){ println!(\"{}\",value()); }\n";
    let mut ids = Vec::new();
    let mut outputs = Vec::new();
    for (label, source, code, phase, needs_check_error, needs_borrow_error) in [
        ("initial", good.to_owned(), 0, true, false, false),
        ("noop", good.to_owned(), 0, true, false, false),
        ("body", good.replace("31", "32"), 0, true, false, false),
        // The const header delta is refused intact. Its valid new signature
        // leaves the literal body ill-typed for the current full body phase.
        ("type-reject", good.replace("uncalled()->i64", "uncalled()->bool"), 1, true, true, false),
        ("recover", good.to_owned(), 0, true, false, false),
        ("move-reject", "fn main(){let a=String::from(\"a\");let b=a;println!(\"{a}{b}\");}\n".to_owned(), 1, true, false, true),
        ("signature-reject", good.replace("->i64", "->MissingType"), 1, false, false, false),
        ("template-reject", format!("fn bad<T:MissingTrait>(x:T)->T{{x}}\n{good}"), 1, false, false, false),
        ("parse-reject", "fn main( {".to_owned(), 1, false, false, false),
        ("final-recover", good.to_owned(), 0, true, false, false),
    ] {
        std::fs::write(&file, &source).unwrap();
        put(&root.join(format!("{label}.source.rs")), source.as_bytes());
        let recorder = Recorder::default();
        let output = root.join(format!("{label}-observed.elf"));
        let mut request = driver::Request {
            cmd: driver::Cmd::Build, session: key.clone(), file: file.clone(), out: output.clone(),
            json: false, timings: false, debug: false, emit_c: None, max_errors: 0,
            run_args: None, harness: false, cc: false, client: Default::default(),
        };
        let _proof = check::collect::provenance::begin();
        observed.caches.objects = None;
        let answer = driver::compile_observed(&request, &mut observed, Some(&recorder));
        let events = recorder.0.into_inner().unwrap();
        assert_eq!(answer.code, code, "{label}: {}", answer.stderr);
        assert!(!answer.ran);
        assert_eq!(events.complete, phase, "{label}");
        if let Some(id) = events.id {
            assert!(!ids.contains(&id)); ids.push(id);
            assert_eq!(events.diagnostics > 0, code != 0);
            if label == "noop" { assert!(events.work.is_empty() && !events.fresh, "noop work={:?} fresh={}", events.work, events.fresh); }
        } else { assert!(events.checked.is_empty() && events.lowered.is_empty()); }
        if needs_check_error { assert!(events.check_errors > 0); }
        if needs_borrow_error { assert!(events.borrow_errors > 0); }
        if label == "type-reject" {
            assert!(events.fresh, "must observe current full collect before keep_err");
            assert_eq!(events.uncalled_return.as_deref(),Some("Bool"),"callback must observe the current rejected signature");
        }
        put(&root.join(format!("{label}.events")), format!("id={:?} work={:?} checked={:?} lowered={:?} fresh={} checkerrors={} borrowerrors={} diagnostics={} complete={}\n",events.id,events.work,events.checked,events.lowered,events.fresh,events.check_errors,events.borrow_errors,events.diagnostics,events.complete).as_bytes());
        put(&root.join(format!("{label}-observed.stderr")), answer.stderr.as_bytes());
        request.out = root.join(format!("{label}-plain.elf"));
        plain.caches.objects = None;
        let control = driver::compile(&request, &mut plain);
        put(&root.join(format!("{label}-plain.stderr")), control.stderr.as_bytes());
        assert_eq!(answer.code, control.code);
        assert_eq!(answer.stderr, control.stderr, "observer changed native diagnostics for {label}");
        for reads in observed.caches.slot_reads.iter().flatten() {
            assert!(!reads.keys(&observed.caches.read_keys).any(|key| matches!(key, driver::graph::ReadKey::Derives(def) if observed.front.kept.iter().flatten().any(|kept|kept.program.fn_defs.contains(def)))), "observer's synthetic function derive note escaped its unlogged scope");
        }
        if code == 0 { outputs.push((label, output, request.out)); }
        else { assert!(!output.exists() && !request.out.exists()); }
    }
    // Check answers can return before a prepared body phase. Use the actual
    // resolver path, which owns the native last-check response cache.
    std::fs::write(&file, format!("type OtherNumber=i64;\n{good}")).unwrap();
    for (label, cached) in [("check-first", false), ("check-cached", true)] {
        let request = driver::Request { cmd:driver::Cmd::Check, session:key.clone(), file:file.clone(), out:root.join("check-unused"), json:false, timings:false, debug:false, emit_c:None, max_errors:0, run_args:None, harness:false, cc:false, client:Default::default() };
        let recorder=Recorder::default(); let _proof=check::collect::provenance::begin();
        let answer=driver::compile_observed(&request,&mut observed,Some(&recorder));
        put(&root.join(format!("{label}.stderr")),answer.stderr.as_bytes()); assert_eq!(answer.code,0);
        let events=recorder.0.into_inner().unwrap();
        assert_eq!(events.id.is_none(),cached,"{label}");
        assert_eq!(events.complete,!cached,"cached Check must not synthesize semantic events");
    }
    observed.settle(); plain.settle(); drop(observed); drop(plain);
    // Retain the upstream regression's actual edit sequence. Its body error
    // loses the current opaque hidden type, while keep_err prefers the older
    // Program. A toy type mismatch does not establish that precondition.
    let preferred_file=root.join("preferred.rs");
    let preferred_key=driver::SessionKey::synthetic(&preferred_file,false);
    let mut preferred=driver::Session::new(root.join("objects-preferred"),true,preferred_key.clone());
    preferred.cache_tried=true;preferred.set_output(driver::OutputKind::Standalone);
    let mut preferred_outputs=Vec::new();
    for (label,text,code) in [
        ("preferred-v1",include_str!("../compiler/native/tests/daemon/sd27_rpit_body_reject_then_revert/v1.rs"),0),
        ("preferred-v2",include_str!("../compiler/native/tests/daemon/sd27_rpit_body_reject_then_revert/v2.rs"),0),
        ("preferred-v3",include_str!("../compiler/native/tests/daemon/sd27_rpit_body_reject_then_revert/v3.rs"),0),
        ("preferred-v4",include_str!("../compiler/native/tests/daemon/sd27_rpit_body_reject_then_revert/v4.rs"),1),
        ("preferred-v5",include_str!("../compiler/native/tests/daemon/sd27_rpit_body_reject_then_revert/v5.rs"),0),
    ] {
        std::fs::write(&preferred_file,text).unwrap();put(&root.join(format!("{label}.source.rs")),text.as_bytes());
        let out=root.join(format!("{label}.elf"));
        let request=driver::Request {cmd:driver::Cmd::Build,session:preferred_key.clone(),file:preferred_file.clone(),out:out.clone(),json:false,timings:false,debug:false,emit_c:None,max_errors:0,run_args:None,harness:false,cc:false,client:Default::default()};
        let recorder=Recorder::default();let _proof=check::collect::provenance::begin();preferred.caches.objects=None;
        let answer=driver::compile_observed(&request,&mut preferred,Some(&recorder));
        put(&root.join(format!("{label}.stderr")),answer.stderr.as_bytes());assert_eq!(answer.code,code,"{label}: {}",answer.stderr);
        let events=recorder.0.into_inner().unwrap();
        if code!=0 {
            assert!(events.complete && events.fresh,"must observe the current failed body phase");
            let kept=preferred.front.kept.iter().flatten().next().expect("older preferred Program");
            assert!(!events.collected.ptr_eq(&std::sync::Arc::downgrade(&kept.program.ast)) || events.opaque_hidden!=kept.program.gx.opaque_hidden.len(),"older preferred Program must differ from actual current rejected state");
            assert!(!out.exists());
        } else {preferred_outputs.push((label,out));}
    }
    preferred.settle();drop(preferred);
    for (label,out) in preferred_outputs {
        let result=std::process::Command::new(out).output().unwrap();put(&root.join(format!("{label}.run.stdout")),&result.stdout);put(&root.join(format!("{label}.run.stderr")),&result.stderr);
        assert!(result.status.success(),"failed application must not replay");assert_eq!(result.stdout,b"0 0 5\n2 1 6\n1\n");
    }
    for (label, first, second) in outputs {
        let mut expected = None;
        for (kind, output) in [("observed", first), ("plain", second)] {
            let result = std::process::Command::new(output).output().unwrap();
            put(&root.join(format!("{label}-{kind}.run.stdout")), &result.stdout);
            put(&root.join(format!("{label}-{kind}.run.stderr")), &result.stderr);
            assert!(result.status.success(), "failed application must not replay");
            if let Some(expected) = &expected { assert_eq!(&result.stdout, expected); }
            else { expected = Some(result.stdout); }
        }
    }
    println!("live native current-phase/Checked/IR/diagnostic borrows, unique call identities, noop work0, early failures and recovery passed with observer-off diagnostic/runtime parity; no typed-crate or speed claim");
}
