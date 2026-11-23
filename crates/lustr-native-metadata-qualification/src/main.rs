//! One native consumer preparation, source-owned metadata and genuine archive.
//! This executable is a research qualifier, not the installed compiler route.
#![allow(dead_code, unused_imports, unexpected_cfgs)]
include!("../../lustr-native-core/src/lib.rs");
#[path="../../lustr-native-core/src/metadata_archive.rs"]
mod archive;
#[path="../../lustr-native-core/src/library_artifact.rs"]
mod library_artifact;
mod retained;

#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

use lustr_metadata_encoder as metadata;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

const SDK_RECEIPT: &str = "4e400b263bf9923616dea44ead2d77213703177d3bfaf6becd6a19a4c5a657a7";
const CFG_VERSION: &str = "1.102.0-nightly (5b7b37aef 2026-11-08)";

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}
fn put(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("write {}: {error}", path.display()))
}
fn native_id(id: def::DiskDefId) -> metadata::NativeId {
    metadata::NativeId { krate: id.krate.0, path: id.hash.0 }
}
fn span(value: diag::Span) -> metadata::Span {
    metadata::Span { lo: value.lo, hi: value.hi }
}
fn invocation(source: &Path, output: &Path) -> Result<Vec<String>, String> {
    let source = source.to_str().ok_or("native metadata source path is not Unicode")?;
    let output = output.to_str().ok_or("native metadata output path is not Unicode")?;
    Ok([
        source,
        "--crate-name",
        "consumer",
        "--edition=2024",
        "--crate-type=lib",
        "--emit=obj",
        "--target=x86_64-unknown-linux-gnu",
        "-Cmetadata=native_consumer_v1",
        "-Cpanic=abort",
        "-Copt-level=0",
        "-Cdebuginfo=0",
        "-Coverflow-checks=on",
        "-Cdebug-assertions=on",
        "-o",
        output,
    ]
    .iter()
    .map(|value| (*value).to_owned())
    .collect())
}
fn command(root: &Path, label: &str, command: &mut std::process::Command, report: &mut Value) -> Result<Vec<u8>, String> {
    let record = format!("{command:?}");
    let result = command.output().map_err(|error| format!("{label}: {error}"))?;
    put(&root.join(format!("{label}.stdout")), &result.stdout)?;
    put(&root.join(format!("{label}.stderr")), &result.stderr)?;
    report["commands"].as_array_mut().unwrap().push(json!({"name":label,"command":record,"status":result.status.code(),
        "stdout_sha256":hash(&result.stdout),"stderr_sha256":hash(&result.stderr)}));
    if !result.status.success() {
        return Err(format!("{label}: {}", String::from_utf8_lossy(&result.stderr)));
    }
    if !result.stderr.is_empty() {
        return Err(format!("{label}: unexpected successful stderr was preserved"));
    }
    Ok(result.stdout)
}

use library_artifact::object_contents;

/// These controls corrupt records from the actual checked and emitted unit.
/// They run only in this research executable, never in a compiler request.
fn encoding_controls(input: &metadata::Input<'_>, sdk: &metadata::Sdk) -> Result<Vec<String>, String> {
    if input.emitted.len() < 2 {
        return Err("metadata qualifier requires its independent multi-function fixture".into());
    }
    let mut controls = Vec::new();
    let mut reject = |name: &str, result: Result<metadata::Encoded, String>| {
        if result.is_ok() {
            Err(format!("metadata encoder accepted {name}"))
        } else {
            controls.push(name.to_owned());
            Ok(())
        }
    };
    let mut emitted = input.emitted.to_vec();
    emitted.pop();
    reject("missing-emitted-instance", metadata::encode(&metadata::Input { emitted: &emitted, ..*input }, sdk))?;
    let mut emitted = input.emitted.to_vec();
    emitted.push(emitted[0].clone());
    reject("duplicate-emitted-instance", metadata::encode(&metadata::Input { emitted: &emitted, ..*input }, sdk))?;
    let mut foreign = input.emitted[0].definition;
    while input.functions.iter().any(|function| function.definition == foreign) {
        foreign.path = foreign.path.wrapping_add(1);
    }
    let mut emitted = input.emitted.to_vec();
    emitted[0].definition = foreign;
    reject("foreign-emitted-instance", metadata::encode(&metadata::Input { emitted: &emitted, ..*input }, sdk))?;
    let mut emitted = input.emitted.to_vec();
    emitted[0].symbol = "invalid symbol".into();
    reject("malformed-emitted-symbol", metadata::encode(&metadata::Input { emitted: &emitted, ..*input }, sdk))?;
    let mut emitted = input.emitted.to_vec();
    emitted[1].symbol = emitted[0].symbol.clone();
    reject("duplicate-emitted-symbol", metadata::encode(&metadata::Input { emitted: &emitted, ..*input }, sdk))?;
    let mut functions = input.functions.to_vec();
    functions[1].definition = functions[0].definition;
    reject("duplicate-definition", metadata::encode(&metadata::Input { functions: &functions, ..*input }, sdk))?;
    let index = input.functions.iter().position(|function| !function.calls.is_empty()).ok_or("qualifier fixture has no checked local call")?;
    let mut functions = input.functions.to_vec();
    let called = functions[index].calls[0];
    functions[index].calls.push(called);
    reject("duplicate-call-edge", metadata::encode(&metadata::Input { functions: &functions, ..*input }, sdk))?;
    let mut functions = input.functions.to_vec();
    functions[index].calls[0] = foreign;
    reject("foreign-call-edge", metadata::encode(&metadata::Input { functions: &functions, ..*input }, sdk))?;
    reject("empty-producer-objects", metadata::encode(&metadata::Input {objects:&[], ..*input},sdk))?;
    let mut objects=input.objects.to_vec(); objects[0].definitions.pop();
    reject("missing-producer-definition", metadata::encode(&metadata::Input {objects:&objects, ..*input},sdk))?;
    let mut objects=input.objects.to_vec(); let duplicate=objects[0].definitions[0]; objects[0].definitions.push(duplicate);
    reject("duplicate-producer-definition", metadata::encode(&metadata::Input {objects:&objects, ..*input},sdk))?;
    let mut objects=input.objects.to_vec(); objects[0].definitions[0]=foreign;
    reject("foreign-producer-definition", metadata::encode(&metadata::Input {objects:&objects, ..*input},sdk))?;
    let mut objects=input.objects.to_vec(); objects[0].member="../consumer.o".into();
    reject("malformed-producer-member", metadata::encode(&metadata::Input {objects:&objects, ..*input},sdk))?;
    for (name, members) in [
        ("empty-rlib-link-members", &[][..]),
        ("duplicate-rlib-link-member", &["consumer.o", "consumer.o"][..]),
        ("malformed-rlib-link-member", &["../consumer.o"][..]),
    ] {
        if metadata::encode_rlib_link(members).is_ok() {
            return Err(format!("metadata encoder accepted {name}"));
        }
        controls.push(name.to_owned());
    }
    Ok(controls)
}

fn verify_archive(bytes: &[u8], stage: &Path) -> Result<Vec<String>, String> {
    let archive = object::read::archive::ArchiveFile::parse(bytes).map_err(|error| error.to_string())?;
    let mut member_names = Vec::new();
    for member in archive.members() {
        let member = member.map_err(|error| error.to_string())?;
        let name = std::str::from_utf8(member.name()).map_err(|error| error.to_string())?;
        if !["lib.rmeta", "lib.rmeta-link", "consumer.o"].contains(&name)
            || member.data(bytes).map_err(|error| error.to_string())? != read(&stage.join(name))?
        {
            return Err("archive member differs from its actual produced bytes".into());
        }
        member_names.push(name.to_owned());
    }
    if member_names != ["lib.rmeta", "lib.rmeta-link", "consumer.o"] {
        return Err("archive member inventory differs from original order".into());
    }
    Ok(member_names)
}

fn compile(source: &Path, root: &Path, receipt: &Path, sysroot: &Path, report: &mut Value) -> Result<(), String> {
    if std::env::var_os("RUSTC_FORCE_RUSTC_VERSION").is_some_and(|value| value != CFG_VERSION) {
        return Err("RUSTC_FORCE_RUSTC_VERSION differs from the pinned metadata context".into());
    }
    // This qualifier loads SDK files once in its one-process host setup. The
    // pure encode call below consumes Sdk without reads or Rust processes.
    // Reusing this setup in the ordinary retained compiler remains unintegrated.
    let sdk = lustr_metadata_sdk::open_bound(receipt, sysroot, SDK_RECEIPT)?;
    report["sdk_receipt_sha256"] = json!(SDK_RECEIPT);
    let stage = root.join("staging");
    std::fs::create_dir(&stage).map_err(|error| format!("create owned staging: {error}"))?;
    let arguments = invocation(source, &stage.join("native.o"))?;
    report["native_arguments"] = json!(arguments);
    report["phase"] = json!("native-prepare");
    let prepared = native_object::prepare(&arguments)?;
    report["native_source_sha256"] = json!(hash(prepared.source().as_bytes()));
    report["phase"] = json!("native-completion");
    let completed = native_consumer::complete(prepared).map_err(|error| format!("native consumer completion: {error:?}"))?;
    let root_span = span(completed.root_span());
    let functions: Vec<_> = completed
        .functions()
        .iter()
        .map(|function| metadata::Function {
            definition: native_id(function.definition()),
            name: function.name().to_owned(),
            name_span: span(function.name_span()),
            span: span(function.span()),
            public: function.visibility() == native_consumer::Visibility::Public,
            docs: function.docs().iter().map(|doc| metadata::DocComment {
                span: span(doc.span),
                text: doc.text.clone(),
                block: doc.block,
            }).collect(),
            inline_never: function.inline_never(),
            parameters: function
                .parameters()
                .iter()
                .map(|parameter| metadata::Parameter {
                    name: parameter.name().to_owned(),
                    span: span(parameter.span()),
                    ty: metadata::IntegerType::from_rust_name(parameter.ty().name()).expect("completed parameter type"),
                })
                .collect(),
            result: metadata::IntegerType::from_rust_name(function.result().name()).expect("completed result type"),
            calls: function.calls().iter().copied().map(native_id).collect(),
        })
        .collect();
    report["phase"] = json!("native-emission");
    let emitted = completed.emit()?;
    let native = emitted.native();
    let prepared = native.prepared();
    let instances: Vec<_> = native
        .functions()
        .iter()
        .map(|instance| {
            Ok(metadata::EmittedFunction {
                definition: native_id(instance.definition()),
                symbol: String::from_utf8(instance.symbol().to_vec()).map_err(|error| error.to_string())?,
            })
        })
        .collect::<Result<_, String>>()?;
    let helpers: Vec<_> = native
        .helpers()
        .iter()
        .map(|symbol| String::from_utf8(symbol.clone()).map_err(|error| error.to_string()))
        .collect::<Result<_, _>>()?;
    let original = native.object();
    put(&stage.join("native.o"), original)?;
    report["phase"] = json!("source-owned-encode");
    let mut definitions: Vec<_> = instances.iter().map(|f|f.definition).collect(); definitions.sort();
    let objects = [metadata::ObjectMember { member:"consumer.o".into(), producer_object_sha256:Sha256::digest(original).into(), definitions }];
    let input = metadata::Input {
        extra_filename: "",
        embed_metadata: None,
            crate_name: prepared.crate_name(),
            metadata: prepared.metadata(),
            source_path: prepared.source_path().to_str().ok_or("native source path is not Unicode")?,
            working_directory: prepared
                .working_directory()
                .ok_or("absent native working directory")?
                .to_str()
                .ok_or("native working directory is not Unicode")?,
            source: prepared.source(),
            root_span,
            profile: metadata::Profile { overflow_checks: prepared.overflow_checks(),
                debug_assertions: prepared.debug_assertions(), panic_abort: true },
            functions: &functions,
            emitted: &instances,
            backend_helpers: &helpers,
            objects: &objects,
        };
    report["encoding_controls"] = json!(encoding_controls(&input, &sdk)?);
    let encoded = metadata::encode(&input, &sdk)?;
    report["stable_crate_id"] = json!(format!("{:016x}", encoded.stable_crate_id));
    report["crate_hash_le_hex"] = json!(encoded.crate_hash.iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    report["native_object_sha256"] = json!(hash(original));
    report["metadata_sha256"] = json!(hash(&encoded.metadata));
    report["backend_helpers"] = json!(helpers);
    report["functions"] = json!(functions.iter().map(|function| json!({"name":function.name,"native_id":[function.definition.krate,function.definition.path],"public":function.public,
        "name_span":[function.name_span.lo,function.name_span.hi],"span":[function.span.lo,function.span.hi],
        "parameters":function.parameters.iter().map(|p| json!({"name":p.name,"span":[p.span.lo,p.span.hi]})).collect::<Vec<_>>(),
        "calls":function.calls.iter().map(|id| json!([id.krate,id.path])).collect::<Vec<_>>() })).collect::<Vec<_>>());
    report["symbol_mapping"] = json!(encoded.symbols.iter().map(|symbol| json!({"native_id":[symbol.definition.krate,symbol.definition.path],"rust_def_index":symbol.rust_def_index,
        "rust_def_path_hash":format!("{:016x}",symbol.rust_def_path_hash),"native":symbol.native_symbol,"rust":symbol.rust_symbol,"public":symbol.public})).collect::<Vec<_>>());
    put(&stage.join("consumer.rmeta"), &encoded.metadata)?;
    put(&stage.join("lib.rmeta"), &archive::wrapper(b".rmeta", &encoded.metadata)?)?;
    let link = metadata::encode_rlib_link(&["consumer.o"])?;
    put(&stage.join("lib.rmeta-link"), &archive::wrapper(b".rmeta-link", &link)?)?;

    let mapping: BTreeMap<_, _> = encoded.symbols.iter().map(|symbol| (symbol.native_symbol.clone(), symbol.rust_symbol.clone())).collect();
    let rename_text: String = mapping.iter().map(|(native, rust)| format!("{native} {rust}\n")).collect();
    put(&stage.join("symbols.map"), rename_text.as_bytes())?;
    let llvm = sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/bin");
    let objcopy = llvm.join("llvm-objcopy");
    let ar = llvm.join("llvm-ar");
    report["artifact_tools"] = json!([{"path":objcopy,"sha256":hash(&read(&objcopy)?)},{"path":ar,"sha256":hash(&read(&ar)?)}]);
    report["phase"] = json!("native-symbol-map");
    command(
        root,
        "objcopy",
        std::process::Command::new(&objcopy).current_dir(&stage).args(["--redefine-syms=symbols.map", "native.o", "consumer.o"]),
        report,
    )?;
    let renamed = read(&stage.join("consumer.o"))?;
    let old = unit_object::symbols(original)?;
    let new = unit_object::symbols(&renamed)?;
    let expected = |names: &std::collections::BTreeSet<Vec<u8>>| {
        names
            .iter()
            .map(|name| mapping.get(String::from_utf8_lossy(name).as_ref()).map_or_else(|| name.clone(), |name| name.as_bytes().to_vec()))
            .collect::<std::collections::BTreeSet<_>>()
    };
    if new.defined != expected(&old.defined)
        || new.functions != expected(&old.functions)
        || new.undefined != expected(&old.undefined)
        || object_contents(original, &mapping)? != object_contents(&renamed, &BTreeMap::new())?
    {
        return Err("actual renamed object changed bytes, relocations or source/helper symbol ownership".into());
    }
    report["renamed_object_sha256"] = json!(hash(&renamed));
    report["phase"] = json!("original-archive");
    command(
        root,
        "archive",
        std::process::Command::new(&ar).current_dir(&stage).args([
            "crsD",
            "--format=gnu",
            "libconsumer.rlib",
            "lib.rmeta",
            "lib.rmeta-link",
            "consumer.o",
        ]),
        report,
    )?;
    let archive = read(&stage.join("libconsumer.rlib"))?;
    let member_names = verify_archive(&archive, &stage)?;
    report["archive_members"] = json!(member_names);
    report["rlib_sha256"] = json!(hash(&archive));
    command(root, "missing-member-control", std::process::Command::new(&ar).current_dir(&stage)
        .args(["crsD", "--format=gnu", "missing-object.rlib", "lib.rmeta", "lib.rmeta-link"]), report)?;
    if verify_archive(&read(&stage.join("missing-object.rlib"))?, &stage).is_ok() {
        return Err("archive verifier accepted an absent emitted object member".into());
    }
    put(&stage.join("duplicate-object.rlib"), &archive)?;
    command(root, "duplicate-member-control", std::process::Command::new(&ar).current_dir(&stage)
        .args(["qD", "duplicate-object.rlib", "consumer.o"]), report)?;
    if verify_archive(&read(&stage.join("duplicate-object.rlib"))?, &stage).is_ok() {
        return Err("archive verifier accepted a duplicate emitted object member".into());
    }
    report["archive_controls"] = json!(["missing-object-member", "duplicate-object-member"]);
    report["phase"] = json!("publication");
    match unit_publication::publish(&stage.join("libconsumer.rlib"), &root.join("libconsumer.rlib"))
        .map_err(|error| format!("publish metadata archive: {error}"))?
    {
        unit_publication::Publication::Durable => (),
        unit_publication::Publication::DurabilityUnknown(error) => {
            return Err(format!("metadata archive published; durability unconfirmed: {error}"));
        }
    }
    report["phase"] = json!("complete");
    Ok(())
}

fn main() {
    raise_stack_limit();
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.first().is_some_and(|argument|argument=="--retained") {
        if let Err(error)=retained::run(&arguments[1..]) {eprintln!("{error}");std::process::exit(3);}
        return;
    }
    if arguments.len() != 4 {
        eprintln!("SOURCE EXISTING_OUTPUT_DIRECTORY EXACT_SDK_RECEIPT PINNED_SDK");
        std::process::exit(2);
    }
    let source = PathBuf::from(&arguments[0]);
    let root = PathBuf::from(&arguments[1]);
    let receipt = PathBuf::from(&arguments[2]);
    let sysroot = PathBuf::from(&arguments[3]);
    let mut report = json!({"schema":"native-consumer-metadata-qualification-v1","passed":false,"commands":[],"phase":"setup",
        "scope":"one-shot native completion, original metadata and archive; retained/public integration unqualified"});
    let result = compile(&source, &root, &receipt, &sysroot, &mut report);
    report["passed"] = json!(result.is_ok());
    if let Err(error) = &result {
        report["error"] = json!(error);
    }
    if let Err(error) = put(&root.join("report.json"), &serde_json::to_vec_pretty(&report).unwrap()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
    match result {
        Ok(()) => println!("native metadata artifact: {}", root.join("libconsumer.rlib").display()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
