# Performance evidence index

[PERFORMANCE.md](../PERFORMANCE.md) explains the measured results, methodology,
regressions and remaining coverage. The [README summary](../README.md#numbers-and-current-qualification)
keeps the headline table visible. Neither document claims general Rust speed
parity or release readiness.

## Completed fixed-corpus campaigns

| Measured candidate | Full table | Every timed value | Original qualification artifact |
| --- | --- | --- | --- |
| `c70eb1c` | [27 comparisons](2026-11-17-c70eb1c.md) | [540 values](2026-11-17-c70eb1c.csv) | [Original ZIP](archives/2026-11-17-c70eb1c-qualification.zip) |
| `afb17a8` | [27 comparisons](2026-11-17-afb17a8.md) | [540 values](2026-11-17-afb17a8.csv) | [Original ZIP](archives/2026-11-17-afb17a8-qualification.zip) |
| `048d824` | [27 comparisons](2026-11-17-048d824.md) | [540 values](2026-11-17-048d824.csv) | [Original ZIP](archives/2026-11-17-048d824-qualification.zip) |
| `dec0ce2` | [27 comparisons](2026-11-17-dec0ce2.md) | [540 values](2026-11-17-dec0ce2.csv) | [Original ZIP](archives/2026-11-17-dec0ce2-qualification.zip) |
| `74e14e7` | [27 comparisons](2026-11-17-74e14e7.md) | [540 values](2026-11-17-74e14e7.csv) | [Original ZIP](archives/2026-11-17-74e14e7-qualification.zip) |
| `40d2ac3` | [27 comparisons](2026-11-17-40d2ac3.md) | [540 values](2026-11-17-40d2ac3.csv) | [Original ZIP](archives/2026-11-17-40d2ac3-qualification.zip) |
| `5a3f493` (schema 2) | [36 comparisons](2026-11-17-5a3f493.md) | [648 values](2026-11-17-5a3f493.csv) | [Original ZIP](archives/2026-11-17-5a3f493-qualification.zip) |

Each ZIP is byte-identical to the original GitHub Actions artifact. The
[manifest](campaigns.json) binds its run, artifact, actual checkout/tree,
archive/raw-data hashes and derived-file hashes. It is retained in Git so
Actions expiration does not remove the original qualification record.
The seven fixed campaigns retain **3,888 timed commands, 198 comparisons and
12,768 recorded command-output files**. The first six preserve their original
schema-1 scope: 540 timings and 27 comparisons each. The seventh adds matched
standalone rustc, with 648 timings and 36 comparisons. Its 1,135 raw records
include 1,080 recorded commands: 648 timed and 432 untimed, with 2,160 output
files. These counts describe recorded commands, not every descendant process.

The archives do not contain the entire compiler binary toolchain. The
`40d2ac3` paired campaign passed its own checks although its separate profiling
step failed; that original source-built job remains failed. The new `5a3f493`
source-built job and its original profile validation both passed. Their source,
results and qualification limits remain separately identified.

These are the separate `lustr-mixed-bench` fixture campaigns produced and
already published by this public repository's GitHub Actions workflow. Review
of these complete artifacts found public fixture/source/build records and
hosted-runner paths, with no private workspace content or credential material.
Their original bytes and hashes are preserved. The private arbitrary-workspace
`lustr-bench <spec.json>` reports described in
[benchmark-method.md](../docs/compiler-set/benchmark-method.md#report-schema-and-publication)
still require the documented sanitization before public use. This archive does
not change that rule or authorize publishing private reports.

From the repository root:

```sh
python3 tools/test-mixed-performance-report.py
python3 tools/check-mixed-performance-tables.py
python3 tools/check-mixed-performance-archives.py
python3 tools/report-mixed-performance.py \
  performance/archives/2026-11-17-5a3f493-qualification.zip \
  --output /absolute/new-report-directory
```

The last command refuses an existing output directory. It verifies the input
before writing a report and never runs a command copied from a log. Corrupted
or incomplete original evidence fails verification. Historical table replay
does not test a newly built compiler.

## Record the next campaign

The [source-built workflow](../.github/workflows/compiler-set.yml) builds matched
pristine Lust, integrated Lust, packaged Lustr and the full Rust/Cargo reference.
All correctness prerequisites must pass before its paired measurements. It
preserves cold, no-op, body/signature edits, check variants, rejection and
compile-plus-test separately, with every slower sample.

The current [schema-2 runner](../docs/compiler-set/native-rustc-comparison.md)
also measures matching standalone rustc on the same native source. It requires
648 timed commands and 36 summaries at twelve pairs, adding rustc→entry to the
existing three contrasts. Its first completed campaign is `5a3f493`, actual
checkout `b3393a4a2907228516546cd34f1a5c60aaf6aaaa`. The six earlier rows remain
schema 1 with their original 540 timings/27 comparisons. They have no native
rustc lane and cannot be retroactively interpreted as that comparison.

The same workflow now captures selected host facts and performs phase profiling
in a separate qualification directory. Host facts are bound into new campaign
identities. Profiling is disabled for all paired samples. The full artifact
retains source/build receipts, commands, outputs, fixture files, profiles,
host records and failure information.

The first [completed phase report](2026-11-17-dec0ce2-profile.md) is linked
separately from its paired timings. Its original traces and host record are
inside the fourth qualification ZIP; their instrumented values are not pooled
with the paired timing values. The [v2 diagnosis at 40d2ac3](2026-11-17-40d2ac3-profile.md)
retains its original framing failure, separately verified reconstruction and
writer correction. It does not relabel the original profiling run as passed.

The [fresh v2 profile at 5a3f493](2026-11-17-5a3f493-profile.md) passed its
original validation. It retains 26 compiler profiles across 15 qualification
commands and 35 successful native responses. Its original summary and Markdown
replay byte for byte without trace reconstruction. Instrumented durations remain
separate from the 3,888 paired timing values. Host snapshots exist for `dec0ce2`,
`40d2ac3` and `5a3f493`; the other four archives contain no host snapshot.

After a new completed campaign, download its **original ZIP**, verify its
Actions artifact digest, and run the reporter against it. Keep the ZIP as a new
dated archive, generated CSV/table as new files, and add its exact identities
to the manifest through a reviewed PR. Run both committed-data checks. Preserve
older files. Do not replace an older report with new timings, infer host facts
for a historical run, or publish a partial/failed campaign as a completed one.

The [upstream audit](../docs/compiler-set/upstream-benchmark-audit.md) lists the
additional larger-workspace, edit-family, runtime and resource campaigns needed
to match the breadth of Lust's own studies. Those remain unmeasured here.

## Unchanged project coverage

The separate [coordinator campaign](../docs/compiler-set/coordinator-performance.md)
runs this repository's actual coordinator package under its normal dev/test
profiles. Its five scenarios preserve test execution, declared cold/warm state,
compiler identity and raw timing values. A strict public projection omits raw
private output and environment paths while retaining their original digests.
This differs from the complete original ZIPs above.

| Measured candidate | Original project evidence and recovery | Timed values | Comparisons |
| --- | --- | ---: | ---: |
| `40d2ac3` | [Completed campaign](projects/2026-11-17-40d2ac3/README.md) | 120 | 5 |
| `5a3f493` | [Completed campaign](projects/2026-11-17-5a3f493/README.md) | 120 | 5 |

These two campaigns retain **240 measured timings and 10 comparisons** outside
the fixed-corpus totals. They measure the same actual coordinator package, not
two different large projects. Normal dev/test debug=2 is outside current
native-object eligibility; primed root rebuilds change only a timestamp and are
not body/signature edits. The latest clean-build comparison is inconclusive;
its other four intervals favor matched Rust. Both full result tables remain
visible, without a pooled or cross-run improvement claim.

For each campaign, the six public files and original public/encrypted ZIPs are
retained byte for byte. Actual authenticated decryption, archive verification
and source-bound public reconciliation passed for **600 original files and
510 output streams in each campaign**.
The private key/plaintext remain private. Both success and failure paths attempt
encrypted retention. The public archive checker verifies both campaigns'
public/ciphertext bindings and replays all ten comparisons without claiming a
new decryption or compiler execution.

## Charts and exact source recovery

The [5a3f493 four-panel chart](plots/2026-11-17-5a3f493/README.md) includes all
36 comparisons, including native cold/rejection losses against matched rustc.
The [40d2ac3 chart](plots/2026-11-17-40d2ac3/README.md) retains its original
27-comparison scope. Both show conditional intervals and exact source/data
bindings, with a JSON projection and PNG beside the SVG. Recreate a plot in a new
directory with Python 3.10+ and Matplotlib 3.10.8:

```bash
python3 tools/plot-mixed-performance.py \
  performance/archives/2026-11-17-5a3f493-qualification.zip \
  --output /absolute/new-plot-directory --png
```

The generator first runs the existing strict evidence replay; it adds no new
estimator or excluded modes. These plots are individual campaigns, not a
cross-run causal trend. The exact merge source bundles preserve
[`b3393a4` for 5a3f493](sources/2026-11-17-5a3f493/README.md) and
[`3b4b0bd` for 40d2ac3](sources/2026-11-17-40d2ac3/README.md), with their declared
prerequisites, after live PR refs move. Recovery requires those prerequisites
and does not restore original compiler binaries or requalify a build.

## Qualification limits

The [40d2ac3 native Linux gate](../docs/compiler-set/measurements/2026-11-17-native-linux-40d2ac3/README.md)
passed 5,527 ordinary differential cases, then failed one of the 5,527 sanitized
cases: the generated `unicode_props_hash` program exceeded its ten-second
execution deadline after successful linking. Later fuzz and benchmark-smoke
stages were not reached. The [bounded local diagnosis](../docs/compiler-set/measurements/2026-11-17-unicode-sanitizer-diagnosis/README.md)
instead encountered a LeakSanitizer fatal error; it neither reproduced the
hosted timeout nor supplied a passing sanitizer result.

The fresh [`5a3f493` full native gate](../docs/compiler-set/measurements/2026-11-17-native-linux-5a3f493/README.md)
also failed at `unicode_props_hash`, after 5,527 ordinary and 5,526 sanitized
passes. Its original ZIP and source identities are retained. The separate
Compiler set run passed eight non-macOS jobs,
while macOS compatibility was cancelled before
any step executed. Source-built correctness, completed timing and source
equality do not convert the failed native gate into a pass or qualify macOS.

## Component copy-policy experiment

The [separately retained component experiment](components/2026-11-17-40d2ac3-link-input/review.md)
contains 24 matched pairs for sixteen copies of a generated 64 KiB input. It
compares the historical copy-and-sync policy with disposable staging and retains
the original ZIP, every pair, source hashes and eight passing isolation/recovery
tests. These 48 component timings are outside the compiler campaign totals.

This does not compile an application or measure a complete link. The pre-merge
compiler already omitted synchronization of disposable copies, so this experiment
does not establish an additional gain caused by the merge. Persistent retention
keeps its synchronization, and complete-work compiler speed remains a separate
measurement.
