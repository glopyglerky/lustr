//! Bind the executable's actual Rust metadata and archive inputs before
//! mounting any dependency definitions or linking their native objects.
use std::{collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}, sync::Arc};

use lustr_metadata_encoder::{Sdk, archive_metadata_matches, archive_object_members};
use object::{Object, ObjectSection, read::archive::ArchiveFile};

use crate::{
    check::collect::Program,
    def,
    diag::Diag,
    driver::imports::{Definitions, Dependency, Emission, ScalarDefinitions},
    intern::Interner,
    library_artifact::unit_object,
    protocol::{ExecutableExtern, ExternMetadata},
    resolve::imports::ResolvedTables,
};

use super::{Stamp, imports::{Identity, Import}, stamp};

struct ExecutableCrate {
    input: ExecutableExtern,
    definitions: Import,
    archive: Option<(Stamp, Dependency)>,
}

pub(super) struct ExecutableImports {
    crates: Vec<ExecutableCrate>,
    direct: usize,
    searched: Vec<(PathBuf, Option<Stamp>)>,
    objects: Vec<Arc<[u8]>>,
}

impl ExecutableImports {
    pub(super) fn open(
        inputs: &[ExecutableExtern], search_paths: &[PathBuf], sdk: &Sdk, standard: Option<&Sdk>, panic_abort: bool,
    ) -> Result<Self, String> {
        let mut crates = Vec::new();
        let mut indices = BTreeMap::new();
        let mut searched = Vec::new();
        let linking = inputs.first().is_some_and(|input| input.archive.is_some());
        for input in inputs {
            let index = crate_index(crates.len())?;
            let definitions = Import::read(&input.metadata, sdk, standard, index, true)?;
            if indices.insert(definitions.identity.clone(), crates.len()).is_some() {
                return Err("duplicate Cargo extern crate identity".into());
            }
            crates.push(ExecutableCrate { input: input.clone(), definitions, archive: None });
        }
        let mut index = 0;
        while index < crates.len() {
            for dependency in crates[index].definitions.dependencies.clone() {
                if indices.contains_key(&dependency) { continue; }
                let resolved = find_dependency(&dependency, search_paths, sdk, standard, crate_index(crates.len())?,
                    linking, &mut searched)?;
                indices.insert(dependency, crates.len());
                crates.push(resolved);
            }
            index += 1;
        }
        require_acyclic(&crates, &indices)?;

        let mut objects = Vec::new();
        let mut defined = BTreeSet::new();
        let mut required = BTreeSet::new();
        for entry in &mut crates {
            let definitions = &entry.definitions;
            let input = &entry.input;
            entry.archive = if let Some(path) = &input.archive {
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
                for code in archive_objects(&bytes, definitions)? {
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
        }
        if !required.is_subset(&defined) {
            return Err("the supplied Cargo archives do not close the imported object references".into());
        }
        let resolved = Self { crates, direct: inputs.len(), searched, objects };
        if !resolved.unchanged(inputs) { return Err("Cargo dependencies changed while loading their metadata graph".into()); }
        Ok(resolved)
    }

    pub(super) fn unchanged(&self, inputs: &[ExecutableExtern]) -> bool {
        self.direct == inputs.len()
            && self.direct_crates().iter().zip(inputs).all(|(item, input)| &item.input == input)
            && self.crates.iter().all(|item| {
                let input = &item.input;
                item.definitions.unchanged(&input.metadata.artifact)
                    && match (&item.archive, &input.archive) {
                        (Some((before, _)), Some(path)) => stamp(path).is_ok_and(|observed| &observed == before),
                        (None, None) => true,
                        _ => false,
                    }
            })
            && self.searched.iter().all(|(path, old)| search_stamp(path).is_ok_and(|current| &current == old))
    }

    fn direct_crates(&self) -> &[ExecutableCrate] {
        &self.crates[..self.direct]
    }

    pub(super) fn unused_externs(&self, used: &BTreeSet<def::CrateId>) -> Vec<&str> {
        self.direct_crates().iter().map(|item| &item.definitions.facts)
            .filter(|facts| !used.contains(&facts.index)).map(|facts| facts.alias.as_str()).collect()
    }
}

fn crate_index(count: usize) -> Result<def::CrateId, String> {
    if count >= 4096 { return Err("too many Cargo crate identities".into()); }
    Ok(def::CrateId(count as u32 + 1))
}

fn search_stamp(path: &Path) -> Result<Option<Stamp>, String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("inspect Cargo metadata {}: {error}", path.display())),
        Ok(_) => stamp(path).map(Some),
    }
}

fn find_dependency(
    dependency: &Identity, search_paths: &[PathBuf], sdk: &Sdk, standard: Option<&Sdk>, index: def::CrateId,
    linking: bool, searched: &mut Vec<(PathBuf, Option<Stamp>)>,
) -> Result<ExecutableCrate, String> {
    // A crate's original filename suffix must remain within one search dir.
    if !dependency.suffix.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte)) {
        return Err("invalid Cargo dependency metadata filename".into());
    }
    let filename = format!("lib{}{}.rmeta", dependency.name, dependency.suffix);
    let mut found: Option<ExecutableCrate> = None;
    let mut directories = BTreeSet::new();
    for directory in search_paths {
        if !directories.insert(directory) { continue; }
        let path = directory.join(&filename);
        let before = search_stamp(&path)?;
        let present = before.is_some();
        searched.push((path.clone(), before));
        if !present { continue; }
        let metadata = ExternMetadata { alias: dependency.name.clone(), artifact: path.clone() };
        let Ok(definitions) = Import::read(&metadata, sdk, standard, index, true) else { continue; };
        if &definitions.identity != dependency { continue; }
        if found.is_some() {
            return Err(format!("multiple Cargo metadata files match {}", dependency.name));
        }
        let input = ExecutableExtern { metadata, archive: linking.then(|| path.with_extension("rlib")) };
        found = Some(ExecutableCrate { input, definitions, archive: None });
    }
    found.ok_or_else(|| format!("no matching Cargo metadata for transitive crate {}", dependency.name))
}

fn require_acyclic(crates: &[ExecutableCrate], indices: &BTreeMap<Identity, usize>) -> Result<(), String> {
    let mut entered = vec![false; crates.len()];
    let mut complete = vec![false; crates.len()];
    for root in 0..crates.len() {
        let mut pending = vec![(root, false)];
        while let Some((index, leaving)) = pending.pop() {
            if complete[index] { continue; }
            if leaving {
                complete[index] = true;
            } else {
                if entered[index] { return Err("Cargo metadata dependency graph contains a cycle".into()); }
                entered[index] = true;
                pending.push((index, true));
                for dependency in &crates[index].definitions.dependencies {
                    pending.push((indices[dependency], false));
                }
            }
        }
    }
    Ok(())
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
    if names.is_empty() == definitions.is_scalar() {
        return Err("Rust link metadata object inventory does not match the checked provider".into());
    }
    let mut objects = Vec::new();
    let mut seen = BTreeSet::new();
    for name in names {
        let member = members.next().ok_or("missing object named in Rust link metadata")?.map_err(|e| e.to_string())?;
        if name.as_bytes() != member.name() || !name.ends_with(".o") || !seen.insert(name) {
            return Err("actual Cargo archive objects differ from the Rust link metadata".into());
        }
        objects.push(member.data(bytes).map_err(|e| e.to_string())?);
    }
    if members.next().is_some() {
        return Err("Cargo archive has unrepresented object inventory".into());
    }
    Ok(objects)
}

impl Definitions for ExecutableImports {
    fn cargo_extern_matches(&self, alias: &str, name: &str) -> bool {
        self.direct_crates().iter().map(|item| &item.definitions.facts)
            .any(|facts| facts.alias == alias && facts.crate_name == name)
    }

    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag> {
        tables.executable_import_uses = Some(BTreeSet::new());
        for item in self.direct_crates() {
            item.definitions.mount(tables, interner)?;
        }
        Ok(())
    }

    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>> {
        for item in self.direct_crates() {
            item.definitions.install(program)?;
        }
        Ok(())
    }

    fn validate_checked(&self, program: &Program) -> Result<(), String> {
        for item in self.direct_crates() {
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

    fn scalar_definitions(&self) -> Option<Vec<Arc<ScalarDefinitions>>> {
        self.direct_crates().iter().all(|item| item.definitions.is_scalar())
            .then(|| self.direct_crates().iter().map(|item| item.definitions.facts.clone()).collect())
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
