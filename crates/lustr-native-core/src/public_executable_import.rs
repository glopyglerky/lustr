//! Bind the executable's actual Rust metadata and archive inputs before
//! mounting any dependency definitions or linking their native objects.
use std::{collections::BTreeSet, path::Path, sync::Arc};

use lustr_metadata_encoder::{Sdk, archive_metadata_matches, archive_object_members};
use object::{Object, ObjectSection, read::archive::ArchiveFile};

use crate::{
    check::collect::Program,
    def,
    diag::Diag,
    driver::imports::{Definitions, Dependency, Emission},
    intern::Interner,
    library_artifact::unit_object,
    protocol::ExecutableExtern,
    resolve::imports::ResolvedTables,
};

use super::{Stamp, imports::Import, stamp};

struct ExecutableCrate {
    definitions: Import,
    archive: Option<(Stamp, Dependency)>,
}

pub(super) struct ExecutableImports {
    crates: Vec<ExecutableCrate>,
    objects: Vec<Arc<[u8]>>,
}

impl ExecutableImports {
    pub(super) fn open(inputs: &[ExecutableExtern], sdk: &Sdk, standard: Option<&Sdk>, panic_abort: bool) -> Result<Self, String> {
        let mut crates = Vec::new();
        let mut objects = Vec::new();
        let mut defined = BTreeSet::new();
        let mut required = BTreeSet::new();
        for (index, input) in inputs.iter().enumerate() {
            let index = def::CrateId(u32::try_from(index + 1).map_err(|_| "too many Cargo crate identities")?);
            let definitions = Import::read(&input.metadata, sdk, standard, index, true)?;
            let archive = if let Some(path) = &input.archive {
                if !definitions.has_direct_integer_abi() {
                    return Err("unrepresented linked integer ABI in Cargo extern metadata".into());
                }
                if definitions.requires_abort && !panic_abort {
                    return Err("Cargo archive requires panic=abort but its executable uses panic=unwind".into());
                }
                let before = stamp(path)?;
                let bytes: Arc<[u8]> =
                    std::fs::read(path).map_err(|e| format!("read Cargo archive {}: {e}", path.display()))?.into();
                if before != stamp(path)? || !definitions.unchanged(&input.metadata.artifact) {
                    return Err("Cargo metadata/archive pair changed while loading".into());
                }
                let mut available = BTreeSet::new();
                for code in archive_objects(&bytes, &definitions)? {
                    let observed = unit_object::symbols(code)?;
                    for symbol in observed.defined {
                        if !defined.insert(symbol) {
                            return Err("Cargo archive definitions collide".into());
                        }
                    }
                    available.extend(observed.functions);
                    required.extend(observed.undefined);
                    objects.push(Arc::from(code));
                }
                if !definitions.exported_symbols().all(|symbol| available.contains(symbol.as_bytes())) {
                    return Err("Cargo archive lacks the actual public metadata symbol".into());
                }
                let identity = path.to_str().ok_or("Cargo archive path is not Unicode")?.into();
                Some((before, Dependency { identity, contents: bytes }))
            } else {
                None
            };
            crates.push(ExecutableCrate { definitions, archive });
        }
        let identities: BTreeSet<_> = crates.iter().map(|item| &item.definitions.identity).collect();
        if identities.len() != crates.len()
            || crates.iter().any(|item| {
                item.definitions
                    .dependencies
                    .iter()
                    .any(|dependency| dependency == &item.definitions.identity || !identities.contains(dependency))
            })
        {
            return Err("the supplied Cargo artifacts do not close the metadata dependency graph".into());
        }
        if !required.is_subset(&defined) {
            return Err("the supplied Cargo archives do not close the imported object references".into());
        }
        Ok(Self { crates, objects })
    }

    pub(super) fn unchanged(&self, inputs: &[ExecutableExtern]) -> bool {
        self.crates.len() == inputs.len()
            && self.crates.iter().zip(inputs).all(|(item, input)| {
                item.definitions.unchanged(&input.metadata.artifact)
                    && match (&item.archive, &input.archive) {
                        (Some((before, _)), Some(path)) => stamp(path).is_ok_and(|observed| &observed == before),
                        (None, None) => true,
                        _ => false,
                    }
            })
    }

    pub(super) fn unused_externs(&self, used: &BTreeSet<def::CrateId>) -> Vec<&str> {
        self.crates.iter().filter(|item| !used.contains(&item.definitions.index)).map(|item| item.definitions.alias.as_str()).collect()
    }
}

fn archive_objects<'a>(bytes: &'a [u8], definitions: &Import) -> Result<Vec<&'a [u8]>, String> {
    let archive = ArchiveFile::parse(bytes).map_err(|e| e.to_string())?;
    let mut members = archive.members();
    let mut section = |name: &[u8], section: &str| -> Result<&'a [u8], String> {
        let member = members.next().ok_or("missing Rust archive metadata member")?.map_err(|e| e.to_string())?;
        if member.name() != name {
            return Err("unexpected Rust archive metadata member".into());
        }
        let wrapper = object::File::parse(member.data(bytes).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        wrapper.section_by_name(section).ok_or("original Rust archive metadata section absent")?.data().map_err(|e| e.to_string())
    };
    let metadata = section(b"lib.rmeta", ".rmeta")?;
    if !archive_metadata_matches(&definitions.metadata, metadata, &definitions.identity.name) {
        return Err("Cargo archive and full metadata disagree".into());
    }
    let links = section(b"lib.rmeta-link", ".rmeta-link")?;
    let names = archive_object_members(links).ok_or("original Rust link metadata inventory is not represented")?;
    let mut objects = Vec::new();
    let mut seen = BTreeSet::new();
    for name in names {
        let member = members.next().ok_or("missing object named in Rust link metadata")?.map_err(|e| e.to_string())?;
        if name.as_bytes() != member.name() || !name.ends_with(".o") || !seen.insert(name) {
            return Err("actual Cargo archive objects differ from the Rust link metadata".into());
        }
        objects.push(member.data(bytes).map_err(|e| e.to_string())?);
    }
    if objects.is_empty() || members.next().is_some() {
        return Err("Cargo archive has empty or unrepresented object inventory".into());
    }
    Ok(objects)
}

impl Definitions for ExecutableImports {
    fn cargo_extern_matches(&self, alias: &str, name: &str) -> bool {
        self.crates.iter().any(|item| item.definitions.alias == alias && item.definitions.identity.name == name)
    }

    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag> {
        tables.executable_import_uses = Some(BTreeSet::new());
        for item in &self.crates {
            item.definitions.mount(tables, interner)?;
        }
        Ok(())
    }

    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>> {
        for item in &self.crates {
            item.definitions.install(program)?;
        }
        Ok(())
    }

    fn validate_checked(&self, program: &Program) -> Result<(), String> {
        for item in &self.crates {
            item.definitions.validate_checked(program)?;
        }
        Ok(())
    }

    fn dependencies(&self) -> Vec<Dependency> {
        self.crates
            .iter()
            .flat_map(|item| {
                let mut deps = item.definitions.dependencies();
                if let Some((_, archive)) = &item.archive {
                    deps.push(Dependency { identity: archive.identity.clone(), contents: archive.contents.clone() });
                }
                deps
            })
            .collect()
    }

    fn take_emission(&self) -> Result<Box<dyn Emission + 'static>, String> {
        Ok(Box::new(Objects(self.objects.clone())))
    }
}

struct Objects(Vec<Arc<[u8]>>);
impl Emission for Objects {
    fn emit(self: Box<Self>) -> Result<Vec<Arc<[u8]>>, String> {
        Ok(self.0)
    }
}
