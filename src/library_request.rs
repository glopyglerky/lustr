//! Parse the actual Rust compiler request used by the pinned Cargo leaf unit.
//! Unsupported options remain refusals; this adapter never reads Rust source.
use crate::{Native, library_contract::{ExecutableExtern, ExternMetadata, LibraryKey, LibraryOptions, SDK_RECEIPT, TARGET}};
use std::{collections::{BTreeMap,BTreeSet}, ffi::OsString, path::PathBuf};

pub(crate) fn parse(args: &[OsString]) -> Option<Native> {
    if !crate::library_contract::supported_host() {return None;}
    let mut source = None;
    let mut name = None;
    let mut kind = None;
    let mut edition = None;
    let mut target = None;
    let mut out_dir = None;
    let mut emits = None;
    let mut error_format = None;
    let mut json = None;
    let mut check_cfg = Vec::new();
    let mut search_paths = Vec::new();
    let mut externs = Vec::new();
    let mut codegen = BTreeMap::new();
    let mut options = LibraryOptions::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].to_str()?;
        let mut value = |flag: &str| -> Option<&str> {
            if arg == flag { i += 1; args.get(i)?.to_str() }
            else { arg.strip_prefix(flag)?.strip_prefix('=') }
        };
        let mut unique = |slot: &mut Option<String>, flag: &str| -> Option<()> {
            if slot.is_some() { return None; }
            *slot = Some(value(flag)?.to_owned()); Some(())
        };
        if arg == "--crate-name" || arg.starts_with("--crate-name=") { unique(&mut name,"--crate-name")?; }
        else if arg == "--crate-type" || arg.starts_with("--crate-type=") { unique(&mut kind,"--crate-type")?; }
        else if arg == "--edition" || arg.starts_with("--edition=") { unique(&mut edition,"--edition")?; }
        else if arg == "--target" || arg.starts_with("--target=") { unique(&mut target,"--target")?; }
        else if arg == "--out-dir" || arg.starts_with("--out-dir=") { unique(&mut out_dir,"--out-dir")?; }
        else if arg == "--emit" || arg.starts_with("--emit=") { unique(&mut emits,"--emit")?; }
        else if arg == "--error-format" || arg.starts_with("--error-format=") { unique(&mut error_format,"--error-format")?; }
        else if arg == "--json" || arg.starts_with("--json=") { unique(&mut json,"--json")?; }
        else if arg == "--check-cfg" || arg.starts_with("--check-cfg=") { check_cfg.push(value("--check-cfg")?.to_owned()); }
        else if arg == "--extern" || arg.starts_with("--extern=") {
            externs.push(value("--extern")?);
        }
        else if arg == "-L" || arg.starts_with("-L") {
            let value = if arg == "-L" { i += 1; args.get(i)?.to_str()? } else { &arg[2..] };
            search_paths.push(std::path::absolute(value.strip_prefix("dependency=")?).ok()?);
        } else if arg == "-C" || arg.starts_with("-C") {
            let value = if arg == "-C" { i += 1; args.get(i)?.to_str()? } else { &arg[2..] };
            let (key,value) = value.split_once('=')?;
            if codegen.insert(key.to_owned(),value.to_owned()).is_some() { return None; }
        } else if arg == "-Z" || arg.starts_with("-Z") {
            let value = if arg == "-Z" { i += 1; args.get(i)?.to_str()? } else { &arg[2..] };
            if options.embed_metadata.is_some() { return None; }
            options.embed_metadata = Some(match value.strip_prefix("embed-metadata=")? {
                "y" | "yes" | "on" | "true" => true,
                "n" | "no" | "off" | "false" => false,
                _ => return None,
            });
        } else if arg == "--force-warn" || arg.starts_with("--force-warn=") {
            if options.force_warn_unused_crate_dependencies || value("--force-warn")? != "unused_crate_dependencies" { return None; }
            options.force_warn_unused_crate_dependencies = true;
        } else if !arg.starts_with('-') && arg.ends_with(".rs") && source.is_none() { source = Some(arg.to_owned()); }
        else { return None; }
        i += 1;
    }
    let binary = kind.as_deref() == Some("bin");
    if !matches!(kind.as_deref(), Some("bin" | "lib" | "rlib")) || edition.as_deref() != Some("2024")
        || target.as_deref().is_some_and(|v| v != TARGET) { return None; }
    let emit = emits?;
    let fields = emit.split(',').collect::<Vec<_>>();
    let check = match (fields.len(), fields.into_iter().collect::<BTreeSet<_>>()) {
        (2, fields) if binary && fields == BTreeSet::from(["dep-info", "link"]) => false,
        (2, fields) if fields == BTreeSet::from(["dep-info", "metadata"]) => true,
        (3, fields) if !binary && fields == BTreeSet::from(["dep-info", "metadata", "link"]) => false,
        _ => return None,
    };
    if (check || binary) && options.embed_metadata.is_some() { return None; }
    let cargo_json = match error_format.as_deref() {
        None | Some("human") if json.is_none() => false,
        Some("json") => {
            let fields = json?.split(',').map(str::to_owned).collect::<Vec<_>>();
            if fields.len() != 4 || fields.iter().map(String::as_str).collect::<BTreeSet<_>>()
                != BTreeSet::from(["diagnostic-rendered-ansi","artifacts","future-incompat","unused-externs-silent"]) { return None; }
            true
        },
        _ => return None,
    };
    let metadata = codegen.remove("metadata")?;
    let extra_filename = codegen.remove("extra-filename").unwrap_or_default();
    let panic = match codegen.remove("panic").as_deref() {
        Some("abort") => "panic=abort",
        None | Some("unwind") => "panic=unwind",
        _ => return None,
    };
    // Rust defaults to embedded LLVM bitcode for an rlib; this native scalar
    // artifact contract requires the caller's explicit no-bitcode selection.
    if !matches!(codegen.remove("embed-bitcode").as_deref(),Some("no"|"off"|"false"|"n")) {return None;}
    for (key,value) in &codegen {
        let represented = match key.as_str() {
            "opt-level" => value == "0",
            "debuginfo" => value == "0" || check && matches!(value.as_str(), "1" | "2"),
            // Cargo provides an optional on-disk incremental cache location.
            // Reuse belongs to the native session rather than Rust's on-disk cache.
            "incremental" => (check || panic == "panic=unwind") && std::path::Path::new(value).is_absolute(),
            "overflow-checks" | "debug-assertions" => matches!(value.as_str(),"yes"|"no"|"on"|"off"|"true"|"false"|"y"|"n"),
            // This artifact path emits no debug sections (nonzero debuginfo
            // is refused above). Cargo's standard debug stripping has no
            // additional work for this request.
            "strip" => matches!(value.as_str(), "none" | "debuginfo"),
            _ => false,
        };
        if !represented { return None; }
    }
    let source = source?;
    let source_path = std::path::absolute(&source).ok()?;
    let (external, executable_dependencies) = if binary {
        (None, Some(executable_externs(externs, check)?))
    } else {
        match externs.as_slice() {
            [] => (None, None),
            [value] => (Some(ExternMetadata::parse(value)?), None),
            _ => return None,
        }
    };
    let context = LibraryKey { crate_name:name?, metadata, extra_filename, source_path,
        working_directory: std::env::current_dir().ok()?, check_cfg, search_paths, external, executable_dependencies,
        sdk_receipt:SDK_RECEIPT.into() };
    if !context.valid() || !check && !context.supports_link_profile(panic == "panic=abort") { return None; }
    if std::env::var("CARGO_CRATE_NAME").is_ok_and(|v|v != context.crate_name) { return None; }
    let out_dir = std::path::absolute(out_dir?).ok()?;
    let output = out_dir.join(if check { context.metadata_name() } else if binary { context.binary_name() } else { context.archive_name() });
    let mut native = vec![(if check { "check" } else { "build" }).into(),source,"-o".into(),output.to_str()?.into(),
        "--target".into(),TARGET.into(),"-C".into(),panic.into()];
    if binary {
        let target = std::env::var("CARGO_BIN_NAME").unwrap_or_else(|_| context.crate_name.clone());
        if !context.matches_target_name(&target) { return None; }
        native.extend(["--bin".into(), target]);
    }
    else { native.push("--lib".into()); }
    // At O0 Rust enables debug assertions by default. Omitted overflow checks
    // inherit the effective debug assertion setting (Session::overflow_checks),
    // including an explicit override. The actual Rust request owns both values;
    // the native Cargo profile must not substitute its defaults afterwards.
    let debug = codegen.get("debug-assertions").map_or("on", String::as_str);
    let overflow = codegen.get("overflow-checks").map_or(debug, String::as_str);
    for (key, value) in [("overflow-checks", overflow), ("debug-assertions", debug)] {
        native.extend(["-C".into(),format!("{key}={value}")]);
    }
    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        let manifest = PathBuf::from(manifest_dir).join("Cargo.toml");
        native.extend(["--manifest-path".into(),std::path::absolute(manifest).ok()?.to_str()?.into()]);
    }
    if cargo_json { native.push("--json".into()); }
    if let Some(embed) = options.embed_metadata {
        native.extend(["--lustr-library-embed-metadata".into(),if embed {"yes"} else {"no"}.into()]);
    }
    if options.force_warn_unused_crate_dependencies {
        native.push("--lustr-library-force-warn-unused-crate-dependencies".into());
    }
    for (flag,value) in [("--lustr-library-name",context.crate_name.as_str()),
        ("--lustr-library-metadata",context.metadata.as_str()),("--lustr-library-extra-filename",context.extra_filename.as_str()),
        ("--lustr-library-cwd",context.working_directory.to_str()?)] {
        native.extend([flag.into(),value.into()]);
    }
    for cfg in context.check_cfg { native.extend(["--lustr-library-check-cfg".into(),cfg]); }
    for path in context.search_paths { native.extend(["--lustr-library-search".into(),path.to_str()?.into()]); }
    if let Some(dependency) = context.external {
        native.extend(["--lustr-library-extern-metadata".into(), format!("{}={}", dependency.alias, dependency.artifact.to_str()?)]);
    }
    if let Some(dependencies) = context.executable_dependencies {
        native.push("--lustr-executable-artifacts".into());
        for input in dependencies {
            let archive = match &input.archive { Some(path) => path.to_str()?, None => "" };
            native.extend(["--lustr-executable-extern".into(),
                format!("{}={}", input.metadata.alias, input.metadata.artifact.to_str()?),
                archive.into()]);
        }
    }
    Some(Native {args:native,output})
}

fn executable_externs(arguments: Vec<&str>, check: bool) -> Option<Vec<ExecutableExtern>> {
    let mut pairs = BTreeMap::<&str, (Option<ExternMetadata>, Option<PathBuf>)>::new();
    for argument in arguments {
        let (alias, path) = argument.split_once('=')?;
        let (metadata, archive) = pairs.entry(alias).or_default();
        match PathBuf::from(path).extension()?.to_str()? {
            "rmeta" if metadata.is_none() => *metadata = Some(ExternMetadata::parse(argument)?),
            "rlib" if !check && archive.is_none() => *archive = Some(std::path::absolute(path).ok()?),
            _ => return None,
        }
    }
    pairs.into_values().map(|(metadata, archive)| {
        (archive.is_none() == check).then_some(ExecutableExtern { metadata: metadata?, archive })
    }).collect()
}
