# Performance review: PR #29 source

> Source-bound checkpoint. Statements about the current candidate, required
> follow-up and measured speed below refer to the named source and date. The
> [active correction](active-correction.md) identifies present implementation
> work; the [current performance policy](practical-performance.md) controls new
> claims. Historical successes, incomplete paths and failed results remain scoped
> to their original implementation.

This review records the existing evidence for **PR #29**, not the latest branch
in general. Useful supported-work gains are established; complete practical
performance acceptance and release readiness are not. Follow the
[architecture plan](architecture-plan.md), [completion checklist](completion-checklist.md)
and [qualification review](qualification-review-2026-11-18.md) together.

## Goal and stopping rule

Keep supported work close to Lust, ordinary complementary Rust close to matched
rustc, and obtain useful gains on supported and mixed workloads. Exact Lust
parity, a speedup on every operation and large gains on ordinary complementary
Rust are not required. A necessary correctness or integration repair does not
need to produce a speedup: qualify its behavior and assess material regressions.
Do not weaken validation, alter the requested program/profile or compromise
compilation correctness to remove a small timing difference.

The existing [practical policy at the reviewed source](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/docs/compiler-set/practical-performance.md)
sets these engineering budgets:

| Work | Review budget and evidence |
| --- | --- |
| Supported warm check and build, each separately | At most **10% relative overhead OR 0.5 ms added latency** versus Lust, whichever allows more. In a qualified complete-package assessment, require a lower paired Lust/Lustr ratio bound of at least `1/1.10`, or an upper paired-bootstrap median added-latency bound of at most 0.5 ms. Both operations must also have a rustc/Lustr ratio interval entirely above one. |
| Ordinary complementary Rust | Initial near-stock review budget of **5% relative overhead OR 0.5 ms added latency**, assessed per operation with paired intervals and unchanged normal profiles. No large speedup is required. |
| Mixed work and other modes | Preserve cold, body/signature edits, rejection, runtime, tails and compile-plus-run/test results. Establish useful representative gains without averaging away slower operations or demanding universal wins. |

These are stopping budgets, not proof of identical performance or universal
correctness. Preserve original strict results and failed qualification. The
separately measured approximately 0.3 ms Cargo package/target lookup cost is **not a compulsory
blocker** and cannot be subtracted from command timings measured on another host.
Required identity and invalidation checks stay intact. See the
[selector evidence and limits](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/docs/compiler-set/remaining-entry-cost.md).

## Source and evidence identity

| Identity | Exact value |
| --- | --- |
| Reviewed [PR #29](https://github.com/glopyglerky/lustr/pull/29) head | `a9fd4c6336b1e39c4f584f7da50a437c5c248a0d` |
| Measured merge checkout | `3d23ce4d740a1022bcd3f0dae60c6c4a79853099` |
| Shared complete source tree | `a1a28f2d9c9271a5f0d04349d5dd3657c59f04fe` |
| Hosted compiler-set run | [44420439480](https://github.com/glopyglerky/lustr/actions/runs/44420439480) |

The separately proposed [PR #30](https://github.com/glopyglerky/lustr/pull/30),
head `b94d3713616804fead3f30b18a530e2e81a9510b`, is outside this timing review.
No equivalent fresh measurements for that head are claimed here.

| Retained artifact | ZIP SHA256 |
| --- | --- |
| [Three-compiler dashboard, 14484151306](https://github.com/glopyglerky/lustr/actions/runs/44420439480/artifacts/14484151306) | `0bc591f1419c2aa6665757656c2d217d40861d0dbc503569f447a48c2f569a26` |
| [Source-built paired evidence, 14484610457](https://github.com/glopyglerky/lustr/actions/runs/44420439480/artifacts/14484610457) | `d7ac3ea87ed5487fc9142faa306aa918b858431770d8ec1ccd771b4c8f99284e` |
| [Public slice, 14483540661](https://github.com/glopyglerky/lustr/actions/runs/44420439480/artifacts/14483540661) | `a99a4203e6743e3fc5186ed706e657e2089a7cc6e2016cca7fd287ba46905b00` |
| [Ordinary-project projection, 14484336199](https://github.com/glopyglerky/lustr/actions/runs/44420439480/artifacts/14484336199) | `0516a4b786e71d2a1a63b8543a7222240c22e3bcd80faec502dbceb2680ad5d8` |

A read-only replay regenerated all **29 dashboard rows**: report JSON, README
and SHA256SUMS matched byte-for-byte. The report JSON SHA256 is
`663f520ee8a4470c66327fe83e3218e3f3036c09a9282c448958ef95408ec9f1`.
Existing verifiers checked fixed/native records and all 36 comparisons, plus the
public slice's schedule and original streams. All five ordinary-project
statistical records were independently recomputed from their numeric samples.
That project's public projection retains stream identities, not original
stdout/stderr bytes; those absent streams were not independently reopened.
No compiler, test or benchmark was executed for this review.

## What the measurements establish

All families have 12 pairs. Times below are separate lane medians, in
milliseconds. Ratios are **paired geometric reference time / Lustr time**, not
ratios of those medians; greater than one favors Lustr. Brackets are the recorded
paired-bootstrap 95% intervals.

| Supported operation | Lustr median | Reference median | Paired ratio [95% interval] |
| --- | ---: | ---: | --- |
| Native scalar body edit versus rustc | 7.550 | 69.714 | 8.228 [7.043, 9.489] |
| Public warm check versus rustc | 2.963 | 13.127 | 4.616 [3.695, 5.597] |
| Public warm build versus rustc | 7.705 | 13.143 | 1.990 [1.427, 3.186] |
| Public warm build versus Lust | 7.705 | 6.457 | 0.814 [0.625, 1.047] |

The rustc comparisons show useful supported-work gains. Near-Lust acceptance is
unresolved: warm build's difference of marginal medians is **+1.249 ms / +19.34%**,
outside the engineering allowance by these central values, while its paired
interval spans parity. This difference is not the paired median added latency.
The older PR #26 campaign used a different host, so changed values alone do not
establish a source-caused regression.

The complete-package practical assessor requires **24 matched pairs and complete
qualification**. These 12-pair broad results cannot be relabeled as its passing
assessment. The [independent-package run 44418360324](https://github.com/glopyglerky/lustr/actions/runs/44418360324)
still stops at baseline differential qualification; the
[shared-native run 44418360410](https://github.com/glopyglerky/lustr/actions/runs/44418360410)
refuses incompatible native source trees. Neither admits timing results. Preserve
their failures; a candidate repair does not retroactively repair a frozen baseline.

### Ordinary complementary Rust

| Operation | rustc median | Lustr median | Change in marginal medians | Paired rustc/Lustr [95% interval] |
| --- | ---: | ---: | ---: | --- |
| Clean dependency build | 737.691 | 755.372 | +2.40% | 0.989 [0.945, 1.036] |
| Primed root build | 292.661 | 281.761 | -3.72% | 1.000 [0.905, 1.099] |
| Primed root check | 98.158 | 102.385 | +4.31% | 0.942 [0.776, 1.106] |
| No-op | 19.482 | 13.229 | -32.09% | 1.563 [1.088, 2.395] |
| Compile plus test | 8,016.383 | 8,067.588 | +0.64% | 1.009 [0.991, 1.030] |

The unchanged normal profiles support the near-stock direction for this measured
project. They do not establish a tight all-operation bound: primed check has
**6.10% paired geometric latency overhead**, despite its 4.31% difference in
marginal medians, and a wide interval. Do not describe all five as statistically
passing 5%. No-op Cargo Fresh and forced timestamp rebuilds retain their stated
scope; they are not content-edit measurements.

### Mixed workloads

Mixed body edit is **1.000× [0.954, 1.045]**, inconclusive. The older source's
body-edit win remains historical. Rejection improves, **1.102× [1.040, 1.174]**.
Four operations have intervals wholly favoring rustc:

| Operation | rustc median | Lustr median | Paired rustc/Lustr [95% interval] |
| --- | ---: | ---: | --- |
| Cold build | 269.869 | 300.132 | 0.892 [0.857, 0.928] |
| Check | 64.474 | 74.343 | 0.778 [0.633, 0.898] |
| Check after body edit | 38.148 | 47.032 | 0.732 [0.558, 0.893] |
| Signature edit | 90.207 | 126.363 | 0.769 [0.688, 0.867] |

These are meaningful end-to-end costs. Their cause is not established by the
separate selector measurement. Diagnose a demonstrated substantial cost after
the compiler boundary is correct; do not seek universal wins or weaken semantics.

## Branch integration gaps at the reviewed PR #29 tree

| Earlier repair | Status in PR #29 | Finite action |
| --- | --- | --- |
| [PR #19](https://github.com/glopyglerky/lustr/pull/19), `b73c83de4775044cc0df79af1d6385645b598cd4`: process fixture ownership | Equivalent serial/exclusive-directory repair and same-timestamp lifetime regression are already in [the current process tests](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/crates/lustr/tests/process.rs#L20). | Keep the current implementation; record the earlier PR as superseded after canonical integration. Do not implement it again. |
| [PR #23](https://github.com/glopyglerky/lustr/pull/23), `ea1cd30c5e8425228a6e9a7c482e8790382f03f8`: reproducible macOS sanitizer acquisition/SDK | Its pinned installer and SDK selector are absent. [Current CI](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/.github/workflows/ci.yml#L170) still updates Homebrew `llvm@23` before asserting 23.1.6. | Port the reviewed acquisition/SDK mechanism with an intentional qualified pin and all current capability/corpus assertions. Do not replace the current workflow wholesale. |
| [PR #27](https://github.com/glopyglerky/lustr/pull/27), `ea11ede857f5d47fb3f856d1a35b8a16e19e7be2`: distinct scanner fixture names | [Two scanner tests](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/compiler/native/tools/lust-port/tests/scan_fixture.rs#L215) still share `fixture("human")`; their allocator removes an existing PID/name directory. | Carry the `unknown-package` name correction into the current native tree and update its current integration patch/tree guard. |

The earlier heads are nonancestors of the reviewed head. These are missing
integration and duplicate work, not proof of an intervening revert. Controlled
fixture reproductions establish the ownership defects; the original hosted
scheduler interleavings were not observed.

## Remaining qualification, without an open-ended speed chase

1. Consolidate the established repairs into one identified candidate and retain
   their failure/regression evidence. Keep PR #30 and any later changes explicitly
   separate until their own source and qualification are bound.
2. Complete the integrated capability boundary and semantic repairs, including
   consistent Unicode behavior. The architecture and qualification documents
   define those conditions; speed tolerance does not replace them.
3. Qualify the actual installable source and components on the claimed platform,
   reusing unchanged valid evidence where applicable. Do not rerun known-invalid
   frozen comparisons without addressing their applicability.
4. Once that candidate is qualified, use one predeclared bounded assessment of
   supported, complementary and representative mixed work. Report every result,
   apply the budgets above and stop tuning when they are met. Any further
   investigation must address a concrete unresolved cost or failed condition.
5. Publish the source-bound tables, paired intervals, tails, losing operations,
   profiles, host identities and artifact links. Keep historical results distinct
   from the delivered candidate and reconcile stale pending claims against the
   [qualification review](qualification-review-2026-11-18.md).
