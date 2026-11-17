//! Object integration with the pinned upstream LLVM backend. Rust supplies all
//! semantic decisions, metadata, ordinary objects and final linking. Only a
//! successfully compiled native definition is omitted from LLVM generation.
use rustc_codegen_ssa::{CompiledModule, CompiledModules, CrateInfo, ModuleKind};
use rustc_metadata::EncodedMetadata;
use rustc_session::{Session, config::OutputFilenames};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

#[derive(Default)]
struct State {
    entries: BTreeMap<String, PathBuf>,
    directory: Option<PathBuf>,
}
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

pub fn install(directory: PathBuf, entries: BTreeMap<String, PathBuf>) {
    let mut state = STATE.get_or_init(Default::default).lock().unwrap();
    assert!(state.entries.is_empty(), "one Rust compilation per mixed driver");
    state.entries = entries;
    state.directory = Some(directory);
}

pub fn contains(symbol: &str) -> bool {
    STATE.get().is_some_and(|state| state.lock().unwrap().entries.contains_key(symbol))
}

pub(crate) fn append_objects(modules: &mut CompiledModules) {
    let Some(state) = STATE.get() else { return };
    for (n, object) in state.lock().unwrap().entries.values().enumerate() {
        // Rust's archive writer uses each object's basename. Distinct names
        // preserve every native member in a mixed rlib and its real metadata
        // object inventory, even when functions live in separate directories.
        let link_object = object.with_file_name(format!("lustr-native-{n}.link.o"));
        fs::copy(object, &link_object).expect("copy current native object for the Rust linker");
        modules.modules.push(CompiledModule {
            name: format!("lustr-native-{n}"),
            kind: ModuleKind::Regular,
            object: Some(link_object),
            global_asm_object: None,
            dwarf_object: None,
            bytecode: None,
            assembly: None,
            llvm_ir: None,
        });
    }
}

fn copy_file(path: &mut Option<PathBuf>, directory: &Path, n: &mut usize) -> io::Result<()> {
    let Some(source) = path else { return Ok(()) };
    let target = directory.join(format!("object-{}-{}", *n, source.file_name().unwrap().to_string_lossy()));
    *n += 1;
    fs::copy(&*source, &target)?;
    *source = target;
    Ok(())
}

fn retain_file(path: &mut Option<PathBuf>, directory: &Path, n: &mut usize) -> io::Result<()> {
    copy_file(path, directory, n)?;
    if let Some(target) = path {
        let _profile = super::mixed_profile::span("retained-object-sync");
        fs::File::open(target)?.sync_all()?;
    }
    Ok(())
}

pub(crate) fn retain_link(
    sess: &Session,
    modules: &mut CompiledModules,
    info: &CrateInfo,
    metadata: &EncodedMetadata,
    outputs: &OutputFilenames,
) -> io::Result<()> {
    let _profile = super::mixed_profile::span("retain-link-recipe");
    // Library metadata must be encoded anew on every source build. Only
    // executable requests can retain and reuse this serialized link recipe.
    if info.crate_types != [rustc_structures::CrateType::Executable] {
        return Ok(());
    }
    let Some(state) = STATE.get() else { return Ok(()) };
    let state = state.lock().unwrap();
    if state.entries.is_empty() {
        return Ok(());
    }
    let directory = state.directory.as_ref().unwrap();
    let retained = directory.join("rust-objects");
    fs::create_dir(&retained)?;
    let clone_module = |m: &CompiledModule| CompiledModule {
        name: m.name.clone(),
        kind: m.kind,
        object: m.object.clone(),
        global_asm_object: m.global_asm_object.clone(),
        dwarf_object: m.dwarf_object.clone(),
        bytecode: m.bytecode.clone(),
        assembly: m.assembly.clone(),
        llvm_ir: m.llvm_ir.clone(),
    };
    let mut cached = CompiledModules {
        modules: modules.modules.iter().map(clone_module).collect(),
        allocator_module: modules.allocator_module.as_ref().map(clone_module),
    };
    let mut n = 0;
    let mut native = state.entries.values();
    for module in cached.modules.iter_mut().chain(cached.allocator_module.iter_mut()) {
        // The original native objects stay immutable. Later successful edits
        // select fresh object paths through the retained certificate; the Rust
        // recipe keeps these original identities for versioned substitution.
        if module.name.starts_with("lustr-native-") {
            module.object = Some(native.next().unwrap().clone());
            continue;
        }
        retain_file(&mut module.object, &retained, &mut n)?;
        retain_file(&mut module.global_asm_object, &retained, &mut n)?;
        retain_file(&mut module.dwarf_object, &retained, &mut n)?;
        retain_file(&mut module.bytecode, &retained, &mut n)?;
    }
    let rlink = directory.join("checked.rlink");
    CompiledModules::serialize_rlink(sess, &rlink, &cached, info, metadata, outputs)?;
    fs::File::open(&rlink)?.sync_all()?;
    fs::File::open(directory)?.sync_all()?;
    Ok(())
}

/// Decode Rust's real, version-bound link recipe and relink through its original
/// backend. Temporary copies keep the checked objects intact when Rust removes
/// its link inputs after success.
pub(crate) fn relink(sess: &Session, directory: &Path, native: &BTreeMap<PathBuf, PathBuf>) -> Result<(), String> {
    let _profile = super::mixed_profile::span("relink");
    let bytes = fs::read(directory.join("checked.rlink")).map_err(|e| e.to_string())?;
    let (mut modules, info, metadata, outputs) =
        CompiledModules::deserialize_rlink(sess, bytes).map_err(|_| "Rust rejected the retained link recipe".to_owned())?;
    let temporary = super::mixed_cache::fresh(directory, "link", "tmp").ok_or("link generation")?;
    fs::create_dir(&temporary).map_err(|e| e.to_string())?;
    let mut n = 0;
    let copying = super::mixed_profile::span("temporary-link-copies");
    for module in modules.modules.iter_mut().chain(modules.allocator_module.iter_mut()) {
        for path in [&mut module.object, &mut module.global_asm_object, &mut module.dwarf_object, &mut module.bytecode] {
            let Some(source) = path.as_ref() else { continue };
            let source = source.canonicalize().map_err(|e| e.to_string())?;
            let selected = native.get(&source).unwrap_or(&source);
            if !source.starts_with(directory) || !selected.starts_with(directory) {
                return Err("cached object escaped its generation".into());
            }
            *path = Some(selected.clone());
            // These copies belong only to this live linker invocation. No
            // persisted certificate or link recipe points to them. The checked
            // originals above remain immutable and durably retained; syncing
            // disposable inputs adds no recovery guarantee.
            copy_file(path, &temporary, &mut n).map_err(|e| e.to_string())?;
        }
    }
    drop(copying);
    let backend = rustc_codegen_llvm::LlvmCodegenBackend::new();
    let _link = super::mixed_profile::span("original-linker");
    backend.link(sess, modules, info, metadata, &outputs);
    Ok(())
}
