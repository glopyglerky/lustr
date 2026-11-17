# Larger public slice and incremental Cargo comparison

`tools/run-upstream-slice-benchmark.py` adds a separate campaign on the pinned
upstream `goport-util-small` source. It compares pristine Lust, integrated
Lust, the packaged Lustr entry, and matching Cargo/rustc with incremental
compilation enabled. It does not change historical scalar-corpus records or
reuse their timings as results for a larger program.

The packaged entry lane invokes standalone `lustr build/check SOURCE`. It is
not the mixed Cargo path. The pinned slice's functions live in nested modules,
outside the mixed bridge's current root-level scalar boundary. This campaign
therefore cannot establish a Cargo acceleration result. The separate
[mixed Cargo campaign](../../crates/lustr-bench/src/bin/lustr-mixed-bench.rs)
measures that path and its real source edits.

## First hosted attempt and recorder correction

The first hosted attempt at `6bf6e3e` failed before timing. Its
[original archive and diagnosis](../../performance/slices/2026-11-17-6bf6e3e/README.md)
retain all 98 files and zero measured samples. The daemon lacked `LUST_STATS=1`;
setting it only on the client cannot enable the daemon's diagnostic renderer.

The corrected runner enables statistics on the untimed qualification daemon
and both qualification requests. Measured commands and daemons remain
uninstrumented, which replay now verifies. It accepts the upstream's exact
`reuse/reuse/-`, `bodies/patch/-` and `bodies/chunks/bodies` retained routes,
while retaining all zero-repeated-work checks. Its 27 tests include compiling
and running the actual pinned statistics reader/renderer. They do not establish
a completed daemon campaign; fresh hosted execution is required.

## Corpus and exact work

The input is `compiler/native/bench/slices/goport-util-small`, preserved from
Lust `240e08d91d4aed0299639f6f11fd522912b248f9`. The runner compares its complete
inventory and the upstream driver with the same pinned pristine checkout
before copying either. These are upstream's pruned ts-rust sources and
documented dependency stand-ins. They are not the complete, unchanged ts-rust
application. Attribution, source changes and removed items remain in the
parent [slice README](../../compiler/native/bench/slices/README.md), the slice's
`spec.json` and `prune.json`.

Preparation changes the one empty `main` to call the existing upstream bench
driver. It adds that driver and an isolated Cargo manifest. All other source
bytes stay unchanged. The manifest explicitly sets edition 2024, dev opt-level
0, debug information 0, panic unwinding, overflow checks, debug assertions and
Cargo incremental compilation. Native commands use TinyCC's dev compilation,
omit `-g` and pass unwind/check settings explicitly. This does not establish
optimization equivalence between TinyCC and LLVM.

The target is x86_64 Linux. The Rust reference must match the imported SDK
commit `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. Native binaries and the
entry come from a source-built release composition; pristine Lust has a
separately retained release build log. The workflow's build receipts and
binary hashes provide provenance, not a proof of reproducible binary builds.

| Scenario | Work and cache state |
| --- | --- |
| Cold check | New lane directories; native daemon and disk compiler caches disabled; new Cargo target and intermediate directories. |
| No-op check | Same unchanged source after excluded priming; live native daemon versus a Cargo artifact reporting `fresh: true`. |
| Cold build | A separate new set of lane directories; linked executable produced by each compiler. |
| No-op build | Unchanged source after excluded priming; retained native daemon versus an actual Cargo Fresh result. |
| Compile plus run | Sum of an unchanged-source build command and execution of its verified artifact. Recorder work between the two spans is excluded. |
| Runtime | Only execution of the already-built driver, bound to the successful build's executable hash and identical-output gate. |

These are not body edits, signature edits, or timestamps presented as edits.
Cold describes compiler-owned state. OS page caches and installed dependency
sources are not flushed. The reference is one explicit Cargo setup, not the
fastest selected result across arbitrary rustc flags.

## Correctness precedes timing

All four compilers must build the slice and run the upstream driver's path
operations successfully. The matching Cargo executable supplies the runtime
oracle. Each other executable must produce the exact same stdout, empty
stderr and exit status 0. Every timed successful build is followed by the
same runtime comparison outside compile-only timing. Runtime records bind
the command to the generated executable's hash.

A separate type-mismatch fixture must fail for the expected diagnostic, not
because a tool is missing, a process crashed, or a command timed out. This
small negative gate does not claim rejection coverage for every slice
construct. Initial and final qualification use separate state directories.

The isolated runtime row executes a short path-operation driver. Its command
wall time includes process startup and exit, so it is not a sustained application
throughput or full ts-rust runtime benchmark. The driver and its exact output
are retained; neither an empty program nor a successful compile supplies the
runtime result.

Untimed native diagnostics must identify a retained check with no repeated
body or code-generation work. The accepted diagnostic forms mirror the
existing `crates/lustr-bench/src/native_evidence.rs` contract. Packaged-entry
traces must identify the native engine and the expected cold-process or
daemon route. Instrumentation stays disabled in measured commands.

The runner starts real `bin/lust daemon` processes for pristine and candidate
lanes. It measures `bin/lustr` for the packaged-entry lane. `lustr daemon` is
not an assumed product command. Every owned daemon must remain alive for its
warm requests and exit successfully after its private socket is removed.
Only runner-created state is used. A socket or shutdown failure fails the
campaign; it cannot silently become a cold sample.

Cargo's machine-readable artifact must identify the selected binary and the
declared dev profile. Cold commands require `fresh: false`; no-op commands
require `fresh: true`. Source inventories and tool hashes must remain stable
through final qualification. The full fixture, command vectors, original
stdout/stderr, subprocess failures, build receipts and selected host facts
are retained. Inherited environment variables are not copied into commands.

## Statistics and retention

The default twelve rounds yield **288 timed observations and 18 comparisons**
across six scenarios and four lanes. The compile-plus-run observation
contains two retained command records. Excluded priming, qualification and
runtime checks remain recorded separately.

Each four-round block runs every cyclic rotation of a seeded shuffled lane
order. Every lane occupies each execution position three times in twelve
rounds. This balances positions; it does not eliminate shared-host noise.

The runner imports the existing strict reporter's `summarize` implementation.
It keeps paired geometric baseline/candidate ratios, conditional bootstrap
95% intervals, medians, and nearest-rank p95s. With twelve observations, p95
is the maximum. It does not substitute a ratio of medians or remove slow
samples. Statistical intervals are exploratory, conditional on these pairs,
and do not prove performance equivalence or release readiness.

`report.json` starts failed. A missing result, mismatched artifact, runtime
disagreement, timeout or failed final gate leaves it failed with no speed
table. Partial observations and original failures remain available in
`raw.jsonl`, `commands/`, and `samples.csv`. Failed work is not a completed
campaign. Build caches are retained under `work/`; they are not needed to
review the recorded output. Archive the original report directory or the
explicit report, fixture, command, receipt and oracle files without claiming
that omitted caches or binaries were preserved.

Replay validates retained source and output hashes, both qualification stages,
daemon shutdowns, Cargo Fresh results, runtime/build identities, every timed
command and the generated table. It reuses the same existing estimator and
never executes a command from the log:

```bash
python3 tools/run-upstream-slice-benchmark.py --replay /absolute/retained-campaign
```

Replay does not freshly build the compiler or restore omitted executable
binaries. The original campaign must also pass its own replay before the
runner returns success.

## Execute in the source-built workflow

Run after the existing source-toolchain and pristine release-build steps:

```bash
python3 tools/test-upstream-slice-benchmark.py
python3 tools/run-upstream-slice-benchmark.py \
  --repository "$GITHUB_WORKSPACE" \
  --pristine-source "$RUNNER_TEMP/lustr-pristine-source" \
  --pristine-lust "$RUNNER_TEMP/lustr-pristine-target/release/lust" \
  --pristine-build-log "$RUNNER_TEMP/lustr-pristine-build.log" \
  --built "$RUNNER_TEMP/lustr-source-built" \
  --output "$RUNNER_TEMP/lustr-public-slice" --pairs 12
```

The output directory must be new, absolute, and outside the source/toolchain
trees. Each child has an explicit timeout and bounded recorded output. Preserve
the output on success and failure. Keep this campaign separate from the
schema-1/2 scalar and mixed-Cargo archives.

Local feasibility checks can use development binaries, but their timings
cannot satisfy the release-composition contract. Environments that refuse
Unix socket binding cannot qualify the retained native path; the actual
failure must be kept and the complete campaign executed elsewhere. No speed
result is implied by adding this runner.
