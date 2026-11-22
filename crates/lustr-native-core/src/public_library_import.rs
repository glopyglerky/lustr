//! Join the real Cargo metadata artifact to the original native resolver.

use std::{path::Path, sync::Arc};

use crate::{
    borrowck::RegionSig,
    check::collect::Program,
    def,
    diag::{Diag, Span},
    driver,
    intern::Interner,
    protocol::ExternMetadata,
    resolve::imports::{Bind, DefInfo, ExternInfo, InsertOut, ResKind, ResolvedTables, Target, VisKind},
    ty::{self, TyIntern},
};
use lustr_metadata_encoder::{Sdk, read_scalar_crate};

use super::{Stamp, stamp};

struct Function {
    name: String,
    hash: def::DefPathHash,
    arity: usize,
    symbol: String,
}

// This route mounts one actual non-SDK dependency. Resolver index zero is the
// source crate. No dependency source jobs or synthetic declarations run.
const DEPENDENCY: def::CrateId = def::CrateId(1);

pub(super) struct Import {
    pub(super) alias: String,
    pub(super) stamp: Stamp,
    crate_name: String,
    stable: def::StableCrateHash,
    functions: Vec<Function>,
    artifact_identity: String,
    metadata: Arc<[u8]>,
    pub(super) sdk: Sdk,
}

impl Import {
    pub(super) fn open(request: &ExternMetadata, sdk: &Sdk) -> Result<Self, String> {
        let path = &request.artifact;
        let before = stamp(path)?;
        let data = std::fs::read(path).map_err(|e| format!("read extern metadata {}: {e}", path.display()))?;
        let leaf = read_scalar_crate(&data, sdk).ok_or("extern artifact does not represent a pinned scalar library")?;
        let expected = format!("lib{}{}.rmeta", leaf.name, leaf.filename_suffix);
        if path.file_name().is_none_or(|name| name != expected.as_str()) || stamp(path)? != before {
            return Err("extern metadata filename or generation changed".into());
        }
        let context = sdk.with_scalar_leaf(&leaf).ok_or("extern metadata crate identity conflicts with the SDK")?;
        let functions = leaf
            .functions
            .iter()
            .map(|function| Function {
                name: function.name.into(),
                hash: def::DefPathHash(function.path_hash),
                arity: function.arity,
                symbol: leaf.symbol(function.name),
            })
            .collect();
        let crate_name = leaf.name.into();
        let stable = def::StableCrateHash(leaf.stable_id);
        Ok(Self {
                alias: request.alias.clone(),
                stamp: before,
                crate_name,
                stable,
                functions,
                artifact_identity: path.to_str().ok_or("extern metadata path is not Unicode")?.into(),
                metadata: data.into(),
                sdk: context,
        })
    }

    pub(super) fn unchanged(&self, path: &Path) -> bool {
        stamp(path).is_ok_and(|observed| observed == self.stamp)
    }

    pub(super) fn owns(&self, id: def::DiskDefId) -> bool {
        id.krate == self.stable
    }

    fn failure(message: &str) -> Diag {
        Diag::new("L0508", Span::default(), message, "Use the actual matching Cargo metadata dependency.")
    }
}

impl driver::imports::Definitions for Import {
    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag> {
        if tables.externs.iter().any(|existing| existing.owner == ty::defs::ROOT_MODULE && existing.name == self.alias)
            || tables.module_defs.iter().chain(tables.def_info.keys()).any(|id| id.krate == DEPENDENCY)
        {
            return Err(Self::failure("extern namespace is already occupied"));
        }
        let root = tables.add_crate_root(interner.intern(&self.alias));
        let root_id = def::DefId { krate: DEPENDENCY, hash: def::DefPathHash(self.stable.0) };
        tables.module_defs[root.0 as usize] = root_id;
        tables.externs.push(ExternInfo {
            name: self.alias.clone(),
            def: root_id,
            is_std_lib: false,
            owner: ty::defs::ROOT_MODULE,
            root_module: Some(root),
            unsupported: None,
        });
        for function in &self.functions {
            let id = def::DefId { krate: DEPENDENCY, hash: function.hash };
            let flat = interner.intern(&format!("$lustr$extern${:016x}${:016x}", self.stable.0, function.hash.0));
            let info = DefInfo { kind: ResKind::Fn, flat, module: root, vis: VisKind::Pub };
            if tables.def_info.insert(id, info).is_some()
                || tables.flat[1].insert(flat, id).is_some()
                || !matches!(
                    tables.insert(
                        root,
                        driver::graph::Namespace::Value,
                        interner.intern(&function.name),
                        Bind { target: Target::Def(id), vis: VisKind::Pub, rank: 2, def_module: root }
                    ),
                    InsertOut::Changed
                )
            {
                return Err(Self::failure("extern function identity conflicts with an existing binding"));
            }
            tables.display_names.insert(flat, interner.intern(&format!("{}::{}", self.alias, function.name)));
        }
        Ok(())
    }

    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>> {
        if program.krate == DEPENDENCY || program.imported_crates.insert(DEPENDENCY, self.stable).is_some() {
            return Err(vec![Self::failure("extern crate identity is already occupied")]);
        }
        let word = program.tys.intern(ty::TyKind::Int(ty::IntTy::U64));
        for function in &self.functions {
            let id = def::DefId { krate: DEPENDENCY, hash: function.hash };
            let info = program
                .resolved
                .as_ref()
                .and_then(|tables| tables.def_info.get(&id))
                .ok_or_else(|| vec![Self::failure("extern signature has no original binding")])?;
            let mut data = ty::defs::DefData::skeleton(ty::defs::DefKind::ForeignFn, None, info.flat, ty::defs::Vis::Pub, Span::default());
            data.module = info.module;
            let params = vec![word; function.arity];
            data.sig = ty::SigData::Fn { params: params.clone(), ret: word };
            data.native_linkage = Some(Box::new(ty::defs::NativeLinkage {
                id: def::DiskDefId { krate: self.stable, hash: function.hash },
                symbol: function.symbol.clone(),
            }));
            program.defs.insert(id, data);
            program.regions.native_fns.insert(
                id,
                RegionSig {
                    n_univ: 0,
                    params,
                    ret: word,
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
        self.validate_checked(program).map_err(|reason| vec![Self::failure(&reason)])
    }

    fn validate_checked(&self, program: &Program) -> Result<(), String> {
        if program.imported_crates.get(&DEPENDENCY) != Some(&self.stable) {
            return Err("checked extern crate identity changed".into());
        }
        for function in &self.functions {
            let id = def::DefId { krate: DEPENDENCY, hash: function.hash };
            let (params, ret, linkage) = program.native_fn(id).ok_or("checked extern signature is absent")?;
            if params.len() != function.arity
                || params.iter().chain([&ret]).any(|&id| program.tys.kind(id) != ty::TyKind::Int(ty::IntTy::U64))
                || linkage.id.krate != self.stable
                || linkage.id.hash != function.hash
                || linkage.symbol != function.symbol
            {
                return Err("checked extern signature or linkage changed".into());
            }
        }
        Ok(())
    }

    fn cargo_crate_name(&self) -> Option<&str> {
        Some(&self.crate_name)
    }
    fn dependencies(&self) -> Vec<driver::imports::Dependency> {
        vec![driver::imports::Dependency { identity: self.artifact_identity.clone(), contents: self.metadata.clone() }]
    }
    fn take_emission(&self) -> Result<Box<dyn driver::imports::Emission + 'static>, String> {
        Err("check-only extern metadata cannot supply linked objects".into())
    }
}
