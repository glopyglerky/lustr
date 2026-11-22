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
 "compiler/native/src/driver/imports.rs","compiler/native/src/resolve/worklist.rs",
 "crates/lustr-native-core/Cargo.toml","crates/lustr-native-core/Cargo.lock",
 "crates/lustr-native-core/src/library_history.rs","crates/lustr-native-core/src/native_library.rs","crates/lustr-native-core/src/native_completion.rs",
 "crates/lustr-native-core/src/library_artifact.rs","crates/lustr-native-core/src/metadata_archive.rs",
 "crates/lustr-native-core/src/public_library.rs","crates/lustr-native-core/src/public_library_import.rs",
 "crates/lustr-native-core/src/unit_object.rs","crates/lustr-native-core/src/unit_publication.rs",
 "crates/lustr-native-core/LICENSE-LLVM","crates/lustr-metadata-encoder/Cargo.toml",
 "crates/lustr-metadata-encoder/src/lib.rs","crates/lustr-metadata-encoder/src/encode.rs",
 "crates/lustr-metadata-encoder/src/extern_metadata.rs","crates/lustr-metadata-encoder/src/sdk.rs",
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
      "supported_context":"Linux x64, edition 2024, Cargo library, explicit panic=abort and embed-bitcode=no, opt-level=0, debuginfo=0; dep-info,metadata for check or dep-info,metadata,link for build, optionally with one direct scalar leaf dependency supplied as metadata; actual unstable -Z embed-metadata choice for build and forced unused_crate_dependencies response.",
      "request_options":"LibraryOptions is per request, outside native SessionKey and Client. Unstable embedding None/true/false are distinct metadata/SVH variants; effective None/true use full archive metadata and false uses its matching header stub. Force-warn/JSON do not enter the metadata hash. Stable -C embed-metadata remains unrepresented.",
      "artifact_variants":"At most three embedding choices for the current native link snapshot, plus the last durable link publication. The metadata-only cache holds a single checked generation and requires no native object, export, or archive work.",
      "semantic_owner":"One existing native prepare/emit continuation; only existing completed scalar declarations/bodies.",
      "unimplemented":["stable -C embed-metadata and wider unstable options","native final executable linking and multi-dependency or transitive crate resolution","wider native completion","other targets/profiles","socket-free public retained host"],
      "inactive_history":"Owner-start budget, default 128 entries and 16 MiB of known payload per native-object, native-definition and artifact cache; current/published/in-flight evidence protected. Lowering the real entry limit exercises the same LRU cut. Actual RSS includes live source, allocator overhead and original native caches.",
      "publication":"All bytes staged/synced before replacements; complete files replace in metadata/dep-info/archive order. Partial multi-name effects and uncertain durability are terminal. No-op exact output bytes leave output inodes/mtimes unchanged.",
      "runtime_limitation":"The pinned dynamic llvm-ar/llvm-objcopy dependency closure may require the exact 208,283,952-byte LLVM shared library. This is artifact-tool closure, not a proved minimal source cut or compiler speed qualification.",
      "upstream_sources":[
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_session/src/options.rs","sha256":"9ac7d7de14cbef1624c9998593b081000dcd496413d2bb3643aac2bd1c9c7e69","role":"Separate tracked stable/unstable embedding options, excluded lint crate-hash state and untracked JSON response mode"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_session/src/session.rs","sha256":"cf93aa2f3af4a4be00930d9c1078c17a91add82ac6206e38e3fee74c724e93a8","role":"Exact effective embedding precedence and default; only unstable channel is admitted"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/rmeta/encoder.rs","sha256":"7f740fbaf9e81f4d518a95eeab485059c4f1a88c636df3104af03ec2817f5b73","role":"Original stub writer/header and full-SVH binding; tracked-option supplement"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/rmeta/mod.rs","sha256":"d6d5d627a4876ddd5d371c2260f0d68303f1e483a5209fcace3893765bfd5948","role":"Original CrateHeader target/name/proc-macro/stub field order"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/fs.rs","sha256":"793c4dae083a30b7073ec080ce24cc02eb1e2048938c6c48d9946fb51e14ecdf","role":"Split full metadata and header-only metadata preparation/publication"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_codegen_ssa/src/back/link.rs","sha256":"0cfbc38cf0c66133b3f7dc10bd558d1fb4773fc1c09054c48a32bc694902a191","role":"Original lib.rmeta wrapper uses stub_or_full while lib.rmeta-link and object inventory remain"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_codegen_ssa/src/back/symbol_export.rs","sha256":"75e01be382f01e791e71d7ede5bc8f3430902b4efebf269f50aead67bd7d0f1f","role":"Both exported symbol providers return no symbols on metadata-only requests before consulting code generation"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_resolve/src/late/weak_lang_items.rs","sha256":"ad42f79421494be53258e42570877f91c2c162ade987fa317a514cb1c941cc42","role":"Record the original missing personality language item for the fixed native no_std leaf and SDK closure"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/locator.rs","sha256":"3e91bea232fdf8b8b01858bb55fb923f63f8c3ea66071cfc8eda9e9cebb9d758","role":"Full standalone metadata and matching stub archive association; no stub-only full-metadata acceptance"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/creader.rs","sha256":"5ad69fef2068ce2cff4a1ab21659359270770e4ee69fa840558a29a3fb706539","role":"Actual explicit-unused-extern inventory and forced empty result emission after required analyses"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_lint_defs/src/lib.rs","sha256":"5bb391853043b7216010b3fccdc09e81158f34c39232a1dc51eab9d816654084","role":"Exact ForceWarn as_str token for the typed response"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_interface/src/passes.rs","sha256":"8ee543fb7a03b80cbd95a58c57a134215d97218afa6551ecc0bd70f02fe75fbc","role":"Actual output/dep-info layout and filename escaping"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_errors/src/json.rs","sha256":"cb6bfcad442469b2bbdcc77ce28b22b3903c4fec489043a82f6dfd008ad12281","role":"Typed diagnostic, artifact and unused-extern JSON representation; semantic diagnostics remain native"},
        {"repository":"rust-lang/cargo","revision":"c07ac2c4d12237effc4b94f4543769b421790045","path":"src/compiler/mod.rs","sha256":"06f232baf25ae3279fa92eaa96a2ff791dfa7dd0c200c8c9e4b58273245a9f44","role":"Actual leaf compiler arguments and explicit/default profile facts"}],
      "source_sha256":{p:sha(ROOT/p) for p in FILES}}
    write(ROOT/"upstream/public-library-integration.json",record)
if __name__=="__main__":update()
