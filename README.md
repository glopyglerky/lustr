# Lustr

**Lustr's only goal is full Rust compilation as a rustc replacement, using Lust
for everything Lust covers and only the necessary upstream implementation for
the remaining work.**

Lustr preserves Lust's frontend, retained checked state and fast incremental
compilation. Complementary Rust support must connect to that compiler and
compose in the same unchanged program. The full Rust source remains in
[blueFe](https://github.com/glopyglerky/blueFe). Lustr imports justified slices
and their required dependencies, with pinned provenance and licenses.

Supported work must stay close to stock Lust speed. Complementary work must stay
close to matching stock rustc speed. The two comparisons are assessed separately.
The first qualified release requires full correct Rust compilation under the
exclusive ownership contract and a measured overall speedup over stock rustc.
Matching stock Lust remains the optimization goal; it must not hold back an
otherwise qualified faster replacement. Every slower case remains visible.
The [exclusive coverage contract](docs/compiler-set/exclusive-coverage-contract.md)
is the canonical direction.

## Current status

**The full replacement is unfinished. No current package is qualified as the
compiler described above.**

| Snapshot | What is actually present |
| --- | --- |
| Pre-correction default `next`, [7cea76c9](https://github.com/glopyglerky/lustr/tree/7cea76c9d3adc587b54ccff168fe07efafd40309) | Earlier native-first entry, library retry and full-Rust compatibility paths. It predates the scalar mixed bridge. |
| Reviewed [PR #32](https://github.com/glopyglerky/lustr/pull/32), [cb0721d4](https://github.com/glopyglerky/lustr/tree/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3) | Retained native work, useful correctness repairs, partial shared semantic work, scalar composition and genuine Rust artifacts. Public retries and whole-consumer Rust analysis still conflict with the required design. |
| Governing documentation, merged [PR #31](https://github.com/glopyglerky/lustr/pull/31), [2fbafe39](https://github.com/glopyglerky/lustr/tree/2fbafe391fdf3d0a59b8cc24e9b0f55be3570cbd) | One controlling direction, explicit selective-import and ownership rules, upstream source/history research, and reconciled active plans. This merge changed no product source. |
| Current [implementation correction](docs/compiler-set/exclusive-coverage-cleanup.md), [PR #33](https://github.com/glopyglerky/lustr/pull/33) | Removes public retries, the whole-program mixed launcher and whole-compiler packaging. Adds the partial import inventory and guarded native package boundaries; repairs Cargo discovery, selection caching and environment preservation. The full complement and speed qualification remain unfinished. |

A narrow bridge cannot redefine Lust's supported subset. Work Lust can compile
but the bridge cannot represent is an integration gap. It must not quietly move
to rustc. A whole-program retry or a complete Rust frontend followed by native
emission of a few functions does not meet the contract.

Lust's support rules come from its pinned implementation and qualification.
Apply them during the normal compilation using the native state already being
produced. Do not first run a scanner, trial compile or duplicate frontend to
choose an implementation. An unexamined capability remains an evidence gap.

Existing repairs and their failures remain useful. The
[retained-work disposition](docs/compiler-set/retained-work-disposition.md)
explains what to keep, retire from production, or use only as research. Source
history and historical evidence remain intact.

## Start here

| Document | Purpose |
| --- | --- |
| [Exclusive coverage contract](docs/compiler-set/exclusive-coverage-contract.md) | Sole goal, exact work sets, import closure, ownership and performance mathematics. |
| [Active correction](docs/compiler-set/active-correction.md) | One current implementation PR, exact evidence, preserved history and remaining work. |
| [Architecture plan](docs/compiler-set/architecture-plan.md) | Implementation order and architecture acceptance. |
| [Design checkpoint](docs/compiler-set/design.md) | Real source state, reusable interfaces and remaining defects. |
| [Completion checklist](docs/compiler-set/completion-checklist.md) | One work list with observable completion conditions. |
| [Implementation cleanup](docs/compiler-set/exclusive-coverage-cleanup.md) | Removed routes, preserved components, focused validation and remaining gaps. |
| [Ownership follow-through](docs/compiler-set/exclusive-coverage-followthrough.md) | Remaining bridge, daemon and acceptance conflicts, native runtime repair and portability priority. |
| [Sole-goal audit](docs/compiler-set/sole-goal-audit.md) | Recorded route cleanup, ownership and release clarification, actual public-entry results and remaining gaps. |
| [Capability result qualification](docs/compiler-set/capability-result-qualification.md) | Compares real generated output and test outcomes through the public compiler; runs installed-package contracts in CI. |
| [Lust upstream review](docs/compiler-set/upstream-lust-review-2026-11-19.md) | Retention, precise invalidation, CI, performance history and reverts. |
| [Rust upstream review](docs/compiler-set/upstream-rust-review-2026-11-19.md) | Existing selective reuse interfaces, dependency closure, CI and performance history. |
| [Support and cost analysis](docs/compiler-set/exclusive-coverage-mathematics.md) | Known capability rules, CI coverage limits and the cost of extra dispatch or semantic work. |
| [Performance policy](docs/compiler-set/practical-performance.md) | Overall speed threshold for the full replacement and separate near-stock optimization targets. |
| [Current performance evaluator](docs/compiler-set/current-performance-assessment.md) | Fixed-workload total-time statistics, declared evidence integrity and explicit limits; no current speed or release claim. |
| [Contributor instructions](AGENTS.md) | Rules for continuing without changing direction. |

## What the research changes

Lust's main edit-loop advantage comes from retaining checked state, patching
eligible bodies, tracking actual semantic reads and emitting affected work.
Its history includes both correctness repairs for missing dependencies and
optimizations reverted after measurement. Lustr must preserve those mechanisms.
Upstream CI's performance smoke is not a near-stock speed gate. See the
[pinned Lust review](docs/compiler-set/upstream-lust-review-2026-11-19.md).

Rust already shares type IR, trait-solving algorithms and supporting components
with a frontend that owns its own representations. Those interfaces provide a
concrete starting point for selective reuse. Importing `rustc_driver` instead
pulls in broad compiler stages and does not establish a minimal complement.
See the [pinned Rust review](docs/compiler-set/upstream-rust-review-2026-11-19.md).

## Performance evidence

The [18 November performance checkpoint](docs/compiler-set/performance-review-2026-11-18.md)
retains prior source-bound measurements, gains, losses and uncertainty. They do
not qualify this implementation correction or the complete replacement. The
[qualification checkpoint](docs/compiler-set/qualification-review-2026-11-18.md)
likewise preserves what actually ran at its recorded sources. Later repairs
need their own evidence; old failures must not be silently reclassified.

Current near-stock claims require the relative review bounds: 10% overhead for
supported warm work and 5% for ordinary complementary work, per case with paired
uncertainty. These are engineering assumptions, not demonstrated current
results or additional mandatory release thresholds. Overall speedup is a
separate end-to-end comparison using workload frequencies fixed before sampling.
Passing both subset targets does not prove that total compilation time improved.
The former 0.5 ms alternate rule remains historical; small absolute
overhead cannot excuse a large relative slowdown. Cold builds, edits, rejection,
memory and compile-plus-test remain separate. Large gains outside Lust's subset
are unnecessary.

## Source and history

The pristine [Lust donor](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9)
is identified by [`upstream/lust.pin`](upstream/lust.pin). The declared Rust SDK
identity is recorded in [`upstream/bluefe-core.pin`](upstream/bluefe-core.pin).
The [source register](docs/compiler-set/sources.md) distinguishes references,
partial imports and broad research dependencies. An SDK dependency or an import
manifest is not proof that Lustr implements the full complement.

The [prior default README](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/README.md)
retains preview commands and measurements in their original context. Earlier
compatibility packages remain historical artifacts. They do not change Lustr's
sole goal or authorize a hidden fallback in a future product.
