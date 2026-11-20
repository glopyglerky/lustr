#!/usr/bin/env python3
"""Regenerate exact source declarations without running a compiler or candidate."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FILES=[
 "src/lib.rs","src/main.rs","src/native_queries.rs","src/library_contract.rs","src/library_request.rs",
 "compiler/native/Cargo.toml","compiler/native/Cargo.lock","compiler/native/build.rs",
 "compiler/native/src/main.rs","compiler/native/src/protocol.rs","compiler/native/src/daemon.rs","compiler/native/src/diag.rs",
 "compiler/native/src/driver/mod.rs","compiler/native/src/driver/compile.rs","compiler/native/src/driver/session.rs",
 "crates/lustr-native-core/Cargo.toml","crates/lustr-native-core/Cargo.lock",
 "crates/lustr-native-core/src/library_history.rs","crates/lustr-native-core/src/native_library.rs","crates/lustr-native-core/src/native_completion.rs",
 "crates/lustr-native-core/src/library_artifact.rs","crates/lustr-native-core/src/metadata_archive.rs",
 "crates/lustr-native-core/src/public_library.rs","crates/lustr-native-core/src/unit_object.rs","crates/lustr-native-core/src/unit_publication.rs",
 "crates/lustr-native-core/LICENSE-LLVM","crates/lustr-metadata-encoder/Cargo.toml",
 "crates/lustr-metadata-encoder/src/lib.rs","crates/lustr-metadata-encoder/src/encode.rs",
 "crates/lustr-metadata-sdk/Cargo.toml","crates/lustr-metadata-sdk/src/lib.rs",
 "tools/build-native-toolchain.rs","tools/install-native-metadata-sdk.py","tools/check-exclusive-coverage.py",
 "tools/update-public-library-source-manifest.py","upstream/metadata-artifact-tools.json","upstream/metadata-dependency-closure.json",
 "docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json",
]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path,data):path.write_text(json.dumps(data,indent=2)+"\n")
def update():
    encoder=ROOT/"crates/lustr-metadata-encoder"
    path=encoder/"source-slices.json";data=json.loads(path.read_text())
    data["adapted_sources_sha256"]={str(p.relative_to(encoder)):sha(p) for p in sorted((encoder/"src").rglob("*.rs"))}
    write(path,data)
    qualifier=ROOT/"crates/lustr-native-metadata-qualification"
    path=qualifier/"source-slices.json";data=json.loads(path.read_text())
    files=["Cargo.toml","Cargo.lock","build.rs"]+sorted(str(p.relative_to(qualifier)) for p in (qualifier/"src").rglob("*.rs"))
    files+=["../lustr-native-core/src/library_history.rs","../lustr-native-core/src/library_artifact.rs","../lustr-native-core/src/metadata_archive.rs",
            "../lustr-native-core/src/unit_object.rs","../lustr-native-core/src/unit_publication.rs"]
    data["adapted_sources_sha256"]={p:sha(qualifier/p) for p in files};write(path,data)
    record={
      "schema":"lustr-public-library-integration-v1","rust_revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494",
      "status":"authored-unqualified","full_rust_coverage":False,"performance_qualified":False,
      "metadata_dependency_closure":"upstream/metadata-dependency-closure.json","artifact_tool_closure":"upstream/metadata-artifact-tools.json",
      "product_owner":"crates/lustr-native-core/src/public_library.rs",
      "shared_artifact_service":"crates/lustr-native-core/src/library_artifact.rs",
      "supported_context":"Linux x64, edition 2024, actual dependency-free Cargo leaf library; explicit panic=abort and embed-bitcode=no, opt-level=0, debuginfo=0; exact dep-info,metadata,link outputs.",
      "semantic_owner":"One existing native prepare/emit continuation; only existing completed scalar declarations/bodies.",
      "unimplemented":["metadata-only cargo check","public --extern ingestion","wider native completion","other targets/profiles","socket-free public retained host"],
      "inactive_history":"Owner-start budget, default 128 entries and 16 MiB of known payload per native-object, native-definition and artifact cache; current/published/in-flight evidence protected. Lowering the real entry limit exercises the same LRU cut. Actual RSS includes live source, allocator overhead and original native caches.",
      "publication":"All bytes staged/synced before replacements; complete files replace in metadata/dep-info/archive order. Partial multi-name effects and uncertain durability are terminal. No-op exact output bytes leave output inodes/mtimes unchanged.",
      "runtime_limitation":"The pinned dynamic llvm-ar/llvm-objcopy dependency closure may require the exact 208,283,952-byte LLVM shared library. This is artifact-tool closure, not a proved minimal source cut or compiler speed qualification.",
      "upstream_sources":[
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_interface/src/passes.rs","sha256":"8ee543fb7a03b80cbd95a58c57a134215d97218afa6551ecc0bd70f02fe75fbc","role":"Actual output/dep-info layout and filename escaping"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_errors/src/json.rs","sha256":"cb6bfcad442469b2bbdcc77ce28b22b3903c4fec489043a82f6dfd008ad12281","role":"Typed diagnostic and artifact JSON representation; semantic diagnostics remain native"},
        {"repository":"rust-lang/cargo","revision":"c07ac2c4d12237effc4b94f4543769b421790045","path":"src/compiler/mod.rs","sha256":"06f232baf25ae3279fa92eaa96a2ff791dfa7dd0c200c8c9e4b58273245a9f44","role":"Actual leaf compiler arguments and explicit/default profile facts"}],
      "source_sha256":{p:sha(ROOT/p) for p in FILES}}
    write(ROOT/"upstream/public-library-integration.json",record)
if __name__=="__main__":update()
