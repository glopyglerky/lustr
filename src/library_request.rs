//! Parse the actual Rust compiler request used by the pinned Cargo leaf unit.
//! Unsupported options remain refusals; this adapter never reads Rust source.
use crate::{
    Native,
    library_contract::{
        DebugInfo, ExecutableExtern, ExternMetadata, LibraryKey, LibraryOptions,
        CompilerCfg, OptLevel, SDK_RECEIPT, SplitDebug, Strip, TARGET, classify_cfg,
    },
};
use std::{collections::{BTreeMap,BTreeSet}, ffi::OsString, path::PathBuf};

pub(crate) fn parse(args: &[OsString]) -> Option<Native> {
    if !crate::library_contract::supported_host() {return None;}
    let mut source = None;
    let mut name = None;
    let mut kind = None;
    let mut test = false;
    let mut custom_test = false;
    let mut allow_dead_code_pub_in_binary = false;
    let mut edition = None;
    let mut target = None;
    let mut out_dir = None;
    let mut output_file = None;
    let mut no_color = false;
    let mut emits = None;
    let mut error_format = None;
    let mut json = None;
    let mut cfg = BTreeSet::new();
    let mut check_cfg = Vec::new();
    let mut features = BTreeSet::new();
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
        else if arg == "--test" {
            if test { return None; }
            test = true;
        }
        else if arg == "--cfg" || arg.starts_with("--cfg=") {
            match classify_cfg(value("--cfg")?)? {
                CompilerCfg::Test => custom_test = true,
                CompilerCfg::Feature(name) => { features.insert(name); }
                CompilerCfg::Custom(spec) => { cfg.insert(spec.to_owned()); }
            }
        }
        else if arg == "--edition" || arg.starts_with("--edition=") { unique(&mut edition,"--edition")?; }
        else if arg == "--target" || arg.starts_with("--target=") { unique(&mut target,"--target")?; }
        else if arg == "--out-dir" || arg.starts_with("--out-dir=") { unique(&mut out_dir,"--out-dir")?; }
        else if arg == "-o" && output_file.is_none() { i += 1; output_file = Some(args.get(i)?.clone()); }
        else if arg == "--color" || arg.starts_with("--color=") {
            if no_color || value("--color")? != "never" { return None; }
            no_color = true;
        }
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
        } else if arg == "-g" {
            options.debug.info = DebugInfo::Full;
        } else if arg == "-C" || arg.starts_with("-C") {
            let value = if arg == "-C" { i += 1; args.get(i)?.to_str()? } else { &arg[2..] };
            let (key,value) = value.split_once('=')?;
            match key {
                // Rust resolves -g and the explicit level in argument order.
                "debuginfo" => options.debug.info = DebugInfo::parse(value)?,
                "split-debuginfo" => options.debug.split = SplitDebug::parse(value)?,
                "strip" => options.debug.strip = Strip::parse(value)?,
                _ if codegen.insert(key.to_owned(),value.to_owned()).is_some() => return None,
                _ => {}
            }
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
        } else if arg == "-Adead_code_pub_in_binary" {
            // Cargo gives library test executables the pinned Rust lint's
            // default Allow level. No dead-code lint is enabled by this request.
            allow_dead_code_pub_in_binary = true;
        } else if (arg == "-" || !arg.starts_with('-') && arg.ends_with(".rs")) && source.is_none() { source = Some(arg.to_owned()); }
        else { return None; }
        i += 1;
    }
    let custom_test = custom_test && !test;
    let test_target = test || custom_test;
    let standalone = !test_target && !externs.is_empty() && matches!(kind.as_deref(), None | Some("bin"))
        && matches!(emits.as_deref(), None | Some("link"));
    let binary = standalone || test_target || kind.as_deref() == Some("bin");
    let represented_kind = if standalone || test_target { matches!(kind.as_deref(), None | Some("bin")) }
        else { matches!(kind.as_deref(), Some("bin" | "lib" | "rlib")) };
    if !represented_kind || (allow_dead_code_pub_in_binary && !test_target) || edition.as_deref() != Some("2024")
        || target.as_deref().is_some_and(|v| v != TARGET) { return None; }
    let emit = if standalone { "link".into() } else { emits? };
    let fields = emit.split(',').collect::<Vec<_>>();
    let check = match (fields.len(), fields.into_iter().collect::<BTreeSet<_>>()) {
        (1, _) if standalone => false,
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
    let metadata = if standalone { codegen.remove("metadata").unwrap_or_default() } else { codegen.remove("metadata")? };
    let extra_filename = codegen.remove("extra-filename").unwrap_or_default();
    let panic = match codegen.remove("panic").as_deref() {
        Some("abort") => "panic=abort",
        None | Some("unwind") => "panic=unwind",
        _ => return None,
    };
    // Rust defaults to embedded LLVM bitcode for an rlib; this native scalar
    // artifact contract requires the caller's explicit no-bitcode selection.
    match codegen.remove("embed-bitcode").as_deref() {
        Some("no"|"off"|"false"|"n") => {}
        None if standalone => {}
        _ => return None,
    }
    let optimization = match codegen.remove("opt-level") {
        Some(value) => OptLevel::parse(&value)?,
        None => OptLevel::None,
    };
    for (key,value) in &codegen {
        let represented = match key.as_str() {
            // Cargo provides an optional on-disk incremental cache location.
            // Reuse belongs to the native session rather than Rust's on-disk cache.
            "incremental" => (check || panic == "panic=unwind") && std::path::Path::new(value).is_absolute(),
            "overflow-checks" | "debug-assertions" => matches!(value.as_str(),"yes"|"no"|"on"|"off"|"true"|"false"|"y"|"n"),
            _ => false,
        };
        if !represented { return None; }
    }
    let source = source?;
    let stdin = source == "-";
    if stdin && !standalone { return None; }
    let source = if stdin { "<anon>".to_owned() } else { source };
    let source_path = std::path::absolute(&source).ok()?;
    let name = match name {
        Some(name) => name,
        None if stdin => "rust_out".into(),
        None if standalone => source_path.file_stem()?.to_str()?.replace('-', "_"),
        None => return None,
    };
    let (external, executable_dependencies) = if binary {
        (None, Some(executable_externs(externs, check)?))
    } else {
        match externs.as_slice() {
            [] => (None, None),
            [value] => (Some(ExternMetadata::parse(value)?), None),
            _ => return None,
        }
    };
    let context = LibraryKey { crate_name:name, metadata, extra_filename, source_path,
        working_directory: std::env::current_dir().ok()?, cfg: cfg.into_iter().collect(), check_cfg, search_paths, external, executable_dependencies,
        standalone, optimization, custom_test, sdk_receipt:SDK_RECEIPT.into() };
    if test && panic != "panic=unwind" { return None; }
    if !context.valid() { return None; }
    if !standalone && std::env::var("CARGO_CRATE_NAME").is_ok_and(|v|v != context.crate_name) { return None; }
    let output = if standalone {
        match (output_file, out_dir) {
            (Some(path), None) if path != "-" => std::path::absolute(path).ok()?,
            (None, dir) => std::path::absolute(dir.unwrap_or_else(|| ".".into())).ok()?.join(context.binary_name()),
            (Some(_), _) => return None,
        }
    } else {
        if output_file.is_some() { return None; }
        let out_dir = std::path::absolute(out_dir?).ok()?;
        out_dir.join(if check { context.metadata_name() } else if binary { context.binary_name() } else { context.archive_name() })
    };
    if !context.valid_output(&output, check, options) { return None; }
    let mut native = vec![(if check { "check" } else { "build" }).into(),source,"-o".into(),output.to_str()?.into(),
        "--target".into(),TARGET.into(),"-C".into(),panic.into()];
    if standalone {
        native.push("--lustr-standalone-executable".into());
    } else if binary {
        let target = std::env::var("CARGO_BIN_NAME").unwrap_or_else(|_| context.crate_name.clone());
        if !context.matches_target_name(&target) { return None; }
        native.extend(["--bin".into(), target]);
    }
    else { native.push("--lib".into()); }
    if test { native.push("--lustr-executable-test".into()); }
    if custom_test { native.push("--lustr-custom-test".into()); }
    // Rust enables debug assertions by default only at O0. Omitted overflow checks
    // inherit the effective debug assertion setting (Session::overflow_checks),
    // including an explicit override. The actual Rust request owns both values;
    // the native Cargo profile must not substitute its defaults afterwards.
    let debug = codegen.get("debug-assertions")
        .map_or(if optimization == OptLevel::None { "on" } else { "off" }, String::as_str);
    let overflow = codegen.get("overflow-checks").map_or(debug, String::as_str);
    for (key, value) in [("overflow-checks", overflow), ("debug-assertions", debug)] {
        native.extend(["-C".into(),format!("{key}={value}")]);
    }
    let cargo_manifest = std::env::var_os("CARGO_MANIFEST_PATH").map(PathBuf::from);
    let resolved_by_cargo = cargo_manifest.is_some();
    let manifest = cargo_manifest.or_else(|| {
        std::env::var_os("CARGO_MANIFEST_DIR").map(|directory| PathBuf::from(directory).join("Cargo.toml"))
    });
    if standalone {
        native.push("--no-default-features".into());
        if !features.is_empty() {
            native.extend(["--features".into(), features.into_iter().collect::<Vec<_>>().join(",")]);
        }
    } else if let Some(manifest) = manifest {
        native.extend(["--manifest-path".into(),std::path::absolute(manifest).ok()?.to_str()?.into()]);
        // The outer Cargo invocation has already resolved this package.
        // Internal graph queries must neither fetch inputs nor change its lockfile.
        if resolved_by_cargo {
            native.push("--frozen".into());
        }
        // Cargo already resolved the active features, including the literal
        // "default" feature when enabled. Keep that set through planning
        // and prevent Lust's developer config from substituting another set.
        native.push("--no-default-features".into());
        if !features.is_empty() {
            native.extend(["--features".into(), features.into_iter().collect::<Vec<_>>().join(",")]);
        }
    } else if !features.is_empty() {
        return None;
    }
    if cargo_json { native.push("--json".into()); }
    if let Some(embed) = options.embed_metadata {
        native.extend(["--lustr-library-embed-metadata".into(),if embed {"yes"} else {"no"}.into()]);
    }
    if options.force_warn_unused_crate_dependencies {
        native.push("--lustr-library-force-warn-unused-crate-dependencies".into());
    }
    if options.debug != Default::default() {
        native.extend(["--lustr-library-debug-options".into(), String::from_utf8(options.debug.encode().to_vec()).ok()?]);
    }
    if optimization != OptLevel::None {
        native.extend(["--lustr-opt-level".into(), optimization.value().into()]);
    }
    for (flag,value) in [("--lustr-library-name",context.crate_name.as_str()),
        ("--lustr-library-metadata",context.metadata.as_str()),("--lustr-library-extra-filename",context.extra_filename.as_str()),
        ("--lustr-library-cwd",context.working_directory.to_str()?)] {
        native.extend([flag.into(),value.into()]);
    }
    for cfg in context.cfg { native.extend(["--lustr-library-cfg".into(),cfg]); }
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
    Some(Native {args:native,output,stdin})
}

fn executable_externs(arguments: Vec<&str>, check: bool) -> Option<Vec<ExecutableExtern>> {
    let mut pairs = BTreeMap::<&str, (Option<ExternMetadata>, Option<PathBuf>)>::new();
    for argument in arguments {
        let (alias, path) = argument.split_once('=')?;
        let (metadata, archive) = pairs.entry(alias).or_default();
        match PathBuf::from(path).extension()?.to_str()? {
            "rmeta" if metadata.is_none() => *metadata = Some(ExternMetadata::parse(argument)?),
            "rlib" if archive.is_none() => *archive = Some(std::path::absolute(path).ok()?),
            _ => return None,
        }
    }
    pairs.into_iter().map(|(alias, (metadata, archive))| {
        if let Some(metadata) = metadata {
            (archive.is_none() == check).then_some(ExecutableExtern { metadata, archive })
        } else {
            let archive = archive?;
            let metadata = ExternMetadata { alias: alias.into(), artifact: archive.clone() };
            Some(ExecutableExtern { metadata, archive: (!check).then_some(archive) })
        }
    }).collect()
}

#[cfg(all(test, target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
mod tests {
    use super::*;

    fn request(emit: &str, extra: &[&str]) -> Option<Native> {
        let mut args = vec![
            "--crate-name", "peer", "--crate-type", "bin", "--edition", "2024",
            "--out-dir", "/tmp/lustr-levels", "-Cmetadata=peer", "-Cembed-bitcode=no", "--emit", emit, "main.rs",
        ];
        args.extend(extra);
        parse(&args.into_iter().map(OsString::from).collect::<Vec<_>>())
    }

    fn sent_options(request: Native) -> [u8; 3] {
        request.args.windows(2)
            .find(|pair| pair[0] == "--lustr-library-debug-options")
            .map_or(*b"000", |pair| pair[1].as_bytes().try_into().unwrap())
    }

    #[test]
    fn metadata_only_preserves_pinned_debug_levels_and_last_option_precedence() {
        let check = "dep-info,metadata";
        for (value, expected) in [
            ("none", b"000"), ("0", b"000"), ("line-directives-only", b"100"),
            ("line-tables-only", b"200"), ("1", b"300"), ("limited", b"300"),
            ("2", b"400"), ("full", b"400"),
        ] {
            let arg = format!("-Cdebuginfo={value}");
            assert_eq!(&sent_options(request(check, &[&arg]).unwrap()), expected, "{value}");
        }
        assert_eq!(sent_options(request(check, &["-g", "-C", "debuginfo=1"]).unwrap()), *b"300");
        assert_eq!(sent_options(request(check, &["-Cdebuginfo=0", "-g"]).unwrap()), *b"400");
        let split = ["-Csplit-debuginfo=packed", "-Csplit-debuginfo=unpacked", "-Cstrip=debuginfo", "-Cstrip=symbols"];
        assert_eq!(sent_options(request(check, &split).unwrap()), *b"022");
        for bad in ["-Cdebuginfo=3", "-Csplit-debuginfo=separate", "-Cstrip=all"] {
            assert!(request(check, &[bad, "-g"]).is_none(), "malformed flag {bad}");
        }
    }

    #[test]
    fn native_full_executable_debug_and_unimplemented_output_modes() {
        let link = "dep-info,link";
        assert!(request(link, &[]).is_some());
        let no_debug = ["-g", "-Cdebuginfo=0", "-Csplit-debuginfo=off", "-Cstrip=debuginfo"];
        assert_eq!(sent_options(request(link, &no_debug).unwrap()), *b"001");
        assert_eq!(sent_options(request(link, &["-g"]).unwrap()), *b"400");
        assert_eq!(sent_options(request(link, &["-Cdebuginfo=2"]).unwrap()), *b"400");
        for unsupported in ["-Cdebuginfo=line-tables-only", "-Cdebuginfo=1", "-Csplit-debuginfo=packed", "-Cstrip=symbols"] {
            assert!(request(link, &[unsupported]).is_none(), "linked mode {unsupported}");
        }
    }
}
