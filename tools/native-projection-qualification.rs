//! Join a pinned rustc projection from dependency metadata to native checking.
use std::{ffi::OsString, path::PathBuf, sync::Arc};

use rustc_driver::{Callbacks, Compilation};
use rustc_hir::{def::Res, def_id::{CRATE_DEF_ID, CRATE_DEF_INDEX, DefId}};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_interface::interface;
use rustc_middle::ty::{self as rust, TyCtxt, Unnormalized};
use rustc_session::config::Input;
use rustc_span::FileName;
use rustc_trait_selection::traits::{Obligation, ObligationCause, ObligationCtxt};

use crate::{
    check::{collect::Program, generics::ImportedProjection}, def, diag::{Diag, Span},
    driver::{self, imports::{Definitions, Dependency, Emission}, graph::Namespace},
    intern::{Interner, Sym}, protocol, resolve::imports::{Bind, DefInfo, ExternInfo, InsertOut,
        ResKind, ResolvedTables, Target, VisKind}, ty,
    upstream_definitions::{ids, integer},
};

struct Projection {
    root: def::DefId,
    receiver: def::DefId,
    family: def::DefId,
    item: def::DiskDefId,
    referent: Option<ty::IntTy>,
    metadata: Arc<[u8]>,
}

impl Projection {
    fn capture(tcx: TyCtxt<'_>, snapshot: &PathBuf, metadata: Arc<[u8]>) -> Result<Self, String> {
        let krate = tcx.crates(()).iter().copied().find(|&krate| tcx.crate_name(krate).as_str() == "schema")
            .ok_or("schema metadata was not loaded")?;
        if tcx.used_crate_source(krate).rmeta.as_ref() != Some(snapshot)
            || std::fs::read(snapshot).map_err(|e| e.to_string())?.as_slice() != metadata.as_ref()
        {
            return Err("upstream did not consume the owned metadata generation".into());
        }
        let root = DefId { krate, index: CRATE_DEF_INDEX };
        let exported = |name, kind| tcx.module_children(root).iter().find_map(|binding| {
            match binding.res {
                Res::Def(actual, id) if actual == kind && binding.ident.name.as_str() == name
                    && binding.vis.is_public() && binding.reexport_chain.is_empty() => Some(id),
                _ => None,
            }
        });
        let receiver = exported("Codec", rustc_hir::def::DefKind::Struct).ok_or("expected public receiver")?;
        let family = exported("Family", rustc_hir::def::DefKind::Trait).ok_or("expected public trait")?;
        let item = tcx.associated_items(family).in_definition_order().find(|item| {
            tcx.item_name(item.def_id).as_str() == "View" && item.is_type()
        }).ok_or("expected public associated type")?.def_id;
        let parameters = tcx.generics_of(item);
        if tcx.generics_of(receiver).count() != 0 || tcx.generics_of(family).count() != 1
            || parameters.parent_count != 1 || parameters.own_params.len() != 1
            || !matches!(parameters.own_params[0].kind, rust::GenericParamDefKind::Lifetime)
        {
            return Err("projection requires a concrete receiver and one lifetime argument".into());
        }
        let receiver_ty = tcx.type_of(receiver).instantiate_identity().skip_normalization();
        let infcx = tcx.infer_ctxt().build(rust::TypingMode::non_body_analysis());
        let obligations = ObligationCtxt::new(&infcx);
        let region = rust::Region::new_placeholder(
            tcx, rust::PlaceholderRegion::new(infcx.create_next_universe(), rust::BoundRegion {
                var: rust::BoundVar::from_u32(0), kind: rust::BoundRegionKind::Anon,
            }),
        );
        let arguments: [rust::GenericArg<'_>; 2] = [receiver_ty.into(), region.into()];
        let alias = rust::Ty::new_projection(tcx, rust::IsRigid::No, item, arguments);
        let env = rust::ParamEnv::empty();
        let cause = ObligationCause::dummy();
        obligations.register_obligation(Obligation::new(tcx, cause.clone(), env,
            rust::PredicateKind::Clause(rust::ClauseKind::WellFormed(alias.into()))));
        let resolved = obligations.normalize(&cause, env, Unnormalized::new(alias));
        let errors = obligations.evaluate_obligations_error_on_ambiguity();
        let referent = if errors.no_errors() {
            if !obligations.resolve_regions(CRATE_DEF_ID, env, []).is_empty() {
                return Err("upstream projection has unresolved lifetime constraints".into());
            }
            let rust::Ref(result_region, referent, rustc_hir::Mutability::Not) = *resolved.kind()
                else { return Err("projection result is not a shared reference".into()); };
            if result_region != region {
                return Err("projection result does not preserve the lifetime argument".into());
            }
            Some(integer(referent).ok_or("projection referent type is unrepresented")?)
        } else {
            let mut has_candidates = false;
            tcx.for_each_relevant_impl(family, receiver_ty, |_| has_candidates = true);
            if has_candidates {
                return Err(format!("upstream projection could not be normalized: {errors:?}"));
            }
            None
        };
        Ok(Self { root: ids(tcx, root).0, receiver: ids(tcx, receiver).0,
            family: ids(tcx, family).0, item: ids(tcx, item).1, referent, metadata })
    }

    fn flat(&self, interner: &Interner, id: def::DefId) -> Sym {
        interner.intern(&format!("$lustr$extern${:016x}${:016x}", self.item.krate.0, id.hash.0))
    }
}

impl Definitions for Projection {
    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag> {
        let failure = || Diag::new("L0508", Span::default(), "conflicting projection import", "Use one exact metadata generation.");
        let alias = "schema";
        if tables.externs.iter().any(|entry| entry.name == alias)
            || tables.module_defs.iter().chain(tables.def_info.keys()).any(|def| def.krate == self.root.krate)
        {
            return Err(failure());
        }
        let module = tables.add_crate_root(interner.intern(alias));
        tables.module_defs[module.0 as usize] = self.root;
        tables.externs.push(ExternInfo { name: alias.into(), def: self.root, is_std_lib: false,
            owner: ty::defs::ROOT_MODULE, root_module: Some(module), unsupported: None });
        for (id, name, kind) in [(self.receiver, "Codec", ResKind::Adt), (self.family, "Family", ResKind::Trait)] {
            let flat = self.flat(interner, id);
            if tables.def_info.insert(id, DefInfo { kind, flat, module, vis: VisKind::Pub }).is_some()
                || tables.flat[0].insert(flat, id).is_some()
                || !matches!(tables.insert(module, Namespace::Type, interner.intern(name),
                    Bind { target: Target::Def(id), vis: VisKind::Pub, rank: 2, def_module: module }), InsertOut::Changed)
            {
                return Err(failure());
            }
            tables.display_names.insert(flat, interner.intern(&format!("{alias}::{name}")));
        }
        Ok(())
    }

    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>> {
        let receiver = self.flat(&program.interner, self.receiver);
        let family = self.flat(&program.interner, self.family);
        let item = program.interner.intern("View");
        program.imported_crates.insert(self.root.krate, self.item.krate);
        program.gx.imported_projections.get_or_insert_with(Default::default).entry(family).or_default().push(
            ImportedProjection {
                definition: self.item, receiver, item,
                referent: self.referent.map(crate::check::generics::ProjectionReferent::Integer),
            });
        Ok(())
    }

    fn validate_checked(&self, program: &Program) -> Result<(), String> {
        let receiver = self.flat(&program.interner, self.receiver);
        let family = self.flat(&program.interner, self.family);
        let item = program.interner.intern("View");
        if program.imported_crates.get(&self.root.krate) == Some(&self.item.krate)
            && program.gx.imported_projections.as_ref().and_then(|map| map.get(&family)).is_some_and(|facts| {
                facts.iter().any(|fact| fact.definition == self.item && fact.receiver == receiver && fact.item == item
                    && fact.referent == self.referent.map(crate::check::generics::ProjectionReferent::Integer))
            })
        {
            Ok(())
        } else {
            Err("checked projection differs from the imported answer".into())
        }
    }

    fn dependencies(&self) -> Vec<Dependency> {
        vec![Dependency { identity: format!("schema::{:?}", self.item), contents: self.metadata.clone() }]
    }

    fn take_emission(&self) -> Result<Box<dyn Emission + 'static>, String> {
        Ok(Box::new(NoObjects))
    }
}

// A checked associated type carries no runtime code into the link.
struct NoObjects;
impl Emission for NoObjects {
    fn emit(self: Box<Self>) -> Result<Vec<Arc<[u8]>>, String> { Ok(Vec::new()) }
}

struct Query {
    metadata: Arc<[u8]>,
    snapshot: PathBuf,
    result: Option<Result<Projection, String>>,
}

impl Callbacks for Query {
    fn config(&mut self, config: &mut interface::Config) {
        // The upstream service sees only a metadata client. The original
        // consumer source is read and compiled exclusively by Lust.
        config.input = Input::Str { name: FileName::anon_source_code("projection"),
            input: "extern crate schema;".into() };
        config.override_queries = Some(|_, providers| {
            providers.queries.typeck_root = |_, _| panic!("projection queried local body typechecking");
            providers.queries.analysis = |_, _| panic!("projection queried local analysis");
        });
    }

    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        self.result = Some(Projection::capture(tcx, &self.snapshot, self.metadata.clone()));
        Compilation::Stop
    }
}

fn capture(metadata: Arc<[u8]>, snapshot: PathBuf) -> Result<Projection, String> {
    std::fs::write(&snapshot, &metadata).map_err(|e| e.to_string())?;
    let mut query = Query { metadata, snapshot, result: None };
    let arguments = vec!["rustc".into(), query.snapshot.to_str().unwrap().into(),
        "--edition=2024".into(), "--crate-name=projection_metadata_client".into(), "--crate-type=lib".into(),
        "--sysroot".into(), env!("LUSTR_IMPORTED_SYSROOT").into(),
        "--extern".into(), format!("schema={}", query.snapshot.display())];
    let result = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&arguments, &mut query));
    if result != std::process::ExitCode::SUCCESS {
        return Err("the upstream metadata reader failed".into());
    }
    query.result.ok_or_else(|| "the upstream projection query did not run".to_string())?
}

pub(super) fn run(args: &[OsString]) -> u8 {
    let [source, metadata, output, scratch, subsequent @ ..] = args else {
        eprintln!("usage: --projection SOURCE SCHEMA_RMETA OUTPUT NEW_SCRATCH [LATER_SCHEMA_RMETA ...]");
        return 2;
    };
    let source = PathBuf::from(source).canonicalize().unwrap();
    let scratch = PathBuf::from(scratch);
    std::fs::create_dir(&scratch).unwrap();
    let scratch = scratch.canonicalize().unwrap();
    let mut key = driver::SessionKey::synthetic(&source, false);
    key.profile.panic_abort = true;
    let mut session = driver::Session::new(scratch.join("native-state"), true, key.clone());
    session.set_output(driver::OutputKind::Standalone);
    let request = driver::Request {
        cmd: driver::Cmd::Build, session: key, file: source, out: output.into(),
        json: false, timings: false, debug: false, emit_c: None, max_errors: 0, run_args: None,
        harness: false, cc: false, library_options: Default::default(), client: Default::default(),
    };
    let mut current: Option<Projection> = None;
    let mut status = 3;
    for (i, metadata) in std::iter::once(metadata).chain(subsequent).enumerate() {
        let metadata: Arc<[u8]> = std::fs::read(metadata).unwrap().into();
        let started = std::time::Instant::now();
        let query_needed = current.as_ref().is_none_or(|projection| projection.metadata != metadata);
        if query_needed {
            let directory = scratch.join(format!("metadata-{i}"));
            std::fs::create_dir(&directory).unwrap();
            match capture(metadata, directory.join("libschema.rmeta")) {
                Ok(projection) => current = Some(projection),
                Err(error) => { eprintln!("projection import: {error}"); return 3; }
            }
        }
        let projection = current.as_ref().unwrap();
        eprintln!("step={i} projection {:?}: {:?}; upstream-queries={} elapsed-us={}",
            projection.item, projection.referent, u8::from(query_needed), started.elapsed().as_micros());
        let previous = std::fs::read(&request.out).ok();
        let started = std::time::Instant::now();
        let response = match driver::prepare_projection_import(&request, &mut session, projection) {
            Ok(driver::Preparation::Ready(prepared)) => {
                let program = prepared.program();
                let names: Vec<_> = prepared.checked_functions().iter().map(|&id| {
                    program.interner.get(program.fn_def(def::LocalDefId(id as u32)).name)
                }).collect();
                eprintln!("step={i} native checked bodies={names:?}");
                assert_eq!(prepared.imported_dependencies()[0].contents, projection.metadata);
                prepared.emit()
            }
            Ok(driver::Preparation::Complete(response) | driver::Preparation::Rejected(response)
                | driver::Preparation::Failed(response) | driver::Preparation::IntegrationIncomplete(response)) => response,
            Err(error) => { eprintln!("projection import: {}", error.0); return 3; }
        };
        eprint!("{}", response.stderr);
        status = response.code as u8;
        if status != 0 {
            assert_eq!(std::fs::read(&request.out).ok(), previous, "rejection changed the published executable");
        }
        eprintln!("step={i} native status={status} elapsed-us={} {}", started.elapsed().as_micros(), session.front.last_visits);
        if status > 1 {
            return status;
        }
    }
    status
}
