//! Read the scalar interface of an actual Rust metadata input.
use rustc_middle::ty::{self, TyCtxt, TypingEnv};
use rustc_session::config::Input;
use rustc_span::{FileName, def_id::{CRATE_DEF_INDEX, DefId}};
use std::{collections::BTreeSet, fs, io::{self, Write}, path::{Path, PathBuf}, process::ExitCode};

struct Request {
    alias: String,
    artifact: PathBuf,
    sdk_lib: PathBuf,
    search_paths: BTreeSet<PathBuf>,
    interface: Option<Interface>,
}

enum Interface {
    Scalar(Vec<u8>),
    RequiresRust,
}

fn integer(ty: ty::Ty<'_>) -> Option<u8> {
    Some(match *ty.kind() {
        ty::Int(ty::IntTy::I8) => 0,
        ty::Int(ty::IntTy::I16) => 1,
        ty::Int(ty::IntTy::I32) => 2,
        ty::Int(ty::IntTy::I64) => 3,
        ty::Int(ty::IntTy::Isize) => 4,
        ty::Uint(ty::UintTy::U8) => 5,
        ty::Uint(ty::UintTy::U16) => 6,
        ty::Uint(ty::UintTy::U32) => 7,
        ty::Uint(ty::UintTy::U64) => 8,
        ty::Uint(ty::UintTy::Usize) => 9,
        _ => return None,
    })
}

fn count(out: &mut impl Write, length: usize) -> io::Result<()> {
    out.write_all(&u32::try_from(length).map_err(io::Error::other)?.to_le_bytes())
}

fn artifact_hash(artifact: &Path) -> io::Result<rustc_span::SourceFileHash> {
    rustc_span::SourceFileHash::new(rustc_span::SourceFileHashAlgorithm::Sha256, fs::File::open(artifact)?)
}

impl Request {
    fn read(&self, tcx: TyCtxt<'_>) -> Result<Interface, Box<dyn std::error::Error>> {
        use rustc_hir::def::{DefKind, Res};
        let mut owner = None;
        let mut dependencies = BTreeSet::new();
        let mut needs_dependency_context = false;
        for &krate in tcx.crates(()) {
            let source = tcx.used_crate_source(krate);
            if [source.rmeta.as_ref(), source.rlib.as_ref()].into_iter().flatten().any(|path| path == &self.artifact) {
                if owner.replace(krate).is_some() {
                    return Err("ambiguous upstream metadata identity".into());
                }
                continue;
            }
            for path in source.paths() {
                let path = path.canonicalize()?;
                if !path.starts_with(&self.sdk_lib) {
                    if !path.parent().is_some_and(|directory| self.search_paths.contains(directory)) {
                        return Err("the loaded metadata is outside the installed SDK and supplied dependency search paths".into());
                    }
                    needs_dependency_context = true;
                }
                dependencies.insert(path);
            }
        }
        let krate = owner.ok_or("Rust did not load the requested metadata")?;
        // The native scalar view cannot represent this graph. Rust has
        // resolved the supplied metadata identities and will own the request.
        if needs_dependency_context || !tcx.trait_impls_in_crate(krate).is_empty() {
            return Ok(Interface::RequiresRust);
        }
        let root = DefId { krate, index: CRATE_DEF_INDEX };
        let mut functions = Vec::new();
        for child in tcx.module_children(root) {
            if !child.vis.is_public() {
                continue;
            }
            let Res::Def(DefKind::Fn, def) = child.res else {
                return Ok(Interface::RequiresRust);
            };
            if !child.reexport_chain.is_empty() || tcx.generics_of(def).count() != 0
                || !tcx.param_env(def).is_empty() || tcx.is_const_fn(def)
            {
                return Ok(Interface::RequiresRust);
            }
            let signature = tcx.normalize_erasing_regions(TypingEnv::fully_monomorphized(), tcx.fn_sig(def).instantiate_identity());
            if !signature.bound_vars().is_empty() {
                return Ok(Interface::RequiresRust);
            }
            let signature = signature.skip_binder();
            if !signature.fn_sig_kind.is_safe() || !matches!(signature.abi(), rustc_abi::ExternAbi::Rust)
                || signature.c_variadic()
            {
                return Ok(Interface::RequiresRust);
            }
            let Some(parameters) = signature.inputs().iter().map(|&ty| integer(ty)).collect::<Option<Vec<_>>>() else {
                return Ok(Interface::RequiresRust);
            };
            let Some(result) = integer(signature.output()) else {
                return Ok(Interface::RequiresRust);
            };
            if parameters.len() > 64 {
                return Ok(Interface::RequiresRust);
            }
            let mut row = Vec::new();
            super::write_string(&mut row, child.ident.name.as_str())?;
            row.write_all(&tcx.def_path_hash(def).local_hash().as_u64().to_le_bytes())?;
            count(&mut row, parameters.len())?;
            row.write_all(&parameters)?;
            row.write_all(&[result])?;
            super::write_string(&mut row, tcx.symbol_name(ty::Instance::mono(tcx, def)).name)?;
            functions.push(row);
        }
        if functions.is_empty() || functions.len() > 65_535 || dependencies.len() > 512 {
            return Ok(Interface::RequiresRust);
        }
        let mut out = b"lustr-external-scalar-v1\0".to_vec();
        super::write_string(&mut out, &artifact_hash(&self.artifact)?.to_string())?;
        super::write_string(&mut out, tcx.crate_name(krate).as_str())?;
        super::write_string(&mut out, tcx.extra_filename(krate))?;
        out.write_all(&tcx.stable_crate_id(krate).as_u64().to_le_bytes())?;
        out.write_all(&tcx.crate_hash(krate).as_fingerprint().to_le_bytes())?;
        out.write_all(&[u8::from(tcx.required_panic_strategy(krate).is_some_and(|strategy| !strategy.unwinds()))])?;
        count(&mut out, dependencies.len())?;
        for path in dependencies {
            super::write_string(&mut out, path.to_str().ok_or("SDK metadata path is not Unicode")?)?;
        }
        count(&mut out, functions.len())?;
        for row in functions {
            out.write_all(&row)?;
        }
        Ok(Interface::Scalar(out))
    }
}

impl rustc_driver::Callbacks for Request {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.input = Input::Str {
            name: FileName::anon_source_code("lustr_metadata_client"),
            input: format!("extern crate {};", self.alias),
        };
        config.override_queries = Some(|_, providers| {
            providers.queries.typeck_root = |tcx, _| tcx.dcx().fatal("metadata selection attempted local body typechecking");
            providers.queries.analysis = |tcx, _| tcx.dcx().fatal("metadata selection attempted local crate analysis");
        });
    }

    fn after_expansion<'tcx>(
        &mut self, _: &rustc_interface::interface::Compiler, tcx: TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        self.interface = Some(self.read(tcx)
            .unwrap_or_else(|error| tcx.dcx().fatal(format!("extern interface requires more shared facts: {error}"))));
        rustc_driver::Compilation::Stop
    }
}

fn inspect(args: &[String]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let [alias, path, search_paths @ ..] = args else {
        return Err("expected one extern name and metadata file".into());
    };
    if alias.is_empty() || !alias.bytes().enumerate().all(|(index, byte)| {
        byte == b'_' || byte.is_ascii_alphabetic() || index != 0 && byte.is_ascii_digit()
    }) {
        return Err("invalid extern name".into());
    }
    let artifact = Path::new(path).canonicalize()?;
    if !matches!(artifact.extension().and_then(|ext| ext.to_str()), Some("rmeta" | "rlib")) {
        return Err("extern selection requires a Rust metadata file or archive".into());
    }
    let hash = artifact_hash(&artifact)?;
    let exe = std::env::current_exe()?;
    let sdk = exe.parent().and_then(Path::parent).ok_or("host SDK root is absent")?.to_owned();
    let sdk_lib = sdk.join("lib").canonicalize()?;
    let mut params = vec!["rustc".into(), artifact.to_str().ok_or("extern path is not Unicode")?.into(),
        "--edition=2024".into(), "--crate-name=lustr_metadata_client".into(), "--crate-type=lib".into(),
        "--emit=metadata".into(), "--sysroot".into(), sdk.to_str().ok_or("SDK path is not Unicode")?.into(),
        "--extern".into(), format!("{alias}={}", artifact.display())];
    let mut directories = BTreeSet::new();
    for path in search_paths {
        params.extend(["-L".into(), format!("dependency={path}")]);
        if let Ok(directory) = Path::new(path).canonicalize() {
            directories.insert(directory);
        }
    }
    let mut request = Request { alias: alias.clone(), artifact, sdk_lib, search_paths: directories, interface: None };
    let result = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&params, &mut request));
    if result != ExitCode::SUCCESS {
        return Ok(result);
    }
    if artifact_hash(&request.artifact)? != hash {
        return Err("extern input changed while Rust was reading it".into());
    }
    let mut output = io::stdout().lock();
    match request.interface.ok_or("no upstream interface produced")? {
        Interface::Scalar(bytes) => {
            output.write_all(&bytes)?;
            Ok(ExitCode::SUCCESS)
        }
        Interface::RequiresRust => {
            output.write_all(b"lustr-extern-requires-rust-v1\0")?;
            write!(output, "{hash}")?;
            Ok(ExitCode::from(crate::protocol::RUST_COMPLEMENT_REQUIRED as u8))
        }
    }
}

pub(super) fn run(args: &[String]) -> ExitCode {
    inspect(args).unwrap_or_else(|error| {
        eprintln!("read Rust extern metadata: {error}");
        ExitCode::FAILURE
    })
}
