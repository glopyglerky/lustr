//! Cargo-planned host scripts compiled by the ordinary native pipeline.
//!
//! The planner supplies the original host unit, not a probe result. Only this
//! module owns its compile/run attempt. Daemon routing never executes scripts.
//! Old Cargo/rustc replay records are deliberately not accepted here.
use super::*;
use crate::driver::{Cmd, Preparation, Request, Session, SessionKey};
use crate::resolve::cargo::BuildScriptUnit;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

const MAGIC: &str = "LBS-native-1\n";

fn plan_key(plan: &BuildScriptUnit) -> u64 {
    let mut h = FxHasher::default();
    MAGIC.hash(&mut h);
    plan.hash(&mut h);
    // Reuse the native code generator's settings identity. Arbitrary LUST_*
    // variables may be script runtime inputs (the upstream histories use
    // LUST_SCEN_*); their changes must not recompile an unchanged host unit.
    // This request uses TinyCC, cc=false, debug=false and no emitted-C mode.
    static COMPILER: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    COMPILER
        .get_or_init(|| {
            let mut h = FxHasher::default();
            crate::driver::cache::compiler_id().hash(&mut h);
            crate::driver::release_c::codegen_key(crate::driver::release_c::on(false, false), false, false).hash(&mut h);
            for name in ["LUST_PGO_COPIES", "LUST_PGO_COPY_MIN"] {
                (name, std::env::var_os(name)).hash(&mut h);
            }
            h.finish()
        })
        .hash(&mut h);
    if crate::driver::release_c::on(false, false) {
        if let Some(path) = std::env::var_os("LUST_PGO_COPIES").filter(|p| p != "*") {
            file_hash(Path::new(&path)).hash(&mut h);
        }
    }
    h.finish()
}

fn env_value(value: Option<&str>) -> String {
    value.map_or_else(|| "N".to_string(), |v| format!("S{v}"))
}

fn compile_env(plan: &BuildScriptUnit, client: ClientEnv) -> Result<Vec<(String, String)>, String> {
    let mut env: BTreeMap<_, _> = client.map_or_else(|| std::env::vars().collect(), |e| e.iter().cloned().collect());
    env.extend(plan.cargo_env.iter().cloned());
    let executable = std::env::current_exe().map_err(|e| format!("cannot locate native toolchain: {e}"))?;
    env.insert("RUSTC".into(), executable.with_file_name("rustc").to_string_lossy().into_owned());
    env.insert("CARGO".into(), executable.with_file_name("cargo").to_string_lossy().into_owned());
    Ok(env.into_iter().collect())
}

fn job_count(client: ClientEnv) -> Result<usize, String> {
    match client_var(client, "CARGO_BUILD_JOBS").filter(|value| !value.is_empty()) {
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|count| *count > 0)
            .ok_or_else(|| "native build-script integration currently represents only positive numeric Cargo job counts".into()),
        None => Ok(std::thread::available_parallelism().map_or(1, usize::from)),
    }
}

fn runtime_env(plan: &BuildScriptUnit, client: ClientEnv, out_dir: &Path) -> Result<Vec<(String, String)>, String> {
    let jobs = job_count(client)?;
    if client_var(client, "CARGO_MAKEFLAGS").is_some_and(|v| !v.is_empty()) {
        return Err("native build-script integration does not yet transfer a client jobserver".into());
    }
    let mut env: BTreeMap<_, _> = client.map_or_else(|| std::env::vars().collect(), |e| e.iter().cloned().collect());
    // Cargo adds crate/bin names only to the compiler command. At script run
    // time a caller's versions (and inactive cfg/feature entries) stay inherited.
    env.extend(plan.cargo_env.iter().filter(|(name, _)| !matches!(name.as_str(), "CARGO_CRATE_NAME" | "CARGO_BIN_NAME")).cloned());
    let target = if plan.target_triple.is_empty() { crate::protocol::host_triple() } else { &plan.target_triple };
    let cfg = crate::resolve::cfg::CfgEnv::new(target, &[], false, plan.target_profile);
    for flag in cfg.flags {
        env.insert(format!("CARGO_CFG_{}", flag.to_uppercase()), String::new());
    }
    let mut pairs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, value) in cfg.pairs {
        pairs.entry(key).or_default().push(value);
    }
    for (key, mut values) in pairs {
        values.sort();
        values.dedup();
        env.insert(format!("CARGO_CFG_{}", key.to_uppercase()), values.join(","));
    }
    for feature in &plan.target_features {
        env.insert(format!("CARGO_FEATURE_{}", feature.replace('-', "_").to_uppercase()), "1".into());
    }
    let mut features = plan.target_features.clone();
    features.sort();
    features.dedup();
    env.insert("CARGO_CFG_FEATURE".into(), features.join(","));
    let executable = std::env::current_exe().map_err(|e| format!("cannot locate native toolchain: {e}"))?;
    let compiler = executable.with_file_name("rustc");
    let cargo = executable.with_file_name("cargo");
    if !compiler.is_file() || !cargo.is_file() {
        return Err("native build-script integration needs the installed Lustr compiler and Cargo entries".into());
    }
    env.extend([
        ("HOST".into(), crate::protocol::host_triple().into()),
        ("TARGET".into(), target.into()),
        ("OUT_DIR".into(), out_dir.to_string_lossy().into_owned()),
        ("OPT_LEVEL".into(), plan.target_opt_level.clone()),
        ("PROFILE".into(), "debug".into()),
        ("DEBUG".into(), plan.target_debug.to_string()),
        ("NUM_JOBS".into(), jobs.to_string()),
        ("RUSTC".into(), compiler.to_string_lossy().into_owned()),
        ("CARGO".into(), cargo.to_string_lossy().into_owned()),
        ("CARGO_ENCODED_RUSTFLAGS".into(), String::new()),
    ]);
    Ok(env.into_iter().collect())
}

struct CompileProof {
    command: Compile,
    sources: Vec<(PathBuf, u64)>,
    absent: Vec<PathBuf>,
}

struct NativeRecord {
    record: Record,
    runtime_reusable: bool,
    absent: Vec<PathBuf>,
    source_times: Vec<(PathBuf, [u8; 13])>,
}

fn read_record(path: &Path) -> Option<NativeRecord> {
    let text = std::fs::read_to_string(path).ok()?;
    let body = text.strip_prefix(MAGIC)?;
    let (header, body) = body.split_once("\nrecord\n")?;
    let mut header = header.lines();
    let reusable = header.next()?;
    let runtime_reusable = match reusable {
        "runtime-reusable=1" => true,
        "runtime-reusable=0" => false,
        _ => return None,
    };
    let mut absent = Vec::new();
    let mut rerun_paths = Vec::new();
    let mut rerun_env = Vec::new();
    let mut source_times = Vec::new();
    for line in header {
        let (kind, value) = line.split_once(' ')?;
        let value = unesc(value);
        match kind {
            "absent" => absent.push(PathBuf::from(value)),
            "rerun-path" => rerun_paths.push(value),
            "rerun-env" => rerun_env.push(value),
            "source-time" => {
                let (stamp, path) = value.split_once(' ')?;
                if stamp.len() != 26 || !stamp.is_ascii() {
                    return None;
                }
                let mut bytes = [0; 13];
                for (index, byte) in bytes.iter_mut().enumerate() {
                    *byte = u8::from_str_radix(&stamp[index * 2..index * 2 + 2], 16).ok()?;
                }
                source_times.push((PathBuf::from(path), bytes));
            }
            _ => return None,
        }
    }
    let mut record = Record::decode(&format!("LBS5\n{body}"))?;
    record.output.rerun_if_changed = rerun_paths;
    record.output.rerun_if_env_changed = rerun_env;
    (record.rustc.is_none() && record.compile.is_some()).then_some(NativeRecord { record, runtime_reusable, absent, source_times })
}

fn write_record(
    path: &Path,
    record: &Record,
    runtime_reusable: bool,
    absent: &[PathBuf],
    source_times: &[(PathBuf, [u8; 13])],
) -> Result<(), String> {
    let encoded = record.encode();
    let mut bytes = MAGIC.as_bytes().to_vec();
    bytes.extend_from_slice(if runtime_reusable { b"runtime-reusable=1\n" } else { b"runtime-reusable=0\n" });
    for path in absent {
        bytes.extend_from_slice(format!("absent {}\n", esc(&path.to_string_lossy())).as_bytes());
    }
    for path in &record.output.rerun_if_changed {
        bytes.extend_from_slice(format!("rerun-path {}\n", esc(path)).as_bytes());
    }
    for name in &record.output.rerun_if_env_changed {
        bytes.extend_from_slice(format!("rerun-env {}\n", esc(name)).as_bytes());
    }
    for (path, time) in source_times {
        let stamp: String = time.iter().map(|byte| format!("{byte:02x}")).collect();
        bytes.extend_from_slice(format!("source-time {stamp} {}\n", esc(&path.to_string_lossy())).as_bytes());
    }
    bytes.extend_from_slice(b"record\n");
    bytes.extend_from_slice(encoded.strip_prefix(b"LBS5\n").expect("record format"));
    let temporary = temporary_path(path, "record");
    let mut created = false;
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        created = true;
        file.write_all(&bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() && created {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(|e| format!("cannot publish native build-script record: {e}"))
}

fn temporary_path(path: &Path, kind: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    path.with_extension(format!("{kind}-{}-{sequence}.pending", std::process::id()))
}

/// Unlike the legacy recorder, an unreadable or missing output is not an empty
/// successful generation. Symlink outputs require a separate represented input
/// contract; following one here could escape the leased output directory.
fn checked_output_hash(output: &BuildOutput, directory: &Path) -> Result<u64, String> {
    let mut paths = vec![directory.to_path_buf()];
    let mut files = Vec::new();
    let mut directories = Vec::new();
    while let Some(path) = paths.pop() {
        let metadata =
            std::fs::symlink_metadata(&path).map_err(|e| format!("cannot inspect build-script output {}: {e}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err("native build-script integration does not yet represent symlink outputs".into());
        }
        if metadata.is_dir() {
            directories.push(path.clone());
            for entry in std::fs::read_dir(&path).map_err(|e| format!("cannot read build-script output directory: {e}"))? {
                paths.push(entry.map_err(|e| format!("cannot read build-script output entry: {e}"))?.path());
            }
        } else if metadata.is_file() {
            let bytes = std::fs::read(&path).map_err(|e| format!("cannot read build-script output {}: {e}", path.display()))?;
            files.push((path, bytes));
        } else {
            return Err("native build-script integration requires ordinary output files".into());
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    directories.sort();
    let mut cfg = output.rustc_cfg.clone();
    cfg.sort();
    let mut check_cfg = output.rustc_check_cfg.clone();
    check_cfg.sort();
    let mut env = output.rustc_env.clone();
    env.sort();
    let mut parts = vec![b"native-outputs-v2".to_vec(), format!("{cfg:?}{check_cfg:?}{env:?}").into_bytes()];
    for path in directories {
        parts.push(b"directory".to_vec());
        parts.push(path.strip_prefix(directory).expect("output descendant").as_os_str().as_encoded_bytes().to_vec());
    }
    for (path, bytes) in files {
        parts.push(b"file".to_vec());
        parts.push(path.strip_prefix(directory).expect("output descendant").as_os_str().as_encoded_bytes().to_vec());
        parts.push(bytes);
    }
    Ok(hash_parts(&parts.iter().map(Vec::as_slice).collect::<Vec<_>>()))
}

fn source_holds(record: &Record) -> bool {
    !record.sources.is_empty() && record.sources.iter().all(|(path, hash)| path.is_file() && file_hash(path) == *hash)
}

fn absent_holds(paths: &[PathBuf]) -> bool {
    paths.iter().all(|path| !path.is_file())
}

fn compile_holds(record: &Record, plan: &BuildScriptUnit, key: u64, binary: &Path, env: &[(String, String)]) -> bool {
    record.script == plan.script
        && record.setup == key
        && source_holds(record)
        && record.compile.as_ref().is_some_and(|c| {
            c.cwd == plan.manifest_dir
                && c.argv == ["native-v1".to_string(), binary.to_string_lossy().into_owned()]
                && binary.is_file()
                && file_hash(binary) == c.bin
                && c.vars.iter().all(|(name, value)| *value == env_value(env.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())))
        })
}

fn observe_source_times(sources: &[(PathBuf, u64)]) -> Option<Vec<(PathBuf, [u8; 13])>> {
    sources.iter().map(|(path, _)| Some((path.clone(), modified_stamp(&std::fs::metadata(path).ok()?)?))).collect()
}

fn source_times_hold(stamps: &[(PathBuf, [u8; 13])], sources: &[(PathBuf, u64)]) -> bool {
    stamps.len() == sources.len()
        && stamps.iter().zip(sources).all(|((path, expected), (source, _))| {
            path == source && std::fs::metadata(path).ok().and_then(|metadata| modified_stamp(&metadata)) == Some(*expected)
        })
}

fn run_holds(native: &NativeRecord, cache: &Path, client: ClientEnv, directory: &Path) -> bool {
    let record = &native.record;
    record.env_inputs.iter().all(|(name, value)| client_var(client, name) == *value)
        // Host source dependencies are implicit rerun inputs even when the
        // script prints only an environment directive. Their unchanged bytes
        // still reuse compiled code; changed timestamps rerun that code.
        && source_times_hold(&native.source_times, &record.sources)
        && record.inputs.iter().all(|input| current_input_hash(input, cache).is_ok_and(|hash| input.hash == hash))
        && record.out_dir.as_deref() == Some(directory)
        && checked_output_hash(&record.output, directory).is_ok_and(|hash| hash == record.output_hash)
}

fn compile_script(plan: &BuildScriptUnit, directory: &Path, binary: &Path, env: Vec<(String, String)>) -> Result<CompileProof, String> {
    for tool in ["RUSTC", "CARGO"] {
        if !env.iter().any(|(name, value)| name == tool && Path::new(value).is_file()) {
            return Err("native build-script compilation needs the installed Lustr tool entries".into());
        }
    }
    let original = std::fs::read_to_string(&plan.script).map_err(|e| format!("cannot read build script: {e}"))?;
    let mut key = SessionKey::synthetic(&plan.script, false);
    key.krate.package_id = format!("native-host:{}", plan.package_id);
    key.krate.target_name = plan.crate_name.clone();
    key.krate.target_explicit = true;
    key.features = plan.features.clone();
    key.target_triple = crate::protocol::host_triple().into();
    key.profile = plan.host_profile;
    let temporary = temporary_path(binary, "host");
    let request = Request {
        cmd: Cmd::Build,
        session: key.clone(),
        file: plan.script.clone(),
        out: temporary.clone(),
        json: false,
        timings: false,
        debug: false,
        emit_c: None,
        max_errors: 0,
        run_args: None,
        harness: false,
        cc: false,
        library_options: Default::default(),
        client: crate::protocol::Client {
            env: Some(env.clone()),
            shown: Some(plan.script.strip_prefix(&plan.workspace_root).unwrap_or(&plan.script).to_path_buf()),
        },
    };
    let mut session = Session::new(directory.join("native-state"), false, key);
    let mut absent = Vec::new();
    let response = match crate::driver::prepare(&request, &mut session) {
        Preparation::Ready(prepared) => {
            if !prepared.source().starts_with(&original) {
                return Err("build-script source changed while native preparation read it".into());
            }
            // Use the ordinary resolver's checked negative lookups. Creating
            // the other candidate for `mod name;` must invalidate this host
            // executable even when every previously read source is unchanged.
            if let Some(tables) = &prepared.program().resolved {
                absent = tables.module_alts.clone();
            }
            prepared.emit()
        }
        Preparation::Complete(response)
        | Preparation::Rejected(response)
        | Preparation::Failed(response)
        | Preparation::IntegrationIncomplete(response) => response,
    };
    if response.code != 0 || response.ran || !temporary.is_file() {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("native build-script compilation failed: {}", response.stderr));
    }
    let observed = (|| -> Result<(Vec<(PathBuf, u64)>, Vec<(String, String)>), String> {
        if std::fs::read_to_string(&plan.script).ok().as_deref() != Some(original.as_str()) {
            return Err("build-script source changed during native compilation".into());
        }
        let mut sources = BTreeMap::from([(plan.script.clone(), hash_parts(&[original.as_bytes()]))]);
        let map = &session.front.map;
        for id in 0..map.file_count() {
            let id = crate::diag::FileId(id as u32);
            let path = PathBuf::from(map.path(id));
            if path == plan.script {
                continue;
            }
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot bind native build-script source {}: {e}", path.display()))?;
            if crate::diag::text_hash(&text) != map.text_hash(id) {
                return Err("build-script source changed during native compilation".into());
            }
            sources.insert(path, hash_parts(&[text.as_bytes()]));
        }
        let mut reads = BTreeMap::new();
        for chunk in &session.front.chunks {
            if crate::expand::deps_stale(&chunk.expand_deps, &session.front.client) {
                return Err("build-script expansion input changed during native compilation".into());
            }
            for dep in &chunk.expand_deps {
                match dep {
                    crate::expand::Dep::File { path, hash } => {
                        let bytes = std::fs::read(path).map_err(|e| format!("cannot bind build-script expansion input: {e}"))?;
                        let mut observed = FxHasher::default();
                        bytes.hash(&mut observed);
                        if observed.finish() != *hash {
                            return Err("build-script expansion input changed during native compilation".into());
                        }
                        sources.insert(path.clone(), hash_parts(&[&bytes]));
                    }
                    crate::expand::Dep::Env { name, value } => {
                        reads.insert(name.clone(), env_value(value.as_deref()));
                    }
                    _ => {}
                }
            }
        }
        Ok((sources.into_iter().collect(), reads.into_iter().collect()))
    })();
    let (sources, vars) = match observed {
        Ok(reads) => reads,
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
    };
    if !absent_holds(&absent) {
        let _ = std::fs::remove_file(&temporary);
        return Err("build-script module lookup changed during native compilation".into());
    }
    std::fs::rename(&temporary, binary).map_err(|e| format!("cannot publish native build-script executable: {e}"))?;
    Ok(CompileProof {
        command: Compile {
            cwd: plan.manifest_dir.clone(),
            vars,
            argv: vec!["native-v1".into(), binary.to_string_lossy().into_owned()],
            bin: file_hash(binary),
        },
        sources,
        absent,
    })
}

/// Pinned Rust `rustc_interface::interface::parse_cfg` requires one name,
/// optionally one string literal, and EOF. Reuse the native lexer for that
/// grammar, but admit only the custom forms the downstream cfg service can
/// preserve. Its existing string consumer does not decode escaped/raw values
/// or alter the separately owned Cargo feature/test/builtin configuration.
pub(crate) fn cfg_directive(kind: &str, value: &str) -> Result<String, String> {
    use crate::lexer::Tok;
    let unsupported = || format!("native build-script integration does not yet represent `{kind}={value}`");
    let interner = crate::intern::Interner::default();
    let lexed = crate::lexer::lex(value, crate::diag::FileId(0), 0, &interner).map_err(|_| unsupported())?;
    let tokens: Vec<_> = lexed.tokens.iter().map(|token| token.tok).collect();
    let custom = |symbol| {
        let name = interner.get(symbol);
        if crate::resolve::cfg::KNOWN_CFG_NAMES.contains(&name)
            || crate::resolve::cfg::UNSTABLE_CFG_NAMES.contains(&name)
            || matches!(name, "proc_macro" | "doc" | "doctest" | "lust_dev")
        {
            return Err(format!(
                "native build-script integration does not yet represent overriding builtin or separately owned cfg `{name}`"
            ));
        }
        Ok(name.to_string())
    };
    match (kind, tokens.as_slice()) {
        ("rustc-cfg", [Tok::Ident(symbol, false), Tok::Eof]) => {
            let name = custom(*symbol)?;
            if value.trim() != name {
                return Err(unsupported());
            }
            Ok(name)
        }
        ("rustc-cfg", [Tok::Ident(symbol, false), Tok::Eq, Tok::Str(index, _), Tok::Eof]) => {
            let name = custom(*symbol)?;
            let literal = &lexed.strings[*index as usize];
            if literal.contains(['"', '\\', '\n', '\r', '\0']) {
                return Err(unsupported());
            }
            let quoted = format!("\"{literal}\"");
            let (written_name, written_value) = value.split_once('=').ok_or_else(unsupported)?;
            if written_name.trim() != name || written_value.trim() != quoted {
                return Err(unsupported());
            }
            Ok(format!("{name}={quoted}"))
        }
        ("rustc-check-cfg", [Tok::Ident(cfg, false), Tok::LParen, Tok::Ident(symbol, false), Tok::RParen, Tok::Eof])
            if interner.get(*cfg) == "cfg" =>
        {
            let name = custom(*symbol)?;
            let canonical = format!("cfg({name})");
            if value.chars().filter(|c| !c.is_whitespace()).collect::<String>() != canonical {
                return Err(unsupported());
            }
            Ok(canonical)
        }
        _ => Err(unsupported()),
    }
}

fn directives(bytes: &[u8]) -> Result<BuildOutput, String> {
    let mut text = String::new();
    for raw in bytes.split(|b| *b == b'\n') {
        let line = match std::str::from_utf8(raw) {
            Ok(line) => line.trim(),
            Err(_) => {
                let raw = raw.trim_ascii_start();
                if raw.starts_with(b"cargo:") {
                    return Err("non-UTF8 native build-script directive".into());
                }
                continue;
            }
        };
        let Some(rest) = line.strip_prefix("cargo:") else { continue };
        let rest = rest.strip_prefix(':').unwrap_or(rest);
        let Some((key, value)) = rest.split_once('=') else {
            return Err("malformed build-script directive".into());
        };
        if !matches!(key, "rerun-if-changed" | "rerun-if-env-changed" | "rustc-cfg" | "rustc-check-cfg" | "rustc-env") {
            return Err(format!("native build-script integration does not yet represent `{key}`"));
        }
        if value.contains('\0') || (key == "rustc-env" && value.split_once('=').is_none_or(|(name, _)| name.is_empty())) {
            return Err("malformed build-script directive value".into());
        }
        if matches!(key, "rustc-cfg" | "rustc-check-cfg") {
            text.push_str(&format!("cargo::{key}={}", cfg_directive(key, value)?));
        } else {
            text.push_str(line);
        }
        text.push('\n');
    }
    let mut output = parse_output(&text);
    // Command environments use the final assignment to each name. Do not let
    // reordering duplicate directives keep an old effective output hash.
    output.rustc_env = output.rustc_env.into_iter().collect::<BTreeMap<_, _>>().into_iter().collect();
    Ok(output)
}

fn input_path(path: &Path) -> Result<PathBuf, String> {
    use std::path::Component;
    if !path.is_absolute() {
        return Err("native build-script input is not absolute".into());
    }
    ordinary_input_path(path)?;
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                clean.pop();
            }
            Component::CurDir => {}
            other => clean.push(other.as_os_str()),
        }
    }
    Ok(clean)
}

/// Keep the bytes observed before an execution. Filesystem timestamps are not
/// a before/after proof: an in-run write can share a clock tick with the start,
/// and the unchanged upstream edit fixtures deliberately use future mtimes.
/// The package tree lets newly declared local inputs use that same snapshot.
/// Excluded subtrees or first-time external inputs remain explicit gaps.
#[derive(Default)]
struct InputTree {
    root: PathBuf,
    files: BTreeMap<PathBuf, u64>,
    directories: BTreeMap<PathBuf, [u8; 13]>,
    unavailable: BTreeSet<PathBuf>,
    excluded: BTreeSet<PathBuf>,
}

// Cargo's declared inputs expose modification time, including future values.
// Bind equality of the already-read observation, never a wall-clock cutoff.
fn modified_stamp(metadata: &std::fs::Metadata) -> Option<[u8; 13]> {
    let observed = metadata.modified().ok()?.duration_since(std::time::UNIX_EPOCH);
    let before_epoch = observed.is_err();
    let duration = observed.unwrap_or_else(|error| error.duration());
    let mut bytes = [0; 13];
    bytes[0] = u8::from(before_epoch);
    bytes[1..9].copy_from_slice(&duration.as_secs().to_le_bytes());
    bytes[9..13].copy_from_slice(&duration.subsec_nanos().to_le_bytes());
    Some(bytes)
}

impl InputTree {
    fn capture(root: &Path, package_cache: Option<&Path>) -> Self {
        let mut tree = Self { root: root.to_path_buf(), ..Default::default() };
        let mut paths = vec![root.to_path_buf()];
        while let Some(path) = paths.pop() {
            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => {
                    tree.unavailable.insert(path);
                    continue;
                }
            };
            if let Some(cache) = package_cache.filter(|_| path != root) {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(name, ".git" | ".hg" | ".svn")
                    || name.starts_with(".lust")
                    || path == cache
                    || metadata.is_dir() && (path.join("CACHEDIR.TAG").exists() || path.join("Cargo.toml").exists())
                {
                    tree.excluded.insert(path);
                    continue;
                }
            }
            if metadata.file_type().is_symlink() {
                tree.unavailable.insert(path);
            } else if metadata.is_file() {
                let Some(modified) = modified_stamp(&metadata) else {
                    tree.unavailable.insert(path);
                    continue;
                };
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        tree.files.insert(path, hash_parts(&[b"native-input-file", &modified, &bytes]));
                    }
                    Err(_) => {
                        tree.unavailable.insert(path);
                    }
                }
            } else if metadata.is_dir() {
                let Some(modified) = modified_stamp(&metadata) else {
                    tree.unavailable.insert(path);
                    continue;
                };
                tree.directories.insert(path.clone(), modified);
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        for entry in entries {
                            match entry {
                                Ok(entry) => paths.push(entry.path()),
                                Err(_) => {
                                    tree.unavailable.insert(path.clone());
                                }
                            }
                        }
                    }
                    Err(_) => {
                        tree.unavailable.insert(path);
                    }
                }
            } else {
                tree.unavailable.insert(path);
            }
        }
        tree
    }

    fn hash(&self, path: &Path, package: bool) -> Result<u64, String> {
        if !path.starts_with(&self.root) {
            return Err(format!("native build-script integration has no pre-execution snapshot for external input {}", path.display()));
        }
        let intersects = |other: &PathBuf| path.starts_with(other) || other.starts_with(path);
        if self.unavailable.iter().any(intersects) || (!package && self.excluded.iter().any(intersects)) {
            return Err(format!("native build-script input is outside the captured ordinary file tree: {}", path.display()));
        }
        if let Some(hash) = self.files.get(path) {
            return Ok(*hash);
        }
        if !self.directories.contains_key(path) {
            return Ok(hash_parts(&[b"native-input-missing"]));
        }
        let mut parts = vec![b"native-directory-input-v3".to_vec()];
        for (directory, modified) in self.directories.iter().filter(|(directory, _)| directory.starts_with(path)) {
            parts.push(b"directory".to_vec());
            parts.push(directory.strip_prefix(path).expect("snapshot descendant").as_os_str().as_encoded_bytes().to_vec());
            parts.push(modified.to_vec());
        }
        for (file, hash) in self.files.iter().filter(|(file, _)| file.starts_with(path)) {
            parts.push(b"file".to_vec());
            parts.push(file.strip_prefix(path).expect("snapshot descendant").as_os_str().as_encoded_bytes().to_vec());
            parts.push(hash.to_le_bytes().to_vec());
        }
        Ok(hash_parts(&parts.iter().map(Vec::as_slice).collect::<Vec<_>>()))
    }
}

fn ordinary_input_path(path: &Path) -> Result<(), String> {
    // A changed ancestor must not turn an ordinary recorded path into an
    // unobserved symlink traversal, even if its destination has equal bytes.
    // Check the written traversal before removing '..': link/../file need
    // not mean the same file as its lexical normalization.
    for ancestor in path.ancestors() {
        if std::fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("native build-script integration does not yet represent symlink runtime inputs".into());
        }
    }
    Ok(())
}

fn current_input_hash(input: &Input, cache: &Path) -> Result<u64, String> {
    ordinary_input_path(&input.path)?;
    InputTree::capture(&input.path, input.package.then_some(cache)).hash(&input.path, input.package)
}

struct RuntimeInputs {
    package: Option<InputTree>,
    known: BTreeMap<PathBuf, u64>,
    setup: Vec<PathBuf>,
}

impl RuntimeInputs {
    fn capture(plan: &BuildScriptUnit, cache: &Path, client: ClientEnv, previous: Option<&Record>) -> Result<Self, String> {
        // A prior explicit contract permits narrow observations. New paths
        // discovered by this run can be returned without a reuse certificate;
        // do not rescan the package for every environment-only edit.
        let package = previous
            .is_none_or(|record| record.output.rerun_if_changed.is_empty() && record.output.rerun_if_env_changed.is_empty())
            .then(|| InputTree::capture(&plan.manifest_dir, Some(cache)));
        let setup = setup_files(&plan.manifest_dir, client).into_iter().map(|p| input_path(&p)).collect::<Result<Vec<_>, _>>()?;
        let mut paths = setup.clone();
        if let Some(record) = previous {
            // A successful but uncacheable run still tells the next request
            // which external paths to observe before its one execution.
            paths.extend(record.output.rerun_if_changed.iter().map(|p| {
                let path = Path::new(p);
                if path.is_absolute() { path.to_path_buf() } else { plan.manifest_dir.join(path) }
            }));
        }
        let mut known = BTreeMap::new();
        for path in paths {
            let Ok(path) = input_path(&path) else {
                continue;
            };
            let input = Input { package: false, path: path.clone(), hash: 0 };
            // Unobserved runtime files prevent reuse, not execution. Setup
            // files are checked separately and must retain their identity.
            if let Ok(hash) = current_input_hash(&input, cache) {
                known.insert(path, hash);
            }
        }
        Ok(Self { package, known, setup })
    }

    fn bind(&self, path: PathBuf, package: bool, cache: &Path) -> Result<Input, String> {
        let path = input_path(&path)?;
        let from_tree = || {
            self.package
                .as_ref()
                .ok_or_else(|| "native build-script input was not observed before execution".to_string())?
                .hash(&path, package)
        };
        let hash = if !package { self.known.get(&path).copied().map(Ok).unwrap_or_else(from_tree)? } else { from_tree()? };
        let input = Input { package, path, hash };
        if current_input_hash(&input, cache)? != hash {
            return Err(format!("build-script input changed during execution: {}", input.path.display()));
        }
        Ok(input)
    }
}

fn run_script(plan: &BuildScriptUnit, binary: &Path, directory: &Path, client: ClientEnv) -> Result<BuildOutput, String> {
    std::fs::create_dir_all(directory).map_err(|e| format!("cannot create native build-script OUT_DIR: {e}"))?;
    let env = runtime_env(plan, client, directory)?;
    execute(binary, &plan.manifest_dir, env)
}

fn execute(binary: &Path, cwd: &Path, env: Vec<(String, String)>) -> Result<BuildOutput, String> {
    let result = Command::new(binary)
        .current_dir(cwd)
        .env_clear()
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("cannot execute native build script: {e}"))?;
    if !result.status.success() {
        return Err(format!("native build script failed ({}): {}", result.status, String::from_utf8_lossy(&result.stderr)));
    }
    directives(&result.stdout)
}

/// The caller retains the returned lease in its checked native continuation.
/// Compilation, execution and record publication each have one terminal outcome;
/// a failure never falls back, retries, or exposes an old successful generation.
pub fn outputs(plan: &BuildScriptUnit, cache: &Path, request: &Request) -> Result<Option<Outputs>, String> {
    outputs_with(plan, cache, request, compile_script, run_script)
}

fn outputs_with<C, R>(plan: &BuildScriptUnit, cache: &Path, request: &Request, compile: C, run: R) -> Result<Option<Outputs>, String>
where
    C: FnOnce(&BuildScriptUnit, &Path, &Path, Vec<(String, String)>) -> Result<CompileProof, String>,
    R: FnOnce(&BuildScriptUnit, &Path, &Path, ClientEnv) -> Result<BuildOutput, String>,
{
    if plan.edition != "2024" || plan.crate_name != "build_script_build" {
        return Err("native build-script integration requires the planned edition-2024 host unit".into());
    }
    if crate::protocol::source_override().is_some() {
        return Err("native build-script retained records require the pinned compiler library sources".into());
    }
    let key = plan_key(plan);
    let directory = cache.join(format!("native-buildscript-{key:016x}"));
    std::fs::create_dir_all(&directory).map_err(|e| format!("cannot create native build-script state: {e}"))?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("lock"))
        .map_err(|e| format!("cannot open native build-script lease: {e}"))?;
    lock.lock().map_err(|e| format!("cannot acquire native build-script lease: {e}"))?;
    let lease = Arc::new(lock);
    let path = cache.join(format!("buildscript-native-{key:016x}.txt"));
    let binary = directory.join("build-script");
    let out_dir = directory.join("out");
    let client = request.client.env.as_deref();
    let env = compile_env(plan, client)?;
    let previous = read_record(&path).filter(|r| compile_holds(&r.record, plan, key, &binary, &env) && absent_holds(&r.absent));
    let record = if let Some(native) = previous.as_ref().filter(|r| r.runtime_reusable && run_holds(r, cache, client, &out_dir)) {
        let record = &native.record;
        return Ok(Some(Outputs {
            out_dir: record.out_dir.clone(),
            rustc_cfg: record.output.rustc_cfg.clone(),
            rustc_check_cfg: record.output.rustc_check_cfg.clone(),
            rustc_env: record.output.rustc_env.clone(),
            lease: Some(lease),
        }));
    } else {
        let runtime_inputs = RuntimeInputs::capture(plan, cache, client, previous.as_ref().map(|r| &r.record))?;
        let proof = if let Some(native) = previous {
            let record = native.record;
            CompileProof {
                command: record.compile.expect("validated native compile record"),
                sources: record.sources,
                absent: native.absent,
            }
        } else {
            compile(plan, &directory, &binary, env)?
        };
        let source_times = observe_source_times(&proof.sources).unwrap_or_default();
        let output = run(plan, &binary, &out_dir, client)?;
        let mut runtime_reusable = true;
        let mut inputs = Vec::new();
        for p in &output.rerun_if_changed {
            let path = if Path::new(p).is_absolute() { PathBuf::from(p) } else { plan.manifest_dir.join(p) };
            match runtime_inputs.bind(path, false, cache) {
                Ok(input) => inputs.push(input),
                Err(_) => runtime_reusable = false,
            }
        }
        if output.rerun_if_changed.is_empty() && output.rerun_if_env_changed.is_empty() {
            match runtime_inputs.bind(plan.manifest_dir.clone(), true, cache) {
                Ok(input) => inputs.push(input),
                Err(_) => runtime_reusable = false,
            }
        }
        for path in &runtime_inputs.setup {
            inputs.push(runtime_inputs.bind(path.clone(), false, cache)?);
        }
        let mut env_inputs: Vec<_> = output.rerun_if_env_changed.iter().map(|name| (name.clone(), client_var(client, name))).collect();
        env_inputs.sort();
        env_inputs.dedup();
        let record = Record {
            script: plan.script.clone(),
            sources: proof.sources,
            inputs,
            env_inputs,
            rustc: None,
            setup: key,
            output_hash: checked_output_hash(&output, &out_dir)?,
            out_dir: Some(out_dir),
            output,
            compile: Some(proof.command),
        };
        if !source_holds(&record) || !absent_holds(&proof.absent) {
            return Err("native build-script source changed before output publication".into());
        }
        runtime_reusable &= source_times_hold(&source_times, &record.sources);
        write_record(&path, &record, runtime_reusable, &proof.absent, &source_times)?;
        record
    };
    Ok(Some(Outputs {
        out_dir: record.out_dir,
        rustc_cfg: record.output.rustc_cfg,
        rustc_check_cfg: record.output.rustc_check_cfg,
        rustc_env: record.output.rustc_env,
        lease: Some(lease),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Fixture {
        root: PathBuf,
        plan: BuildScriptUnit,
        request: Request,
        compiled: Cell<usize>,
        ran: Cell<usize>,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("lustr-host-record-{}-{id}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            let script = root.join("build.rs");
            std::fs::write(&script, "fn main() {}\n").unwrap();
            std::fs::write(root.join("Cargo.toml"), "[package]\nname='host-control'\nversion='0.1.0'\nedition='2024'\n").unwrap();
            let plan = BuildScriptUnit {
                script: script.clone(),
                manifest_dir: root.clone(),
                workspace_root: root.clone(),
                package_id: "host-control@0.1.0".into(),
                crate_name: "build_script_build".into(),
                edition: "2024".into(),
                features: Vec::new(),
                host_profile: crate::protocol::ProfileFlags::DEV_UNWIND,
                target_triple: String::new(),
                target_profile: crate::protocol::ProfileFlags::DEV_UNWIND,
                target_features: Vec::new(),
                target_opt_level: "0".into(),
                target_debug: true,
                cargo_env: Vec::new(),
                planning_key: 1,
            };
            let request = Request {
                cmd: Cmd::Build,
                session: SessionKey::synthetic(&script, false),
                file: script,
                out: root.join("consumer"),
                json: false,
                timings: false,
                debug: false,
                emit_c: None,
                max_errors: 0,
                run_args: None,
                harness: false,
                cc: false,
                library_options: Default::default(),
                client: crate::protocol::Client { env: Some(Vec::new()), shown: None },
            };
            Self { root, plan, request, compiled: Cell::new(0), ran: Cell::new(0) }
        }
        fn cache(&self) -> PathBuf {
            self.root.join(".lust-cache")
        }
        fn record(&self) -> PathBuf {
            self.cache().join(format!("buildscript-native-{:016x}.txt", plan_key(&self.plan)))
        }
        // Synthetic boundary controls exercise the same record/attempt owner.
        // The public retained campaign separately compiles the real build.rs.
        fn compile(&self, plan: &BuildScriptUnit, _: &Path, binary: &Path, _: Vec<(String, String)>) -> Result<CompileProof, String> {
            self.compiled.set(self.compiled.get() + 1);
            std::fs::write(binary, "native compiler control artifact").unwrap();
            Ok(CompileProof {
                command: Compile {
                    cwd: plan.manifest_dir.clone(),
                    argv: vec!["native-v1".into(), binary.to_string_lossy().into_owned()],
                    vars: Vec::new(),
                    bin: file_hash(binary),
                },
                sources: vec![(plan.script.clone(), file_hash(&plan.script))],
                absent: Vec::new(),
            })
        }
        fn run(&self, _: &BuildScriptUnit, _: &Path, directory: &Path, env: ClientEnv) -> Result<BuildOutput, String> {
            self.ran.set(self.ran.get() + 1);
            std::fs::create_dir_all(directory).unwrap();
            Ok(BuildOutput {
                rerun_if_env_changed: vec!["INPUT".into()],
                rustc_env: vec![("OBSERVED".into(), client_var(env, "INPUT").unwrap_or_else(|| "unset".into()))],
                ..Default::default()
            })
        }
        fn outputs(&self) -> Result<Outputs, String> {
            outputs_with(&self.plan, &self.cache(), &self.request, |p, d, b, e| self.compile(p, d, b, e), |p, b, d, e| self.run(p, b, d, e))
                .map(|out| out.unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn unchanged_records_do_no_compile_or_run_and_client_values_remain_distinct() {
        let mut fixture = Fixture::new();
        assert_eq!(fixture.outputs().unwrap().rustc_env, [("OBSERVED".into(), "unset".into())]);
        drop(fixture.outputs().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 1));
        fixture.request.client.env =
            Some(vec![("LUSTR_TRACE_PATH".into(), "another-request.trace".into()), ("UNREAD".into(), "changed".into())]);
        drop(fixture.outputs().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 1));
        fixture.request.client.env = Some(vec![("INPUT".into(), String::new())]);
        assert_eq!(fixture.outputs().unwrap().rustc_env, [("OBSERVED".into(), String::new())]);
        fixture.request.client.env = Some(vec![("INPUT".into(), "a".into())]);
        assert_eq!(fixture.outputs().unwrap().rustc_env, [("OBSERVED".into(), "a".into())]);
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 3));
    }

    #[test]
    fn actual_compile_environment_reads_invalidate_without_unread_environment_noise() {
        let mut fixture = Fixture::new();
        let run = |fixture: &Fixture| {
            outputs_with(
                &fixture.plan,
                &fixture.cache(),
                &fixture.request,
                |p, d, b, env| {
                    let read = env_value(env.iter().find(|(name, _)| name == "COMPILE_READ").map(|(_, value)| value.as_str()));
                    let mut proof = fixture.compile(p, d, b, env)?;
                    proof.command.vars.push(("COMPILE_READ".into(), read));
                    Ok(proof)
                },
                |p, b, d, env| fixture.run(p, b, d, env),
            )
        };
        drop(run(&fixture).unwrap());
        fixture.request.client.env = Some(vec![("LUSTR_TRACE_PATH".into(), "next.trace".into()), ("UNREAD".into(), "changed".into())]);
        drop(run(&fixture).unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 1));
        fixture.request.client.env = Some(vec![("COMPILE_READ".into(), String::new())]);
        drop(run(&fixture).unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (2, 2));
    }

    #[test]
    fn implicit_host_source_timestamps_rerun_without_recompiling_unchanged_bytes() {
        let fixture = Fixture::new();
        drop(fixture.outputs().unwrap());
        let future = std::time::SystemTime::now() + std::time::Duration::from_secs(3600);
        std::fs::File::options()
            .write(true)
            .open(&fixture.plan.script)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(future))
            .unwrap();
        drop(fixture.outputs().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 2));
        drop(fixture.outputs().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 2));
    }

    #[test]
    fn a_new_alternative_module_rechecks_the_host_without_running_a_stale_binary() {
        let fixture = Fixture::new();
        let alternative = fixture.root.join("other/mod.rs");
        let first = outputs_with(
            &fixture.plan,
            &fixture.cache(),
            &fixture.request,
            |p, d, b, e| {
                let mut proof = fixture.compile(p, d, b, e)?;
                proof.absent.push(alternative.clone());
                Ok(proof)
            },
            |p, b, d, e| fixture.run(p, b, d, e),
        );
        drop(first.unwrap());
        let saved = std::fs::read(fixture.record()).unwrap();
        std::fs::create_dir_all(alternative.parent().unwrap()).unwrap();
        std::fs::write(&alternative, "pub fn f() {}\n").unwrap();
        let second = outputs_with(
            &fixture.plan,
            &fixture.cache(),
            &fixture.request,
            |_, _, _, _| {
                fixture.compiled.set(fixture.compiled.get() + 1);
                Err("ambiguous module from the native resolver".into())
            },
            |_, _, _, _| panic!("ran the stale host after its module absence changed"),
        );
        assert!(matches!(second, Err(error) if error.contains("ambiguous module")));
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (2, 1));
        assert_eq!(std::fs::read(fixture.record()).unwrap(), saved);
    }

    #[test]
    fn positive_job_counts_are_represented_and_untransferred_jobservers_remain_explicit() {
        let fixture = Fixture::new();
        assert!(job_count(Some(&[])).unwrap() > 0);
        assert_eq!(job_count(Some(&[("CARGO_BUILD_JOBS".into(), "2".into())])).unwrap(), 2);
        for (name, value, reason) in [
            ("CARGO_MAKEFLAGS", "--jobserver-auth=3,4", "jobserver"),
            ("CARGO_BUILD_JOBS", "-1", "job counts"),
            ("CARGO_BUILD_JOBS", "0", "job counts"),
            ("CARGO_BUILD_JOBS", "invalid", "job counts"),
        ] {
            let env = vec![(name.to_string(), value.to_string())];
            assert!(matches!(runtime_env(&fixture.plan, Some(&env), &fixture.root), Err(error) if error.contains(reason)));
        }
    }

    #[test]
    fn failed_run_has_one_attempt_and_keeps_prior_record_for_proven_recovery() {
        let mut fixture = Fixture::new();
        drop(fixture.outputs().unwrap());
        let before = std::fs::read(fixture.record()).unwrap();
        fixture.request.client.env = Some(vec![("INPUT".into(), "changed".into())]);
        let result = outputs_with(
            &fixture.plan,
            &fixture.cache(),
            &fixture.request,
            |_, _, _, _| panic!("runtime input change recompiled the script"),
            |_, _, _, _| {
                fixture.ran.set(fixture.ran.get() + 1);
                Err("original script failure".into())
            },
        );
        assert!(matches!(result, Err(error) if error == "original script failure"));
        assert_eq!(fixture.ran.get(), 2);
        assert_eq!(std::fs::read(fixture.record()).unwrap(), before);
        fixture.request.client.env = Some(Vec::new());
        drop(fixture.outputs().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 2));
    }

    #[test]
    fn changed_source_cannot_be_rebound_to_old_compilation_after_a_run() {
        let fixture = Fixture::new();
        let result = outputs_with(
            &fixture.plan,
            &fixture.cache(),
            &fixture.request,
            |p, d, b, e| fixture.compile(p, d, b, e),
            |p, b, d, e| {
                let output = fixture.run(p, b, d, e)?;
                std::fs::write(&p.script, "fn main() { changed(); }\n").unwrap();
                Ok(output)
            },
        );
        assert!(matches!(result, Err(error) if error.contains("source changed")));
        assert!(!fixture.record().exists());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 1));
    }

    #[test]
    fn changed_runtime_input_keeps_the_result_but_never_certifies_post_run_bytes() {
        let fixture = Fixture::new();
        let input = fixture.root.join("input");
        std::fs::write(&input, "before").unwrap();
        let run = || {
            outputs_with(
                &fixture.plan,
                &fixture.cache(),
                &fixture.request,
                |p, d, b, e| fixture.compile(p, d, b, e),
                |_, _, directory, _| {
                    fixture.ran.set(fixture.ran.get() + 1);
                    std::fs::create_dir_all(directory).unwrap();
                    let observed = std::fs::read_to_string(&input).unwrap();
                    if observed != "after" {
                        std::fs::write(&input, "after").unwrap();
                    }
                    Ok(BuildOutput {
                        rerun_if_changed: vec!["input".into()],
                        rustc_env: vec![("VALUE".into(), observed)],
                        ..Default::default()
                    })
                },
            )
        };
        assert_eq!(run().unwrap().unwrap().rustc_env, [("VALUE".into(), "before".into())]);
        assert!(!read_record(&fixture.record()).unwrap().runtime_reusable);
        assert_eq!(run().unwrap().unwrap().rustc_env, [("VALUE".into(), "after".into())]);
        assert!(read_record(&fixture.record()).unwrap().runtime_reusable);
        drop(run().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 2));
    }

    #[test]
    fn newly_declared_external_input_is_uncacheable_until_observed_before_a_later_request() {
        let fixture = Fixture::new();
        let external = fixture.root.with_extension("external");
        std::fs::write(&external, "outside").unwrap();
        let run = || {
            outputs_with(
                &fixture.plan,
                &fixture.cache(),
                &fixture.request,
                |p, d, b, e| fixture.compile(p, d, b, e),
                |_, _, directory, _| {
                    fixture.ran.set(fixture.ran.get() + 1);
                    std::fs::create_dir_all(directory).unwrap();
                    Ok(BuildOutput {
                        rerun_if_changed: vec![external.to_string_lossy().into_owned()],
                        rustc_env: vec![("VALUE".into(), std::fs::read_to_string(&external).unwrap())],
                        ..Default::default()
                    })
                },
            )
        };
        assert_eq!(run().unwrap().unwrap().rustc_env, [("VALUE".into(), "outside".into())]);
        assert!(!read_record(&fixture.record()).unwrap().runtime_reusable);
        drop(run().unwrap());
        assert!(read_record(&fixture.record()).unwrap().runtime_reusable);
        drop(run().unwrap());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 2));
        std::fs::remove_file(external).unwrap();
    }

    #[test]
    fn runtime_snapshots_distinguish_missing_inputs_and_ignore_future_mtimes() {
        let fixture = Fixture::new();
        let data = fixture.root.join("data");
        std::fs::create_dir(&data).unwrap();
        let input = data.join("input");
        std::fs::write(&input, "same").unwrap();
        let future = std::time::SystemTime::now() + std::time::Duration::from_secs(3600);
        std::fs::File::options().write(true).open(&input).unwrap().set_times(std::fs::FileTimes::new().set_modified(future)).unwrap();
        let before = RuntimeInputs::capture(&fixture.plan, &fixture.cache(), Some(&[]), None).unwrap();
        assert!(before.bind(input.clone(), false, &fixture.cache()).is_ok());
        assert!(before.bind(data.clone(), false, &fixture.cache()).is_ok());
        let missing = data.join("missing");
        assert!(before.bind(missing.clone(), false, &fixture.cache()).is_ok());
        // The old untagged sentinel collided with a real file containing
        // these bytes; missing and present must be distinct observations.
        std::fs::write(&missing, "<missing>").unwrap();
        assert!(before.bind(missing, false, &fixture.cache()).is_err());
        assert!(before.bind(data, false, &fixture.cache()).is_err());
        assert!(before.bind(input.clone(), false, &fixture.cache()).is_ok());
        std::fs::File::options()
            .write(true)
            .open(&input)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(future + std::time::Duration::from_secs(60)))
            .unwrap();
        assert!(before.bind(input, false, &fixture.cache()).is_err(), "mtime-only changes are observable build-script inputs");
        drop(fixture.outputs().unwrap());
        let saved = read_record(&fixture.record()).unwrap();
        let narrow = RuntimeInputs::capture(&fixture.plan, &fixture.cache(), Some(&[]), Some(&saved.record)).unwrap();
        assert!(narrow.package.is_none(), "an explicit prior rerun contract must not scan the package on every edit");
    }

    #[test]
    fn directory_fingerprints_include_empty_entries_and_nonlossy_file_names() {
        use std::os::unix::ffi::OsStringExt;
        let fixture = Fixture::new();
        let directory = fixture.root.join("directory");
        std::fs::create_dir(&directory).unwrap();
        let hashes = || {
            // Hold freshness constant so a timestamp change cannot hide a
            // missing directory-name or raw-filename identity in this control.
            std::fs::File::open(&directory)
                .unwrap()
                .set_times(std::fs::FileTimes::new().set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000)))
                .unwrap();
            (
                InputTree::capture(&directory, None).hash(&directory, false).unwrap(),
                checked_output_hash(&BuildOutput::default(), &directory).unwrap(),
            )
        };
        let empty = hashes();
        std::fs::create_dir(directory.join("first")).unwrap();
        let first = hashes();
        assert_ne!(empty.0, first.0);
        assert_ne!(empty.1, first.1);
        std::fs::rename(directory.join("first"), directory.join("second")).unwrap();
        let second = hashes();
        assert_ne!(first.0, second.0);
        assert_ne!(first.1, second.1);
        std::fs::remove_dir(directory.join("second")).unwrap();
        assert_eq!(empty, hashes());
        let path_a = directory.join(std::ffi::OsString::from_vec(vec![b'f', 0xfe]));
        let path_b = directory.join(std::ffi::OsString::from_vec(vec![b'f', 0xff]));
        assert_eq!(path_a.to_string_lossy(), path_b.to_string_lossy());
        std::fs::write(&path_a, "same bytes").unwrap();
        let a = hashes();
        std::fs::rename(path_a, path_b).unwrap();
        let b = hashes();
        assert_ne!(a.0, b.0);
        assert_ne!(a.1, b.1);
    }

    #[test]
    fn a_symlink_parent_traversal_cannot_certify_an_unrelated_lexical_path() {
        let fixture = Fixture::new();
        let external = fixture.root.with_extension("outside");
        std::fs::create_dir_all(external.join("sub")).unwrap();
        std::fs::write(external.join("data"), "outside-before").unwrap();
        std::fs::write(fixture.root.join("data"), "unrelated-local").unwrap();
        std::os::unix::fs::symlink(external.join("sub"), fixture.root.join("link")).unwrap();
        let run = || {
            outputs_with(
                &fixture.plan,
                &fixture.cache(),
                &fixture.request,
                |p, d, b, e| fixture.compile(p, d, b, e),
                |_, _, directory, _| {
                    fixture.ran.set(fixture.ran.get() + 1);
                    std::fs::create_dir_all(directory).unwrap();
                    Ok(BuildOutput {
                        rerun_if_changed: vec!["link/../data".into()],
                        rustc_env: vec![("VALUE".into(), std::fs::read_to_string(fixture.root.join("link/../data")).unwrap())],
                        ..Default::default()
                    })
                },
            )
        };
        assert_eq!(run().unwrap().unwrap().rustc_env, [("VALUE".into(), "outside-before".into())]);
        assert!(!read_record(&fixture.record()).unwrap().runtime_reusable);
        std::fs::write(external.join("data"), "outside-after").unwrap();
        assert_eq!(run().unwrap().unwrap().rustc_env, [("VALUE".into(), "outside-after".into())]);
        assert!(!read_record(&fixture.record()).unwrap().runtime_reusable);
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 2));
        std::fs::remove_dir_all(external).unwrap();
    }

    #[test]
    fn publication_failure_never_creates_a_success_record_or_retries() {
        let fixture = Fixture::new();
        std::fs::create_dir_all(fixture.record()).unwrap();
        let result = fixture.outputs();
        assert!(matches!(result, Err(error) if error.contains("cannot publish")));
        assert!(fixture.record().is_dir());
        assert_eq!((fixture.compiled.get(), fixture.ran.get()), (1, 1));
    }

    #[test]
    fn generated_output_lease_survives_clones_until_the_last_consumer_drops() {
        let fixture = Fixture::new();
        let output = fixture.outputs().unwrap();
        let clone = output.clone();
        let lock = OpenOptions::new().read(true).write(true).open(output.out_dir.as_ref().unwrap().parent().unwrap().join("lock")).unwrap();
        assert!(lock.try_lock().is_err());
        drop(output);
        assert!(lock.try_lock().is_err());
        drop(clone);
        lock.try_lock().unwrap();
    }

    #[test]
    fn malformed_or_unrepresented_directives_and_missing_outputs_fail_closed() {
        for stdout in [
            b"cargo::rustc-env=X=\xff".as_slice(),
            b"cargo::rustc-link-lib=missing",
            b"cargo::error=failed",
            b"cargo::rustc-env=NAME",
            b"cargo::rustc-cfg=a\0b",
        ] {
            assert!(directives(stdout).is_err(), "{stdout:?}");
        }
        assert_eq!(directives(b"cargo::rustc-env=X=old\ncargo::rustc-env=X=new\n").unwrap().rustc_env, [("X".into(), "new".into())]);
        let fixture = Fixture::new();
        assert!(checked_output_hash(&BuildOutput::default(), &fixture.root.join("missing")).is_err());
        let old = Record {
            script: fixture.plan.script.clone(),
            sources: Vec::new(),
            inputs: Vec::new(),
            env_inputs: Vec::new(),
            rustc: None,
            setup: 0,
            output_hash: 0,
            out_dir: None,
            output: BuildOutput::default(),
            compile: None,
        };
        let path = fixture.root.join("legacy");
        std::fs::write(&path, old.encode()).unwrap();
        assert!(read_record(&path).is_none());
    }

    #[test]
    fn cfg_directives_preserve_only_represented_literals_and_custom_names() {
        assert_eq!(directives(b"cargo::rustc-cfg=bs_a\ncargo::rustc-check-cfg=cfg(bs_a)\n").unwrap().rustc_cfg, ["bs_a"]);
        let output = directives(b"cargo::rustc-cfg=mode = \"a=b\"\n").unwrap();
        let mut cfg = crate::resolve::cfg::CfgEnv::default();
        cfg.add_rustc_cfg(&output.rustc_cfg[0]);
        assert_eq!(cfg.pairs, [("mode".into(), "a=b".into())]);
        for line in [
            "cargo::rustc-cfg=abc xyz",
            "cargo::rustc-cfg=mode=bare",
            "cargo::rustc-cfg=mode=1",
            "cargo::rustc-cfg=mode=\"a\\nb\"",
            "cargo::rustc-cfg=mode=r#\"raw\"#",
            "cargo::rustc-cfg=target_os=\"other\"",
            "cargo::rustc-cfg=feature=\"unplanned\"",
            "cargo::rustc-cfg=test",
            "cargo::rustc-cfg=all(foo)",
            "cargo::rustc-check-cfg=cfg(one,two)",
            "cargo::rustc-check-cfg=cfg(mode, values(\"a\"))",
            "cargo::rustc-check-cfg=cfg(any())",
        ] {
            assert!(directives(line.as_bytes()).is_err(), "{line}");
        }
    }

    #[test]
    fn a_real_failed_script_executes_once_and_uses_only_the_supplied_environment() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::new();
        let script = fixture.root.join("script");
        std::fs::write(&script, "#!/bin/sh\nprintf 'x' >> count\nprintf '%s' \"${INPUT-unset}\" > observed\nexit 7\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let result = execute(&script, &fixture.root, vec![("INPUT".into(), "request-value".into())]);
        assert!(matches!(result, Err(error) if error.contains("native build script failed")));
        assert_eq!(std::fs::read(fixture.root.join("count")).unwrap(), b"x");
        assert_eq!(std::fs::read(fixture.root.join("observed")).unwrap(), b"request-value");
    }
}
