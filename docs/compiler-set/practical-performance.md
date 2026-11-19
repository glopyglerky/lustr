# Practical performance assessment

The [exclusive coverage contract](exclusive-coverage-contract.md) requires
supported work near stock Lust speed and complementary work near matched stock
rustc speed. Assess them separately. The complete compiler must also satisfy
ownership, source selection, correctness and actual artifact integration.
A coordinator or full-driver wrapper cannot pass architecture by meeting a
timing budget.

Revised 19 November 2026 to enforce the repeated near-stock requirement. The
relative budgets below remain engineering choices, not the user's exact
numerical wording, universal guarantees or evidence of a pass. Fix assessment
and workload definitions before a new campaign; do not relax them after seeing
the results.

The [candidate policy at cb0721d4](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/practical-performance.md)
and [18 November checkpoint](performance-review-2026-11-18.md) allowed either a
relative bound or 0.5 ms added latency. Preserve those historical assessments
and their original decisions. Their absolute alternative no longer establishes
a new near-stock claim. A 0.2 ms baseline plus 0.4 ms is three times slower,
despite satisfying the old absolute allowance.

## Supported warm work

For each matched warm check and build, require the 10% relative overhead bound.
Match source, configuration, complete commands, artifacts and applicable
correctness. No averaging a slow operation away with a faster one.

Replay exact-source evidence before assessment. Require the lower paired 95%
Lust/Lustr ratio bound to be at least `1 / 1.10`. Both warm operations also need
a matched Rust/Lustr ratio interval entirely above one. Report paired absolute
added-latency intervals and the former 0.5 ms reference for diagnosis, but they
cannot override a failed or inconclusive relative comparison. These are
conditional intervals for that repeated workload and host. Noise is a reason
to improve the predeclared measurement design, not to report a pass.

The paired ratio statistic is the geometric mean of within-pair
reference/candidate ratios, with the existing paired bootstrap interval. It is
not a ratio of marginal medians or a bound on every individual request. Retain
sample distributions and latency tails independently.

`tools/assess-warm-performance.py` and its version-1 schema replay the historical
practical rule. Their `meets_practical_target` field is not current near-stock
admission. The integrated selective compiler and its complete current campaign
remain unfinished; no current performance pass is inferred from that tool.

The existing complete-package campaign requires 24 matched pairs plus applicable
qualification. A different proposed design must be justified and fixed before
measurement. Prior 12-pair characterization cannot silently become its passing
result. Retain sources and failed/strict results when interpreting old data.

## Ordinary complementary work

Use unchanged projects, dependencies, requested features and normal profiles.
The complementary implementation must be the actual integrated selective
implementation. A historical packaged-rustc measurement retains its historical
scope and does not qualify the new complement.

Use 5% relative overhead as the per-operation near-stock review budget. Require
the lower paired 95% stock-Rust/Lustr ratio bound to be at least `1 / 1.05`.
Report absolute added latency separately without an alternate passing rule.
Investigate an out-of-budget or inconclusive result. Gains on Lust-supported
work do not cancel it. No large complementary speedup is required. Do not reduce
debug information, alter panic behavior or substitute an easier program/profile
to improve the ratio.

## Other work and overhead diagnosis

Measure boundary overhead as components are integrated, before accepting a
design that spends Lust's gain on repeated work. Qualification probes and
independent oracles run outside the user request. Actual dispatch, required
preparation, conversions, initialization, transport and publication stay inside
its measurement. A no_std or transport change needs applicable operation and
retention evidence before replacing the existing path; neither is a prerequisite
that can delay the sole compiler goal.

Compare equivalent work. If stock Lust cannot produce a requested artifact,
identify the matching supported operation and separately report the extra
artifact service and complete public request. A Lust run and a full Cargo
library build are not interchangeable denominators. Do not call an isolated
body-emission ratio the full compilation speedup.

Keep cold and cached-cold builds, no-op, body/signature/type/trait/macro/dependency
edits, rejection, priming, linking, tails, memory, application execution and
compile-plus-run/test visible as separate results. The warm allowance does not
silently admit regressions in those modes. Review each material regression
against the requested work, without inventing a new universal numeric bound.

Record pristine Lust, integrated Lust, public Lustr and matching stock Rust
identities. Observe actual compiler phases and ownership. Separate worker time,
loading/transport/validation, complementary queries, metadata and linking.
Inclusive spans cannot be added as independent costs. Instrumented profiles and
cross-host differences do not establish the cause of a latency change.

The [Lust review](upstream-lust-review-2026-11-19.md) records why retained state
and precise invalidation dominate its edit advantage. The
[Rust review](upstream-rust-review-2026-11-19.md) records current thread-default
changes and the need to match explicit frontend/backend concurrency. Compiler
build/CI speed and generated compiler request latency remain different measures.

## Publication and retained evidence

Preserve raw samples, workload/profile/cache definitions, source and binary
hashes, runner and C/linker identities, failure records and replay commands.
Historical gains and losses remain in the [performance checkpoint](performance-review-2026-11-18.md).
This document makes no new measured performance claim.

Publish an installable replacement only after the exact components satisfy the
exclusive contract and relevant runtime, rejection, dependency, macro, panic/drop,
invalidation and platform qualification. State all tested scope and unresolved
limits. Source cleanup, a manifest, a guard, or a passing compatibility path
cannot establish complete Rust coverage or the required performance.
