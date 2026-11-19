# Practical performance assessment

The [exclusive coverage contract](exclusive-coverage-contract.md) requires
supported work near stock Lust speed and complementary work near matched stock
rustc speed. Assess them separately. The complete compiler must also satisfy
ownership, source selection, correctness and actual artifact integration.
A coordinator or full-driver wrapper cannot pass architecture by meeting a
timing budget.

This policy retains the existing engineering budgets and assessment rules from
the [candidate policy at cb0721d4](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/practical-performance.md)
and [18 November checkpoint](performance-review-2026-11-18.md). The numeric
budgets were engineering choices informed by prior measurements. They are not
the user's exact numerical wording, universal guarantees or evidence that the
current source passes. Do not relax them after observing a new campaign.

## Supported warm work

For each matched warm check and build, allow 10% relative overhead or 0.5 ms
added latency, whichever allowance is larger. Preserve the original strict
comparison alongside the practical decision. Match source, configuration,
complete commands, artifacts and applicable correctness. No averaging a slow
operation away with a faster one.

The complete-package assessor replays its evidence first. Its existing paired
ratio interval assesses the relative budget, and a separately reported paired
bootstrap interval assesses median added latency. The upper added-latency bound
must be at most 0.5 ms, or the lower Lust/Lustr ratio bound at least `1 / 1.10`.
Both warm operations also need a Rust/Lustr ratio interval entirely above one.
These are conditional intervals for that repeated workload and host.

The existing complete-package campaign requires 24 matched pairs plus applicable
qualification. A different proposed design must be justified and fixed before
measurement. Prior 12-pair characterization cannot silently become its passing
result. Retain sources and failed/strict results when interpreting old data.

## Ordinary complementary work

Use unchanged projects, dependencies, requested features and normal profiles.
The complementary implementation must be the actual integrated selective
implementation. A historical packaged-rustc measurement retains its historical
scope and does not qualify the new complement.

Use 5% relative overhead or 0.5 ms absolute overhead as the per-operation
near-stock review budget, whichever allowance is larger. Report paired intervals
and uncertainty for each operation. Investigate an out-of-budget result. Gains
on Lust-supported work do not cancel it. No large complementary speedup is
required. Do not reduce debug information, alter panic behavior or substitute
an easier program/profile to improve the ratio.

## Other work and overhead diagnosis

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
