# Direct packaged-native comparison with matched rustc

The [fixed-corpus runner](../../crates/lustr-bench/src/bin/lustr-mixed-bench.rs)'s schema 2 adds the missing **matched standalone rustc
→ packaged native entry** comparison on the identical supported scalar source.
It keeps the existing pristine→integrated Lust, pristine→entry and matched
Cargo/rustc→mixed comparisons. The existing recorder and statistical method
remain unchanged. The six retained schema-1 campaigns have no such native
rustc lane and cannot be reinterpreted as measurements of this comparison.

Implementation and local gates are distinct from a completed hosted campaign.
Fresh CI must construct the whole toolchain, pass all existing correctness
prerequisites and execute the extended measurement before any new ratio is
published.

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
complete Rust SDK as the mixed compiler. Its command flags are:

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
