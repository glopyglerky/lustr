//! Join the real Cargo metadata artifact to the original native resolver.

use std::{path::Path, sync::Arc};

use crate::{
    borrowck::RegionSig,
    check::{collect::Program, generics::{ImportedProjection, ProjectionReferent}},
    def,
    diag::{Diag, Span},
    driver,
    driver::graph::Namespace,
    driver::imports::{CargoDefinitions, ProjectionDefinition, ScalarFunction},
    intern::{Interner, Sym},
    protocol::ExternMetadata,
    resolve::imports::{Bind, DefInfo, ExternInfo, InsertOut, ResKind, ResolvedTables, Target, VisKind},
    ty::{self, TyIntern},
};
use lustr_metadata_encoder::{IntegerType, Sdk, read_executable_crate, read_link_crate, read_scalar_crate};

use super::{Stamp, stamp};

fn native_integer_type(ty: IntegerType) -> ty::IntTy {
    match ty {
        IntegerType::Isize => ty::IntTy::Isize,
        IntegerType::I8 => ty::IntTy::I8,
        IntegerType::I16 => ty::IntTy::I16,
        IntegerType::I32 => ty::IntTy::I32,
        IntegerType::I64 => ty::IntTy::I64,
        IntegerType::I128 => ty::IntTy::I128,
        IntegerType::Usize => ty::IntTy::Usize,
        IntegerType::U8 => ty::IntTy::U8,
        IntegerType::U16 => ty::IntTy::U16,
        IntegerType::U32 => ty::IntTy::U32,
        IntegerType::U64 => ty::IntTy::U64,
        IntegerType::U128 => ty::IntTy::U128,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Identity {
    pub name: String,
    pub svh: [u8; 16],
    pub suffix: String,
}

pub(super) struct Import {
    pub(super) facts: Arc<CargoDefinitions>,
    pub(super) stamp: Stamp,
    pub(super) identity: Identity,
    pub(super) requires_abort: bool,
    pub(super) code_free: bool,
    pub(super) dependencies: Vec<Identity>,
    artifact_identity: String,
    pub(super) metadata: Arc<[u8]>,
    pub(super) embedded_archive: Option<Arc<[u8]>>,
    pub(super) sdk: Sdk,
}

impl Import {
    pub(super) fn open(request: &ExternMetadata, sdk: &Sdk) -> Result<Self, String> {
        Self::read(request, sdk, None, def::CrateId(1), false)
    }

    pub(super) fn read(
        request: &ExternMetadata, sdk: &Sdk, standard: Option<&Sdk>, index: def::CrateId,
        executable: bool,
    ) -> Result<Self, String> {
        let path = &request.artifact;
        let before = stamp(path)?;
        let bytes: Arc<[u8]> = std::fs::read(path).map_err(|e| format!("read extern input {}: {e}", path.display()))?.into();
        let from_archive = executable && path.extension().is_some_and(|ext| ext == "rlib");
        let data: &[u8] = if from_archive { super::executable::embedded_metadata(&bytes)? } else { &bytes };
        let (leaf, projection) = if executable {
            read_executable_crate(data, sdk)
        } else {
            read_scalar_crate(data, sdk).map(|leaf| (leaf, None))
        }
        .or_else(|| standard.and_then(|context| read_link_crate(data, context)).map(|leaf| (leaf, None)))
        .ok_or("extern artifact has no represented interface in the pinned SDK")?;
        let extension = if from_archive { "rlib" } else { "rmeta" };
        let expected = format!("lib{}{}.{extension}", leaf.name, leaf.filename_suffix);
        if path.file_name().is_none_or(|name| name != expected.as_str()) || stamp(path)? != before {
            return Err("extern metadata filename or generation changed".into());
        }
        let context = if executable { sdk.clone() } else {
            sdk.with_scalar_leaf(&leaf).ok_or("extern metadata crate identity conflicts with the SDK")?
        };
        let functions = leaf
            .functions
            .iter()
            .map(|function| ScalarFunction {
                name: function.name.into(),
                hash: def::DefPathHash(function.path_hash),
                parameters: function.parameters.iter().copied().map(native_integer_type).collect(),
                result: native_integer_type(function.result),
                symbol: leaf.symbol(function.name),
            })
            .collect();
        let identity = Identity { name: leaf.name.into(), svh: leaf.svh, suffix: leaf.filename_suffix.into() };
        let dependencies = leaf.dependencies.iter().map(|dep| Identity {
            name: dep.name.into(), svh: dep.svh, suffix: dep.filename_suffix.into(),
        }).collect();
        let code_free = projection.as_ref().is_some_and(|item| item.code_free);
        let projection = projection.map(|item| {
            let definition = |(name, hash): (&str, u64)| (name.into(), def::DefPathHash(hash));
            ProjectionDefinition {
                receiver: definition(item.receiver),
                family: definition(item.family),
                item: definition(item.item),
                referent: item.referent.map(|referent| match referent {
                    lustr_metadata_encoder::ProjectionReferent::Integer(kind) => ProjectionReferent::Integer(native_integer_type(kind)),
                    lustr_metadata_encoder::ProjectionReferent::IntegerSlice(kind) => ProjectionReferent::IntegerSlice(native_integer_type(kind)),
                    lustr_metadata_encoder::ProjectionReferent::Str => ProjectionReferent::Str,
                }),
            }
        });
        let facts = Arc::new(CargoDefinitions {
            alias: request.alias.clone(),
            crate_name: identity.name.clone(),
            stable: def::StableCrateHash(leaf.stable_id),
            index,
            functions,
            projection,
        });
        Ok(Self {
                facts,
                stamp: before,
                identity,
                requires_abort: leaf.requires_abort,
                code_free,
                dependencies,
                artifact_identity: path.to_str().ok_or("extern metadata path is not Unicode")?.into(),
                metadata: if from_archive { Arc::from(data) } else { bytes.clone() },
                embedded_archive: from_archive.then_some(bytes),
                sdk: context,
        })
    }

    pub(super) fn unchanged(&self, path: &Path) -> bool {
        stamp(path).is_ok_and(|observed| observed == self.stamp)
    }

    pub(super) fn owns(&self, id: def::DiskDefId) -> bool {
        id.krate == self.facts.stable
    }

    pub(super) fn has_direct_integer_abi(&self) -> bool {
        use driver::library::completion::has_direct_integer_abi;
        self.facts.functions.iter().all(|function| {
            has_direct_integer_abi(function.result)
                && function.parameters.iter().copied().all(has_direct_integer_abi)
        })
    }

    pub(super) fn exported_symbols(&self) -> impl Iterator<Item = &str> {
        self.facts.functions.iter().map(|function| function.symbol.as_str())
    }

    fn flat_definition(&self, interner: &Interner, hash: def::DefPathHash) -> Sym {
        interner.intern(&format!("$lustr$extern${:016x}${:016x}", self.facts.stable.0, hash.0))
    }

    fn projection_fact(&self, interner: &Interner, projection: &ProjectionDefinition) -> (Sym, ImportedProjection) {
        let family = self.flat_definition(interner, projection.family.1);
        let fact = ImportedProjection {
            definition: def::DiskDefId { krate: self.facts.stable, hash: projection.item.1 },
            receiver: self.flat_definition(interner, projection.receiver.1),
            item: interner.intern(&projection.item.0),
            referent: projection.referent,
        };
        (family, fact)
    }

    fn failure(message: &str) -> Diag {
        Diag::new("L0508", Span::default(), message, "Use the actual matching Cargo metadata dependency.")
    }
}

impl driver::imports::Definitions for Import {
    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag> {
        if tables.externs.iter().any(|existing| existing.owner == ty::defs::ROOT_MODULE && existing.name == self.facts.alias)
            || tables.module_defs.iter().chain(tables.def_info.keys()).any(|id| id.krate == self.facts.index)
        {
            return Err(Self::failure("extern namespace is already occupied"));
        }
        let root = tables.add_crate_root(interner.intern(&self.facts.alias));
        let root_id = def::DefId { krate: self.facts.index, hash: def::DefPathHash(self.facts.stable.0) };
        tables.module_defs[root.0 as usize] = root_id;
        tables.externs.push(ExternInfo {
            name: self.facts.alias.clone(),
            def: root_id,
            is_std_lib: false,
            owner: ty::defs::ROOT_MODULE,
            root_module: Some(root),
            unsupported: None,
        });
        let mut bind = |name: &str, hash, kind, namespace, namespace_index: usize| {
            let id = def::DefId { krate: self.facts.index, hash };
            let flat = self.flat_definition(interner, hash);
            let info = DefInfo { kind, flat, module: root, vis: VisKind::Pub };
            if tables.def_info.insert(id, info).is_some()
                || tables.flat[namespace_index].insert(flat, id).is_some()
                || !matches!(
                    tables.insert(
                        root,
                        namespace,
                        interner.intern(name),
                        Bind { target: Target::Def(id), vis: VisKind::Pub, rank: 2, def_module: root }
                    ),
                    InsertOut::Changed
                )
            {
                return Err(Self::failure("extern definition identity conflicts with an existing binding"));
            }
            tables.display_names.insert(flat, interner.intern(&format!("{}::{name}", self.facts.alias)));
            Ok(())
        };
        for function in &self.facts.functions {
            bind(&function.name, function.hash, ResKind::Fn, Namespace::Value, 1)?;
        }
        if let Some(projection) = &self.facts.projection {
            bind(&projection.receiver.0, projection.receiver.1, ResKind::Adt, Namespace::Type, 0)?;
            bind(&projection.family.0, projection.family.1, ResKind::Trait, Namespace::Type, 0)?;
        }
        Ok(())
    }

    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>> {
        if program.krate == self.facts.index || program.imported_crates.values().any(|stable| *stable == self.facts.stable)
            || program.imported_crates.insert(self.facts.index, self.facts.stable).is_some() {
            return Err(vec![Self::failure("extern crate identity is already occupied")]);
        }
        for function in &self.facts.functions {
            let id = def::DefId { krate: self.facts.index, hash: function.hash };
            let info = program
                .resolved
                .as_ref()
                .and_then(|tables| tables.def_info.get(&id))
                .ok_or_else(|| vec![Self::failure("extern signature has no original binding")])?;
            let mut data = ty::defs::DefData::skeleton(ty::defs::DefKind::ForeignFn, None, info.flat, ty::defs::Vis::Pub, Span::default());
            data.module = info.module;
            let params: Vec<_> = function.parameters.iter().map(|&kind| program.tys.intern(ty::TyKind::Int(kind))).collect();
            let ret = program.tys.intern(ty::TyKind::Int(function.result));
            data.sig = ty::SigData::Fn { params: params.clone(), ret };
            data.native_linkage = Some(Box::new(ty::defs::NativeLinkage {
                id: def::DiskDefId { krate: self.facts.stable, hash: function.hash },
                symbol: function.symbol.clone(),
            }));
            program.defs.insert(id, data);
            program.regions.native_fns.insert(
                id,
                RegionSig {
                    n_univ: 0,
                    params,
                    ret,
                    facts: Vec::new(),
                    names: Vec::new(),
                    early: Vec::new(),
                    fn_bounds: Vec::new(),
                    trait_bounds: Vec::new(),
                    item_bounds: Vec::new(),
                    opaque_caps: None,
                    header: Vec::new(),
                    bounds: Vec::new(),
                },
            );
        }
        if let Some(projection) = &self.facts.projection {
            let (family, fact) = self.projection_fact(&program.interner, projection);
            program.gx.imported_projections.get_or_insert_with(Default::default).entry(family).or_default().push(fact);
        }
        self.validate_checked(program).map_err(|reason| vec![Self::failure(&reason)])
    }

    fn validate_checked(&self, program: &Program) -> Result<(), String> {
        if program.imported_crates.get(&self.facts.index) != Some(&self.facts.stable) {
            return Err("checked extern crate identity changed".into());
        }
        for function in &self.facts.functions {
            let id = def::DefId { krate: self.facts.index, hash: function.hash };
            let (params, ret, linkage) = program.native_fn(id).ok_or("checked extern signature is absent")?;
            if params.len() != function.parameters.len()
                || params.iter().zip(&function.parameters).any(|(&id, &kind)| program.tys.kind(id) != ty::TyKind::Int(kind))
                || program.tys.kind(ret) != ty::TyKind::Int(function.result)
                || linkage.id.krate != self.facts.stable
                || linkage.id.hash != function.hash
                || linkage.symbol != function.symbol
            {
                return Err("checked extern signature or linkage changed".into());
            }
        }
        if let Some(projection) = &self.facts.projection {
            let (family, expected) = self.projection_fact(&program.interner, projection);
            if !program.gx.imported_projections.as_ref().and_then(|map| map.get(&family)).is_some_and(|facts| {
                facts.iter().any(|fact| fact.definition == expected.definition
                    && fact.receiver == expected.receiver && fact.item == expected.item
                    && fact.referent == expected.referent)
            }) {
                return Err("checked projection differs from the imported definition".into());
            }
        }
        Ok(())
    }

    fn cargo_crate_name(&self) -> Option<&str> {
        Some(&self.facts.crate_name)
    }
    fn dependencies(&self) -> Vec<driver::imports::Dependency> {
        vec![driver::imports::Dependency {
            identity: self.artifact_identity.clone(),
            contents: self.embedded_archive.as_ref().unwrap_or(&self.metadata).clone(),
        }]
    }
    fn take_emission(&self) -> Result<Box<dyn driver::imports::Emission + 'static>, String> {
        Err("Cargo links the imported library artifacts at their final consumer".into())
    }
}
