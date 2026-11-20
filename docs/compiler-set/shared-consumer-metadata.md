# Genuine consumer metadata: required completion service

> Source-bound dependency investigation. The later
> [native Cargo continuation](native-cargo-continuation.md) and
> [selective metadata serializer](../../crates/lustr-metadata-encoder/README.md)
> record bounded follow-through. The serializer and native consumer qualifier
> are not yet connected into a product metadata path. The original full-SDK
> encoder probe below is research evidence, not the complementary implementation.

The shared definition service does not yet produce Rust metadata for native-owned
consumer bodies. Inspection at repository commit
`bd1d2d0067decc1771eed98884e64ff1fd12b3d4` and a direct experiment against the
declared Rust source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` identify the next
required contract. No analysis query was replaced with an unconditional success.

## Observed encoder dependency

The [original probe and record](evidence/shared-metadata-dependency-20261119/verification.json)
call `rustc_metadata::fs::encode_and_write_metadata` after expansion on a
`no_std` library containing one ordinary, nongeneric function over `u64`.
Consumer `typeck_root` and complete `analysis` queries remain panic traps.
The encoder demands `analysis` before completing metadata, the probe exits 101,
and no requested `.rmeta` is produced. This is a dependency experiment, not a
compiler acceptance or metadata qualification result.

The exact pinned source path is:

1. `rustc_metadata/src/rmeta/encoder.rs`: `encode_crate_root` constructs
   `LocalDefsRemapping`; `encode_def_path_table` also reads the definition table.
2. `rustc_metadata/src/rmeta/mod.rs`: `LocalDefsRemapping::new` calls
   `tcx.definitions()`.
3. `rustc_middle/src/ty/context.rs`: `TyCtxt::definitions` requires
   `analysis(())` before freezing definitions.

The evidence records the original probe source, compiler identity, commands,
backtrace and complete upstream source-file hashes. `$SDK` and `$PROBE` in the
public command record denote the exact SDK and privately owned probe directory;
the original command record's hash is preserved.

## Required integration

A passing native `PreparedRequest` alone cannot satisfy this completion query.
The original analysis in `rustc_interface/src/passes.rs` also establishes
declaration, attribute, visibility, privacy, lint and diagnostic obligations.
Some original privacy and late-lint checks demand typed Rust bodies. The cold
native path has no joined completion proof for those obligations. The existing
mixed backend's retained-certificate path is not such a cold proof.

The next service must own the exact accepted local definition/body inventory,
map it to original upstream identities, run applicable original declaration
checks, and supply remaining obligations from actual native checked facts.
Unrepresented obligations must remain explicit. An alternative source-owned
upstream seam could let the original encoder consume a validated completed
definition table without claiming original whole-crate analysis ran. Neither
route may fabricate `TypeckResults`, return unconditional analysis success, or
rerun the whole frontend solely to regenerate metadata facts.

After that prerequisite, the instance-graph interface is usable. The original
`exported_generic_symbols` query collects monomorphizations for rlibs even when
no generic export is ultimately found. A closed native library can supply a
real `MonoItemPartitions` containing its exact mapped and emitted
`Instance::mono` definitions. Keep the original exported-symbol query so it
derives its answer from that graph. Do not return a fabricated empty graph or
alter shared-generics/profile flags to avoid the query.

The first bounded qualification can cover nongeneric, non-const, non-inline
functions admitted from actual native checked capability. It must encode
genuine metadata, compose the exact native objects into a genuine archive, and
run a separately compiled matching-Rust caller. Retain traps proving no consumer
Rust body checking or MIR generation, plus invalid native source, uncovered
definition/body, symbol ownership, output failure, artifact preservation and
recovery controls. This experiment does not complete checklist steps 2 or 3.


## Retained native candidate, 20 November 2026

The separate [native metadata qualifier](../../crates/lustr-native-metadata-qualification/README.md#retained-native-candidate)
now has an authored retained mode using the real native Session, unchanged
source, existing semantic read validation and original per-function TinyCC
emission. Common completed facts feed the selective serializer without a second
consumer frontend. SDK facts, native and renamed objects, metadata and archive
bytes are retained at their appropriate generations. Prepared IR survives Check
and dropped continuations, and failed emission/publication cannot authorize old
machine code as the new source's result.

This is a pending source checkpoint, not a passing test or product admission.
The earlier source-bound Clang campaign remains unchanged. Exact new-source
hosted compilation, independent Rust-consumer results, narrow retained histories
and original traces are required. Public Cargo/library routing, general metadata
obligations, complete Rust coverage and speed qualification remain unfinished.
