# Lustr compiler performance

**The full acceleration goal is not complete.** The current toolchain can use the
imported Rust implementation for complete-language requests and demonstrates
bounded native/mixed compilation. Neither source availability nor a green CI
characterization job establishes Lust-equivalent speed for full projects.

Start with **[the complete measured tables](RESULTS.md)**. The root README shows
selected decision-relevant rows; the full report keeps every mode, slower result
and uncertain comparison. No upstream speed ratio is reused as a Lustr result.

## Evidence index

| Campaign | Candidate head | Evidence and reproducible tables |
|---|---|---|
| 2026-11-17, run 44056401150 | `c70eb1c25085884622a95666021fce0528e7325b` | [First completed paired campaign](runs/44056401150/README.md) |
| 2026-11-17, run 44063047987 | `afb17a8af2cce9facdf03ba6cf12b8533cb3a171` | [Second completed campaign](runs/44063047987/README.md) |

Each directory retains the **original, hash-verified qualification ZIP**, its
original report and raw JSONL, a convenient paired CSV, immutable source/run
identities, and generated Markdown. The ZIP preserves command logs, source
fixtures, probes, correctness/build records and other original evidence after
GitHub's temporary artifact retention expires. Do not execute binaries or import
caches from an evidence archive. These are data for inspection, not a toolchain
installation.

The two campaigns are separate observations, not pooled samples. Their heads
have the same compiler sources; the later commit adjusts the macOS CI utility
environment. They are useful repeat observations, not a compiler optimization
before/after comparison. Do not assume that separate hosted runners have equal
hardware, frequency or competing load.

Historical failed campaigns remain relevant:

| Run | Outcome and meaning |
|---|---|
| 43965806036 | The expected stale-output failure reproduced, then missing `rg` stopped qualification. No completed speed campaign. |
| 44046766887 | Correctness passed; an overstrict probe rejected pristine Lust's zero-body-work `mode=bodies` route. No paired timing campaign completed. |
| 44046767054 | Native macOS could not link Linux's `personality` syscall. This was not a measured speed regression. |
| 44056401176 | macOS linkage was repaired; native shell tests then encountered Bash/GNU-utility mismatches. Separate platform qualification, not speed admission. |

## What is compared

| Family | Baseline | Candidate | Question answered |
|---|---|---|---|
| Native integration | Pristine upstream `lust` | Integrated `lust` | Does the reviewed upstream patch change supported-work cost? |
| Native user entry | Pristine upstream `lust` | Packaged `lustr` | What does the product entry add, including rejection/full-compiler routing? |
| Mixed Cargo project | Matching original Cargo/rustc | Packaged mixed `lustr` | Does the actual combined Cargo journey outperform the complete compiler? |

Do not compare native scalar times directly to mixed Cargo times: these are
**different workloads and boundaries**. Cross-family division would produce a
misleading speedup. The `integrated` lane is not the complete mixed compiler.
The historical Cranelift lane is not Lust acceleration and has separate records.

The native fixture contains a scalar kernel, `main` and a real test. The mixed
workspace adds a procedural macro, async caller and a `no_std` library using
`alloc`. Actual results and invalid-source rejection are verified. This is a
small real Cargo graph, but it is not a representative large-application corpus.

Nine modes are kept separate: cold build, no-op build, body edit, signature edit,
check after prior edits, unchanged check, body-edit check, invalid-program
rejection, and compile-plus-test. The `check` row is not a fresh dependency-free
cold check. Cargo no-op measures Cargo freshness/dispatch and may invoke no
compiler. Compile-plus-test includes test execution; post-build value assertions
are outside compile-only timings.

## Profiles, caches and measurement boundary

The compiler implementations are built using their recorded release profiles.
The application fixtures use development opt-level 0 with debug information
disabled; normal unwind and required checking behavior are retained. These are
**not default-debug Cargo-build or optimized release-application speed claims**.
Ordinary Cargo-check correctness and its freshly encoded metadata are separate
qualified behaviors.

Each cold pair receives fresh target, intermediate, mixed, native and XDG cache
directories. Native cold commands disable compiler disk/daemon caches. The OS
page cache is not flushed, so "cold" does not mean a freshly booted machine.
Warm modes use the same lane's retained caches and real daemon. Setup, priming,
mode probes, source hashing and summary generation are not timed samples.
Measured child wall time includes launching the requested command, opening its
captured output streams and waiting for its exit using 100-microsecond polling.
That polling and scheduler noise matter for millisecond-scale work.

Counters/traces are enabled in separate diagnostic probes and disabled during
measurements. A zero-body-work route can still perform collection, parsing,
metadata work or linking. The probe keeps that route distinct from direct reuse;
no counter label itself proves a speedup.

The original two campaigns record compiler/build identities, target, profiles,
commands and cache scope, but do **not** record CPU model, RAM, affinity, clock
policy or simultaneous host load. This is a known evidence gap, not information
that can be reconstructed from `ubuntu-24.04`. The reporting tool provides a
restricted `capture` command for future before/after environment snapshots;
it deliberately excludes hostname, username, tokens and a full environment dump.

## Statistics and interpretation

For pair i, `r_i = baseline_seconds_i / candidate_seconds_i`. The reported speed
ratio is `exp(mean(log(r_i)))`. A value above 1 is faster; below 1 is slower.
Candidate latency change is `(1 / ratio - 1) × 100%`: positive means slower.
Do not call a 0.60x speed ratio a 40% latency increase; its inverse implies about
67% more time. The paired ratio is not the ratio of the displayed medians.

The runner uses seeded, position-balanced randomized order, and reports separate
medians, nearest-rank p95, all paired ratios, 10,000 paired bootstrap resamples,
per-mode percentile 95% intervals and an exact sign-flip probability when the
sample count permits. The implementation and tests are in
[`crates/lustr-bench/src/stats.rs`](../../crates/lustr-bench/src/stats.rs) and
[`order.rs`](../../crates/lustr-bench/src/order.rs). Seeded randomness makes the
analysis reproducible; it does not make shared-host timing deterministic.

With twelve samples, nearest-rank p95 is the maximum observation. It is not a
well-estimated production tail. Bootstrap validity depends on the suitability
of treating pairs as independent observations. Per-mode intervals are
exploratory: 27 comparisons do not receive a simultaneous-coverage or
multiple-testing guarantee. An interval crossing 1 means **inconclusive**, not
"same speed". No average is allowed to hide a failing edit or rejection mode.
No performance or release admission is granted by the characterization runner.

## Independent evidence replay

The standard-library-only report tool checks the archive SHA-256, exact
head/merge/tree, build receipt hashes, raw JSONL digest, every command's output
hash and expected status, unchanged paired inputs and sample completeness. It
recomputes every summary from raw timings, including the seeded bootstrap and
sign test, rather than trusting a stored label. A failed, partial, altered or
inconsistent campaign is rejected; no fabricated zero or missing row is shown
as a successful measurement.

After checkout, replay the committed evidence without network access:

```sh
python3 tools/test_performance_report.py
python3 tools/performance_report.py replay --root docs/performance --check
```

To import an additional completed campaign, add its exact run, head, actual
checkout, artifact ID and API-reported SHA-256 to `campaigns.json`, then run the
fetch command in an authorized environment with an Actions-read token. The token
stays in the environment, is never printed or stored, and is removed from
redirected artifact requests. Review all generated changes before merging.

```sh
python3 tools/performance_report.py fetch --root docs/performance
python3 tools/performance_report.py replay --root docs/performance --check
```

Importing/replaying evidence does **not** rerun the compiler. Actual builds and
benchmarks remain separate executed operations.

## Reproduce fresh compiler measurements

Use the source-built job in
[`.github/workflows/compiler-set.yml`](../../.github/workflows/compiler-set.yml)
at the selected exact source. It is the authoritative construction recipe,
including the pinned SDK, pristine Lust, TinyCC, dependency locks and profiles.
Do not substitute an old installed package or debug-built compiler.

The campaign pins upstream Lust `240e08d91d4aed0299639f6f11fd522912b248f9` and
Rust source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` (the matching
`nightly-2026-11-09` SDK). Pin values belong to the campaign, not an assertion
that these are today's latest upstreams. Build both Lust lanes against the same
TinyCC and compiler profile. Preserve requested application features, dependencies,
panic strategy, assertions, profiles and build scripts.

After the workflow's exact toolchain-construction and correctness steps, the
measurement command is:

```sh
cargo build --manifest-path crates/lustr-bench/Cargo.toml \
  --release --locked --bin lustr-mixed-bench
python3 tools/performance_report.py capture --output /absolute/host-before.json
/absolute/bench-target/release/lustr-mixed-bench \
  /absolute/pristine-lust-source /absolute/pristine-build/release/lust \
  /absolute/source-built-output /absolute/new-measurement-directory 12
python3 tools/performance_report.py capture --output /absolute/host-after.json
```

Set the build's target directory to `/absolute/bench-target` when using that
example path. The output directory must not exist before the campaign. Valid
pair counts are multiples of six from 12 through 96; more pairs are not a
substitute for a representative corpus or controlled hardware. Keep raw failed
runs too, without listing them as completed timing campaigns.

## What remains to establish the speed goal

| Area | Present evidence | Remaining obligation |
|---|---|---|
| Supported native paths | Two small-fixture campaigns; all modes retained | Representative unchanged supported projects, longer edit sequences and stable host repetitions |
| Product entry | Direct comparison to pristine Lust, including costly rejection | Attribute and remove unnecessary overhead without skipping correct fallback or diagnostics |
| Mixed Cargo | Actual proc-macro/async/no_std graph, retained edit probes | Broader eligible functions, real projects and improved measured full-command time |
| Large scaling | Not covered by these campaign tables | Small/medium/large crates; body, type, trait, macro and dependency edits; record work counters |
| Release profile | Imported compiler provides completeness | Matched release-build latency, runtime, size and profiling semantics |
| Runtime speed | Value and test-result assertions, not runtime throughput | Separate repeated execution benchmarks with equivalent optimization and semantics |
| Memory/CPU/energy | Not measured in these two campaigns | Peak/cumulative process and daemon accounting; independent CPU/resource runs |
| Platforms | Linux x86-64 timings; separate compatibility/layout checks | Real native macOS, Windows and other promised platform timing/behavior |
| Incremental safety | Bounded signature/dependency/macro invalidation cases | Broader invalidation, restart, failure, cancellation and concurrency coverage |

The immediate optimization targets are the measured mixed cold/edit/signature
costs and native-entry rejection latency. Their causes must be isolated using
phase traces or profiling outside timed samples. Importing full Rust removes the
need to rebuild every language feature; it does not remove process, parsing,
macro expansion, context hashing, metadata or linker costs automatically.

## How this relates to upstream Lust

The pinned [upstream README](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md)
separates cold checking, warm edits, daemon memory and generated-program runtime.
It discloses adapter substitutions, feature differences, host variability and
slower unoptimized executables. Lustr follows that reporting discipline rather
than copying its ratios. The original mechanisms—retained frontend state,
precise dependency invalidation, parallel per-function work and lightweight
native code generation—are the intended acceleration baseline.

Official Rust [performance-testing guidance](https://rustc-dev-guide.rust-lang.org/tests/perf.html)
compares exact compiler versions over workload/configuration matrices.
[Cargo timings](https://doc.rust-lang.org/cargo/reference/timings.html) help expose
compilation units and scheduling; they do not measure every internal compiler
phase or establish semantic equivalence. Existing historical Lustr/BlueFe
measurements under `docs/compiler-set/measurements` remain historical, separately
scoped evidence, not results for the current mixed compiler.
