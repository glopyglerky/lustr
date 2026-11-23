# Construction receipt for native metadata's SDK dependencies

The standalone [construction tool](../../../../tools/build-metadata-sdk-receipt.py)
produces a receipt for the exact SDK dependencies of the fixed scalar `no_std`
reference library. It uses the original metadata reader at Rust revision
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. It does not accept a user source path,
run during consumer compilation, decide ownership, or add a compiler-private
dependency to the native encoder.

The actual run passed five commands: the declared compiler's version query,
one build of the fixed reference, and original `-Zls=root` reads of the
reference, `core` and `compiler_builtins` metadata. The tool verifies the pinned
rustc, driver and LLVM bytes before use; records every command, original status
and stdout/stderr; and checks those tools and both dependency artifact pairs
again before publishing the receipt. Every command exited zero without
diagnostics. The fixed source hash is
`c88b4bbcdc31261137f1c98a7ed706202ffcd687fd6d60a5f73fc818cd5236ba`.

## Actual dependency facts

[receipt.json](receipt.json) is the original successful receipt, unchanged, with
SHA-256 `4e400b263bf9923616dea44ead2d77213703177d3bfaf6becd6a19a4c5a657a7`.
It records the exact profile, complete original root observations, artifact
sizes/hashes and source contracts. The context is Linux x86-64, edition 2024,
an rlib, panic abort, optimization/debug level zero, overflow checks and debug
assertions enabled. It does not qualify other profiles or targets.

| Crate number | Name | StableCrateId | Serialized SVH bytes, hex | Kind / privacy |
| --- | --- | --- | --- | --- |
| 1 | `core` | `e323c6d5b6a89c14` | `3611d18169958ffa22f1ed1712df6206` | Unconditional (2), public |
| 2 | `compiler_builtins` | `c54c708a7154aa88` | `e645f60e594802de4a0775e61599c208` | Unconditional (2), private |

Both host hashes are absent. The extra filenames are `-b13a99ac1c8a27b3` and
`-fb8df4e3730e7261`. Their order and privacy come from the original metadata of
the fixed reference; they are not guessed from `core` alone. Each entry binds
both the installed `.rmeta` and matching `.rlib`.

The original enum in
[`rustc_crate_store/src/cstore.rs`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_crate_store/src/cstore.rs)
defines `MacrosOnly`, `Conditional` and `Unconditional` in ordinals 0, 1 and 2.
The original
[`Svh` display](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_data_structures/src/svh.rs)
prints its fingerprint as a 128-bit hexadecimal integer; the
[`Fingerprint` serialization](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_data_structures/src/fingerprint.rs)
uses little-endian bytes. The receipt converts that representation and checks
it directly against bytes 24–39 of each metadata header.

The pinned metadata format version is 12, and its exact compiler version string
is `rustc 1.102.0-nightly (5b7b37aef 2026-11-08)`. Header magic, compiler version,
SVH and both root offsets are checked independently of the textual root output.
The original root reader and layout are recorded in
[`rustc_metadata/src/rmeta/decoder.rs`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_metadata/src/rmeta/decoder.rs)
and
[`rmeta/mod.rs`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_metadata/src/rmeta/mod.rs).

## Refusals and preserved earlier failures

All 14 focused [construction controls](../../../../tools/test-metadata-sdk-receipt.py)
passed. They cover unknown, duplicate or reordered dependency rows; malformed
root/version/hash/offset data; an SDK path escaping through a symlink; altered
artifact bytes; an unpinned SDK before any compiler command; failed command
status with binary-valued stdout retained and one observed attempt; refusal of
a user-source argument; and preservation of an existing output directory.
These are evaluator/construction controls, not native compiler coverage.

The initial control run had one wrong expected hexadecimal spelling of the
reference's original decimal StableCrateId. The parsed tool value was correct.
That failed run and its test source remain in the record; the corrected control
compares the same original decimal identity with its exact hexadecimal value.

An earlier direct original-reader attempt on the installed `core` **rlib**
panicked with `MemDecoder exhausted`. Its original backtrace is preserved.
This SDK ships split metadata: its core rlib is 3,039,568 bytes and its companion
rmeta is 54,594,630 bytes. The original reader succeeds on that rmeta, which is
the explicit input used by the construction tool. There is no failed-read retry
inside the tool and no alteration of the SDK artifacts.

## Preservation and boundary

[records.tar.gz](records.tar.gz) contains construction commands, status records,
stdout/stderr, the fixed reference source, all control results, and the earlier
original-reader observations. Public records normalize only the local SDK,
source and output directory prefixes. [record-identities.json](record-identities.json)
retains original and published byte counts and hashes for every record.
[verification.json](verification.json) states the exact run and its limits.
The receipt itself has no local absolute paths and is preserved byte for byte.

The SDK and generated reference binaries are not copied into the repository.
Their identities remain in the receipt and original command records. The tool
has no production caller or Cargo product dependency in this commit. A host
adapter may validate its installed receipt once and supply immutable dependency
facts to the native serializer; it must not call this construction tool on a
user request. This SDK identity proof does not establish consumer declaration,
privacy or lint completion, native instance ownership, valid produced metadata,
retained integration, full Rust coverage or release admission.

## Standard-prelude scalar libraries

The pinned SDK's original reader reported 19 dependencies for the unchanged
standard-prelude scalar provider. The corresponding `no_std` provider has
two. The standard root has no missing weak language item. The explicit
`no_std` root has the existing missing `EhPersonality` item. The
[standard receipt](standard-receipt.json) records the observed identities,
order, conditional dependency, and artifact hashes.

Construction verifies both original artifacts for every standard dependency
and installs the small receipt. Native requests use its data in the existing
metadata encoder and decoder. The scalar machine code comes from Lust's
checked bodies. The added source support retains actual pattern binding IDs
and resolves immutable integer locals through the native checker.

The installed compiler contract checks a standard-prelude provider and
application through unmodified default Cargo check, build, and run. A
separate pinned Rust invocation links against the genuine native provider
archive and metadata. The scope covers the represented scalar leaves.
