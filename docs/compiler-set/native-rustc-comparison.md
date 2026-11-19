# Direct packaged-native comparison with matched rustc

> Historical comparison of the recorded mixed package. The fixed-corpus
> comparator, independent oracle and original evidence remain available. Its
> required `rustc-full` package layout is retired and does not qualify the
> current exclusive native compiler or the unfinished full replacement.

The [fixed-corpus runner](../../crates/lustr-bench/src/bin/lustr-mixed-bench.rs)'s schema 2 adds the missing **matched standalone rustc
→ packaged native entry** comparison on the identical supported scalar source.
It keeps the existing pristine→integrated Lust, pristine→entry and matched
Cargo/rustc→mixed comparisons. The existing recorder and statistical method
remain unchanged. The six retained schema-1 campaigns have no such native
rustc lane and cannot be reinterpreted as measurements of this comparison.

The first complete schema-2 campaign executed at `5a3f493`, actual checkout
`b3393a4a2907228516546cd34f1a5c60aaf6aaaa`, in
[source-built run 44111326777](https://github.com/glopyglerky/lustr/actions/runs/44111326777/job/134137398335).
That job constructed the toolchain, passed the existing correctness prerequisites
and completed all 648 timed commands and 36 comparisons. The
[original archive](../../performance/archives/2026-11-17-5a3f493-qualification.zip)
and [complete table](../../performance/2026-11-17-5a3f493.md) record the actual
result. Source-built qualification and these bounded timing
results do not supply a complete native-gate, all-platform or release pass.

## First measured result

All values below come from that single campaign. Medians are milliseconds;
ratios are paired geometric means of rustc time divided by packaged Lustr time,
with conditional 95% intervals. They are not ratios of the displayed medians.

| Mode | Matched rustc median | Packaged Lustr median | Ratio [95% interval] |
| --- | ---: | ---: | --- |
| Cold build | 38.147 | 40.285 | 0.914 [0.876, 0.950] |
| No-op build | 37.982 | 3.106 | 12.208 [12.037, 12.383] |
| Body edit | 38.224 | 5.590 | 6.894 [6.797, 7.004] |
| Signature edit | 38.424 | 4.856 | 7.935 [7.759, 8.131] |
| Check | 11.619 | 2.069 | 5.621 [5.446, 5.812] |
| No-op check | 11.356 | 1.778 | 6.426 [6.267, 6.605] |
| Check after body edit | 11.285 | 1.882 | 5.970 [5.797, 6.179] |
| Rejection | 17.038 | 18.016 | 0.945 [0.930, 0.959] |
| Compile plus test | 53.267 | 30.960 | 1.740 [1.705, 1.776] |

**Cold build and rejection favor rustc.** The seven other intervals favor the
packaged native entry on this supported scalar corpus. Warm native state is
being compared with new nonincremental rustc invocations, as specified below.
These ratios do not describe Cargo Fresh, arbitrary Rust projects or mixed
Cargo acceleration. The same campaign records slower mixed Cargo body and
signature edits; the [four-panel chart](../../performance/plots/2026-11-17-5a3f493/README.md)
retains every contrast and regression.

## Exact work and compiler settings

All four native lanes consume the same `native_source` bytes for each matched
mode/sample. They target Linux x86-64, edition 2024, unoptimized development
code without debug information, ordinary unwinding, overflow checks and debug
assertions. Pristine, integrated and entry commands explicitly select the
target and panic/check settings for build/check. Their test command uses the
native harness's `DEV_UNWIND` defaults with `debug: false`; that contract is
checked against the pinned [protocol defaults](../../compiler/native/src/protocol.rs)
and [native harness](../../compiler/native/src/testrun.rs).

The reference is the constructor's `rustc-reference`, using the same pinned
complete Rust SDK as the mixed compiler. The first campaign's
[recorded identity](../../performance/plots/2026-11-17-5a3f493/plot-data.json)
is Rust 1.102.0-nightly, commit `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`,
with LLVM 23.1.5. The separate native correctness gate's Rust 1.99.0 oracle
is a different reference. The measured command flags are:

```text
--edition=2024 --target=x86_64-unknown-linux-gnu
-C opt-level=0 -C debuginfo=0 -C panic=unwind
-C overflow-checks=on -C debug-assertions=on
```

Build adds `--emit=link`; check adds `--emit=metadata`. Output paths are owned
by the lane. Check therefore includes rustc's metadata emission. Runtime
assertions after ordinary builds remain outside compile-only wall time.

Compile plus test includes **both** `rustc --test` and actual execution of the
resulting binary with `--test-threads=1 --color never`. A small internal command
adapter performs those two commands under the existing recorder's single outer
wall-time measurement. Its own process overhead is included. Both original
child argument vectors, exits and signals remain in stderr receipts alongside
compiler/test diagnostics. A failed compile never executes or retries a test.
This adapter adds no clock, timing sample or statistical estimator.

## Cache and no-op meaning

The standalone rustc lane is explicitly **nonincremental**, its ordinary direct
command behavior. Every no-op invokes rustc again. It is not Cargo's `Fresh`
decision and is not a claim about the fastest possible rustc configuration.
The separate Cargo reference remains the comparison for actual Cargo behavior.

Lust lanes retain their original native daemons and same-lane caches for warm
modes. Native cold disables daemon/disk compiler caching. Each pair owns fresh
lane state; OS page caches are not flushed. Rustc receives no incremental
directory or persistent compiler daemon. These differences are part of the
explicit tool contract, not hidden cache equivalence.

## Coverage, gates and identity

Schema 2 has four native lanes and two Cargo lanes, all nine existing modes,
and twelve matched observations at the default setting: **648 timed commands
and 36 paired summaries**. Counts must be a multiple of four between 12 and 96,
so complete randomized cyclic blocks cover the four-lane schedule. Historical
schema-1 recordings remain five-lane, 540-command, 27-summary campaigns.

The new reference gate requires accepted checks/builds, runtime output,
executed passing tests, invalid-source rejection, executed failing tests and
successful recovery in both the entry and rustc lanes. Passing/failing test
counts are checked explicitly, including zero ignored/filtered tests. Existing
native retained-state probes still apply only to the three Lust lanes; no
native daemon or retention claim is invented for rustc.

All compiler and adapter executables are hash-identified. Every native compiler
receipt binds to its lane's declared executable path, including untimed retained
probes. Exact source, operation and flag contracts cover timed native commands
and the new reference gates; they do not extend to Cargo argument validation or
the older untimed retained-probe argument lists. The reporter checks the
two-child rustc test receipts, verifies every original stdout/stderr digest,
requires identical paired source, exact lane/mode/sample coverage, order
balance and complete reference gates, then replays the existing statistics.
Missing samples, changed flags, substituted executables and unexecuted tests
are rejected.

The [existing statistics implementation](../../crates/lustr-bench/src/stats.rs)
still computes the paired geometric mean of baseline/candidate time,
with the existing seeded 10,000-resample bootstrap. The sign-flip statistic is
exact at the default twelve pairs and through sixteen pairs; it is unavailable
above sixteen pairs.
New lanes do not create a new estimator or a pooled speed claim. The report
keeps all four contrasts, and the chart adds a fourth panel. Historical CSV
and Markdown bytes reproduce unchanged under the version-aware reporter.

## Reproduction and interpretation

The [source-built workflow](../../.github/workflows/compiler-set.yml) invokes the
existing runner after its complete compiler gates. For a separately built and
qualified toolchain, the command shape remains:

```bash
/absolute/lustr-mixed-bench \
  /absolute/pristine-lust-source /absolute/pristine-build/lust \
  /absolute/source-built-package /absolute/new-measurement-directory 12
```

The output must be a new absolute directory. Use the exact source, matched
release components and pinned TinyCC construction described in the
[mixed compiler instructions](mixed-compiler.md#build-and-qualify). Report or
plot the resulting original ZIP with the existing strict tools; do not reuse
an older binary package as a new qualified candidate.

A direct supported-scalar result does not establish arbitrary-project speed,
general native Rust support, mixed-function speed, release-profile compilation
or generated-program runtime throughput. A faster native lane can coexist with
slower mixed Cargo work. Preserve both observations and their separate workloads.
