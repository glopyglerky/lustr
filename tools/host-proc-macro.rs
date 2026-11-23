//! Run the pinned Rust implementation for host procedural macro crates.
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_attr_ir;
extern crate rustc_codegen_llvm;
extern crate rustc_codegen_ssa;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_session;
extern crate rustc_structures;
extern crate rustc_target;

#[cfg(feature = "upstream-semantic")]
include!("../crates/lustr-native-core/src/lib.rs");

#[cfg(feature = "upstream-semantic")]
#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

use rustc_session::{EarlyDiagCtxt, config};
use rustc_structures::CrateType;
use std::process::ExitCode;

#[path = "host-crate-interface.rs"]
mod host_crate_interface;
#[path = "host-freestanding.rs"]
mod host_freestanding;

rustc_driver::override_c_allocator_in_binary!();


use std::{fs, io::{self, Write}, path::{Path, PathBuf}};
use rustc_session::config::{OutFileName, OutputType};
use rustc_span::{SourceFileHash, SourceFileHashAlgorithm, def_id::LOCAL_CRATE};

struct Definition {
    name: String,
    filename: String,
    start: u32,
    end: u32,
    source_digest: String,
}

struct HostProcMacro {
    record_spans: bool,
    output: Option<PathBuf>,
    definitions: Vec<Definition>,
}

impl rustc_driver::Callbacks for HostProcMacro {
    fn config(&mut self, compiler: &mut rustc_interface::interface::Config) {
        if compiler.opts.crate_types != [CrateType::ProcMacro]
            || compiler.opts.target_triple.tuple() != config::host_tuple()
        {
            EarlyDiagCtxt::new(config::ErrorOutputType::default())
                .early_fatal("the host procedural macro compiler requires a host proc-macro crate");
        }
    }

    fn after_analysis(
        &mut self,
        _: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        if !self.record_spans || !tcx.sess.opts.output_types.contains_key(&OutputType::Exe) {
            return rustc_driver::Compilation::Continue;
        }
        let OutFileName::Real(path) = rustc_session::output::out_filename(
            tcx.sess, CrateType::ProcMacro, tcx.output_filenames(()), tcx.crate_name(LOCAL_CRATE),
        ) else {
            return rustc_driver::Compilation::Continue;
        };
        self.output = Some(path);

        let source_map = tcx.sess.source_map();
        for &definition in &tcx.resolutions(()).proc_macros {
            let id = tcx.local_def_id_to_hir_id(definition);
            if !rustc_attr_ir::find_attr!(tcx.hir_attrs(id), ProcMacro) {
                continue;
            }
            let span = tcx.hir_span(id);
            let file = source_map.lookup_source_file(span.lo());
            if source_map.span_to_snippet(span).is_err() { continue; }
            let filename = file.name.prefer_remapped_unconditionally().to_string();
            let source_digest = file.name.clone().into_local_path()
                .and_then(|path| fs::read_to_string(path).ok())
                .filter(|source| file.src_hash.matches(source))
                .map(|source| SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, source).to_string())
                .unwrap_or_default();

            self.definitions.push(Definition {
                name: tcx.hir_name(id).to_string(),
                filename,
                start: file.relative_position(span.lo()).0,
                end: file.relative_position(span.hi()).0,
                source_digest,
            });
        }
        rustc_driver::Compilation::Continue
    }
}

fn write_string(out: &mut impl Write, value: &str) -> io::Result<()> {
    let size = u32::try_from(value.len()).map_err(io::Error::other)?;
    out.write_all(&size.to_le_bytes())?;
    out.write_all(value.as_bytes())
}

fn write_receipt(artifact: &Path, definitions: &[Definition]) -> io::Result<()> {
    let mut filename = artifact.as_os_str().to_owned();
    filename.push(".lustr-spans");
    let mut temporary = filename.clone();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let digest = SourceFileHash::new(SourceFileHashAlgorithm::Sha256, fs::File::open(artifact)?)?;
    let mut out = fs::File::create_new(&temporary)?;
    let result = (|| {
        out.write_all(b"lustr-proc-macro-spans-v2\0")?;
        write_string(&mut out, &digest.to_string())?;
        out.write_all(&u32::try_from(definitions.len()).map_err(io::Error::other)?.to_le_bytes())?;
        for definition in definitions {
            write_string(&mut out, &definition.name)?;
            write_string(&mut out, &definition.filename)?;
            out.write_all(&definition.start.to_le_bytes())?;
            out.write_all(&definition.end.to_le_bytes())?;
            write_string(&mut out, &definition.source_digest)?;
        }
        out.sync_all()?;
        fs::rename(&temporary, filename)?;
        fs::File::open(artifact.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")))?.sync_all()
    })();
    if result.is_err() {
        drop(out);
        let _ = fs::remove_file(temporary);
    }
    result
}

fn main() -> ExitCode {
    let diagnostics = EarlyDiagCtxt::new(config::ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&diagnostics);
    let args = rustc_driver::args::raw_args(&diagnostics);
    if args.get(1).is_some_and(|arg| arg == "--lustr-read-crate-interface") {
        return host_crate_interface::run(&args[2..]);
    }
    rustc_driver::install_ice_hook(rustc_driver::DEFAULT_BUG_REPORT_URL, |_| ());
    rustc_driver::install_ctrlc_handler();
    if args.get(1).is_some_and(|arg| arg == "--lustr-build-freestanding") {
        return host_freestanding::run(host_freestanding::Artifact::Freestanding, &args[0], &args[2..]);
    }
    if args.get(1).is_some_and(|arg| arg == "--lustr-build-static-library") {
        return host_freestanding::run(host_freestanding::Artifact::StaticLibrary, &args[0], &args[2..]);
    }
    if args.get(1).is_some_and(|arg| arg == "--lustr-build-rust-unit") {
        return host_freestanding::run(host_freestanding::Artifact::CargoUnit, &args[0], &args[2..]);
    }
    let record_spans = std::env::current_exe().ok().is_some_and(|path| {
        path.ancestors().nth(3).is_some_and(|directory| {
            directory.join("lustr-macro-expansion/bin/lustr-expand-proc-macro").is_file()
        })
    });
    let mut host = HostProcMacro { record_spans, output: None, definitions: Vec::new() };
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::compiler_entrypoint(&args, &mut host);
    });
    if status == ExitCode::SUCCESS
        && let Some(path) = host.output
        && let Err(error) = write_receipt(&path, &host.definitions)
    {
        eprintln!("cannot record host procedural macro definitions: {error}");
        return ExitCode::FAILURE;
    }
    status
}
