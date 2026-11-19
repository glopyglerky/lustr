//! Exercise genuine upstream metadata services inside one native compilation.
#![feature(rustc_private)]
#![allow(dead_code, unused_imports)]
extern crate rustc_abi;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;

include!("../crates/lustr-native-core/src/lib.rs");
#[path = "../crates/lustr-native-core/src/upstream_definitions.rs"]
mod upstream_definitions;

#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

mod qualification {
    use super::*;
    use rustc_driver::{Callbacks, Compilation};
    use rustc_interface::interface;
    use rustc_middle::ty::TyCtxt;
    use std::path::PathBuf;

    struct Compile {
        source: PathBuf,
        output: PathBuf,
        scratch: PathBuf,
        alias: String,
        archive: upstream_definitions::Archive,
        completed: bool,
        status: u8,
    }

    impl Callbacks for Compile {
        fn config(&mut self, config: &mut interface::Config) {
            // A consumer Rust body query is a qualification failure. No fake
            // TypeckResults or skip-success response stands in for its work.
            config.override_queries = Some(|_, providers| {
                providers.queries.typeck_root = |_, _| panic!("import service attempted consumer Rust typechecking");
                providers.queries.analysis = |_, _| panic!("import service attempted complete Rust analysis");
            });
        }

        fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
            let action = std::env::var("LUSTR_QUALIFICATION_ACTION").unwrap_or_else(|_| "build".into());
            assert!(matches!(action.as_str(), "build" | "check" | "abandon"));
            eprintln!("shared-action={action}");
            let adapter_control = match std::env::var("LUSTR_QUALIFICATION_ADAPTER_FAILURE").as_deref() {
                Ok("missing") => upstream_definitions::abi::Control::MissingTool,
                Ok("failed") => upstream_definitions::abi::Control::FailedTool,
                Ok("malformed") => upstream_definitions::abi::Control::MalformedObject,
                Ok("empty") => upstream_definitions::abi::Control::EmptyObject,
                Err(_) => upstream_definitions::abi::Control::Normal,
                _ => panic!("unknown adapter qualification control"),
            };
            let provider = match upstream_definitions::Provider::capture(tcx, &self.alias, &self.archive, adapter_control) {
                Ok(provider) => provider,
                Err(error) => {
                    eprintln!("import outcome: {error:?}");
                    self.status = match error { upstream_definitions::Error::Unintegrated(_) => 2, upstream_definitions::Error::Failed(_) => 3 };
                    self.completed = true;
                    return Compilation::Stop;
                }
            };
            if std::env::var_os("LUSTR_QUALIFICATION_TAKE_EMISSION").is_some() {
                drop(driver::imports::Definitions::take_emission(&provider).unwrap());
            }
            let mut key = driver::SessionKey::synthetic(&self.source, false);
            key.target_triple = tcx.sess.target.llvm_target.to_string();
            key.profile = protocol::ProfileFlags {
                panic_abort: !tcx.sess.panic_strategy().unwinds(),
                overflow_checks: tcx.sess.overflow_checks(),
                debug_assertions: tcx.sess.opts.debug_assertions,
            };
            let mut session = driver::Session::new(self.scratch.join("native-state"), false, key.clone());
            let request = driver::Request {
                cmd: if action == "check" { driver::Cmd::Check } else { driver::Cmd::Build },
                session: key, file: self.source.clone(), out: self.output.clone(),
                json: false, timings: false, debug: false, emit_c: None, max_errors: 0,
                run_args: None, harness: false, cc: false, client: Default::default(),
            };
            match driver::prepare_imported(&request, &mut session, &provider) {
                Ok(driver::Preparation::Ready(prepared)) => {
                    assert!(!prepared.checked_functions().is_empty());
                    assert_eq!(prepared.imported_dependencies().len(), if provider.adapter_count() == 0 { 1 } else { 3 });
                    assert_eq!(prepared.imported_dependencies()[0].contents.as_ref(), self.archive.contents());
                    assert!(driver::imports::Definitions::take_emission(&provider).is_err(), "the owned code plan cannot be transferred twice");
                    let current = driver::imports::Definitions::dependencies(&provider);
                    assert_eq!(prepared.imported_dependencies().len(), current.len());
                    assert!(prepared.imported_dependencies().iter().zip(&current).all(|(owned, current)|
                        owned.identity == current.identity && owned.contents == current.contents));
                    eprintln!("imported-emission-transfer=once dependencies=preserved");
                    eprintln!("imported-generation={}", prepared.imported_dependencies()[0].identity);
                    let upstream_source = tcx.sess.source_map().files().iter()
                        .find_map(|file| file.src.as_ref().filter(|source| source.as_str() == prepared.source()))
                        .expect("native request and imported global context must use the same original source snapshot").clone();
                    assert_eq!(upstream_source.as_str(), prepared.source());
                    eprintln!("shared request: native-checked={} imported-inputs={} integer-adapters={} consumer-rust-typeck=0 consumer-rust-analysis=0",
                        prepared.checked_functions().len(), prepared.imported_dependencies().len(), provider.adapter_count());
                    for dependency in prepared.imported_dependencies() {
                        if dependency.identity == "imported-LLVM-adapter-IR" { std::fs::write(self.scratch.join("integer-adapters.ll"), &dependency.contents).unwrap(); }
                        if dependency.identity == "imported-LLVM-adapter-tool" { std::fs::write(self.scratch.join("integer-adapter-tool.txt"), &dependency.contents).unwrap(); }
                    }
                    if std::env::var_os("LUSTR_QUALIFICATION_OBJECT_FAILURE").is_some() {
                        std::fs::create_dir_all(self.scratch.join("native-state/imported-0.o")).unwrap();
                    }
                    if std::env::var_os("LUSTR_QUALIFICATION_REMOVE_SNAPSHOT").is_some() {
                        std::fs::remove_file(self.archive.path()).unwrap();
                    }
                    if action == "abandon" {
                        drop(prepared);
                        self.status = 0;
                        eprintln!("shared-prepared-request=abandoned");
                    } else {
                        let response = prepared.emit();
                        eprint!("{}", response.stderr);
                        self.status = response.code as u8;
                    }
                }
                Ok(driver::Preparation::Complete(response) | driver::Preparation::Rejected(response)
                    | driver::Preparation::Failed(response) | driver::Preparation::IntegrationIncomplete(response)) => {
                    eprint!("{}", response.stderr);
                    self.status = response.code as u8;
                }
                Err(error) => {
                    eprintln!("import outcome: Unintegrated({})", error.0);
                    self.status = 2;
                }
            }
            if self.status == 0 {
                let previous = std::fs::read(&self.output).ok();
                assert!(driver::prepare_imported(&request, &mut session, &provider).is_err(), "an imported generation must not be reused without a dependency contract");
                assert!(matches!(driver::prepare(&request, &mut session), driver::Preparation::Failed(response) if response.code == 3),
                    "ordinary native preparation must not reuse unkeyed imported state");
                assert_eq!(std::fs::read(&self.output).ok(), previous);
                eprintln!("imported-generation-reuse=refused previous-artifact=preserved");
            }
            self.completed = true;
            Compilation::Stop
        }
    }

    pub fn run() -> u8 {
        let arguments: Vec<_> = std::env::args_os().skip(1).collect();
        if arguments.first().is_some_and(|argument| argument == "--front-image-control") {
            unsafe extern "C" fn unused_sink(_: *mut std::ffi::c_void, _: i32, _: *const u8, _: usize) { panic!("refused entry called the sink") }
            assert!(!embedding_environment_supported());
            unsafe {
                assert_eq!(lustr_native_object_v1(0, std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
                assert_eq!(lustr_native_v2(0, std::ptr::null(), std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
                assert_eq!(lustr_native_retained_v1(0, std::ptr::null(), 0, std::ptr::null(), std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
                assert_eq!(lustr_native_session_v1(0, std::ptr::null(), 0, std::ptr::null(), std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
            }
            eprintln!("front-image entry guards: 4 passed");
            return 0;
        }
        if arguments.first().is_some_and(|argument| argument == "--front-client-control") {
            unsafe extern "C" fn unused_sink(_: *mut std::ffi::c_void, _: i32, _: *const u8, _: usize) { panic!("refused entry called the sink") }
            assert!(embedding_environment_supported());
            for key in [c"LUST_FRONT_STATE", c"LUST_FRONT_ARENA"] {
                let environment = [key.as_ptr(), c"requested".as_ptr()];
                unsafe {
                    assert_eq!(lustr_native_retained_v1(0, std::ptr::null(), 2, environment.as_ptr(), std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
                    assert_eq!(lustr_native_session_v1(0, std::ptr::null(), 2, environment.as_ptr(), std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
                }
            }
            eprintln!("front-image client guards: 4 passed");
            return 0;
        }
        assert!(arguments.len() == 4, "SOURCE RLIB OUTPUT NEW_SCRATCH");
        let source = PathBuf::from(&arguments[0]).canonicalize().unwrap();
        let output_path = PathBuf::from(&arguments[2]);
        let scratch = PathBuf::from(&arguments[3]);
        std::fs::create_dir(&scratch).unwrap();
        // Both the upstream loader and object extraction consume this one
        // owned snapshot, never independently reread a mutable dependency.
        let archive = match upstream_definitions::Archive::snapshot(std::path::Path::new(&arguments[1]), &scratch.join("libpeer.rlib")) {
            Ok(archive) => archive,
            Err(reason) => {
                eprintln!("import outcome: Failed({reason})");
                return 3;
            }
        };
        let library = archive.path().to_owned();
        let mut callbacks = Compile {
            source: source.clone(), output: output_path, scratch, alias: "peer".into(),
            archive, completed: false, status: 3,
        };
        // These are the declared qualification profile, not a routing change
        // to a user's existing request. Unwind is exercised separately below.
        let panic = std::env::var("LUSTR_QUALIFICATION_PANIC").unwrap_or_else(|_| "abort".into());
        assert!(matches!(panic.as_str(), "abort" | "unwind"));
        let args = vec!["rustc".into(), source.to_str().unwrap().into(), "--edition=2024".into(),
            "--crate-name=consumer".into(), "--sysroot".into(), env!("LUSTR_IMPORTED_SYSROOT").into(),
            "--extern".into(), format!("peer={}", library.display()), "-Copt-level=0".into(),
            "-Cdebug-assertions=yes".into(), "-Coverflow-checks=yes".into(), format!("-Cpanic={panic}")];
        let upstream_status = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut callbacks));
        if upstream_status != std::process::ExitCode::SUCCESS || !callbacks.completed { return 3; }
        callbacks.status
    }
}

fn main() -> std::process::ExitCode {
    raise_stack_limit();
    std::process::ExitCode::from(qualification::run())
}
