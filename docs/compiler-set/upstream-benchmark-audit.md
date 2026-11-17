# Upstream benchmark audit

Reviewed 17 November 2026. This note audits published evidence and benchmark
source. It does not report a new compiler run. Lustr's measured results, raw
timing projections and reproduction instructions belong in
[PERFORMANCE.md](../../PERFORMANCE.md).

## Reference and reported upstream results

Upstream `t3dotgg/lust-lang` default branch `next` was still
`240e08d91d4aed0299639f6f11fd522912b248f9` at this audit, matching
[`upstream/lust.pin`](../../upstream/lust.pin). Its
[README at that revision](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md)
reports these separate October 2026 experiments:

| Experiment | Reported Lust observation | Comparison and boundary |
| --- | --- | --- |
| Warm body edit, `ts_goport`, 148k lines | 23.3 ms median, 27.8 ms p95, 28 accepted states | Cargo check median 1,386 ms, reported 59x. One-function edit followed by reversion. |
| Warm body edit, `goport_core`, 430k lines | 18.8 ms median, 28.9 ms p95, 28 accepted states | Cargo check median 4,742 ms, reported 252x. Same favorable edit shape in a larger crate. |
| Signature edit, those two crates | 53.5/109.1 ms medians; 4,446/9,924 ms p95 | Medians conceal expensive tail cases. Preserve both. |
| Cold check including dependencies, same features | 15.7 s and 27.4 s in two sandboxes | Cargo 37.1 s and 71.8 s, reported 2.4x/2.6x. Lust uses dependency adapters. |
| Generated application runtime, 24 larger fixture projects | Lust-built total 64.0 s | Cargo debug 15.7 s; release 0.93 s. Lust was 3.8x to 4.9x slower per project than debug. |
| Daemon memory, older 523k-line tree | 4.84 GB after first check; 5.7 GB peak over 300 requests | Newer v0.7.0 tree held 6.6 GB after first check; its 300-request peak was unmeasured. |

These are upstream results on its stated compiler and corpus revisions, not
Lustr observations. The README identifies Lust `d80937461`, the ts-rust PR
branch `7459613a7` for the warm table, and fork `ae1c76940` for cold checking.
The machines were shared-host KVM sandboxes with 16 vCPUs of a Ryzen 9 9950X,
32 GB RAM and rustc 1.99.0. Upstream reports large load-dependent variation.
Those machine facts must not be copied into Lustr's runner records.

The ts-rust input has explicit source rewrites and disabled development
features. Lust's adapters replace real dependency crates. The older
[slice corpus](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/bench/slices/README.md)
also removes unsupported items and supplies documented stand-ins. Lustr's
unchanged Cargo graph measures a different capability. Keep a preserved
upstream corpus campaign and an unchanged-project campaign separately named.

## What Lustr already records

The [existing paired protocol](mixed-compiler.md#paired-integration-measurements)
and [published report](../../PERFORMANCE.md) already provide five lanes, nine
modes, twelve matched observations per lane/mode, all 540 timed values,
correctness prerequisites, compiler identities and independent statistics
replay. The comparisons isolate pristine Lust, integrated Lust, the product
entry and the matched complete Rust/Cargo path. Reported regressions remain
visible. The missing work is broader measurement and durable full evidence,
not inventing a replacement for this recorder.

| Coverage | Existing Lustr evidence | Additional campaign needed |
| --- | --- | --- |
| Cold, no-op, body/signature edits, checks, rejection, compile plus test | Small native scalar corpus and fixed mixed Cargo graph | Larger unchanged workspaces and more function shapes, retaining every existing mode |
| Upstream supported path | Direct pristine/integrated/entry comparison | Pinned upstream slices and full supported ts-rust edit workload |
| Edit invalidation | Current bounded body/signature fixtures | Generic, macro-body, trait-obligation, inference, cross-module signature, return-type, derive, type/impl/module-tree and multi-file edits |
| Invalid programs | Dedicated rejection mode | Accepted and rejected states within each edit family, with cold/warm agreement and recovery |
| Application execution | Correctness runs and compile-plus-test timing | Separate generated-program throughput and end-to-end edit/build/test latency |
| Resources | Paired wall time | CPU, scoped peak RSS, aggregate process-tree memory if measured, daemon growth and restart costs |
| Provenance | Exact source, tools, commands and raw evidence digests | Hardware/OS/affinity fields with explicit unknowns, and durable complete logs beyond Actions retention |

An unsupported case is a coverage result. A timeout is a failure to complete.
Neither may become a fast successful sample. Compiler release-profile
optimization and release-profile application builds also need separate labels.

## Reuse upstream methodology carefully

[Upstream speed documentation](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/docs/speed.md)
separates cold work, daemon startup, no-op, cached restart, edit kinds, build,
runtime and memory. Its slice campaign uses 120 body edits across 24 functions,
five cycles of other edits, warm/cold comparisons and observable program
agreement. Per-request native timings, phase counters and patch/reuse reasons
survive in
[checked-in slice JSON](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/bench/slices/results/2026-10-15-w-bench4-tsrust-all4.json).
That file records an older compiler, `4f4c96f84`. It cannot substitute for the
later README's full ts-rust campaign. No checked-in per-state JSON for that
October 17 headline campaign was located during this audit.

Do not interchange estimators. Upstream
[`speed_slice.py`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/speed_slice.py)
uses ratios of medians and interpolated p95 values, choosing the faster
Cargo configuration by each row's median. Lustr reports a paired geometric
ratio and nearest-rank p95. Its twelve-sample p95 equals the maximum. Show
absolute samples, sample counts, estimator definitions and conditional
intervals alongside either comparison. Different historical runs cannot
establish a causal improvement between compiler commits.

Upstream [`tsedit.py`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/tsedit.py)
accepts an explicit `--lust` path and writes every edit state with `--json`.
Before using its output for Lustr qualification, apply an owned validator:
the upstream script intentionally permits false rejects, and its timed
accepted-state selection checks `cargo_ok` alone. Its `cargo timeout` class
also does not enter the `hangs` count used by the final failure predicate.
Require success from both candidates for accepted-state speed claims, preserve
both rejection outcomes, and fail qualification on every compiler timeout.
Keep these checks outside the pristine source reference.

## CI and admission

Upstream's
[CI workflow](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/.github/workflows/ci.yml)
calls [`scripts/ci.sh`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/ci.sh),
whose benchmark section runs smoke cases. The full ts-rust campaign is opt-in.
[`speed-row.sh`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/speed-row.sh)
checks correctness, explicitly does not fail for poor speed, and deletes its
temporary JSON after printing. A successful upstream speed row therefore
does not establish a performance admission gate or permanent evidence archive.

At the pinned upstream commit,
[Linux CI](https://github.com/t3dotgg/lust-lang/actions/runs/39780031006/job/119828625752)
reached 363 passing daemon scenarios, then failed the sanitized differential
suite with 5,525 passes and one `unicode_props_hash` build timeout at its
10-second deadline.
[macOS ARM64 CI](https://github.com/t3dotgg/lust-lang/actions/runs/39780031006/job/119828626209)
failed linking on undefined `_personality`. These are observations of the
pristine upstream run; Lustr's separately recorded platform results must be
assessed at their own exact source revisions.

Prioritize the measured mixed cold/body/signature overhead and entry rejection
cost, with profiling kept outside timing. Retain the failing predecessor and
rerun the complete matched campaign after a fix. Then expand the corpus and
resource measurements above. Correctness, independent review, supported-path
preservation and general mixed coverage remain separate release requirements
under the [integration policy](upstream-lust.md).
