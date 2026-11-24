# Selective metadata serializer dependency cut

The exact pinned SDK successfully checked the serialization library for
`x86_64-unknown-none`, where a `std` dependency cannot compile. Its normal
dependency graph is `core`/`alloc` compatible. Receipt JSON, filesystem access
and SHA256 artifact verification live in the separate host SDK adapter.

Four focused controls passed:

- The independently compiled original scalar reference has the same stable
  crate identity.
- Original `rustc-stable-hash` agrees with the core namespace extraction over
  segmented inputs crossing its short/buffer boundaries.
- The POSIX path adapter agrees with the original host `Path`/`OsStr` hashing,
  including repeated separators, dot/parent components and Unicode names.
- The extracted ODHT writer has identical complete bytes to the unchanged
  original crate through colliding hashes, replacements and growth. The
  original independent decoder retrieves the same values.

The five recorded commands also check the host adapter and preserve both
normal/build dependency trees. The encoder's original hash/ODHT crates appear
only as development oracles; they do not enter the bare-target library graph.

[verification.json](verification.json) binds every source input, the exact tools,
commands, environment and raw streams. Source hashes were unchanged across the
checks. The original operation provenance and licenses are recorded with the
[encoder](../../../../crates/lustr-metadata-encoder/source-slices.json).

Review then replaced the initial scalar group scan with the original applicable
SSE2 group match/empty operations and original word fallback/probe sequence.
[simd-followup/verification.json](simd-followup/verification.json) preserves the
new exact sources and repeated bare-target plus four original-oracle results.
The preceding record remains the actual earlier source checkpoint; its bytes
and checks are not attributed to this follow-up. Preserving those upstream fast
operations is a source-level decision, not a new latency measurement.

This evidence covers the serializer dependency cut and the named original
operations. It does not prove a complete compiler, downstream Rust acceptance,
new metadata output targets, compiler-wide `no_std`, retained integration or
socket independence. These commands are correctness/build controls, not compiler
latency measurements. The admitted metadata target remains Linux x86-64.
