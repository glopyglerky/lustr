#!/usr/bin/env python3
"""Regenerate exact source declarations without running a compiler or candidate."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FILES=[
 "Cargo.toml","Cargo.lock","build.rs","src/lib.rs","src/main.rs","src/native_queries.rs","src/library_contract.rs","src/library_request.rs",
 "compiler/native/Cargo.toml","compiler/native/Cargo.lock","compiler/native/build.rs",
 "compiler/native/src/main.rs","compiler/native/src/protocol.rs","compiler/native/src/daemon.rs","compiler/native/src/diag.rs",
 "compiler/native/src/cgen.rs","compiler/native/src/cgen/ll.rs","compiler/native/src/cgen/ll_debug.rs",
 "compiler/native/src/check/collect.rs",
 "compiler/native/src/cc.rs","compiler/native/src/cc/pgo.rs","compiler/native/src/ir.rs",
 "compiler/native/src/lower.rs","compiler/native/src/driver/ir_cache.rs","compiler/native/src/tcc.rs",
 "compiler/native/src/cgen/rust_tests.rs","compiler/native/src/runtime/core.c","compiler/native/src/runtime/fmt.c",
 "compiler/native/src/runtime/panic.c",
 "compiler/native/src/driver/mod.rs","compiler/native/src/driver/cache.rs","compiler/native/src/driver/compile.rs","compiler/native/src/driver/session.rs",
 "compiler/native/src/driver/imports.rs","compiler/native/src/resolve/worklist.rs","compiler/native/src/resolve/imports.rs",
 "compiler/native/src/driver/graph.rs","compiler/native/src/check/generics.rs",
 "compiler/native/src/check/prog_shift.rs","compiler/native/src/check/delta_audit.rs",
 "compiler/native/src/borrowck/regions/supers.rs",
 "compiler/native/src/check/outlives.rs",
 "compiler/native/src/resolve/cargo_plan.rs",
 "crates/lustr-native-core/Cargo.toml","crates/lustr-native-core/Cargo.lock",
 "crates/lustr-native-core/src/library_history.rs","crates/lustr-native-core/src/native_library.rs","crates/lustr-native-core/src/native_completion.rs",
 "crates/lustr-native-core/src/library_artifact.rs","crates/lustr-native-core/src/metadata_archive.rs",
 "crates/lustr-native-core/src/public_library.rs","crates/lustr-native-core/src/public_library_import.rs",
 "crates/lustr-native-core/src/public_executable_import.rs",
 "crates/lustr-native-core/src/unit_object.rs","crates/lustr-native-core/src/unit_publication.rs",
 "crates/lustr-native-core/LICENSE-LLVM","crates/lustr-metadata-encoder/Cargo.toml",
 "crates/lustr-metadata-encoder/src/lib.rs","crates/lustr-metadata-encoder/src/encode.rs",
 "crates/lustr-metadata-encoder/src/extern_metadata.rs","crates/lustr-metadata-encoder/src/sdk.rs",
 "crates/lustr-metadata-encoder/src/upstream/predefined_symbols.rs",
 "crates/lustr-metadata-sdk/Cargo.toml","crates/lustr-metadata-sdk/src/lib.rs",
 "tools/build-native-toolchain.rs","tools/native-test-runtime.rs","tools/install-native-metadata-sdk.py","tools/check-exclusive-coverage.py",
 "tools/update-public-library-source-manifest.py","upstream/metadata-artifact-tools.json","upstream/metadata-dependency-closure.json",
 "docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json",
 "docs/compiler-set/evidence/metadata-sdk-receipt-20261120/standard-receipt.json",
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
      "status":"implemented-partial","full_rust_coverage":False,"performance_qualified":False,
      "qualification_record":"docs/compiler-set/evidence/compiler-review-20261122/README.md; results apply only to the exact source and package identities recorded there, not every future regeneration of this inventory",
      "metadata_dependency_closure":"upstream/metadata-dependency-closure.json","artifact_tool_closure":"upstream/metadata-artifact-tools.json",
      "product_owner":"crates/lustr-native-core/src/public_library.rs",
      "shared_artifact_service":"crates/lustr-native-core/src/library_artifact.rs",
      "supported_context":"Linux x64, edition 2024, embed-bitcode=no and opt-level=0. Native builds accept debuginfo=0. Supported executables and native scalar leaf libraries with std or no_std crate policy accept unsplit and unstripped full Rust debug information using the native LLVM emitter and Clang. The debug emitter represents scalar values, selected aggregates and variants, references, function pointers and local bindings across normal control flow. Scalar leaf libraries and executables with a closed scalar archive graph also link under unwind. The native emitter can lower direct integer library dependency calls with Rust debug information. Executable dependencies resolve transitive metadata in Cargo search paths while retaining the explicit extern namespace. Metadata-only checks also accept the ordinary Cargo dev profile with unwind, debuginfo=2 and the optional incremental cache location. Checked scalar signatures carry every integer type. Linked artifacts require the existing direct ABI for 32-bit, 64-bit and pointer-sized integers. Cargo library checks and builds may use one direct scalar leaf dependency supplied as metadata. Cargo executable checks use a closed scalar metadata graph. Executable builds additionally use the matching archives. Executable dep-info covers source files and actual include and environment reads from the native resolver and expander. The admitted extern lint uses native checked reads. Supported Cargo unit and integration tests with the unwind strategy link the pinned Rust libtest runtime and keep body checking and emission in Lust. Tests may use the supported native main return forms and link the represented native path dev dependencies.",
      "request_options":"LibraryOptions is per request, outside native SessionKey and Client. At O0, omitted overflow-checks inherits the effective debug-assertions setting; admitted explicit values take precedence. The direct Rust adapter preserves last-option precedence, while the bounded Cargo artifact adapter preserves debug level (-g and -C debuginfo), split and strip precedence. Other duplicate codegen options are refused. Debug choices travel on public requests outside the semantic session key. Native executable and scalar leaf library debug requests use full unsplit Rust DWARF for the represented source and types. Other debug levels and split sidecars remain unimplemented for linked output. Unstable embedding None/true/false are distinct metadata/SVH variants; effective None/true use full archive metadata and false uses its matching header stub. Force-warn/JSON do not enter the metadata hash. Stable -C embed-metadata remains unrepresented.",
      "artifact_variants":"At most three embedding choices for the current native link snapshot, plus the last durable link publication. The metadata-only cache holds a single checked generation and requires no native object, export, or archive work.",
      "semantic_owner":"One existing native prepare/emit continuation; only existing completed scalar declarations/bodies.",
      "unimplemented":["std-root libraries with direct user dependencies and wider dependent executables","wider Rust source-level debug information","general Rust panic cleanup and exception semantics","stable -C embed-metadata and wider unstable options","general native library and executable dependency graphs","wider native completion","other targets/profiles","socket-free public retained host"],
      "inactive_history":"Owner-start budget, default 128 entries and 16 MiB of known payload per native-object, native-definition and artifact cache; current/published/in-flight evidence protected. Lowering the real entry limit exercises the same LRU cut. Actual RSS includes live source, allocator overhead and original native caches.",
      "publication":"All bytes staged/synced before replacements; complete files replace in metadata/dep-info/archive order. Partial multi-name effects and uncertain durability are terminal. No-op exact output bytes leave output inodes/mtimes unchanged.",
      "runtime_limitation":"The pinned dynamic llvm-ar/llvm-objcopy dependency closure may require the exact 208,283,952-byte LLVM shared library. This is artifact-tool closure, not a proved minimal source cut or compiler speed qualification.",
      "upstream_sources":[
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_driver_impl/src/args.rs","sha256":"0a22c186ea65003951856d1ef0b44e5e9b9136e9b248d483dc64de936144e33f","role":"Line-based compiler response-file expansion and early diagnostics at the Rust entry; no shell-style argument files are admitted"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_driver_impl/src/lib.rs","sha256":"28d3ed858c3db58e159cfe6a6e6e47fa15b64fe354e565a7c94ef24b563c7b44","role":"make_input reads UTF-8 stdin and distinguishes anonymous input from Rustdoc source coordinates"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_interface/src/interface.rs","sha256":"374372a8fa5c5b46ad6d98db08c4b3791808618495ab93f4ed2a34b0719b9cc2","role":"parse_cfg and parse_check_cfg distinguish active flags and values from declarations about expected cfg usage"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_interface/src/passes.rs","sha256":"8ee543fb7a03b80cbd95a58c57a134215d97218afa6551ecc0bd70f02fe75fbc","role":"Authoritative file-stem crate naming for direct compiler inputs"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_interface/src/util.rs","sha256":"a72d09c63337f969f489d5ef4f8ac4266c0516c6126603b34be05500dbf6f90e","role":"build_output_filenames binds the caller's explicit -o and directory selection"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_codegen_ssa/src/back/metadata.rs","sha256":"cb11be9978528bdd6950b7c07c61a9ad914569cc0a20281cb7a564a7bae1881b","role":"DefaultMetadataLoader reads the original lib.rmeta member and .rmeta wrapper section in an rlib"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_attr_ir/src/attr.rs","sha256":"cc70605cd3eb285592beafaea8970c0ea0f1bb1e54f57416dbbb12ecfa9da8b2","role":"Original parsed attribute wrapper and cross-crate documentation accessor"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_attr_ir/src/data_structures.rs","sha256":"6b61fd75a4a88bb80dfdcc71bdb5325c9083bfa99f5bf03baa9c19250a0852ba","role":"Original DocComment variant and field order in the pinned attribute schema"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_ast/src/token.rs","sha256":"d99c9f5a247cb2f24b759fa719d0c1c884970f32f7ade2b08424a2bea69d3fa9","role":"Original sugared line and block doc comment kinds"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/locator.rs","sha256":"3e91bea232fdf8b8b01858bb55fb923f63f8c3ea66071cfc8eda9e9cebb9d758","role":"Original crate-location rules prefer a matching metadata file and can read full metadata from a matching Rust library"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"src/librustdoc/doctest.rs","sha256":"7ae0bd8b117707b936a99aed1e9ede843342ceeaba16465cdc8b96fa1521113d","role":"Original standalone doctest builder request, including the response-file codegen unit choice and generated stdin source"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_span/src/lib.rs","sha256":"9682c3b7d227039f040afff82aae4d2fd8133692bc9ef84f0974323f8a0acca0","role":"Original SourceFile multibyte character sequence and normalized position wire order"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_span/src/analyze_source_file.rs","sha256":"15cb5f3992b3e6f178b1bfac78ed136dd95f6d5b87c3eb4e4587da69b132734d","role":"Original generic byte-offset analysis for line starts and multibyte characters"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_session/src/options.rs","sha256":"9ac7d7de14cbef1624c9998593b081000dcd496413d2bb3643aac2bd1c9c7e69","role":"Pinned debug, split and strip option value parsing; separate embedding options, excluded lint crate-hash state and untracked JSON response mode"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_session/src/config.rs","sha256":"20518f46c9a57d05f1353c1fcc2b3f8108b09c99efd87d6f009b71254441dcaf","role":"select_debuginfo orders -g and -C debuginfo by argument position; DocTestStr retains its originating file and signed line offset"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_span/src/source_map.rs","sha256":"9fa60e859a71b59b6787028a569a17d9fa0b5f047af9bb12de14e9c34af6eeb3","role":"doctest_offset_line reports document coordinates while retaining the generated source line index"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_session/src/session.rs","sha256":"cf93aa2f3af4a4be00930d9c1078c17a91add82ac6206e38e3fee74c724e93a8","role":"Effective embedding and profile options; resolve_path rejects a relative include from an anonymous input"},
        {"repository":"TinyCC/tinycc","revision":"43c7708b85681a2fd4451c8a541af4494a8919b2","path":"tccdbg.c","sha256":"62c14837a1e68283d4505e4837d3cc2f7e3df422a727124a5129cc71f74399fa","role":"Original x86-64 frame descriptor emission and section relocation for one native function per object"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/rmeta/encoder.rs","sha256":"7f740fbaf9e81f4d518a95eeab485059c4f1a88c636df3104af03ec2817f5b73","role":"Original stub writer/header and full-SVH binding; tracked-option supplement"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/rmeta/mod.rs","sha256":"d6d5d627a4876ddd5d371c2260f0d68303f1e483a5209fcace3893765bfd5948","role":"Original CrateHeader target/name/proc-macro/stub field order"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/fs.rs","sha256":"793c4dae083a30b7073ec080ce24cc02eb1e2048938c6c48d9946fb51e14ecdf","role":"Split full metadata and header-only metadata preparation/publication"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_middle/src/ty/context.rs","sha256":"cea6dd6e43c84fb14476a18923aa3b4acb2138c8bff6b849f4b55e2b20544ae3","role":"Executable crate types do not need encoded metadata; encode_and_write_metadata publishes the requested empty file after checking"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_codegen_ssa/src/back/link.rs","sha256":"0cfbc38cf0c66133b3f7dc10bd558d1fb4773fc1c09054c48a32bc694902a191","role":"Original lib.rmeta wrapper uses stub_or_full while lib.rmeta-link and object inventory remain"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_codegen_ssa/src/back/symbol_export.rs","sha256":"75e01be382f01e791e71d7ede5bc8f3430902b4efebf269f50aead67bd7d0f1f","role":"Both exported symbol providers return no symbols on metadata-only requests before consulting code generation"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_resolve/src/late/weak_lang_items.rs","sha256":"ad42f79421494be53258e42570877f91c2c162ade987fa317a514cb1c941cc42","role":"Record the original missing personality language item for the fixed native no_std leaf and SDK closure"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/locator.rs","sha256":"3e91bea232fdf8b8b01858bb55fb923f63f8c3ea66071cfc8eda9e9cebb9d758","role":"Full standalone metadata and matching stub archive association; no stub-only full-metadata acceptance"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_metadata/src/creader.rs","sha256":"5ad69fef2068ce2cff4a1ab21659359270770e4ee69fa840558a29a3fb706539","role":"Actual explicit-unused-extern inventory and forced empty result emission after required analyses"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_lint_defs/src/lib.rs","sha256":"5bb391853043b7216010b3fccdc09e81158f34c39232a1dc51eab9d816654084","role":"Exact ForceWarn as_str token for the typed response"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_interface/src/passes.rs","sha256":"8ee543fb7a03b80cbd95a58c57a134215d97218afa6551ecc0bd70f02fe75fbc","role":"Exact output/dep-info layout, filename escaping, and sorted escaped env-dep records for actual expansion reads"},
        {"repository":"rust-lang/rust","revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494","path":"compiler/rustc_errors/src/json.rs","sha256":"cb6bfcad442469b2bbdcc77ce28b22b3903c4fec489043a82f6dfd008ad12281","role":"Typed diagnostic, artifact and unused-extern JSON representation; semantic diagnostics remain native"},
        {"repository":"rust-lang/cargo","revision":"c07ac2c4d12237effc4b94f4543769b421790045","path":"src/compiler/mod.rs","sha256":"06f232baf25ae3279fa92eaa96a2ff791dfa7dd0c200c8c9e4b58273245a9f44","role":"Actual leaf compiler arguments and explicit/default profile facts"}],
      "source_sha256":{p:sha(ROOT/p) for p in FILES}}
    write(ROOT/"upstream/public-library-integration.json",record)
if __name__=="__main__":update()
