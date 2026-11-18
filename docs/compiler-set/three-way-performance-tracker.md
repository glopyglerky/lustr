# Lustr, Lust, and rustc performance tracker

This is a documentation-only starting snapshot, not a newly executed campaign or a three-way Cargo claim. The prior "Rust / Lustr" speed ratios compare **matched standard Rust/Cargo time to Lustr time**, never upstream Lust to Lustr. A ratio above 1 means Lustr was faster than standard Rust/Cargo for that specific workload.

The three requested lanes are:

- **Lustr:** the packaged primary entry for native workloads, or the routed mixed compiler under Cargo.
- **Lust:** pristine, pinned upstream Lust with its original native compiler. Do not silently replace this with the integrated Lust lane.
- **rustc:** the matching pinned standard Rust compiler, invoked directly for the native scalar fixture or through standard Cargo for Cargo/project/slice rows.

All durations below are **median wall-clock milliseconds (ms), lower is better**. They are observed medians, *not* ratios of paired geometric means. Every row is tagged by benchmark family so unlike operations are not silently compared. The fixed/native and Cargo runners report 12 measured pairs each on the [PR #18 source-built run](https://github.com/glopyglerky/lustr/actions/runs/44245528716). The real-project campaign additionally used two excluded warmup pairs per scenario. The [exact archived evidence](https://github.com/glopyglerky/lustr/blob/84bb8997117b1a582e90a9c9d3ab88985b51753e/docs/compiler-set/evidence/native-eligibility-hosted-20261118/README.md) retains source, package, command, paired-interval and original-artifact identities.

## Missing-result labels

- **N/A:** standalone pristine Lust cannot perform that exact operation/graph/semantics as the declared workflow. Full Cargo coordinator/project rows have no standalone Lust Cargo lane, even when Lust can compile some functions in a separate native fixture. An unsupported language feature must likewise be N/A, with the reason retained.
- **NT:** the operation is supported in principle but that lane was not measured, or a prior attempt failed to qualify. Do not substitute a timing from a different fixture, host, profile or run. Do not report zero, a guessed duration or a speed ratio.
- **Measured:** pass matching source, behavior/output, target, flags and profile gates first; retain raw command/timing records and hash-pinned tool identities. An unsuccessful gate makes the campaign NT until it is rerun successfully, with the failure retained separately.

For every subsequent source revision, retain an additional source-bound snapshot instead of overwriting these historical numbers. Present all compiler columns in the requested order **Lustr | Lust | rustc**. Identify when rustc is standalone versus Cargo, and which Lustr entry was timed. Keep the separately recorded *integrated Lust* performance lane available as underlying evidence, but do not conflate it with pristine Lust.

## Fixed Cargo workflow, same-run snapshot

This is the mixed **Cargo** workload with opt0/debug0 on the fixed corpus. There is no pristine Lust Cargo coordinator lane for these same operations. These N/A cells say nothing about pristine Lust's separate native-supported fixture. Source-bound [fixed-workload table](https://github.com/glopyglerky/lustr/blob/84bb8997117b1a582e90a9c9d3ab88985b51753e/docs/compiler-set/evidence/native-eligibility-hosted-20261118/fixed-workload/README.md).

| Operation | Lustr (ms) | Lust (ms) | rustc via Cargo (ms) |
| --- | ---: | ---: | ---: |
| Body edit | 41.462 | N/A | 45.679 |
| Invalid-code rejection | 29.376 | N/A | 31.010 |
| Compile and test | 69.912 | N/A | 69.378 |
| No-op | 8.969 | N/A | 8.640 |
| No-op check | 8.831 | N/A | 8.338 |
| Check after body edit | 23.306 | N/A | 22.469 |
| Ordinary check | 41.290 | N/A | 37.233 |
| Cold build | 160.697 | N/A | 143.185 |
| Signature edit | 55.081 | N/A | 48.044 |

Two Lustr modes had matched speedups against standard Cargo; seven had matched slowdowns. Original ratios and paired 95% intervals are in the linked evidence. They are more defensible for the direction of a speed comparison than dividing the two displayed medians.

## Native-supported scalar workflow, same-run snapshot

This is the *different* native single-source fixture. Each mode actually exercised the three compiler choices. The Lustr column is the packaged primary **native entry**; Lust is pristine original Lust; rustc is a newly invoked, **nonincremental standalone rustc** reference for each sample. Native warm Lust/Lustr retain their own daemon/cache state. In particular, the standalone rustc no-op is a new compiler invocation and **is not Cargo Fresh**. Source-bound [native comparison table](https://github.com/glopyglerky/lustr/blob/84bb8997117b1a582e90a9c9d3ab88985b51753e/docs/compiler-set/evidence/native-eligibility-hosted-20261118/fixed-workload/README.md).

| Operation | Lustr native entry (ms) | Pristine Lust (ms) | Standalone rustc (ms) |
| --- | ---: | ---: | ---: |
| Body edit | 5.175 | 5.269 | 36.882 |
| Invalid-code rejection | 15.913 | 1.861 | 14.959 |
| Compile and test | 28.136 | 28.345 | 50.719 |
| No-op | 2.852 | 3.102 | 34.490 |
| No-op check | 1.535 | 1.584 | 10.122 |
| Check after body edit | 1.604 | 1.749 | 9.786 |
| Ordinary check | 1.799 | 2.081 | 10.672 |
| Cold build | 41.679 | 38.652 | 35.942 |
| Signature edit | 4.497 | 4.572 | 35.614 |

The 15.913 ms packaged-native rejection is a visible regression from pristine Lust's 1.861 ms median. Do not lose that result inside the successful mixed-Cargo rejection figure. The separate integrated-Lust lane is also retained in the archived original; this three-column presentation deliberately reports the pristine reference requested by the user.

## Public upstream native slice, same-run snapshot

This six-operation slice is the *adapted/pruned* public goport-util-small fixture, not full unchanged ts-rust or a full-Rust compatibility proof. Lustr is the packaged standalone entry, Lust is pristine upstream, and standard Rust uses Cargo. Native daemon retention and Cargo Fresh semantics differ for no-op operations. Source-bound [slice evidence](https://github.com/glopyglerky/lustr/blob/84bb8997117b1a582e90a9c9d3ab88985b51753e/docs/compiler-set/evidence/native-eligibility-hosted-20261118/public-slice/README.md).

| Operation | Lustr entry (ms) | Pristine Lust (ms) | rustc via Cargo (ms) |
| --- | ---: | ---: | ---: |
| Cold check | 160.549 | 131.451 | 258.758 |
| No-op check | 30.118 | 1.151 | 8.446 |
| Cold build | 289.436 | 262.545 | 356.352 |
| No-op build | 31.843 | 3.142 | 8.529 |
| Compile and run | 30.894 | 3.981 | 9.067 |
| Runtime only | 0.877 | 0.881 | 0.720 |

Warm primary-entry operations are substantially slower than pristine Lust on this slice despite the integrated native implementation retaining its speed. This is an observed primary-entry overhead, not proof of any single cause.

## Unchanged full-Rust coordinator project, same-run snapshot

This campaign uses the actual `crates/lustr` project with normal dev/test defaults, including debuginfo=2 and two Cargo jobs. Standalone Lust cannot compile that full Rust graph, so its cells are N/A. These workloads are **not** native object acceleration. Source-bound [project timing table](https://github.com/glopyglerky/lustr/blob/84bb8997117b1a582e90a9c9d3ab88985b51753e/docs/compiler-set/evidence/native-eligibility-hosted-20261118/project-public/README.md).

| Operation | Lustr (ms) | Lust (ms) | rustc via Cargo (ms) |
| --- | ---: | ---: | ---: |
| Clean dependency build | 522.133 | N/A | 518.860 |
| Primed root build | 146.981 | N/A | 147.217 |
| Primed root check | 57.071 | N/A | 56.018 |
| No-op | 9.989 | N/A | 9.987 |
| Compile and test | 4801.302 | N/A | 4775.826 |

None of these five paired intervals established a Lustr speedup. The clean dependency build and no-op intervals were below one; the other three crossed one. Do not interpret the latter as proven equivalence.

## Future campaign and report requirements

1. Emit a source-bound **three-column table in every new benchmark report**. For fixed native and public slice, populate all three from the same qualified campaign, using pristine Lust. For mixed Cargo and full-Rust projects, retain explicit Lust N/A with a scope reason. If a new native-capable workload fails to run, report NT and retain the reason instead of inventing a result.
2. Print both median and p95 **milliseconds per lane** and keep all raw timing samples, pairing/order, source/tool hashes, host, compiler/version flags, cache policy, oracle/correctness and coverage results. Separately retain paired ratios and confidence intervals; do not calculate them as ratios of displayed medians.
3. Distinguish **native standalone**, **mixed Cargo**, **public slice**, and **unchanged real project**. Never cross-fill a missing value with a timing taken from a different benchmark family. Include a compiler-coverage/support matrix or a per-row N/A reason.
4. Keep **pristine Lust** source and the full Rust implementation pinned and unchanged. The integration's internal Lust lane remains a separate fourth trace for speed-retention diagnosis, not a substitute for a pristine baseline.
5. Compare a change to its parent with controlled before/after runs on matching host/workload/profiles, preserving all nine mixed Cargo modes and the six native/pubic slice modes. A prior successful snapshot does not establish a causal speedup for a new code change.
6. Keep failed and historical campaigns immutable and linked. A missing Rust/Lust lane must be conspicuous in the published results, not silently omitted.

This first snapshot is fully sourced from the completed [PR #18 Compiler set run](https://github.com/glopyglerky/lustr/actions/runs/44245528716), checked-out merge `df8f55aa0430ef81f27e7f403d102256b7fc298b`, tree `fdb6af7d1ea50c4b63fd8b6a217633979deedf22`, compiled full-Rust SDK `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. Existing source heads and archived artifacts were not changed for this report. See the [PR #16 tracking acceptance contract](https://github.com/glopyglerky/lustr/pull/16#issuecomment-6972172800) for implementation and CI requirements.
