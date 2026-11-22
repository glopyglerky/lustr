# Cargo artifact-path evaluator correction

The public-library campaign on `dae72eddb614a7b855212fd9656013f5d0fc82c6`
failed after its first successful Cargo/native build, before any independent
consumer. Its assertion required a delivered rlib below `target`. The original
failure remains failed: zero completed public histories, consumers and runtime
assertions. This evaluator correction does not change the compiler or qualify
that campaign retroactively.

The original [failed job](https://github.com/glopyglerky/lustr/actions/runs/44720879498/job/136235971869)
and its public-native-library artifact `14621004066` were inspected on
22 November 2026. The downloaded ZIP matches GitHub's SHA256
`3a3aab0aaf67e60ed39901887eeb19019df552332e7e61de42b63e2288b4fccd`.
Its source receipt records merge checkout `7714f4683a7f175fea5bb486d9247bf755858f69`
with the same tree as the PR head, `e470676112c43d5ebfc79aa3f62a5b0602df640b`.
The original Cargo JSON reports a successful non-fresh build and both actual
intermediate filenames. No original consumer executed before the assertion.

## Pinned upstream behavior

Cargo `c07ac2c4d12237effc4b94f4543769b421790045` explicitly
[disables rlib uplift for nightly Cargo and rustc](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/build_runner/compilation_files.rs#L469)
unless its temporary opt-out is set. Rmeta is never uplifted by that path.
The [channel and opt-out predicates](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/workspace/features.rs#L1603)
use the actual Cargo/rustc channel and
`__CARGO_TEMPORARY_STOP_RLIB_UPLIFT_OPT_OUT`. The campaign constructs an explicit
environment without either test override. Cargo's
[artifact-message producer](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/mod.rs#L724)
therefore reports the original intermediate paths.

An independent Linux x86-64 run used the unchanged original SDK: Rust
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` and the Cargo revision above.
A dependency-free edition-2024 no_std library, with the campaign's development
profile and separate target/build directories, compiled successfully. Cargo
reported both `.rlib` and `.rmeta` in its intermediate output directory.
Changing its u64 function to return bool then produced the expected compiler
failure (Cargo exit 101). Both previous intermediate pathnames had been removed
by Cargo. No candidate compiler or generated application ran in this witness;
it supplies neither candidate coverage nor timing evidence.

## Exact checks retained

The evaluator now requires exactly the two expected Cargo filenames, each
resolving to its actual requested compiler output with identical bytes. A copied
archive below `target`, even with identical bytes, is rejected. Missing,
duplicate, extra, wrong-target and executable artifacts are rejected.

Before subsequent Cargo requests, read-only descriptors retain the actual
previously delivered archive and metadata inodes. Their contents, size, inode
and modification time must remain unchanged across the request, including when
Cargo unlinks their pathnames. This preserves an independent observation of the
old publication rather than inventing a target-directory copy. Existing checks
still require surviving named outputs to match, exact native rejection, no
success/artifact notification on error, zero artifact work for rejection/no-op,
and the original direct-request output-preservation assertions. Descriptors
close in the campaign's existing final cleanup.

All 16 existing evaluator controls remain. Six added controls cover the real
unlifted pair, identical-byte path substitution, malformed artifact inventories,
unlink/replacement of the observed pathname and in-place mutation of the held
inode, including equal-length changed bytes with restored modification time
and a buffered caller's unchanged position. All 22 controls passed locally. The nine Cargo histories, four eviction
replays, four metadata-option replays, 16 original consumers and 4,194,432
required runtime assertions remain unchanged requirements. A fresh complete
public campaign on this source remains necessary; no passing campaign, full
Rust coverage, performance result or release is claimed here.
