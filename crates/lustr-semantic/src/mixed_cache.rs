//! Retain checked Rust work, with fresh Rust expansion and actual HIR hashes.
//! No rustc arena reference or local compiler id crosses an invocation.
use rustc_data_structures::{
    fingerprint::Fingerprint,
    stable_hash::{StableHash, StableHasher},
};
use rustc_middle::ty::TyCtxt;
use rustc_span::{SourceFileHash, SourceFileHashAlgorithm};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[path = "../../lustr-native-core/src/mixed_symbols.rs"]
mod native_symbols;

fn digest(bytes: &[u8]) -> String {
    // Reuse the pinned compiler's SHA-256 implementation. No process or new
    // hashing implementation is needed on this per-function path.
    SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, bytes).to_string()
}

pub fn native_request(tcx: TyCtxt<'_>, header: &str, source: &str, symbol: &str, rust_abi: &str, c_abi: &str) -> Value {
    json!({"schema":"lustr.checked-native-unit","version":1,
        "rustc":env!("LUSTR_BUILD_RUSTC_COMMIT"),"native":env!("LUST_BUILD_HASH"),
        "host":env!("LUSTR_BUILD_RUSTC_HOST"),"target":tcx.sess.opts.target_triple.to_string(),
        "overflow_checks":tcx.sess.overflow_checks(),"debug_assertions":tcx.sess.opts.debug_assertions,
        "panic":format!("{:?}",tcx.sess.panic_strategy()),
        "optimization":format!("{:?}",tcx.sess.opts.optimize),"debug_info":format!("{:?}",tcx.sess.opts.cg.debuginfo),
        "header":header,"source_sha256":digest(source.as_bytes()),"symbol":symbol,
        "rust_abi":rust_abi,"c_abi":c_abi})
}

fn native_bytes(path: &Path, symbol: &str) -> Option<Vec<u8>> {
    const LIMIT: u64 = 64 * 1024 * 1024;
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > LIMIT {
        return None;
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    fs::File::open(path).ok()?.take(LIMIT + 1).read_to_end(&mut bytes).ok()?;
    native_symbols::check_renamed_bytes(&bytes, symbol.as_bytes()).ok()?;
    Some(bytes)
}

pub fn native_digest(path: &Path, symbol: &str) -> Option<String> {
    native_bytes(path, symbol).map(|bytes| digest(&bytes))
}

fn native_path(directory: &Path, kernel: &Value) -> Option<PathBuf> {
    let identity = text(kernel, "identity")?;
    if identity.len() != 16 || !identity.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let path = match text(kernel, "object") {
        Some(path) => PathBuf::from(path),
        None => directory.join(identity).join("native-link.o"),
    };
    // Reject a substituted symlink before resolving the owned generation.
    if !fs::symlink_metadata(&path).ok()?.is_file() {
        return None;
    }
    let path = path.canonicalize().ok()?;
    path.starts_with(directory).then_some(path)
}

/// Reuse only the native object after the CURRENT complete Rust analysis and
/// scalar body/FnAbi checks. This never reuses Rust analysis, metadata or link
/// objects after a semantic-context miss. The isolated native request must be
/// identical, including all source bytes and its actual Rust export/ABI.
pub fn copy_native(compiler: &super::Compiler, definition: &str, request: &Value, output: &Path) -> Option<String> {
    if compiler.check_only || !compiler.reuse_allowed {
        return None;
    }
    let state = compiler.cached.as_ref()?;
    if state["check_only"] != false {
        return None;
    }
    let directory = PathBuf::from(text(state, "directory")?).canonicalize().ok()?;
    if !directory.starts_with(compiler.index.parent()?.canonicalize().ok()?) {
        return None;
    }
    let mut matches = state["kernels"]
        .as_array()?
        .iter()
        .filter(|kernel| text(kernel, "definition") == Some(definition) && kernel["native_request"] == *request);
    let kernel = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let source = format!("{}{}{}", text(kernel, "prefix")?, text(kernel, "body")?, text(kernel, "suffix")?);
    if text(request, "source_sha256")? != digest(source.as_bytes()) {
        return None;
    }
    let symbol = text(request, "symbol")?;
    if text(kernel, "symbol")? != symbol {
        return None;
    }
    let bytes = native_bytes(&native_path(&directory, kernel)?, symbol)?;
    let sha256 = digest(&bytes);
    if text(kernel, "object_sha256")? != sha256 {
        return None;
    }
    // Use exactly the bytes just validated. A hard link would expose the last
    // successful generation to later linker writes; a second read could differ.
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(output).ok()?;
    let retained = file.write_all(&bytes).and_then(|_| file.sync_all()).and_then(|_| fs::File::open(output.parent().unwrap())?.sync_all());
    if retained.is_err() {
        drop(file);
        let _ = fs::remove_file(output);
        return None;
    }
    Some(sha256)
}

pub fn context(tcx: TyCtxt<'_>, native: &BTreeSet<String>) -> Option<Value> {
    let _profile = super::mixed_profile::span("context-fingerprint");
    let mut owners = std::collections::BTreeMap::new();
    for owner in tcx.hir_crate_items(()).owners() {
        let key = format!("{:?}", tcx.def_path_hash(owner.def_id.to_def_id()));
        // Same hashing as pinned rustc_middle::hir::hash_owner_nodes
        // (Rust Project Developers, MIT OR Apache-2.0). Cargo need not request
        // rustc incremental compilation for this separate retained boundary.
        let (attrs, body) = tcx.with_stable_hashing_context(|mut hcx| {
            let attributes = tcx.hir_attr_map(owner);
            let attrs = attributes.opt_hash.unwrap_or_else(|| {
                let mut h = StableHasher::new();
                attributes.map.stable_hash(&mut hcx, &mut h);
                attributes.define_opaque.stable_hash(&mut hcx, &mut h);
                h.finish::<Fingerprint>()
            });
            let body = if native.contains(&key) {
                "native-body".into()
            } else {
                let nodes = tcx.hir_owner_nodes(owner);
                let value = nodes.opt_hash.unwrap_or_else(|| {
                    let mut h = StableHasher::new();
                    nodes.node().stable_hash(&mut hcx, &mut h);
                    nodes.bodies.stable_hash(&mut hcx, &mut h);
                    h.finish::<Fingerprint>()
                });
                format!("{value:?}")
            };
            (format!("{attrs:?}"), body)
        });
        owners.insert(key, json!([attrs, body]));
    }
    let mut dependencies = std::collections::BTreeMap::new();
    for &krate in tcx.crates(()) {
        dependencies.insert(format!("{:?}", tcx.stable_crate_id(krate)), format!("{:?}", tcx.crate_hash(krate)));
    }
    Some(json!({"owners":owners,"dependencies":dependencies}))
}

pub fn read(index: &Path) -> Option<Value> {
    let state: Value = serde_json::from_slice(&fs::read(index).ok()?).ok()?;
    (state["schema"] == "lustr.checked-mixed-link"
        && state["version"] == 5
        && state["rustc"] == env!("LUSTR_BUILD_RUSTC_COMMIT")
        && state["native"] == env!("LUST_BUILD_HASH"))
    .then_some(state)
}

pub fn publish(index: &Path, directory: &Path, context: Value, kernels: Vec<Value>, check_only: bool) -> std::io::Result<()> {
    let _profile = super::mixed_profile::span("publish-certificate");
    let state = json!({"schema":"lustr.checked-mixed-link","version":5,"check_only":check_only,
        "rustc":env!("LUSTR_BUILD_RUSTC_COMMIT"),"native":env!("LUST_BUILD_HASH"),
        "directory":directory,"context":context,"kernels":kernels});
    fs::create_dir_all(index.parent().unwrap())?;
    let tmp = index.with_extension("next");
    let mut f = fs::OpenOptions::new().write(true).create(true).truncate(true).open(&tmp)?;
    f.write_all(&serde_json::to_vec(&state)?)?;
    f.sync_all()?;
    fs::rename(&tmp, index)?;
    fs::File::open(index.parent().unwrap())?.sync_all()
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str()
}

// The first retained boundary permits only a decimal-literal edit in a direct
// scalar `parameter ^ literal` or primitive wrapping-method body. This
// certificate has no new declarations,
// lookups, calls, lint scopes or cross-function uses. General body edits still
// use the complete compiler; expanding this certificate needs its own checks.
fn literal_edit(old: &str, new: &str) -> bool {
    fn split(s: &str) -> Option<(&str, &str, &str)> {
        let body = s.trim().strip_prefix('{')?.strip_suffix('}')?.trim();
        let (parameter, literal) = if let Some(parts) = body.split_once('^') {
            parts
        } else {
            let (parameter, call) = body.split_once('.')?;
            let (method, argument) = call.split_once('(')?;
            if !matches!(
                method.trim(),
                "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "wrapping_shl" | "wrapping_shr" | "rotate_left" | "rotate_right"
            ) {
                return None;
            }
            (parameter, argument.strip_suffix(')')?)
        };
        let parameter = parameter.trim();
        let literal = literal.trim();
        if parameter.is_empty()
            || !parameter.bytes().enumerate().all(|(n, b)| b == b'_' || b.is_ascii_alphabetic() || (n > 0 && b.is_ascii_digit()))
            || literal.is_empty()
            || !literal.bytes().all(|b| b.is_ascii_digit())
            || literal.len() > 1 && literal.starts_with('0')
            || literal.parse::<u64>().ok()? > i64::MAX as u64
        {
            return None;
        }
        let start = literal.as_ptr() as usize - s.as_ptr() as usize;
        Some((&s[..start], literal, &s[start + literal.len()..]))
    }
    match (split(old), split(new)) {
        (Some((a, b, c)), Some((x, y, z))) => a == x && c == z && b.len() == y.len(),
        _ => false,
    }
}

/// Expansion is refreshed even for proc macros with unreported external reads.
/// A changed expansion, definition, dependency or attribute returns to complete
/// Rust analysis in the SAME invocation, reusing the expansion already done.
pub fn attempt(compiler: &mut super::Compiler, tcx: TyCtxt<'_>) -> Option<()> {
    let _profile = super::mixed_profile::span("retained-attempt");
    if !compiler.enabled || !compiler.reuse_allowed || !super::mixed_diagnostics::quiet() {
        return None;
    }
    let state = compiler.cached.clone()?;
    if state["check_only"].as_bool()? != compiler.check_only {
        return None;
    }
    let kernels = state["kernels"].as_array()?;
    if kernels.is_empty() {
        return None;
    }
    let directory = PathBuf::from(state["directory"].as_str()?);
    let owned = directory.canonicalize().ok()?;
    if !owned.starts_with(compiler.index.parent()?.canonicalize().ok()?) {
        return None;
    }
    let excluded: BTreeSet<String> = kernels.iter().map(|k| text(k, "definition").map(str::to_owned)).collect::<Option<_>>()?;
    let current = context(tcx, &excluded)?;
    if current != state["context"] {
        super::trace("reuse-miss=semantic-context");
        return None;
    }
    let mut changed = BTreeSet::new();
    let definitions: std::collections::BTreeMap<_, _> = tcx
        .hir_crate_items(())
        .owners()
        .map(|owner| (format!("{:?}", tcx.def_path_hash(owner.def_id.to_def_id())), owner.def_id))
        .collect();
    for kernel in kernels {
        let name = text(kernel, "name")?;
        let candidate = compiler.candidates.get(name)?;
        if !super::represented_body(tcx, *definitions.get(text(kernel, "definition")?)?, candidate, super::BodyBindings::Retained) {
            return None;
        }
        if candidate.header != text(kernel, "header")? {
            super::trace("reuse-miss=header");
            return None;
        }
        // Retained Rust code contains source locations. Until their remapping
        // is implemented, preserve the line/column after each edited body.
        let old = text(kernel, "body")?;
        if old == candidate.body {
            continue;
        }
        changed.insert(text(kernel, "definition")?.to_owned());
        if !literal_edit(old, &candidate.body) {
            super::trace("reuse-miss=uncertified-body-edit");
            return None;
        }
        let position = |s: &str| (s.bytes().filter(|b| *b == b'\n').count(), s.rsplit('\n').next().unwrap_or("").len());
        if position(old) != position(&candidate.body) {
            super::trace("reuse-miss=source-locations");
            return None;
        }
    }
    let body_queries = super::mixed_profile::span("changed-body-queries");
    for owner in tcx.hir_crate_items(()).owners() {
        if changed.contains(&format!("{:?}", tcx.def_path_hash(owner.def_id.to_def_id()))) {
            // Use the existing Rust body queries for new obligations and
            // rejection. This checks the changed body, not every Rust body.
            tcx.ensure_ok().typeck_root(owner.def_id);
            tcx.ensure_ok().check_unsafety(owner.def_id);
            tcx.ensure_ok().mir_borrowck(owner.def_id);
            tcx.ensure_ok().check_transmutes(owner.def_id);
            tcx.ensure_ok().has_ffi_unwind_calls(owner.def_id);
            tcx.ensure_ok().check_liveness(owner.def_id);
            tcx.ensure_ok().mir_drops_elaborated_and_const_checked(owner.def_id);
            let kernel = kernels
                .iter()
                .find(|k| text(k, "definition") == Some(format!("{:?}", tcx.def_path_hash(owner.def_id.to_def_id())).as_str()))?;
            let candidate = compiler.candidates.get(text(kernel, "name")?)?;
            if !super::represented_body(tcx, owner.def_id, candidate, super::BodyBindings::Current) {
                return None;
            }
        }
    }
    drop(body_queries);
    tcx.sess.dcx().abort_if_errors();
    if !super::mixed_diagnostics::quiet() {
        super::trace("reuse-miss=diagnostics");
        return None;
    }
    let mut updated = kernels.clone();
    for kernel in &mut updated {
        let name = text(kernel, "name")?.to_owned();
        let candidate = compiler.candidates.get(&name)?;
        if candidate.body == text(kernel, "body")? {
            continue;
        }
        let identity = text(kernel, "identity")?.to_owned();
        if identity.len() != 16 || !identity.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let unit = owned.join(&identity);
        if !unit.canonicalize().ok()?.starts_with(&owned) {
            return None;
        }
        let source = unit.join("kernel.rs");
        let generated = format!("{}{}{}", text(kernel, "prefix")?, candidate.body, text(kernel, "suffix")?);
        fs::write(&source, &generated).ok()?;
        kernel["native_request"]["source_sha256"] = json!(digest(generated.as_bytes()));
        if compiler.check_only {
            let (code, _, diagnostic) =
                super::native::check(&source, &name, &identity, tcx.sess.overflow_checks(), tcx.sess.opts.debug_assertions)?;
            super::trace(&diagnostic);
            if code != 0 {
                return None;
            }
            kernel["body"] = json!(candidate.body);
            continue;
        }
        let output = fresh(&unit, "edit", "o")?;
        let (code, native_symbol, diagnostic) =
            super::native::compile(&source, &output, &name, &identity, tcx.sess.overflow_checks(), tcx.sess.opts.debug_assertions)?;
        super::trace(&diagnostic);
        if code != 0 {
            return None;
        }
        let symbol = text(kernel, "symbol")?.to_owned();
        let linked = fresh(&unit, "renamed", "o")?;
        {
            let _profile = super::mixed_profile::span("native-symbol-rename");
            super::mixed_symbol_rename::rename(&output, &native_symbol, &symbol, &linked).ok()?;
        }
        // The current certificate still describes the previous successful
        // body. Keep its object immutable if a later edit, link or dep-info
        // step fails. Only the final-success publication selects this object.
        {
            let _profile = super::mixed_profile::span("edited-native-object-sync");
            fs::File::open(&linked).ok()?.sync_all().ok()?;
            fs::File::open(&unit).ok()?.sync_all().ok()?;
        }
        kernel["object"] = json!(linked);
        kernel["object_sha256"] = json!(native_digest(&linked, &symbol)?);
        kernel["body"] = json!(candidate.body);
    }
    if compiler.check_only {
        super::trace(&format!("reused-check-context=1 full-analysis=0 native-check-edits={}", changed.len()));
    } else {
        let mut objects = BTreeMap::new();
        for kernel in &updated {
            let identity = text(kernel, "identity")?;
            if identity.len() != 16 || !identity.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            let original = owned.join(identity).join("native-link.o").canonicalize().ok()?;
            let selected = native_path(&owned, kernel)?;
            if !original.starts_with(&owned) || native_digest(&selected, text(kernel, "symbol")?)? != text(kernel, "object_sha256")? {
                return None;
            }
            objects.insert(original, selected);
        }
        super::mixed_objects::relink(tcx.sess, &owned, &objects).ok()?;
        super::trace("reused-rust-objects=1 full-analysis=0");
    }
    // Both check and link certificates commit only after the entire compiler
    // invocation succeeds, including the original metadata/dep-info writers.
    compiler.directory = owned;
    compiler.context = Some(current);
    compiler.records = updated;
    compiler.reused = true;
    Some(())
}

pub fn fresh(parent: &Path, prefix: &str, extension: &str) -> Option<PathBuf> {
    for n in 0u64.. {
        let p = parent.join(format!("{prefix}-{n}.{extension}"));
        if !p.exists() {
            return Some(p);
        }
    }
    None
}
