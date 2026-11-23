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
#[path = "native-projection-qualification.rs"]
mod native_projection_qualification;

#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

mod qualification {
    use super::*;
    use rustc_driver::{Callbacks, Compilation};
    use rustc_interface::interface;
    use rustc_middle::ty::TyCtxt;
    use rustc_session::config::Input;
    use rustc_span::{FileName, SourceFileHash, SourceFileHashAlgorithm};
    use std::{ffi::OsString, path::PathBuf, sync::Arc};

    enum Imported {
        Archive(upstream_definitions::Archive),
        Metadata(upstream_definitions::BoundFile),
    }

    struct MetadataQuery<'a> {
        alias: &'a str,
        file: &'a upstream_definitions::BoundFile,
        answer: Option<Result<upstream_definitions::Provider, upstream_definitions::Error>>,
    }

    impl Callbacks for MetadataQuery<'_> {
        fn config(&mut self, config: &mut interface::Config) {
            config.input = Input::Str {
                name: FileName::anon_source_code("lustr_metadata_client"),
                input: format!("extern crate {};", self.alias),
            };
            config.override_queries = Some(|_, providers| {
                providers.queries.typeck_root = |_, _| panic!("metadata client attempted body typechecking");
                providers.queries.analysis = |_, _| panic!("metadata client attempted crate analysis");
            });
        }

        fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
            self.answer = Some(upstream_definitions::Provider::capture_metadata(tcx, self.alias, self.file));
            Compilation::Stop
        }
    }

    fn obtain_metadata(alias: &str, file: &upstream_definitions::BoundFile) -> Result<upstream_definitions::Provider, String> {
        let args = vec!["rustc".into(), file.path().to_string_lossy().into_owned(),
            "--edition=2024".into(), "--crate-name=lustr_metadata_client".into(),
            "--crate-type=lib".into(), "--emit=metadata".into(),
            "--sysroot".into(), env!("LUSTR_IMPORTED_SYSROOT").into(),
            "--extern".into(), format!("{alias}={}", file.path().display())];
        let mut query = MetadataQuery { alias, file, answer: None };
        let status = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut query));
        if status != std::process::ExitCode::SUCCESS {
            return Err("upstream metadata loading failed".into());
        }
        query.answer.ok_or("upstream metadata query was not run")?.map_err(|error| format!("{error:?}"))
    }

    fn metadata_history(args: &[OsString]) -> Result<u8, String> {
        let [alias, source, scratch, metadata @ ..] = args else {
            return Err("usage: --metadata-history CRATE SOURCE NEW_SCRATCH RMETA [RMETA ...]".into());
        };
        let alias = alias.to_str().filter(|name| {
            !name.is_empty() && name.bytes().enumerate().all(|(index, byte)| {
                byte == b'_' || byte.is_ascii_alphabetic() || index != 0 && byte.is_ascii_digit()
            })
        }).ok_or("invalid crate alias")?;
        if metadata.is_empty() { return Err("no metadata requests supplied".into()); }
        let source = PathBuf::from(source).canonicalize().map_err(|error| error.to_string())?;
        let scratch = PathBuf::from(scratch);
        std::fs::create_dir(&scratch).map_err(|error| error.to_string())?;
        let mut key = driver::SessionKey::synthetic(&source, false);
        key.profile.panic_abort = false;
        let mut session = driver::Session::new(scratch.join("native-state"), true, key.clone());
        session.set_output(driver::OutputKind::Standalone);
        let request = driver::Request {
            cmd: driver::Cmd::Check, session: key, file: source, out: scratch.join("program"),
            json: false, timings: false, debug: false, emit_c: None, max_errors: 0,
            run_args: None, harness: false, cc: false, library_options: Default::default(), client: Default::default(),
        };
        let mut current: Option<(upstream_definitions::BoundFile, upstream_definitions::Provider)> = None;
        let mut status = 0;
        for (step, path) in metadata.iter().enumerate() {
            let bytes: Arc<[u8]> = std::fs::read(path).map_err(|error| error.to_string())?.into();
            let refresh = current.as_ref().is_none_or(|(file, _)| file.contents() != bytes.as_ref());
            if refresh {
                let staging = scratch.join(format!("metadata-{step}"));
                std::fs::create_dir(&staging).map_err(|error| error.to_string())?;
                let name = std::path::Path::new(path).file_name().ok_or("missing metadata filename")?;
                let file = upstream_definitions::BoundFile::snapshot_bytes(bytes, &staging.join(name))?;
                let provider = obtain_metadata(alias, &file)?;
                current = Some((file, provider));
            }
            let (file, provider) = current.as_ref().unwrap();
            let response = match driver::prepare_projection_import(&request, &mut session, provider) {
                Ok(driver::Preparation::Ready(prepared)) => {
                    let program = prepared.program();
                    let checked: Vec<_> = prepared.checked_functions().iter().map(|&id| {
                        program.interner.get(program.fn_def(def::LocalDefId(id as u32)).name)
                    }).collect();
                    assert_eq!(prepared.imported_dependencies()[0].contents.as_ref(), file.contents());
                    eprintln!("step={step} checked={checked:?}");
                    prepared.emit()
                }
                Ok(driver::Preparation::Complete(response) | driver::Preparation::Rejected(response)
                    | driver::Preparation::Failed(response) | driver::Preparation::IntegrationIncomplete(response)) => response,
                Err(error) => return Err(error.0.into()),
            };
            status = response.code as u8;
            eprint!("{}", response.stderr);
            eprintln!("step={step} upstream-queries={} native-status={status} {}",
                u8::from(refresh), session.front.last_visits);
            if request.out.exists() { return Err("metadata check produced an executable".into()); }
            if status > 1 { return Ok(status); }
        }
        Ok(status)
    }

    fn native_span(args: &[OsString]) -> Result<u8, String> {
        let args: Vec<_> = args.iter().map(|arg| arg.to_str().ok_or("non-Unicode native span argument"))
            .collect::<Result<_, _>>()?;
        let [source, start, end, digest, output, name, metadata, overflow, assertions] = args.as_slice() else {
            return Err("usage: --native-span SOURCE START END SHA256 OBJECT CRATE METADATA OVERFLOW ASSERTIONS".into());
        };
        let snapshot = std::fs::read_to_string(source).map_err(|error| error.to_string())?;
        let actual = SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, &snapshot).to_string();
        if actual != *digest {
            return Err("native span identifies another source snapshot".into());
        }
        let range = start.parse().map_err(|_| "invalid native item start")?
            ..end.parse().map_err(|_| "invalid native item end")?;
        let args = vec!["--crate-name".into(), (*name).into(), "--edition=2024".into(),
            "--crate-type=lib".into(), "--emit=obj".into(), "--target=x86_64-unknown-linux-gnu".into(),
            "-Cpanic=abort".into(), "-Copt-level=0".into(), "-Cdebuginfo=0".into(),
            format!("-Cmetadata={metadata}"), format!("-Coverflow-checks={overflow}"),
            format!("-Cdebug-assertions={assertions}"), (*source).into(), "-o".into(), (*output).into()];
        let prepared = native_object::prepare_item(&args, &snapshot, range)?;
        let emitted = native_object::emit(prepared)?;
        let [function] = emitted.functions() else { return Err("expected one emitted native function".into()) };
        let program = emitted.prepared().program();
        let name = program.interner.get(program.fn_def(def::LocalDefId(0)).name);
        let symbol = std::str::from_utf8(function.symbol()).map_err(|error| error.to_string())?;
        match emitted.publish_object()? {
            unit_publication::Publication::Durable => {
                println!("native object export: {name} {symbol}");
                Ok(0)
            }
            unit_publication::Publication::DurabilityUnknown(error) => Err(format!("native object publication durability unknown: {error}")),
        }
    }

    struct Compile {
        source: PathBuf,
        output: PathBuf,
        scratch: PathBuf,
        alias: String,
        input: Imported,
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
            let metadata = matches!(&self.input, Imported::Metadata(_));
            let action = std::env::var("LUSTR_QUALIFICATION_ACTION")
                .unwrap_or_else(|_| if metadata { "check" } else { "build" }.into());
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
            let (provider, input_path, input_bytes) = match &self.input {
                Imported::Archive(archive) => (
                    upstream_definitions::Provider::capture(tcx, &self.alias, archive, adapter_control),
                    archive.path(), archive.contents(),
                ),
                Imported::Metadata(file) => (
                    upstream_definitions::Provider::capture_metadata(tcx, &self.alias, file),
                    file.path(), file.contents(),
                ),
            };
            let provider = match provider {
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
                run_args: None, harness: false, cc: false, library_options: Default::default(), client: Default::default(),
            };
            match driver::prepare_imported(&request, &mut session, &provider) {
                Ok(driver::Preparation::Ready(prepared)) => {
                    assert!(!prepared.checked_functions().is_empty());
                    assert_eq!(prepared.imported_dependencies().len(), if provider.adapter_count() == 0 { 1 } else { 3 });
                    assert_eq!(prepared.imported_dependencies()[0].contents.as_ref(), input_bytes);
                    assert!(driver::imports::Definitions::take_emission(&provider).is_err(), "no owned code plan remains in the provider");
                    let current = driver::imports::Definitions::dependencies(&provider);
                    assert_eq!(prepared.imported_dependencies().len(), current.len());
                    assert!(prepared.imported_dependencies().iter().zip(&current).all(|(owned, current)|
                        owned.identity == current.identity && owned.contents == current.contents));
                    if metadata {
                        eprintln!("imported-emission-transfer=none dependencies=preserved");
                    } else {
                        eprintln!("imported-emission-transfer=once dependencies=preserved");
                    }
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
                        std::fs::remove_file(input_path).unwrap();
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
        if arguments.first().is_some_and(|arg| arg == "--projection") {
            return native_projection_qualification::run(&arguments[1..]);
        }
        if arguments.first().is_some_and(|arg| arg == "--metadata-history") {
            return metadata_history(&arguments[1..]).unwrap_or_else(|error| {
                eprintln!("metadata history: {error}");
                3
            });
        }
        if arguments.first().is_some_and(|arg| arg == "--native-span") {
            return native_span(&arguments[1..]).unwrap_or_else(|error| {
                eprintln!("native item span: {error}");
                3
            });
        }
        if arguments.first().is_some_and(|argument| argument == "--front-image-control") {
            unsafe extern "C" fn unused_sink(_: *mut std::ffi::c_void, _: i32, _: *const u8, _: usize) { panic!("refused entry called the sink") }
            assert!(!embedding_environment_supported());
            unsafe {
                assert_eq!(lustr_native_object_v1(0, std::ptr::null(), unused_sink, std::ptr::null_mut()), 3);
            }
            eprintln!("front-image object entry guard: passed");
            return 0;
        }
        let (metadata, alias, arguments) = if arguments.first().is_some_and(|arg| arg == "--metadata") {
            assert!(arguments.len() == 6, "--metadata CRATE SOURCE RMETA OUTPUT NEW_SCRATCH");
            (true, arguments[1].to_str().expect("Rust crate name").to_owned(), &arguments[2..])
        } else {
            assert!(arguments.len() == 4, "SOURCE RLIB OUTPUT NEW_SCRATCH");
            (false, "peer".into(), arguments.as_slice())
        };
        let source = PathBuf::from(&arguments[0]).canonicalize().unwrap();
        let output_path = PathBuf::from(&arguments[2]);
        let scratch = PathBuf::from(&arguments[3]);
        std::fs::create_dir(&scratch).unwrap();
        // Both the upstream loader and object extraction consume this one
        // owned snapshot, never independently reread a mutable dependency.
        let supplied = std::path::Path::new(&arguments[1]);
        let snapshot = if metadata {
            let filename = supplied.file_name().expect("Rust metadata filename");
            upstream_definitions::BoundFile::snapshot(supplied, &scratch.join(filename)).map(Imported::Metadata)
        } else {
            upstream_definitions::Archive::snapshot(supplied, &scratch.join("libpeer.rlib")).map(Imported::Archive)
        };
        let input = match snapshot {
            Ok(input) => input,
            Err(reason) => {
                eprintln!("import outcome: Failed({reason})");
                return 3;
            }
        };
        let library = match &input {
            Imported::Archive(archive) => archive.path(),
            Imported::Metadata(file) => file.path(),
        }.to_owned();
        let mut callbacks = Compile {
            source: source.clone(), output: output_path, scratch, alias: alias.clone(),
            input, completed: false, status: 3,
        };
        // These are the declared qualification profile, not a routing change
        // to a user's existing request. Unwind is exercised separately below.
        let panic = std::env::var("LUSTR_QUALIFICATION_PANIC")
            .unwrap_or_else(|_| if metadata { "unwind" } else { "abort" }.into());
        assert!(matches!(panic.as_str(), "abort" | "unwind"));
        let mut args = vec!["rustc".into(), source.to_str().unwrap().into(), "--edition=2024".into(),
            "--crate-name=consumer".into(), "--sysroot".into(), env!("LUSTR_IMPORTED_SYSROOT").into(),
            "--extern".into(), format!("{alias}={}", library.display()), "-Copt-level=0".into(),
            "-Cdebug-assertions=yes".into(), "-Coverflow-checks=yes".into(), format!("-Cpanic={panic}")];
        if metadata {
            args.push("--emit=metadata".into());
        }
        let upstream_status = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut callbacks));
        if upstream_status != std::process::ExitCode::SUCCESS || !callbacks.completed { return 3; }
        callbacks.status
    }
}

fn main() -> std::process::ExitCode {
    raise_stack_limit();
    std::process::ExitCode::from(qualification::run())
}
