# Lustr performance: measured work, not inherited speed claims

## Status on 17 November 2026

Lustr is **not yet a generally qualified, speed-preserving full-Rust compiler**.
The source-built mixed compiler executes the qualified Cargo graph correctly,
and the original supported native path remains operational. The completed
campaigns nevertheless show substantial mixed Cargo compilation overhead.
A successful measurement job means the campaign completed, not that Lustr won.

The latest measured PR9 head is `afb17a8af2cce9facdf03ba6cf12b8533cb3a171`.
[Compiler set run 44063047987](https://github.com/glopyglerky/lustr/actions/runs/44063047987)
passed all nine jobs. This includes real source-built mixed execution, failed-link
baseline reproduction and corrected recovery, retained checking with fresh Rust
metadata, mixed libraries, the fourteen-test compiler-entry/runtime contract,
target-layout comparisons, and the completed paired campaign. Cross-target
layout comparisons are not native execution on those targets.

[Native CI run 44063047989](https://github.com/glopyglerky/lustr/actions/runs/44063047989)
passed Linux x86-64 but failed macOS ARM64. macOS now builds and reaches the
original differential suite: 5,522 cases passed and four failed. Three fixtures
import Linux-only socket extensions; `runtime_inline_helpers` differs on signed
zero output. These remain unresolved failures, not skipped successes. Subsequent
native suites that the failed gate did not reach are unqualified. The
[portability correction](docs/compiler-set/native-differential-portability.md)
addresses those fixtures and a separately reproduced Unix-address buffer bound
bug. Its exact-source native CI result must be assessed independently of these
historical runs.

## The three comparisons

1. **Pristine Lust → integrated Lust** isolates the declared native integration
   from the packaged entry. It is the direct supported-path preservation check.
2. **Pristine Lust → packaged Lustr entry** includes product routing and request
   handling. A valid native fast path and an expensive rejection route can coexist.
3. **Matched Cargo/rustc → mixed Lustr** compares the same real Cargo graph and
   build modes, including the imported full Rust implementation, native work,
   object integration, and actual Cargo behavior. This is not a comparison
   against an independently reconstructed Rust language implementation.

There is one small native scalar corpus and one fixed small Cargo graph with a
real procedural macro, async code and a `no_std`/`alloc` dependency. These are real
compiler executions, but they are not a representative application population.
Upstream Lust's ts-rust measurements are useful methodology and separate upstream
evidence; none of its headline speedups is a measured Lustr speedup.

## Latest complete-work numbers

Twelve matched pairs per mode, on the Linux x86-64 `ubuntu-24.04` GitHub runner.
Times are milliseconds. Ratio is the geometric mean of **baseline time divided
by candidate time**, with a conditional paired bootstrap 95% interval. A ratio
below 1 means Lustr took more time. It is not the ratio of the displayed medians.

| Cargo workload | Cargo/rustc median | Mixed Lustr median | Ratio [95% interval] |
|---|---:|---:|---:|
| Cold build | 191.887 | 258.039 | 0.662 [0.522, 0.749] |
| No-op build | 11.343 | 12.470 | 0.904 [0.873, 0.932] |
| Body edit | 61.529 | 105.185 | 0.581 [0.574, 0.588] |
| Signature edit | 66.527 | 119.235 | 0.558 [0.548, 0.570] |
| Check | 50.841 | 58.420 | 0.872 [0.864, 0.880] |
| No-op check | 11.382 | 12.238 | 0.930 [0.911, 0.953] |
| Check after body edit | 31.616 | 33.988 | 0.930 [0.922, 0.937] |
| Rejection | 43.893 | 43.179 | 1.011 [0.988, 1.032] |
| Compile plus test | 92.747 | 94.409 | 0.980 [0.971, 0.989] |

Body and signature edits take approximately 71% and 79% longer by the displayed
medians. Cold build is approximately 34% longer at the median. Its paired
geometric estimate is worse because one candidate sample took 1,026.532 ms.
That sample remains in the data; it has not been discarded as inconvenient noise.
Rejection is inconclusive in this campaign, not a demonstrated win.

For the small native fixture, the latest packaged-entry no-op comparison is
1.109 [1.087, 1.135], while body editing is 1.003 [0.975, 1.032], inconclusive.
The packaged rejection path is 0.120 [0.113, 0.128]: pristine Lust's median is
2.621 ms and the product entry's is 23.128 ms. The entry can invoke full Rust
validation and diagnostics after a native rejection. Returning the first Lust
error would risk rejecting valid full-Rust programs; it is not an admissible
performance shortcut. This is an end-user negative-path cost, not evidence that
equivalent frontend phases alone became eight times slower.

All 27 latest comparisons, including native integration, packaged entry, every
median and p95, are in the [generated full table](performance/2026-11-17-afb17a8.md).
All 540 timed command values are in the [lossless timing CSV](performance/2026-11-17-afb17a8.csv).
The [previous campaign table](performance/2026-11-17-c70eb1c.md) and
[its 540 values](performance/2026-11-17-c70eb1c.csv) remain alongside it. Do not
average unrelated modes into a headline that hides a regression.

The earlier campaign already showed mixed Cargo cold/body/signature ratios of
0.726/0.596/0.552. Some small native observations changed classification between
campaigns. `afb17a8` changes macOS CI provisioning, not the measured Linux
compiler architecture. Absolute time differences between shared-host campaigns
are not a causal speed improvement or regression attributable to that commit.

## Exact provenance

Both campaigns use original Lust revision
`240e08d91d4aed0299639f6f11fd522912b248f9`, tree
`0ab1b09815802894e6078de0af59273cbdf46cb0`. The complete Rust reference is the
matching imported SDK, rustc `1.102.0-nightly` dated 2026-11-08, commit
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, host
`x86_64-unknown-linux-gnu`, LLVM 23.1.5. Do not substitute the native macOS CI
oracle's different Rust version into this performance record.

| Campaign | PR head | Actual checkout | Actual tree |
|---|---|---|---|
| Earlier | `c70eb1c25085884622a95666021fce0528e7325b` | `e7e9cf2a224b2590b895298439e5ede814c6229b` | `bf575b9343774b50646a78a5aa22eb52960de840` |
| Latest | `afb17a8af2cce9facdf03ba6cf12b8533cb3a171` | `bc5e0f9cd1168b6b064b9cda9a1455f4fc97d143` | `3ccebbefb604e0c45715200bcabc3b32b182d3ef` |

The latest actual checkout is a PR merge checkout; its tree equals the recorded
PR-head tree. Report the measured checkout rather than inventing a head checkout.

**Earlier original artifact:** [run 44056401150, artifact 14318695838](https://github.com/glopyglerky/lustr/actions/runs/44056401150/artifacts/14318695838).
ZIP SHA-256: `dc6b49c7d8bfd543a79965f5e4299b8f6a9bf481c98c8ffb7d3482b32adf0ac9`.
Original `raw.jsonl` SHA-256: `75ebcfb474458760500a52fa6ef63ccbe4177009ed813a6135bdf8b15279288f`.

**Latest original artifact:** [run 44063047987, artifact 14321378285](https://github.com/glopyglerky/lustr/actions/runs/44063047987/artifacts/14321378285).
ZIP SHA-256: `3963c513a7a2e6d790cae2f576e80927039d9f7d4e1fe41786068b9650c29050`.
Original `raw.jsonl` SHA-256: `2fe01a5ecd484a42da2b6b198fc251a81eecdd569447de9f1d344dfba9f45a21`.

The artifacts retain exact compiler/component hashes, build profiles, commands,
expected and observed exits, stdout/stderr, source hashes, order records, retained
state probes, source/build receipts and correctness artifacts. The committed
CSV files are explicitly **derived timing projections**, not substitute original
logs. Their columns preserve original pair IDs, seconds and every timed value.
The generated tables bind them to the original raw digest and checkout.

GitHub artifacts have finite retention: the latest was reported to expire on
2027-02-15. Both original ZIP files are now committed byte for byte under
[`performance/archives`](performance/archives), with their existing SHA-256
identities bound by [`campaigns.json`](performance/campaigns.json). This preserves
the complete original qualification artifacts after Actions expiry, including
all command outputs, source/build receipts and timing records. The CSV remains
a derived projection. The archive does not contain every compiler binary or
reproduce an execution by itself.

`python3 tools/check-mixed-performance-archives.py` verifies both original ZIPs,
all 3,536 recorded command-output files and exact replay of all 54 published
comparisons. CI performs this check alongside the CSV replay. No timing samples
were removed.

## Work and statistical protocol

Compiler components use their manifest release profiles. Fixture applications
use dev opt-level 0, debug 0 and normal unwind/checks. The reference and candidate
use the matched SDK and equivalent fixture inputs. Native retained-state probes
are separate untimed checks; their accepted `bodies-without-body-work` route
means zero checked bodies/codegen in the specified counters, not zero total work.
An earlier pristine trace still performed collection. It is kept distinct from
an explicit `reuse` route.

Cold trials use fresh per-pair Cargo target, intermediate, mixed, native and XDG
runtime cache directories. Native cold disables daemon/disk compiler caches.
OS page caches are **not** flushed. Warm trials retain each lane's caches and
original native daemons. A Cargo no-op may not invoke the compiler at all.

The runner uses randomized cyclic lane order, seed `327849038930`. Each of the
five lanes has nine modes and twelve observations: 540 timed commands, producing
27 paired summaries. The complete artifact has 938 records, including 884 timed
and untimed commands and 1,768 corresponding output files. Untimed priming,
correctness execution, lockfile preparation and probes are retained but are not
silently counted as timing samples. No timed samples were excluded.

The measurement is command wall time through observed child exit with 100 µs
polling. Tracing and compiler statistics are disabled for measured commands.
Runtime assertions after ordinary builds are outside compile-only timing;
`compile-plus-test` executes the defined tests. It is not an isolated benchmark
of generated application execution speed.

For matched times `(a_i, b_i)`, the estimate is
`exp(mean(log(a_i) - log(b_i)))`. The implementation uses 10,000 paired bootstrap
resamples, SplitMix64 with rejection-index sampling, percentile intervals and an
exact sign-flip calculation for these twelve-pair series. The independent replay
checks the existing Rust implementation's results, not a different estimator.
The p95 is nearest rank. With twelve samples it equals the sample maximum; this
is a descriptive number, not a well-established tail-latency guarantee.

Intervals are exploratory, conditional on this workload and sampling setup.
Bootstrap independence and sign-flip exchangeability can be compromised by
shared-host/cache effects. There is no multiple-comparison correction and no
predeclared equivalence margin. An interval containing 1 is **inconclusive**, not
proof of parity. An interval excluding 1 on this corpus is not universal speed
preservation or release admission. Both paired and absolute data must remain
visible when evaluating a proposed fix.

## Reproduce the report without running a compiler

Python 3.10 or newer, standard library only:

```sh
python3 tools/test-mixed-performance-report.py
python3 tools/check-mixed-performance-archives.py
python3 tools/report-mixed-performance.py \
  performance/archives/2026-11-17-afb17a8-qualification.zip \
  --output /absolute/new-report-directory
```

The output directory must not already exist. The command verifies the entire
input before creating its report and never executes recorded command arguments.
It writes `README.md`, `timings.csv` and a fuller derived `samples.json` containing
identities, samples and original verified summaries. Compare generated Markdown
and CSV bytes with the committed campaign files. ZIP input records its archive
hash; extracted-directory input cannot claim a ZIP digest.

Verification covers final completed status, original raw digest, matching report
records, output sizes/digests, source stability and paired inputs, exact mode/lane
coverage, unique sample IDs, expected exits, no timeout/signal, order balance,
retained-route records and replay of all 27 summaries. Unknown measurement
phases, altered evidence and missing comparisons fail closed. The initial raw
identity record is intentionally marked failed until the campaign completes;
that initial state is not the final result.

The original reporting implementation passed 24 synthetic regression
tests and replayed both original artifacts: 54 summaries, 1,080 timed command
values and 3,536 command-output files. These are reporting/evidence checks,
**not new compiler tests**. They do not rerun the actual programs or independently
establish all semantic properties represented by the original correctness jobs.

To rerun actual compilation, use the exact source and the
[existing source-build workflow](.github/workflows/compiler-set.yml) and
[build-and-qualify instructions](docs/compiler-set/mixed-compiler.md#build-and-qualify).
Build pinned pristine Lust and the candidate under matched profiles and TinyCC,
run every correctness prerequisite, then the existing `lustr-mixed-bench` runner.
Do not use an unmatched previous binary package, rerun only winning modes, or
replace original commands with dry runs. Profiling must be a separate run so
instrumentation does not contaminate the paired timing campaign.

## What is not measured, and what must improve

This campaign does not establish large unchanged-workspace performance, ts-rust
edit-loop performance, all native Rust support, all mixed-function shapes,
release-profile compilation, isolated application runtime throughput, total CPU
cost, aggregate memory, daemon memory growth, energy, or platform-wide parity.
CPU model, RAM, affinity and governor information were not present in the paired
report. Do not infer them from another runner or upstream Lust's hardware.
Future campaigns should record those fields explicitly where available, including
unknown values, and use enough repetitions for the desired tail conclusions.

The current workflow now collects selected host facts before timing, through
`tools/capture-benchmark-host.py`. It records CPU model, logical CPU count,
process affinity, physical memory, visible cgroup limits, available frequency
governors, OS/kernel/architecture and selected runner image/run identities.
Unreadable fields remain null with an explicit reason. It omits hostnames,
usernames and arbitrary environment variables. These are observations, not
resource controls: affinity is not pinned, physical memory is not a process
budget, and a shared host can change frequency and load during the run.

`LUSTR_BENCH_HOST_RECORD` supplies that record to the paired runner. The runner
copies its original bytes to `host.json`, binds its SHA-256 and parsed contents
into the initial/final campaign identities, and the reporter verifies the
binding. Older campaigns remain without host facts. The collector does not
add CPU/RSS, energy or application-throughput measurements.

## Profile overhead without contaminating timing

The mixed driver has optional phase profiling under `LUSTR_MIXED_PROFILE_DIR`.
The default path performs no profiling clock or file operations. A separate
`tools/profile-mixed-compiler.py` run invokes the existing real mixed qualifier
in a new work directory. It records per-process spans and exact command labels
for the same execution, edit, invalidation, failure and recovery journey.
Nested span durations overlap; their sum is not a command duration. Instrumented
values diagnose where time is spent and are not paired benchmark samples.

The paired runner clears its command environment, so it does not pass the
profiling variable into measured compiler invocations. Its original correctness
requirements and timing modes remain intact. A profiling failure stays a failed
workflow step; an independent valid timing campaign can still run and retain
its evidence.

The current candidate also removes forced disk synchronization from temporary
object copies used only by the current relink. Persistent retained objects,
native object publication and success certificates keep their durability
operations. The temporary files are never named by a persisted certificate.
This targets avoidable work; it is not a measured speedup until an exact-source
campaign completes. The existing failed-link/recovery qualification remains
mandatory.

## Broader coverage and upstream comparison

The [upstream benchmark audit](docs/compiler-set/upstream-benchmark-audit.md)
maps Lust's actual README, large edit workloads, generated-program runtime,
memory observations and CI behavior against these Lustr campaigns. It preserves
their different corpora, compiler versions, dependency adapters and estimators.
The [performance index](performance/README.md) explains where each original
archive, projection and reproduction command lives.

The next performance work is to profile mixed body/signature/cold compilation,
then remove measured redundant work while preserving checked Rust objects, native
retained state, real metadata, invalidation, diagnostics and panic semantics.
The roughly 1 ms no-op median gap does not explain the 44–53 ms edit gaps. This
is a useful prioritization signal, not a profile of a specific expensive function.
The product rejection route also needs targeted measurement without narrowing
full-Rust acceptance or suppressing diagnostics.

A fix must retain its failing predecessor, exact before/after source identities,
correctness evidence and a controlled complete-work comparison. The native
macOS failures, broader mixed coverage, representative performance evidence,
independent review and release admission remain open. Neither this document,
the reporter's tests, nor a green measurement campaign closes those requirements.
