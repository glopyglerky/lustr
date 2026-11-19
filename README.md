# Lustr

**One compiler: Lust for everything it supports, with imported upstream Rust
providing the complement inside the same compilation.**

Lustr aims to replace rustc while preserving Lust's fast frontend and incremental
compilation. An unchanged program can use both implementations. The compiler
uses resolved semantic facts before emission and shares the checked state,
dependencies and runtime contracts needed to compose their output.

The required product does not try the whole program with Lust and then retry it
with rustc. It does not require independently reimplementing the rest of Rust or
running an extra complete Rust frontend on every Lust-supported request.

## Start here

| Document | Purpose |
| --- | --- |
| [Architecture plan](docs/compiler-set/architecture-plan.md) | Fixed implementation contract, semantic ownership and the complementary subset. |
| [Completion checklist](docs/compiler-set/completion-checklist.md) | Exact remaining work, in dependency order, with completion criteria. |
| [Design checkpoint](docs/compiler-set/design.md) | What default and the reviewed candidate actually implement. |
| [Qualification checkpoint](docs/compiler-set/qualification-review-2026-11-18.md) | Source-bound correctness and CI results, including failures and newer work. |
| [Performance checkpoint](docs/compiler-set/performance-review-2026-11-18.md) | Measured gains, regressions, uncertainty, artifact links and practical tolerances. |
| [Contributor instructions](AGENTS.md) | Rules that keep subsequent work on this architecture. |

## Implementation status

**The general integrated compiler is unfinished.** The audited candidate has a
working scalar mixed bridge, genuine Rust artifacts and useful retained native
compilation. It still performs whole-crate Rust analysis on cold mixed requests,
then separately checks a native projection. Standalone entry still contains
whole-program retry. These are remaining implementation tasks, not the final
architecture.

This documentation branch starts from default `next` at
[`7cea76c9`](https://github.com/glopyglerky/lustr/tree/7cea76c9d3adc587b54ccff168fe07efafd40309),
which predates that mixed bridge. The architecture audit covers
[PR #29](https://github.com/glopyglerky/lustr/pull/29), head
`a9fd4c6336b1e39c4f584f7da50a437c5c248a0d`.
[PR #30](https://github.com/glopyglerky/lustr/pull/30) is a later callback-correctness
continuation. The qualification checkpoint identifies each tested source and
its remaining failures. Documentation publication does not merge either
implementation into default.

| Area | Audited PR #29 result | Remaining requirement |
| --- | --- | --- |
| Lust compilation | Native frontend and retained daemon preserved. | Use the full actual supported subset in mixed programs; the scalar bridge is smaller. |
| Mixed artifacts | Real native objects compose with genuine Rust metadata and libraries. | Share general semantic state, ABI/runtime ownership and dependencies across the boundary. |
| Linux native correctness | 5,528 ordinary and 5,528 sanitizer differential cases, 364 daemon scenarios, 1,200 fuzz edits passed. | Qualify the final changed candidate and reconcile the declared SDK's semantics. |
| Linux assembled mixed package | Real Cargo mixed execution, retained metadata, 17 public-entry tests and real I/O/link failure recovery passed. | Broaden coverage beyond the qualified scalar interface. |
| macOS native correctness | Ordinary and daemon gates passed; 47 differential cases failed on incompatible callback calls. | Qualify PR #30's proposed repair and complete the full native gate. |
| Unicode | A declared-SDK constant/runtime disagreement is documented. | Generate consistent version-bound semantics and reconcile all relevant differential failures. |
| Full replacement release | Not qualified. | Complete the architecture, correctness, exact package and performance conditions below. |

See the [qualification checkpoint](docs/compiler-set/qualification-review-2026-11-18.md)
for primary jobs, artifact identities and the later PR #30 failures. Passing
Cargo compatibility on a platform does not establish mixed native compilation
on that platform.

## Measured speed

The target is **close to Lust on supported work and near stock rustc on ordinary
complementary Rust**. Useful mixed-work gains matter; exact Lust parity and
large gains outside Lust's subset are not requirements. Correctness comes first.

These are existing PR #29 measurements, not timings of this documentation branch
or PR #30. Each comparison has 12 pairs. Ratios are paired geometric reference
time divided by Lustr time; they are not ratios of the marginal medians.

| Workload | Lustr median | Reference median | Paired speed ratio, 95% interval |
| --- | ---: | ---: | ---: |
| Native scalar body edit vs rustc | 7.550 ms | 69.714 ms | **8.228×** [7.043, 9.489] |
| Public warm check vs rustc | 2.963 ms | 13.127 ms | **4.616×** [3.695, 5.597] |
| Public warm build vs rustc | 7.705 ms | 13.143 ms | **1.990×** [1.427, 3.186] |
| Public warm build vs pristine Lust | 7.705 ms | 6.457 ms | 0.814× [0.625, 1.047] |

The warm-build difference in marginal medians is +1.249 ms, or +19.34%, versus
Lust. Near-Lust acceptance remains unresolved; the interval spans parity. The
ordinary full-Rust project's largest marginal-median overhead is +4.31%, with
wide paired uncertainty. Mixed cold build, check, check after a body edit and
signature edit all favor rustc in this campaign. Those losses remain visible in
the [complete performance checkpoint](docs/compiler-set/performance-review-2026-11-18.md).

The existing engineering budgets permit 10% or 0.5 ms added supported warm
latency, whichever is larger, and use 5% or 0.5 ms as the ordinary-Rust review
budget. They do not require every operation to beat rustc. The broad 12-pair
campaign is useful characterization; it is not the complete-package practical
assessor's qualified 24-pair acceptance result.

The [hosted dashboard](https://github.com/glopyglerky/lustr/actions/runs/44420439480/artifacts/14484151306)
retains all 29 rows. The performance checkpoint links original input artifacts,
source/tree hashes, profiles, uncertainty and replay details. No speedup from
upstream Lust or an earlier compiler-tuning experiment is transferred to the
complete product.

## What completes the product

The [completion checklist](docs/compiler-set/completion-checklist.md) requires:

1. One recorded integration baseline that preserves existing unique repairs.
2. The actual Lust capability and checked-state boundary, beyond isolated scalars.
3. Shared semantic ownership with imported Rust services, avoiding duplicated
   complete frontend work on the supported path.
4. Correct cross-boundary ABI, runtime, metadata and retained edit behavior.
5. Integrated standalone and Cargo entry without whole-program error-driven retry.
6. Version-matched correctness and exact package qualification on each advertised
   platform.
7. A finite practical performance assessment, clear source-bound README results,
   then publication of that qualified package.

A proposed optimization is useful when it removes an observed cost while
preserving these contracts. A few tenths of a millisecond alone do not justify
destabilizing compilation.

## Source and history

The pristine [Lust source](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9)
is pinned by [`upstream/lust.pin`](upstream/lust.pin). The imported Rust identity
is recorded in [`upstream/bluefe-core.pin`](upstream/bluefe-core.pin).
Explicit candidate integration patches retain their own identities and
qualification. [Upstream preservation](docs/compiler-set/upstream-lust.md)
requires provenance and meaningful evidence; necessary correctness repairs do
not have to make the compiler faster.

The root `src/` entry, native compiler, `lustr-native-core` and
`lustr-semantic` have different capabilities at different revisions. The
[design checkpoint](docs/compiler-set/design.md) gives the source-bound map.
The separate `crates/lustr` coordinator and earlier BlueFe releases remain
historical compatibility work. They do not complete this product.

The [previous default README](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/README.md)
preserves earlier preview commands, dependencies and measurements in their
original context. Original licenses, source history and evidence remain intact.
