# Bounded native artifact reuse — 2026-11-15

This PRIVATE experiment adds a separate process-local retained ABI. A successful unchanged executable can be reused only after request, source/runtime-tree, observed-read, negative-module, path-binding, Cargo-graph and output checks. Every source edit recompiles from original source; no transformed AST is retained. Runtime-source changes retire the instance. Unknown reads recompute. The product, published package and original one-shot route are separate. Full host header/linker observation, product lifecycle, native rlib/general Cargo and full native language/speed acceptance remain OPEN. The measurements assume a stable host installation and do not admit the prototype as a general build cache.

Nine samples, shared Intel Core Ultra 5 235T Linux x86_64, CPUs 10–13, Cargo jobs 2. Both native components are opt3/CGU1/ThinLTO/line-table builds from the pinned Rust 1.102 SDK. Applications use opt0/debug0/unwind/overflow checks/debug assertions. Prior direct ABI is the successful v34 component; the new retained process starts afresh for each sample, retaining one artifact during that sample. Cold includes process startup and the first request; retained repeat/no-op includes framed request/response and logging, with ABI-call timing separately recorded. Prior direct and single-file stock start a process per request. Coordinator stock uses real Cargo library/bin compilation, fresh target AND build directories per sample, then actual Cargo Fresh. OS pages are not flushed. Engines interleave with alternating order. This compares equivalent requested native program behavior, not rlib interoperability or equal process lifetimes.

All engines emit real ELF and compare stdout/stderr/status for the mixed Drop/unwind workload or five real unchanged-coordinator CLIs (including actual plan refusals and invalid-CLI status 2). Rejection is status 1 and recovery compiles again. No rustc fallback exists in either native probe; stock is an independent oracle. Raw source/compiler hashes, commands, timings, statuses and diagnostics are retained. Compile-plus-runtime is not Cargo test.

## Unchanged full coordinator

Medians ms, lower is better.

| Phase | Prior direct native | Retained ABI | Stock Cargo | Prior + five CLI | Retained + five CLI | Stock + five CLI |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 295.181 | 290.666 | 487.250 | 299.486 | 294.021 | 490.034 |
| repeat | 288.505 | 10.975 | 11.381 | 292.251 | 16.801 | 13.986 |
| noop | 285.644 | 8.401 | 11.559 | 289.526 | 12.011 | 14.322 |

## Mixed native inference

| Phase | Prior direct native | Retained ABI | Stock rustc |
| --- | ---: | ---: | ---: |
| cold | 192.079 | 195.408 | 93.741 |
| repeat | 197.023 | 8.016 | 92.204 |
| noop | 191.412 | 7.409 | 90.083 |
| body-edit | 185.268 | 186.954 | 88.362 |
| signature-edit | 176.570 | 174.244 | 86.292 |
| type-edit | 184.863 | 181.914 | 86.652 |
| macro-edit | 184.451 | 182.457 | 90.690 |
| rejection | 110.240 | 101.499 | 31.825 |
| recovery | 189.112 | 183.881 | 86.808 |

## Evidence and limits

Raw events: 1352. Raw SHA256 `67fc013dc3fb194e22306bfc4157673a9b450478979dc027b3712cad396fbb49`; summary SHA256 `6de335114343e0e56e9d08804b2b1d91d5143c5e81d2c4643306cc3562d77ffa`. All nine values for each row remain in summary.json. Native cold/edit regressions must remain visible; a faster no-op does not complete the native speed requirement. Exact first symlink stale-success failure and the conservative coordinator-proof miss were preserved before correction. Independent frozen review found two P1s (directory alias binding and inherited disk-cache bypass) and one P2 (entry allocation before node budget); root corrected them and qualified the actual regression. Later environment/format/test/report changes are not independently reviewed.

The first timing harness stopped after sample zero returned a correct cache hit for an identical nominal body-edit variant. Its source/raw logs remain in measure1; the successful campaign uses odd variant indices so every edit changes source bytes. All mixed nine-sample scopes settled in measure2. A later coordinator harness assertion incorrectly expected a plan refusal to succeed; its real status 2 and raw logs remain. Only coordinator measurements were then run in measure3, whose raw log imports the fully settled mixed scope with provenance tags. No compiler/runtime failure was hidden or retried through another compiler. Host OS/header/linker assumptions and shared-host noise limit these results. No cache admission, product rollout, release/privacy acceptance, whole-stack memory safety or complete Rust support is claimed.
