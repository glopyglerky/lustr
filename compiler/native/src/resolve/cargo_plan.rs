//! Product Cargo unit planning. The pinned Cargo prints its resolved unit graph
//! before compilation; only Lustr's read-only compiler queries may run here.
//! Source units then enter the existing retained native dependency compiler.
use super::*;
use crate::protocol::{Cmd, ProfileFlags};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FileIdentity {
    path: PathBuf,
    canonical: PathBuf,
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
    mode: u32,
}

impl FileIdentity {
    fn read(path: &Path) -> Result<Self, String> {
        let canonical = path.canonicalize().map_err(|e| format!("cannot identify Cargo planning tool {}: {e}", path.display()))?;
        let m = std::fs::metadata(path).map_err(|e| format!("cannot inspect Cargo planning tool: {e}"))?;
        if !m.is_file() || m.mode() & 0o111 == 0 {
            return Err("Cargo planning tools must be executable regular files".into());
        }
        Ok(Self {
            path: path.to_owned(),
            canonical,
            device: m.dev(),
            inode: m.ino(),
            length: m.len(),
            modified: (m.mtime(), m.mtime_nsec()),
            changed: (m.ctime(), m.ctime_nsec()),
            mode: m.mode(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Tools {
    receipt: String,
    files: Vec<(FileIdentity, String)>,
}

fn pin(key: &str) -> Result<&'static str, String> {
    let prefix = format!("{key} = \"");
    include_str!("../../../../upstream/bluefe-core.pin")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
        .ok_or_else(|| format!("the source package has no {key} pin"))
}

impl Tools {
    fn inspect(cargo: &Path) -> Result<Self, String> {
        if crate::protocol::host_triple() != "x86_64-unknown-linux-gnu" {
            return Err("Cargo unit planning requires the declared Linux nightly Cargo identity".into());
        }
        let bin = cargo.parent().ok_or("Cargo planning tool has no package directory")?;
        let root = bin.parent().ok_or("Cargo planning tool has no package root")?;
        let receipt = std::fs::read_to_string(root.join("lib/lustr-selection-tools.txt"))
            .map_err(|e| format!("cannot read Cargo planning tool receipt: {e}"))?;
        let mut lines = receipt.lines();
        if lines.next() != Some("lustr-native-selection-toolchain-v2")
            || lines.next() != Some(format!("source={}", pin("source")?).as_str())
            || lines.next() != Some("host=x86_64-unknown-linux-gnu")
        {
            return Err("Cargo planning tool receipt does not match the declared package".into());
        }
        let mut expected = BTreeMap::new();
        for line in lines {
            let (hash, name) = line.split_once("  ").ok_or("malformed Cargo planning tool receipt")?;
            if hash.len() != 64
                || !hash.bytes().all(|c| c.is_ascii_hexdigit())
                || !["bin/cargo", "bin/lustr", "bin/rustc"].contains(&name)
                || expected.insert(name, hash).is_some()
            {
                return Err("malformed Cargo planning tool receipt".into());
            }
        }
        if expected.len() != 3
            || expected.get("bin/cargo") != Some(&pin("cargo_sha256")?)
            || expected.get("bin/lustr") != expected.get("bin/rustc")
            || bin.join("lustr").canonicalize().ok() != bin.join("rustc").canonicalize().ok()
        {
            return Err("Cargo unit planning requires the pinned Cargo and this package's native query entry".into());
        }
        let files = expected
            .into_iter()
            .map(|(name, hash)| Ok((FileIdentity::read(&root.join(name))?, hash.to_owned())))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self { receipt, files })
    }

    fn verified(cargo: &Path) -> Result<Self, String> {
        static VERIFIED: OnceLock<Mutex<Option<Tools>>> = OnceLock::new();
        let tools = Self::inspect(cargo)?;
        let mut previous = VERIFIED.get_or_init(|| Mutex::new(None)).lock().unwrap();
        if previous.as_ref() == Some(&tools) {
            return Ok(tools);
        }
        let mut seen = HashSet::new();
        for (file, expected) in &tools.files {
            if !seen.insert(file.canonical.clone()) {
                continue;
            }
            let output = crate::protocol::host_runtime::sha256_command()
                .arg(&file.path)
                .output()
                .map_err(|e| format!("cannot verify Cargo planning tool bytes: {e}"))?;
            let actual = std::str::from_utf8(&output.stdout).ok().and_then(|text| text.split_whitespace().next());
            if !output.status.success() || actual != Some(expected.as_str()) {
                return Err("Cargo planning tool bytes differ from the declared package".into());
            }
        }
        if Self::inspect(cargo)? != tools {
            return Err("Cargo planning tools changed during verification".into());
        }
        *previous = Some(tools.clone());
        Ok(tools)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    graph: u64,
    package: String,
    target: String,
    kind: Vec<String>,
    check: bool,
    test: bool,
    profile: ProfileFlags,
    tools: Tools,
}

fn graph_unchanged(graph: &CrateGraph, req: &GraphRequest) -> bool {
    let mut h = FxHasher::default();
    (graph_key(req), watch_stamp(&graph.watch)).hash(&mut h);
    h.finish() == graph.key
}

fn configuration_supported(req: &GraphRequest) -> Result<(), String> {
    // Cargo's unit graph does not expose [env], runner or injected compiler
    // arguments. Until those facts have a native representation, a new graph
    // cannot silently discard configuration. The existing single-unit path
    // keeps its previous admission and cost; this bounds the added service.
    let mut dirs = req.manifest_path.parent().into_iter().flat_map(Path::ancestors).map(|p| p.join(".cargo")).collect::<Vec<_>>();
    if let Some(home) = req.var("CARGO_HOME").map(PathBuf::from).or_else(|| req.var("HOME").map(|p| Path::new(&p).join(".cargo"))) {
        dirs.push(home);
    }
    for dir in dirs {
        for name in ["config", "config.toml"] {
            match std::fs::symlink_metadata(dir.join(name)) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                _ => return Err("native Cargo unit planning does not yet represent Cargo configuration-file execution settings".into()),
            }
        }
    }
    let current;
    let env = if let Some(env) = &req.env {
        env.as_slice()
    } else {
        current = cargo_env_of(&std::env::vars().collect::<Vec<_>>());
        current.as_slice()
    };
    for (name, value) in env {
        if !value.is_empty() && injected_compiler_setting(name) {
            return Err(format!("native Cargo unit planning does not yet represent {name}"));
        }
    }
    Ok(())
}

fn injected_compiler_setting(name: &str) -> bool {
    matches!(name, "RUSTFLAGS" | "CARGO_ENCODED_RUSTFLAGS" | "CARGO_BUILD_RUSTFLAGS")
        // Cargo sets TMPDIR for integration-test source and runtime inputs.
        || (name.starts_with("CARGO_TARGET_") && !matches!(name, "CARGO_TARGET_DIR" | "CARGO_TARGET_TMPDIR"))
        || name.starts_with("CARGO_PROFILE_")
        || name.starts_with("CARGO_UNSTABLE_")
        || matches!(name, "CARGO_BUILD_TARGET" | "CARGO_BUILD_RUSTC" | "CARGO_BUILD_RUSTC_WRAPPER" | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
}

pub(super) fn select(
    graph: &CrateGraph,
    req: &GraphRequest,
    package: &PackageInfo,
    target: &TargetInfo,
    test: bool,
    command: Cmd,
    profile: ProfileFlags,
) -> Result<UnitInfo, String> {
    let mut cmd = cargo_command(req.env.as_deref());
    let cargo = PathBuf::from(cmd.get_program());
    let tools = Tools::verified(&cargo)?;
    let key = Key {
        graph: graph.key,
        package: package.id_repr.clone(),
        target: target.name.clone(),
        kind: target.kind.clone(),
        check: command == Cmd::Check,
        test,
        profile,
        tools,
    };
    static CACHE: OnceLock<Mutex<HashMap<Key, Arc<UnitInfo>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(unit) = cache.lock().unwrap().get(&key).cloned() {
        // load_graph just validated this exact graph key and watch stamp.
        // A retained hit must not repeat that manifest/configuration scan.
        return Ok((*unit).clone());
    }
    configuration_supported(req)?;
    if !graph_unchanged(graph, req) {
        return Err("Cargo graph changed before unit planning".into());
    }
    if let Some(dir) = req.manifest_path.parent() {
        cmd.current_dir(dir);
    }
    let mode = if test {
        "test"
    } else if command == Cmd::Check {
        "check"
    } else {
        "build"
    };
    cmd.args([mode, "--unit-graph", "-Z", "unstable-options", "--manifest-path"])
        .arg(&req.manifest_path)
        .args(["--package", &package.id_repr]);
    if target.kind.iter().any(|kind| kind == "bin") {
        cmd.args(["--bin", &target.name]);
    } else if target.kind.iter().any(|kind| kind == "lib" || kind == "rlib") {
        cmd.arg("--lib");
    } else if test && target.kind == ["test"] {
        cmd.args(["--test", &target.name]);
    } else {
        return Err("native Cargo unit planning does not yet represent this root target kind".into());
    }
    if !req.features.is_empty() {
        cmd.args(["--features", &req.features.join(",")]);
    }
    if !req.default_features {
        cmd.arg("--no-default-features");
    }
    if req.all_features {
        cmd.arg("--all-features");
    }
    if !req.target_triple.is_empty() {
        cmd.args(["--target", &req.target_triple]);
    }
    cmd.args(req.cargo_net.args());
    let output = cmd.output().map_err(|e| format!("cannot obtain the Cargo unit graph: {e}"))?;
    if !output.status.success() {
        return Err(format!("Cargo unit planning failed: {}", String::from_utf8_lossy(&output.stderr).trim_end()));
    }
    if !graph_unchanged(graph, req) || Tools::inspect(&cargo)? != key.tools {
        return Err("Cargo planning inputs changed during the request".into());
    }
    let text = std::str::from_utf8(&output.stdout).map_err(|_| "Cargo unit graph is not UTF-8")?;
    let mut position = 0;
    let json = json_value(text.as_bytes(), &mut position)?;
    json_ws(text.as_bytes(), &mut position);
    if position != text.len() {
        return Err("Cargo unit graph has trailing output".into());
    }
    let mut h = FxHasher::default();
    // A host script's semantic plan can remain valid across a check and a
    // build of the same target. The plan cache still binds the actual mode;
    // the script record separately hashes all of BuildScriptUnit's facts.
    (key.graph, &key.tools).hash(&mut h);
    let unit = decode(&json, graph, req, package, target, mode, profile, h.finish())?;
    cache.lock().unwrap().insert(key, Arc::new(unit.clone()));
    Ok(unit)
}

fn field<'a>(value: &'a Json, name: &str) -> Result<&'a Json, String> {
    let Json::Obj(fields) = value else { return Err("Cargo unit graph expected an object".into()) };
    let mut found = fields.iter().filter(|(key, _)| key == name);
    let result = &found.next().ok_or_else(|| format!("Cargo unit graph is missing {name}"))?.1;
    if found.next().is_some() {
        return Err(format!("Cargo unit graph repeats {name}"));
    }
    Ok(result)
}

fn string(value: &Json) -> Result<&str, String> {
    value.as_str().ok_or_else(|| "Cargo unit graph expected a string".into())
}
fn array(value: &Json) -> Result<&[Json], String> {
    if let Json::Arr(values) = value { Ok(values) } else { Err("Cargo unit graph expected an array".into()) }
}
fn boolean(value: &Json) -> Result<bool, String> {
    if let Json::Bool(value) = value { Ok(*value) } else { Err("Cargo unit graph expected a boolean".into()) }
}
fn index(value: &Json) -> Result<usize, String> {
    if let Json::Num(n) = value
        && n.is_finite()
        && *n >= 0.0
        && n.fract() == 0.0
        && *n < usize::MAX as f64
    {
        return Ok(*n as usize);
    }
    Err("Cargo unit graph has an invalid index".into())
}
fn strings(value: &Json) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    for entry in array(value)? {
        let entry = string(entry)?;
        if entry.is_empty() || values.iter().any(|s| s == entry) {
            return Err("Cargo unit graph has empty or duplicated names".into());
        }
        values.push(entry.to_owned());
    }
    Ok(values)
}

#[derive(Clone)]
struct Unit<'a> {
    package: &'a PackageInfo,
    target: &'a TargetInfo,
    mode: String,
    host: bool,
    features: Vec<String>,
    profile: ProfileFlags,
    opt_level: String,
    debug: bool,
    dependencies: Vec<(usize, String)>,
}

fn decode(
    json: &Json,
    graph: &CrateGraph,
    req: &GraphRequest,
    package: &PackageInfo,
    target: &TargetInfo,
    mode: &str,
    profile: ProfileFlags,
    planning_key: u64,
) -> Result<UnitInfo, String> {
    if index(field(json, "version")?)? != 1 {
        return Err("unsupported Cargo unit graph version".into());
    }
    let roots = array(field(json, "roots")?)?;
    if roots.len() != 1 {
        return Err("Cargo unit graph must select exactly one root".into());
    }
    let root = index(&roots[0])?;
    let expected_profile = if mode == "test" { "test" } else { "dev" };
    let mut units = Vec::new();
    for row in array(field(json, "units")?)? {
        let id = string(field(row, "pkg_id")?)?;
        let package =
            graph.packages.iter().find(|pkg| pkg.id_repr == id).ok_or("Cargo unit names a package absent from the metadata graph")?;
        if package.source.is_some() {
            return Err("native Cargo source integration for registry or git dependencies is incomplete".into());
        }
        let declared = field(row, "target")?;
        let name = string(field(declared, "name")?)?;
        let kind = strings(field(declared, "kind")?)?;
        let source = Path::new(string(field(declared, "src_path")?)?);
        let edition = string(field(declared, "edition")?)?;
        let crate_types = strings(field(declared, "crate_types")?)?;
        let target = package
            .targets
            .iter()
            .find(|t| t.name == name && t.kind == kind && t.src_path == source && t.edition == edition && t.crate_types == crate_types)
            .ok_or("Cargo unit target differs from the metadata graph")?;
        if edition != "2024" || kind.iter().any(|kind| kind == "proc-macro") {
            return Err("native Cargo integration requires edition-2024 source units without proc macros".into());
        }
        if row.get("is_std").is_some_and(|value| !matches!(value, Json::Bool(false))) {
            return Err("native Cargo build-std integration is incomplete".into());
        }
        let host = match field(row, "platform")? {
            Json::Null => true,
            Json::Str(platform) if platform == &req.target_triple && platform == crate::protocol::host_triple() => false,
            _ => return Err("native Cargo unit platform differs from the supported host".into()),
        };
        let mode = string(field(row, "mode")?)?.to_owned();
        let features = strings(field(row, "features")?)?;
        if features.iter().any(|f| !package.features.contains(f)) {
            return Err("Cargo unit activates a feature absent from its package".into());
        }
        let p = field(row, "profile")?;
        if string(field(p, "name")?)? != expected_profile {
            return Err("native Cargo unit profile integration is incomplete".into());
        }
        let panic_abort = match string(field(p, "panic")?)? {
            "abort" => true,
            "unwind" => false,
            _ => return Err("unknown Cargo panic profile".into()),
        };
        let profile = ProfileFlags {
            panic_abort,
            overflow_checks: boolean(field(p, "overflow_checks")?)?,
            debug_assertions: boolean(field(p, "debug_assertions")?)?,
        };
        let opt_level = string(field(p, "opt_level")?)?.to_owned();
        if !["0", "1", "2", "3", "s", "z"].contains(&opt_level.as_str()) {
            return Err("unknown Cargo optimization level".into());
        }
        let debug = index(field(p, "debuginfo")?)? != 0;
        let mut dependencies = Vec::new();
        for dep in array(field(row, "dependencies")?)? {
            let to = index(field(dep, "index")?)?;
            let name = string(field(dep, "extern_crate_name")?)?.to_owned();
            if name.is_empty()
                || dependencies.iter().any(|(i, n)| *i == to || *n == name)
                || dep.get("noprelude").is_some_and(|v| !matches!(v, Json::Bool(false)))
            {
                return Err("Cargo unit dependency is duplicated or requires unsupported binding".into());
            }
            dependencies.push((to, name));
        }
        units.push(Unit { package, target, mode, host, features, profile, opt_level, debug, dependencies });
    }
    let selected = units.get(root).ok_or("Cargo unit root index is out of range")?;
    if selected.package.id_repr != package.id_repr
        || selected.target.name != target.name
        || selected.target.kind != target.kind
        || selected.target.src_path != target.src_path
        || selected.mode != mode
    {
        return Err("Cargo unit graph selected a different root request".into());
    }
    // The retained native worklist shares profile semantics across target
    // crates. Do not silently flatten package-specific profile overrides.
    if selected.profile != profile {
        return Err("native Cargo root profile differs from the compiler request".into());
    }
    let mut visited = HashSet::new();
    let mut active = HashSet::new();
    let mut bindings = HashMap::new();
    let (deps, build_script) =
        dependencies(root, &units, graph, req, selected.profile, profile, planning_key, &mut active, &mut visited, &mut bindings)?;
    if visited.len() != units.len() {
        return Err("Cargo unit graph contains unbound units".into());
    }
    Ok(UnitInfo {
        package: package.clone(),
        target: target.clone(),
        package_id: package_id_string(graph, package),
        features: selected.features.clone(),
        cargo_env: package_cargo_env(package, target),
        deps,
        build_script,
    })
}

#[allow(clippy::too_many_arguments)]
fn dependencies(
    index: usize,
    units: &[Unit<'_>],
    graph: &CrateGraph,
    req: &GraphRequest,
    planned_profile: ProfileFlags,
    actual_profile: ProfileFlags,
    planning_key: u64,
    active: &mut HashSet<usize>,
    visited: &mut HashSet<usize>,
    bindings: &mut HashMap<(String, String), usize>,
) -> Result<(Vec<DepUnit>, Option<BuildScriptUnit>), String> {
    if !active.insert(index) {
        return Err("Cargo unit graph contains a dependency cycle".into());
    }
    visited.insert(index);
    let unit = units.get(index).ok_or("Cargo dependency index is out of range")?;
    if unit.profile != planned_profile {
        return Err("native Cargo per-unit profile integration is incomplete".into());
    }
    let mut deps = Vec::new();
    let mut script = None;
    for (to, name) in &unit.dependencies {
        let dep = units.get(*to).ok_or("Cargo dependency index is out of range")?;
        if dep.mode == "run-custom-build" {
            if script.is_some()
                || dep.package.id_repr != unit.package.id_repr
                || dep.target.kind != ["custom-build"]
                || dep.features != unit.features
                || dep.dependencies.len() != 1
            {
                return Err("Cargo build-script run unit does not match its consumer".into());
            }
            let host_index = dep.dependencies[0].0;
            let host = units.get(host_index).ok_or("Cargo build-script compile index is out of range")?;
            if host.mode != "build"
                || !host.host
                || host.package.id_repr != dep.package.id_repr
                || host.target.src_path != dep.target.src_path
                || host.target.kind != ["custom-build"]
                || host.features != dep.features
                || !host.dependencies.is_empty()
            {
                return Err("native Cargo host build-dependency integration is incomplete".into());
            }
            visited.insert(*to);
            visited.insert(host_index);
            script = Some(BuildScriptUnit {
                script: host.target.src_path.clone(),
                manifest_dir: host.package.manifest_path.parent().ok_or("Cargo script has no manifest directory")?.to_owned(),
                workspace_root: graph.workspace_root.clone(),
                package_id: package_id_string(graph, host.package),
                crate_name: host.target.name.replace('-', "_"),
                edition: host.target.edition.clone(),
                features: host.features.clone(),
                host_profile: host.profile,
                target_triple: if req.target_triple.is_empty() { crate::protocol::host_triple().into() } else { req.target_triple.clone() },
                target_profile: actual_profile,
                target_features: unit.features.clone(),
                target_opt_level: dep.opt_level.clone(),
                target_debug: dep.debug,
                cargo_env: package_cargo_env(host.package, host.target),
                planning_key,
            });
        } else if unit.mode == "test" && unit.target.kind == ["test"]
            && dep.mode == "build" && dep.target.kind == ["bin"] && dep.package.id_repr == unit.package.id_repr
            && name.bytes().eq(dep.target.name.bytes().map(|byte| if byte == b'-' { b'_' } else { byte }))
        {
            // Cargo builds same-package executables for CARGO_BIN_EXE_*.
            // They participate in the plan but are not Rust extern crates.
            dependencies(*to, units, graph, req, planned_profile, actual_profile, planning_key, active, visited, bindings)?;
        } else {
            let expected_mode = if unit.mode == "test" { "build" } else { unit.mode.as_str() };
            if dep.mode != expected_mode || !dep.target.kind.iter().any(|kind| kind == "lib" || kind == "rlib") {
                return Err("native Cargo dependency requires an unimplemented unit kind".into());
            }
            let binding = (dep.package.id_repr.clone(), dep.target.name.clone());
            if bindings.insert(binding, *to).is_some_and(|previous| previous != *to) {
                return Err("native Cargo dependency has distinct unit contexts that cannot share one source crate".into());
            }
            let (nested, build_script) =
                dependencies(*to, units, graph, req, planned_profile, actual_profile, planning_key, active, visited, bindings)?;
            deps.push(DepUnit {
                extern_name: name.clone(),
                package_id: package_id_string(graph, dep.package),
                package_name: dep.package.name.clone(),
                version: dep.package.version.clone(),
                source: dep.package.source.clone(),
                lib_src: dep.target.src_path.clone(),
                lib_name: dep.target.name.clone(),
                edition: dep.target.edition.clone(),
                manifest_dir: dep.package.manifest_path.parent().ok_or("Cargo dependency has no manifest directory")?.to_owned(),
                cargo_env: package_cargo_env(dep.package, dep.target),
                build_cfgs: Vec::new(),
                build_check_cfgs: Vec::new(),
                features: dep.features.clone(),
                deps: nested,
                build_script,
            });
        }
    }
    active.remove(&index);
    Ok((deps, script))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(id: &str, name: &str, kind: &str) -> PackageInfo {
        PackageInfo {
            id_repr: id.into(),
            name: name.into(),
            version: "0.1.0".into(),
            edition: "2024".into(),
            manifest_path: PathBuf::from(format!("/plan/{id}/Cargo.toml")),
            authors: Vec::new(),
            description: None,
            homepage: None,
            license: None,
            license_file: None,
            repository: None,
            readme: None,
            rust_version: None,
            source: None,
            targets: vec![TargetInfo {
                kind: vec![kind.into()],
                crate_types: vec![kind.into()],
                name: name.into(),
                src_path: PathBuf::from(format!("/plan/{id}/main.rs")),
                edition: "2024".into(),
                test_flag: true,
            }],
            dependencies: Vec::new(),
            features: vec!["fast".into(), "unrelated".into()],
            resolved_features: vec!["unrelated".into()],
        }
    }

    fn unit(id: &str, name: &str, kind: &str, features: &str, deps: &str) -> String {
        format!(
            r#"{{"pkg_id":"{id}","target":{{"name":"{name}","kind":["{kind}"],"crate_types":["{kind}"],"src_path":"/plan/{id}/main.rs","edition":"2024"}},"platform":null,"mode":"build","features":[{features}],"profile":{{"name":"dev","opt_level":"0","debuginfo":2,"panic":"unwind","overflow_checks":true,"debug_assertions":true}},"dependencies":[{deps}]}}"#
        )
    }

    fn fixture() -> (CrateGraph, GraphRequest, Json) {
        let graph = CrateGraph {
            key: 3,
            watch: Vec::new(),
            stamp: 4,
            workspace_root: PathBuf::from("/plan"),
            packages: vec![package("root", "app", "bin"), package("dep", "util", "lib"), package("unused", "other", "lib")],
            workspace_members: vec!["root".into(), "dep".into(), "unused".into()],
        };
        let req = GraphRequest { manifest_path: graph.packages[0].manifest_path.clone(), default_features: true, ..Default::default() };
        let root = unit("root", "app", "bin", "", r#"{"index":1,"extern_crate_name":"renamed","noprelude":false}"#);
        let dep = unit("dep", "util", "lib", r#""fast""#, "");
        let json = parse_json(&format!(r#"{{"version":1,"roots":[0],"units":[{root},{dep}]}}"#)).unwrap();
        (graph, req, json)
    }

    fn change(object: &mut Json, name: &str, value: Json) {
        let Json::Obj(fields) = object else { panic!("object required") };
        *fields.iter_mut().find(|(key, _)| key == name).unwrap() = (name.to_owned(), value);
    }

    fn units(json: &mut Json) -> &mut Vec<Json> {
        let Json::Obj(fields) = json else { panic!("object required") };
        let Json::Arr(units) = &mut fields.iter_mut().find(|(key, _)| key == "units").unwrap().1 else { panic!("array required") };
        units
    }

    fn decoded(graph: &CrateGraph, req: &GraphRequest, json: &Json) -> Result<UnitInfo, String> {
        decode(json, graph, req, &graph.packages[0], &graph.packages[0].targets[0], "build", ProfileFlags::DEV_UNWIND, 42)
    }

    #[test]
    fn product_cargo_plan_uses_exact_features_edges_and_renames() {
        let (graph, req, json) = fixture();
        let unit = decoded(&graph, &req, &json).unwrap();
        assert!(unit.features.is_empty());
        assert_eq!(unit.deps.len(), 1);
        assert_eq!(unit.deps[0].extern_name, "renamed");
        assert_eq!(unit.deps[0].features, ["fast"]);
        assert_eq!(unit.deps[0].lib_name, "util");
        assert!(unit.deps[0].deps.is_empty());
        assert!(unit.build_script.is_none());
        assert!(
            !unit.deps[0].features.contains(&"unrelated".into()),
            "workspace metadata features must not replace selected unit features"
        );
    }

    #[test]
    fn product_cargo_plan_refuses_other_roots_missing_units_and_cycles() {
        let (graph, req, original) = fixture();
        for roots in [vec![], vec![Json::Num(0.0), Json::Num(1.0)], vec![Json::Num(1.0)], vec![Json::Num(99.0)]] {
            let mut json = original.clone();
            change(&mut json, "roots", Json::Arr(roots));
            assert!(decoded(&graph, &req, &json).is_err());
        }
        let mut json = original.clone();
        change(&mut units(&mut json)[0], "dependencies", parse_json(r#"[{"index":99,"extern_crate_name":"missing"}]"#).unwrap());
        assert!(decoded(&graph, &req, &json).is_err());
        let mut json = original.clone();
        change(&mut units(&mut json)[1], "dependencies", parse_json(r#"[{"index":1,"extern_crate_name":"cycle"}]"#).unwrap());
        assert!(decoded(&graph, &req, &json).unwrap_err().contains("cycle"));
        let mut json = original;
        change(&mut units(&mut json)[0], "dependencies", Json::Arr(Vec::new()));
        assert!(decoded(&graph, &req, &json).unwrap_err().contains("unbound"));
    }

    #[test]
    fn product_cargo_plan_refuses_changed_context_or_unsupported_features() {
        let (graph, req, original) = fixture();
        for (field_name, value) in [
            ("mode", Json::Str("check".into())),
            ("platform", Json::Str("aarch64-apple-darwin".into())),
            ("features", Json::Arr(vec![Json::Str("missing".into())])),
        ] {
            let mut json = original.clone();
            change(&mut units(&mut json)[0], field_name, value);
            assert!(decoded(&graph, &req, &json).is_err());
        }
        let mut json = original;
        let profile = field(&units(&mut json)[1], "profile").unwrap().clone();
        let mut profile = profile;
        change(&mut profile, "overflow_checks", Json::Bool(false));
        change(&mut units(&mut json)[1], "profile", profile);
        assert!(decoded(&graph, &req, &json).unwrap_err().contains("per-unit profile"));
    }

    #[test]
    fn product_cargo_plan_preserves_duplicate_root_and_binding_failures() {
        let (graph, req, original) = fixture();
        let mut json = original.clone();
        let Json::Obj(fields) = &mut json else { unreachable!() };
        fields.push(("roots".into(), Json::Arr(vec![Json::Num(0.0)])));
        assert!(decoded(&graph, &req, &json).unwrap_err().contains("repeats"));
        let mut json = original;
        change(
            &mut units(&mut json)[0],
            "dependencies",
            parse_json(r#"[{"index":1,"extern_crate_name":"renamed"},{"index":1,"extern_crate_name":"second"}]"#).unwrap(),
        );
        assert!(decoded(&graph, &req, &json).unwrap_err().contains("duplicated"));
    }

    #[test]
    fn product_cargo_plan_binds_host_script_context_without_running_it() {
        let (mut graph, req, mut json) = fixture();
        let mut script = graph.packages[0].targets[0].clone();
        script.name = "build-script-build".into();
        script.kind = vec!["custom-build".into()];
        script.crate_types = vec!["bin".into()];
        script.src_path = PathBuf::from("/plan/root/build.rs");
        graph.packages[0].targets.push(script);
        let mut host = parse_json(&unit("root", "build-script-build", "custom-build", "", "")).unwrap();
        let mut target = field(&host, "target").unwrap().clone();
        change(&mut target, "src_path", Json::Str("/plan/root/build.rs".into()));
        change(&mut target, "crate_types", parse_json(r#"["bin"]"#).unwrap());
        change(&mut host, "target", target);
        let mut profile = field(&host, "profile").unwrap().clone();
        change(&mut profile, "debuginfo", Json::Num(0.0));
        change(&mut host, "profile", profile);
        let mut run = host.clone();
        change(&mut run, "mode", Json::Str("run-custom-build".into()));
        change(&mut run, "dependencies", parse_json(r#"[{"index":2,"extern_crate_name":"build_script_build"}]"#).unwrap());
        let rows = units(&mut json);
        change(&mut rows[0], "dependencies", parse_json(r#"[{"index":1,"extern_crate_name":"build_script_build"}]"#).unwrap());
        rows[1] = run;
        rows.push(host);
        let selected = decoded(&graph, &req, &json).unwrap();
        assert!(selected.deps.is_empty());
        let script = selected.build_script.unwrap();
        assert_eq!(script.script, PathBuf::from("/plan/root/build.rs"));
        assert_eq!(script.manifest_dir, PathBuf::from("/plan/root"));
        assert_eq!(script.workspace_root, graph.workspace_root);
        assert_eq!(script.crate_name, "build_script_build");
        assert_eq!(script.host_profile, ProfileFlags::DEV_UNWIND);
        assert_eq!(script.target_profile, ProfileFlags::DEV_UNWIND);
        assert_eq!(script.target_triple, crate::protocol::host_triple());
        assert_eq!(script.planning_key, 42);
        assert!(!script.script.exists(), "unit decoding must not run or read a script body");

        let mut changed = json.clone();
        change(&mut units(&mut changed)[2], "dependencies", parse_json(r#"[{"index":0,"extern_crate_name":"host_dep"}]"#).unwrap());
        assert!(decoded(&graph, &req, &changed).unwrap_err().contains("host build-dependency"));
        let mut changed = json.clone();
        change(&mut units(&mut changed)[2], "features", parse_json(r#"["fast"]"#).unwrap());
        assert!(decoded(&graph, &req, &changed).is_err());
        let mut changed = json;
        change(&mut units(&mut changed)[1], "features", parse_json(r#"["fast"]"#).unwrap());
        assert!(decoded(&graph, &req, &changed).is_err());
    }

    #[test]
    fn product_cargo_plan_refuses_unrepresented_compiler_settings() {
        for name in [
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_BUILD_RUSTFLAGS",
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS",
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
            "CARGO_PROFILE_DEV_PANIC",
            "CARGO_UNSTABLE_BUILD_STD",
            "CARGO_BUILD_TARGET",
            "CARGO_BUILD_RUSTC",
            "CARGO_BUILD_RUSTC_WRAPPER",
            "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        ] {
            assert!(injected_compiler_setting(name), "unrepresented compiler setting {name}");
        }
        for name in ["CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "CARGO_NET_OFFLINE", "CARGO_TERM_COLOR"] {
            assert!(!injected_compiler_setting(name), "noncompiler setting {name}");
        }
    }

    #[test]
    fn product_cargo_plan_binds_unique_source_and_explicit_target() {
        let root = std::env::temp_dir().join(format!("lustr-source-binding-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let source = root.join("main.rs");
        let other = root.join("other.rs");
        std::fs::write(&source, "fn main() {}\n").unwrap();
        std::fs::write(&other, "compile_error!(\"must not read\");\n").unwrap();
        let (mut graph, mut req, _) = fixture();
        req.manifest_path = root.join("Cargo.toml");
        let package = &mut graph.packages[0];
        package.manifest_path = req.manifest_path.clone();
        package.targets[0].src_path = source.clone();
        let mut second = package.targets[0].clone();
        second.name = "second".into();
        second.src_path = other.clone();
        package.targets.push(second);
        assert_eq!(select_source_target(&graph, &req, "bin", "inferred", &source, false).unwrap().1.name, "app");
        assert!(select_source_target(&graph, &req, "bin", "second", &source, true).is_err());
        assert!(select_source_target(&graph, &req, "bin", "missing", &source, true).is_err());
        assert!(select_source_target(&graph, &req, "lib", "app", &source, true).is_err());
        std::fs::remove_file(&other).unwrap();
        std::os::unix::fs::symlink(&source, &other).unwrap();
        assert!(select_source_target(&graph, &req, "bin", "app", &source, false).is_err(), "unqualified aliases are ambiguous");
        assert_eq!(select_source_target(&graph, &req, "bin", "second", &source, true).unwrap().1.name, "second");
        std::fs::remove_file(&source).unwrap();
        assert!(select_source_target(&graph, &req, "bin", "app", &source, true).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn product_cargo_plan_graph_key_distinguishes_missing_and_invalid_config() {
        let root = std::env::temp_dir().join(format!("lustr-config-kind-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".cargo")).unwrap();
        let req = GraphRequest { manifest_path: root.join("Cargo.toml"), env: Some(Vec::new()), ..Default::default() };
        let absent = graph_key(&req);
        let config = root.join(".cargo/config.toml");
        std::fs::create_dir(&config).unwrap();
        assert_ne!(graph_key(&req), absent);
        assert!(configuration_supported(&req).is_err());
        std::fs::remove_dir(&config).unwrap();
        std::fs::write(&config, "").unwrap();
        assert_ne!(graph_key(&req), absent);
        assert!(configuration_supported(&req).is_err());
        std::fs::remove_file(&config).unwrap();
        assert_eq!(graph_key(&req), absent);
        let mut flagged = req;
        flagged.env = Some(vec![("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS".into(), "--cfg custom".into())]);
        assert!(configuration_supported(&flagged).unwrap_err().contains("RUSTFLAGS"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
