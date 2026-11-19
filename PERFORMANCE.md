# Lustr performance: complete work and retained evidence

The [governing architecture](docs/compiler-set/architecture-plan.md) requires
one integrated compiler, with Lust for supported work and imported Rust for its
complement. These measurements describe their recorded implementations, including
transitional compatibility paths. A timing result does not qualify a coordinator
or whole-program retry as the completed compiler.

## Recorded snapshot, 18 November 2026

The [PR #26 evidence](performance/2026-11-18-43a7e07/README.md) contains the recorded
completed 29-row dashboard, all 59 comparisons, raw samples, original artifact
identities and independent replay. Warm public check/build medians are
1.407/3.333 ms for Lustr, 1.098/3.127 ms for pristine Lust and 8.443/8.623 ms
for matching Cargo/rustc. Ordinary full-Rust project medians range from 0.06%
faster to 1.04% slower than stock Rust. Mixed opt0/debug0 Cargo work retains
two wins and seven regressions.

These measurements retain their original source and historical interpretation.
They do not qualify the current exclusive compiler or a selective complement.
The [current performance policy](docs/compiler-set/practical-performance.md)
requires relative evidence for a new near-stock claim; the former 0.5 ms alternate
rule cannot excuse a large relative slowdown. Correctness, ownership and platform
release conditions remain mandatory. No historical failed gate, sample or
performance target below has been rewritten.

## Historical status on 17 November 2026

The latest completed paired Cargo campaign records **1.079× faster body-edit
builds [1.070, 1.088]** on the retained scalar fixture, and faster rejection.
Seven other Cargo modes are slower. Lustr is **not yet a generally qualified,
speed-preserving full-Rust compiler**. Cold builds, signature edits, checks,
normal-profile projects and broader native participation remain priorities.
Native warm results against nonincremental standalone rustc do not establish
Cargo acceleration. Upstream Lust's own headline ratios retain their own scope.

The latest fixed and project campaigns measure PR13 candidate
`6bf6e3e3384a99fbaeab2c66aa7c97bc73231996`, actual merge checkout
`5a3c5666a16d46629a4787e37ca9b20e65720ccc`, source tree
`02652f033e84c0957dd2b99eb562db8171289e50`. The package combines
[in-process symbol checks](docs/compiler-set/mixed-symbol-validation.md),
[validated in-process renaming](docs/compiler-set/mixed-symbol-rename.md) and
[direct entry into the Cargo coordinator](docs/compiler-set/cargo-dispatch.md).
It also corrects Cargo command/alias routing. The measurements below cover
the complete combined package. They do not isolate any one optimization.

In [source-built run 44179667793](https://github.com/glopyglerky/lustr/actions/runs/44179667793/job/134380277750),
target layouts, the unchanged failed-link negative control and recovery, mixed
execution, retained checks with fresh metadata, mixed libraries and all sixteen
entry/runtime tests passed. Original profiling, all 36 fixed comparisons and
the five normal-profile project scenarios also passed. The enclosing job still
**failed** its separate public-slice recorder before collecting timing samples.
The [original failure and correction](performance/slices/2026-11-17-6bf6e3e/README.md)
remain explicit. The corrected recorder has 27 passing tests; it needs fresh
hosted daemon execution. Seven other non-macOS jobs passed, macOS compatibility
was cancelled without executing steps, and the separate full native Linux gate
remains pending. This is not a release or all-platform pass.

The preceding `5a3f493` source-built job passed its original profile and both
campaigns. The prior `40d2ac3` job failed separate profiling when concurrent
writes joined two trace records. Its paired measurements and project campaign
had passed independently. The [original failure and diagnostic reconstruction](performance/2026-11-17-40d2ac3-profile.md)
remain retained alongside subsequent successful original traces.

The separate full native Linux gates at
[`40d2ac3`](docs/compiler-set/measurements/2026-11-17-native-linux-40d2ac3/README.md)
and [`5a3f493`](docs/compiler-set/measurements/2026-11-17-native-linux-5a3f493/README.md)
each failed after 5,527 ordinary passes and 5,526 sanitized passes, with one
`unicode_props_hash` generated-program execution timeout at the unchanged
ten-second deadline. Linking succeeded; fuzz and benchmark smoke were not
reached. The [bounded local diagnosis](docs/compiler-set/measurements/2026-11-17-unicode-sanitizer-diagnosis/README.md)
encountered a separate LeakSanitizer tracing failure and did not establish the
hosted timeout's cause. The current run uses two workers while preserving cases,
sanitizers and deadlines; a pending run supplies no passing gate.

Earlier complete Linux gates at
[`dec0ce2`](docs/compiler-set/measurements/2026-11-17-native-linux-dec0ce2/README.md)
and [`74e14e7`](docs/compiler-set/measurements/2026-11-17-native-linux-74e14e7/README.md)
remain attributed to their exact sources. Cross-target layout checks and
Rust/Cargo corpus checks do not establish native execution on other platforms.
Earlier macOS failures and [portable fixture/Unix-address corrections](docs/compiler-set/native-differential-portability.md)
remain preserved; recent cancellations establish no native macOS qualification.

## Which work is being compared?

| Workload | Reference and candidate | Evidence and boundary |
| --- | --- | --- |
| Fixed real Cargo graph | Matching Cargo/rustc → mixed Lustr | Real procedural macro, async code and a no_std/alloc dependency, with native objects and complete Rust compilation; explicit dev debug=0. |
| Actual unchanged coordinator package | Matching Cargo/rustc → primary Lustr entry | Five scenarios under normal dev/test defaults, including debug=2; outside native-object eligibility. Forced rebuilds change timestamps, not source bytes. |
| Historical unchanged Lust compiler project | Cargo/LLVM → explicit coordinator/Cranelift | November 8 whole-project builds with an older SDK and custom unwind-enabled backend; two measured pairs, no current mixed-compiler claim. |
| Supported native scalar corpus | Pristine Lust → integrated Lust | Preservation check for the declared native integration. |
| Same native corpus | Pristine Lust → packaged Lustr entry | Includes entry routing; rejection can add full-Rust validation. |
| Same native corpus | Matching standalone rustc → packaged native entry | Identical source and explicit dev semantics; rustc is nonincremental. First measured in schema 2 at `5a3f493`. |
| Larger native workspaces, broad edit families, startup/restart, isolated runtime and resources | Separate campaigns | Not established by these tables or a one-repetition CI smoke check. |

The fixed native corpus and Cargo graph are small. Older
[native product experiments](docs/compiler-set/measurements/2026-11-15-native-inference-v34/README.md)
have different architectures, producer-version confounds and retained limits.
They remain separate from current-entry measurements. The
[upstream audit](docs/compiler-set/upstream-benchmark-audit.md) maps Lust's
actual studies against this coverage.

## Latest fixed-corpus results

Twelve matched pairs per mode, Linux x86-64, candidate `6bf6e3e`. Times are
milliseconds. The ratio is the geometric mean of **baseline time divided by
candidate time**, with a conditional paired-bootstrap 95% interval. Below one
means the candidate took longer. It is not the ratio of the displayed medians.

### Mixed Cargo graph

Both lanes execute Cargo on the same source and dependency graph with the
matching SDK. The candidate can reuse checked scalar bodies and native objects;
the full Rust compiler still provides valid metadata and unsupported semantics.
The fixture's dev profile explicitly uses debug=0. It is neither a default-profile
large-project result nor a comparison with standalone Lust replacing real
Cargo dependency crates.

| Cargo workload | Cargo/rustc median | Mixed Lustr median | Ratio [95% interval] |
| --- | ---: | ---: | --- |
| Cold build | 123.510 | 143.232 | 0.862 [0.848, 0.873] |
| No-op build | 7.955 | 8.434 | 0.950 [0.938, 0.962] |
| Body edit | 38.612 | 35.761 | 1.079 [1.070, 1.088] |
| Signature edit | 40.552 | 47.090 | 0.857 [0.854, 0.861] |
| Check | 30.699 | 35.375 | 0.867 [0.856, 0.878] |
| No-op check | 7.762 | 8.138 | 0.945 [0.937, 0.952] |
| Check after body edit | 19.068 | 20.267 | 0.944 [0.935, 0.954] |
| Rejection | 26.971 | 25.604 | 1.059 [1.045, 1.074] |
| Compile plus test | 60.414 | 61.512 | 0.979 [0.970, 0.988] |

The body-edit build takes **7.4% less median time**, with a paired estimate of
**1.079× [1.070, 1.088]** against matching Cargo/rustc. Rejection also favors
Lustr. Seven other modes remain slower: cold builds take about **16% longer**,
signature edits **16% longer**, and checks **15% longer** by the medians.
This is a narrow Cargo improvement on the retained scalar-body fixture, not
general Cargo acceleration. No historical sample was removed or replaced.

[All 36 comparisons and p95s](performance/2026-11-17-6bf6e3e.md),
[all 648 timed values](performance/2026-11-17-6bf6e3e.csv), and the
[vector chart with exact source data](performance/plots/2026-11-17-6bf6e3e/README.md)
are retained. No average across unrelated modes serves as a headline.

![All four paired comparisons, including slower modes and conditional intervals](performance/plots/2026-11-17-6bf6e3e/performance.svg)

The eight campaigns' mixed cold/body/signature ratios are respectively
0.726/0.596/0.552, 0.662/0.581/0.558, 0.732/0.601/0.554,
0.717/0.614/0.553, 0.756/0.612/0.547, 0.743/0.622/0.558 and
0.773/0.624/0.548, followed by 0.862/1.079/0.857. Different source revisions and uncontrolled shared-host
conditions prevent a causal cross-run improvement claim. The earlier `afb17a8`
cold candidate outlier of 1,026.532 ms remains in its original samples; no
slower observation was excluded to obtain the published results.

## Actual project under its normal defaults

The [latest verified coordinator campaign](performance/projects/2026-11-17-6bf6e3e/README.md)
uses the unchanged actual `crates/lustr` package, its library, executables,
integration tests and existing fixture builds. Both lanes retain normal dev/test
profiles, debug=2, features and panic behavior. This profile falls outside
native-object eligibility; these results measure compatibility and overhead.

| Project scenario | Matched Rust median (ms) | Lustr median (ms) | Ratio [95% interval] |
| --- | ---: | ---: | --- |
| Clean dependency build | 495.682 | 500.290 | 0.9905 [0.9836, 0.9973] |
| Primed root build | 146.147 | 147.562 | 0.9932 [0.9777, 1.0083] |
| Primed root check | 47.891 | 49.520 | 0.9672 [0.9547, 0.9808] |
| No-op build | 8.872 | 9.904 | 0.9130 [0.8930, 0.9393] |
| Compile plus test | 4583.380 | 4653.394 | 0.9832 [0.9793, 0.9879] |

Four intervals favor matched Rust; primed root build is inconclusive. Median
overhead is about 0.9% for clean build, 1.0% for primed root build, 3.4% for
primed check, 11.6% for no-op and 1.5% for compile plus test. These descriptive
percentages use the displayed medians, without a pooled estimator. The
[preceding project campaign](performance/projects/2026-11-17-5a3f493/README.md)
and its predecessor retain their original values. Changes across hosts and
revisions do not establish a causal optimization effect.

The forced build, check and compile-plus-test scenarios change only a source
timestamp. Source bytes stay unchanged. These are root rebuilds and do not
measure body/signature edits or demonstrate native acceleration. The project
has no registry dependencies.
Cold state clearing leaves OS page caches and dependency sources warm.
Each scenario contains twelve measured pairs after two excluded warmup pairs.
Both lanes pass initial/final executing tests and structured E0502 rejection
gates. The test gate reports 40 passed and nine existing explicitly ignored
tests. Source/tool guards, command contracts, Cargo fingerprints, owned state
transitions and observed successful daemon shutdown are required. Shutdown
receipts observe zero native child exits and do not claim native work.

Each original campaign contains 120 measured values plus warmups and gates.
The three campaigns retain **360 timings and fifteen comparisons**, separately from
the fixed-corpus totals. Their public projections contain every timing, pair
order, original stream hash, host snapshot and gate count. They do not expose
private original command streams or environments.

## Historical full-project Cargo acceleration with Cranelift

The [November 8 whole-project experiment at `42ab5dc`](docs/compiler-set/verification.md#qualified-full-project-result)
compiled the unchanged upstream Lust compiler's `lust` and `lust-core` binaries
through real Cargo. Both lanes used their project's default dev profile and
the same pinned `nightly-2026-11-05` SDK. The candidate explicitly selected
custom Cranelift revision `8edf4d040acb77048e04b52a379e93dccae888e6`, built with
its `unwinding` feature, while the reference used LLVM.

| Historical Cargo scenario | LLVM median | Cranelift median | Ratio of medians, LLVM / Cranelift |
| --- | ---: | ---: | ---: |
| Empty target directory | 59.536 s | 39.400 s | 1.511 |
| Warm, no source change | 19.532 ms | 21.149 ms | 0.924 |

There were only two measured pairs per scenario. The cold result represents
33.8% less median build time for that configuration; no-op builds were slower.
These descriptive ratios do not use the current paired geometric estimator and
have no confidence interval. The full build timings exclude runtime probes,
unit tests, verification and setup. Both lanes passed the same 391 unit tests
and runtime/rejection probes; the recorded compiler commands verify the backend.
The legacy project's pinned Cargo layout adjustment and test-only stack setting
were applied equally to both lanes and remain part of its qualification scope.

The [summary and identities](docs/compiler-set/evidence/lust-project-42ab5dc-summary.json),
[complete compressed report](docs/compiler-set/evidence/lust-project-42ab5dc.json.gz)
and [backend provenance](docs/compiler-set/evidence/lust-project-42ab5dc-backend.json)
retain the original evidence. This is an opt-in compatibility-coordinator path,
not the current mixed native compiler or an automatically selected backend.
Its rustc source `2d20b42b883dcfc014f3c2c56081eeb6091c2179` differs from the
current campaign SDK. The stock rustup Cranelift preview failed panic recovery;
the [matching custom backend requirement](crates/lustr/README.md#try-the-complementary-development-backend)
cannot be replaced by a panic flag or an arbitrary installed backend. Requalify
the complete current package and measure it before claiming a present Cargo gain.

## Native entry versus pristine Lust and matching rustc

This is the same completed `6bf6e3e` campaign and twelve matched pairs per mode
as the mixed Cargo table. Times are median milliseconds and ratios use the
paired geometric estimator, distinct from the historical Cranelift medians.

| Native workload | Pristine Lust median | Standalone rustc median | Lustr median | rustc / Lustr ratio [95% interval] |
| --- | ---: | ---: | ---: | --- |
| Cold build | 38.140 | 33.773 | 41.044 | 0.823 [0.788, 0.861] |
| No-op build | 3.071 | 30.632 | 2.788 | 11.038 [10.858, 11.223] |
| Body edit | 4.913 | 33.449 | 4.878 | 6.893 [6.580, 7.208] |
| Signature edit | 4.102 | 31.613 | 3.949 | 8.017 [7.800, 8.251] |
| Check | 1.758 | 8.695 | 1.597 | 5.402 [5.167, 5.614] |
| No-op check | 1.431 | 8.771 | 1.432 | 6.050 [5.843, 6.286] |
| Check after body edit | 1.456 | 8.433 | 1.445 | 5.768 [5.574, 5.929] |
| Rejection | 1.619 | 13.642 | 14.160 | 0.967 [0.949, 0.986] |
| Compile plus test | 27.539 | 46.894 | 27.595 | 1.696 [1.640, 1.758] |

Seven native comparisons favor Lustr over this standalone rustc configuration;
cold and rejection favor rustc. Body edits are 6.893× and signature edits
8.017× faster by the paired estimate. Native no-op is 11.038× faster, with
2.788 ms versus 30.632 ms medians. **Every rustc no-op starts a nonincremental
compiler invocation.** This is not Cargo's cached no-op or a comparison against
the fastest possible rustc configuration.

The [exact reference contract](docs/compiler-set/native-rustc-comparison.md)
uses identical supported source and edition 2024, target x86_64 Linux, opt-level
0, debug 0, unwind, overflow checks and debug assertions. Build emits a linked
program; check emits metadata. Rust compile-plus-test includes `rustc --test`,
actual execution with one test thread, and the adapter process's overhead.
Both child commands and exit/signal receipts are retained. Passing/failing
tests, invalid-source rejection, accepted runtime and recovery are required.

Against pristine Lust, native cold, body edits and compile-plus-test are
inconclusive. Native no-op favors Lustr at 1.092 [1.070, 1.113], signature edit
at 1.045 [1.010, 1.078], and check at 1.046 [1.008, 1.081]. Packaged rejection
is slower at 0.122 [0.113, 0.132]; the entry may perform full-Rust validation
after native rejection. The full language and diagnostic contract remains
part of that measured cost.

The pristine→integrated comparison has one interval above one (check) and
eight inconclusive intervals. An inconclusive interval does not prove
equivalence; classification uses unrounded bounds. Packaged rejection takes
14.160 ms versus pristine Lust's 1.619 ms and standalone rustc's 13.642 ms.
Returning the first native error could incorrectly reject valid full Rust.
This complete user-visible negative path performs differing semantic work.

## Exact source, retention and replay

All eight fixed campaigns use pristine Lust revision
`240e08d91d4aed0299639f6f11fd522912b248f9`, tree
`0ab1b09815802894e6078de0af59273cbdf46cb0`. The matching imported Rust SDK is
rustc 1.102.0-nightly, commit `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`,
commit date 2026-11-08, host x86_64-unknown-linux-gnu, LLVM 23.1.5.
The native correctness gate uses a separate Rust 1.99.0 oracle.

| Candidate / schema | Actual measured checkout | Repository tree | Host / profile availability |
| --- | --- | --- | --- |
| `c70eb1c` / 1 | `e7e9cf2a224b2590b895298439e5ede814c6229b` | `bf575b9343774b50646a78a5aa22eb52960de840` | Neither recorded |
| `afb17a8` / 1 | `bc5e0f9cd1168b6b064b9cda9a1455f4fc97d143` | `3ccebbefb604e0c45715200bcabc3b32b182d3ef` | Neither recorded |
| `048d824` / 1 | `fe4704ec6951b8c22c003476886602368810f637` | `5aec709fcd0df39004973a33320492ba82eb4fc1` | Neither recorded |
| `dec0ce2` / 1 | `dbacce1d14c8df60427eeca8dc7865156c98dcb4` | `c29f3bda282c0870c163303d865abb9be81bd066` | Host; completed v1 profile |
| `74e14e7` / 1 | `d0caf843d519763ae3b7bc8a3fdd1c26b12b307d` | `c15cdcb3481f2ee34651a8aa6c901d24df05ff8f` | Neither recorded |
| `40d2ac3` / 1 | `3b4b0bd67401085129d26be1afbd20e96c9b9d02` | `78318c4b01f8eb6f9518ea331efe69f0c457b1ce` | Host; failed v2 original, separate diagnostic reconstruction |
| `5a3f493` / 2 | `b3393a4a2907228516546cd34f1a5c60aaf6aaaa` | `2fca60e57829890b10a4a23da57bafab2581b092` | Host; completed original v2 profile |
| `6bf6e3e` / 2 | `5a3c5666a16d46629a4787e37ca9b20e65720ccc` | `02652f033e84c0957dd2b99eb562db8171289e50` | Host; completed profile with in-process symbol validation |

The latest actual checkout is a GitHub merge commit whose tree equals its
recorded PR head. Its [incremental source bundle](performance/sources/2026-11-17-6bf6e3e/README.md)
preserves that exact object and declares both prerequisites. An isolated restore
reproduced the tree and all recorded source-entry names, modes and blob IDs.
That inventory check does not freshly hash every blob or recover the compiler
binaries. [campaigns.json](performance/campaigns.json) records every full head,
run/artifact ID and original/derived digest.

The latest [original qualification artifact](https://github.com/glopyglerky/lustr/actions/runs/44179667793/artifacts/14374511846)
has ZIP SHA-256
`c2126b1dd44170adf52cd66c53059c2447f54ad8a2483dfaec1b97f1c60fcd01`;
its original raw JSONL SHA-256 is
`3ed362da12116a92f9dfcaf5c4dc07e9c3dd8f9b3a8f1add3f659bbb90d342d5`.

Eight byte-identical original fixed-fixture ZIPs preserve **4,536 timed values,
234 comparisons and 14,928 command-output files** beyond Actions expiry. They
contain exact commands, exits, stdout/stderr, source/build receipts, component
hashes, correctness outputs, cache probes, host/profile records where collected,
and original failures. They do not contain every compiler binary. CSVs and
tables are derived projections. CI replays each archive and verifies exact
published table bytes; older artifacts are not overwritten.

Actual-project evidence has a separate publication boundary. Each campaign's
six public files and original public/encrypted ZIPs are retained byte for byte.
Authenticated recovery and source-bound reconciliation passed for **600 private
original files and 510 streams in each campaign**. The original private plaintext and
key remain private. Public CI checks file inventories, original ZIP digests,
ciphertext bindings, all 360 measured values and fifteen statistical comparisons.

[Recovery receipts](performance/projects/2026-11-17-6bf6e3e/recovery/result.json)
distinguish the locally rebuilt existing replay from the unavailable original
CI binaries. The recorded-source exporter reproduces public samples and README
byte for byte; other projection receipts agree semantically. Original compiler,
runtime, constructor and runner binaries are not freshly requalified. Exporter
runtime identity was not recorded by campaign version 1 and remains null.
Public replay does not perform another decryption or compiler execution.

## Work and statistical protocol

Compiler components use their manifest release profiles. Fixed-fixture
applications use dev opt-level 0, debug 0 and normal unwind/checks. The matched
SDK and identical paired inputs remain part of each recording. The actual
project keeps its separate normal debug=2 dev/test profile.

Schema 2 has four native lanes and two Cargo lanes, nine modes and twelve
measured observations per lane/mode: **648 timed commands and 36 summaries**.
Its original record contains 1,135 raw records, including 1,080 timed/untimed
commands and 2,160 output files. The six schema-1 campaigns remain five-lane,
540-command, 27-summary recordings, each with 938 raw records, 884 commands
and 1,768 outputs. Untimed priming, correctness, lockfile work and probes remain
in the archives without becoming timing samples. The different contrasts reuse
some measured lanes; the displayed comparisons are not independent experiments.

Native retention probes are separate untimed checks. A
`bodies-without-body-work` route requires zero relevant checked-body/codegen
counters; it does not mean zero total work. Earlier pristine traces still
performed collection, distinct from an explicit `reuse` route.

Cold trials use fresh per-pair Cargo target, intermediate, mixed, native and
XDG runtime cache directories. Native cold disables daemon/disk compiler
caches. OS page caches are not flushed. Warm trials retain each lane's native
daemon and caches. Matching standalone rustc remains nonincremental and starts
a compiler process on every request. A Cargo no-op may not invoke a compiler.
These are declared tool behaviors, not equivalent cache implementations.

The recorder uses randomized cyclic lane order, seed `327849038930`. Schema 2
accepts pair counts that are multiples of four from twelve through ninety-six,
so complete cyclic blocks cover all four native lanes. It measures command
wall time through observed child exit, with 100-microsecond polling. Tracing
and compiler statistics are disabled for measured commands. Runtime assertions
after ordinary builds are outside compile-only timing; compile-plus-test
includes actual execution. It is not an isolated runtime-throughput benchmark.

For matched times `(a_i, b_i)`, the estimate is
`exp(mean(log(a_i) - log(b_i)))`. The existing statistics implementation uses
10,000 paired bootstrap resamples, SplitMix64 with rejection-index sampling,
and percentile intervals. Sign-flip calculation is exact for the recorded
twelve-pair campaigns and through sixteen pairs; it is unavailable above
sixteen. The replay checks this existing implementation, not a replacement
estimator. The p95 is nearest rank; with twelve samples it equals the sample
maximum and is descriptive rather than a strong tail guarantee.

Intervals are exploratory and conditional on the workload and sampling setup.
Shared-host/cache effects can compromise bootstrap independence and sign-flip
exchangeability. There is no multiple-comparison correction or predeclared
equivalence margin. An interval containing one is inconclusive. An interval
excluding one does not establish universal performance or release admission.
Absolute observations, paired results and every slower sample remain visible.

## Reproduce the evidence

Fixed-fixture reporting uses Python 3.10 or newer and the standard library:

```sh
python3 tools/test-mixed-performance-report.py
python3 tools/check-mixed-performance-tables.py
python3 tools/check-mixed-performance-archives.py
python3 tools/report-mixed-performance.py \
  performance/archives/2026-11-17-6bf6e3e-qualification.zip \
  --output /absolute/new-report-directory
```

The output directory must not exist. Validation finishes before publication;
no recorded command arguments are executed. Outputs are `README.md`,
`timings.csv` and a fuller derived `samples.json` with identities and original
summaries. ZIP input binds its archive digest; an extracted directory cannot
claim a ZIP hash. All eight original CSV/Markdown reports replay byte for byte.

Verification covers completed status, raw digest, matching records, all output
sizes/hashes, source stability, identical paired inputs, exact schema coverage,
unique sample IDs, expected exits, no timeout/signal, balanced order, retention
probes and statistical replay. Both schemas bind timed Cargo commands to the
exact lane executable, operation/flags, source inventory, working directory and
owned target/intermediate directories. Compile-plus-test requires one executed
passing scalar-contract test. The current verifier closes two historical gaps:
executable relabeling and compiled-but-unexecuted tests. Original recordings
already performed the required work and replay unchanged.

Schema 2 additionally binds native executable, source, operation and flags,
including the two-child standalone rustc test receipts. That stronger native
argument contract does not retroactively extend to schema-1 native commands or
every older untimed probe. Altered executable identities, missing comparisons
and omitted test execution fail verification.

All 80 reporting regressions passed. Archive/table replay covers 234 summaries,
4,536 timing values and 14,928 output files. These checks validate evidence;
they do not rerun the measured compilers. Original correctness gates supply
their separate executed evidence.

For public project replay, with the matching SDK installed:

```bash
cargo +nightly-2026-11-09 build --manifest-path crates/lustr-bench/Cargo.toml \
  --release --locked --example replay --target-dir target/project-replay
python3 tools/check-coordinator-performance-archives.py \
  --replay target/project-replay/release/examples/replay
```

Private recovery additionally requires the authorized recipient key, authenticated
decryption, archive verification before extraction, exact ordinary-file modes
and the recorded Git source. The [recovery procedure](docs/compiler-set/coordinator-performance.md)
and per-campaign receipts distinguish that operation from public replay.

To execute another actual campaign, use the exact source and
[source-built workflow](.github/workflows/compiler-set.yml) or
[build-and-qualify instructions](docs/compiler-set/mixed-compiler.md#build-and-qualify).
Build matching pristine, integrated and packaged components with the pinned
SDK/TinyCC, run every correctness prerequisite, then the existing recorder.
Keep all modes and original outcomes; a historical or unmatched package is
not a newly qualified candidate. Profiling runs separately from paired timing.
New completed artifacts receive new files and manifest entries.

## Host and measurement limits

The latest fixed campaign records Intel Xeon 6975P-C, four visible logical CPUs
and observed affinity 0–3, 16,464,306,176 bytes of physical memory, Linux
7.0.0-1014-aws and Ubuntu image 20261111.127.1. CPU quota, memory limit and
frequency governors are unreported. The project has its own before-campaign
snapshot from the same job. Only `dec0ce2`, `40d2ac3`, `5a3f493` and `6bf6e3e` have fixed
host snapshots; missing facts in the other four campaigns remain unknown.
Matching model/image descriptions do not establish controlled physical-host
or load equivalence between jobs.

The host collector binds its exact bytes and parsed facts to campaign identities.
These are observations: affinity is not pinned, physical memory is not a process
budget, and load/frequency can change. CPU time, RSS, aggregate process memory,
daemon growth, energy and isolated application runtime are unmeasured. Twelve
observations do not support broad tail-latency claims. Upstream Lust's reported
hardware and corpus facts are not substituted for Lustr's missing measurements.

## Phase attribution and retained history

The [current original profile](performance/2026-11-17-6bf6e3e-profile.md) passes
strict replay with in-process symbol validation. Its body-edit observation
records 1.821 ms for the native request, 0.008261 ms for symbol validation,
0.043122 ms for symbol renaming and 15.320 ms for the original linker. The
remaining cold/signature/check costs require full-work profiling; an eliminated
process does not eliminate Rust analysis, metadata, invalidation or linking.
See the full profile for all commands, nested spans and exact identities.

The historical profile below records the predecessor's observed bottleneck.
Differences between instrumented runs are not a controlled causal estimator.

The [preceding original v2 qualification](performance/2026-11-17-5a3f493-profile.md)
passed with 26 compiler profiles, 15 qualification commands and 35 successful
native responses containing measured emitter detail. Its 132 original input
files remain in the qualification ZIP. The original summary and Markdown
reproduce byte for byte, without reconstructing a trace or weakening validation.

Profiling is separate under `LUSTR_MIXED_PROFILE_DIR`; paired commands clear
their environments and do not inherit it. Spans can nest and compiler processes
can overlap. Summing spans does not produce command wall time. The following
values are instrumented observations, not additional paired speed samples.

| Instrumented duration (ms) | Body: one request | Signature: five requests summed |
| --- | ---: | ---: |
| Complete native request | 37.680779 | 176.700542 |
| Client request setup | 0.119698 | 0.377320 |
| Protocol exchange | 37.551081 | 176.282993 |
| Remote emitter | 35.257744 | 162.980178 |
| libtcc | 0.197719 | 2.016269 |
| Symbol-inspection process call (`nm`) | 34.545005 | 158.645402 |
| Native-object file synchronization | 0.409056 | 1.888154 |
| Directory synchronization | 0.009334 | 0.038698 |
| Exchange outside emitter, unattributed | 2.293337 | 13.302815 |

Remote emitter durations retain their own clock domain. The outside-emitter
residual includes transport, queueing, native frontend work before emission,
retained-state work afterward, guards and response production; it is not a
measured frontend phase. Signature values sum five requests and must not be
compared directly with the command's wall duration.

Across all 35 successful responses, the complete `nm` process-call span ranges
from 30.808 to 34.545 ms, median 31.857 ms. It includes lookup, launch, execution,
output capture and waiting. The actual child binary identity, emitted object
bytes and child CPU/spawn breakdown are absent. This is a dominant observed
span whose cause remains unresolved. Explaining that historical delay requires
same-runner tool and object identity plus spawn/wait attribution. The measured `6bf6e3e` package removes this subprocess through checked
in-process object validation; its complete-command measurements appear above. Undefined-symbol validation and durable object/certificate
synchronization remain required.

The [completed v1 profile](performance/2026-11-17-dec0ce2-profile.md) and the
[later failed v2 original with separate reconstruction](performance/2026-11-17-40d2ac3-profile.md)
keep their original status. The writer correction constructs a complete record
and holds the append-file lock through all writes. Its concurrent-process test
fails the predecessor and passes the correction; the new hosted profile now
validates that correction in the source-built toolchain. All 26 strict profiler
tests pass. Failed native responses keep their original diagnostic bytes so
the daemon's fresh/warm comparison preserves its meaning.

The [24-pair temporary-copy experiment](performance/components/2026-11-17-40d2ac3-link-input/review.md)
is a component result, outside compiler timing totals. Disposable copies omit
synchronization, while durable retained objects and certificates keep it.
The experiment neither measures a complete link nor establishes an additional
merge gain; the preceding candidate already omitted disposable-copy synchronization.

## Remaining performance work

The [upstream audit](docs/compiler-set/upstream-benchmark-audit.md) identifies
larger native workspaces, wider edit families, accepted/rejected recovery timing,
daemon startup/cached restart, release compilation, isolated application runtime
and CPU/memory studies. The direct same-source native/rustc comparison is now
measured. The unchanged coordinator adds real-project compatibility and overhead
coverage; its default profile does not establish large native acceleration.

The performance priority is complete Cargo work: cold builds, source body and
signature edits, checks and executing tests. The in-process symbol
validator, renaming and direct Cargo entry have now passed complete source-built
correctness and paired measurements together. The next work is to reduce the
remaining cold, signature and check costs, and expand to unchanged projects with
real source edits and observed native participation. Keep default-profile
fallback and no-op overhead visible; debug=2 linked generation remains outside
native-object eligibility.
The full native sanitizer runtime failure and native macOS execution remain
separate correctness requirements.
An optimization must preserve checked Rust objects, retained native state,
real metadata, invalidation, diagnostics, panic behavior and complete work.
Retain its predecessor, exact sources, correctness results and all slower
observations, then rerun the complete paired campaign. Cross-run changes alone
do not isolate the optimization's effect.

The recovered [public upstream slice runner](docs/compiler-set/upstream-slice-benchmark.md)
adds supported-native scale, explicit incremental Cargo, no-op and generated
program execution scenarios. It uses upstream's pruned source and dependency
stand-ins. It does not measure source edits or run the mixed compiler through
Cargo, so a successful future result must retain that separate scope. Its first hosted attempt
failed before timing because daemon statistics were not enabled; the original
zero-sample failure is retained and the recorder is corrected. A fresh hosted
run remains required before adding slice measurements.

Broader mixed coverage, representative workload speed and release admission
remain open. [Older work and the Darwin loader/lifetime investigation](docs/compiler-set/retained-work-disposition.md)
retain their own scope. Source identity, successful replay and a green
measurement step cannot substitute for those requirements.
