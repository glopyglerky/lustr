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

**The installed toolchain compiles native and mixed Rust workloads through Cargo.
Full Rust coverage and overall performance qualification are still in progress.**

The implementation lane is [PR #33](https://github.com/glopyglerky/lustr/pull/33).
Each qualification record applies to its recorded source, package and workload.

| Capability | Implemented and exercised scope |
| --- | --- |
| Public scalar Cargo path | Genuine metadata, libraries, checks, represented dependencies and executable artifacts through the retained native owner. |
| Linux x86-64 native unwind | Native executable and library tests cover frame walking, panic propagation, destructor cleanup and catching in represented configurations. |
| Imported associated types | Cargo provider and consumer tests exercise lifetime-generic associated types through native checking and linking. |
| Default Cargo profiles | Represented programs have native check, build, run and test paths. Native Rust debug information covers the types and locations exercised by the debugger tests. |
| Mixed host compilation | Pinned Rust supplies async semantics and the linker while Lust emits selected closed functions. Tests cover ordinary executables, static Rust libraries and freestanding `no_std` ELF executables. |
| Full language, ecosystem and platforms | Wider native artifacts, selective complementary semantics, real dependency/proc-macro graphs and the full target/profile matrix remain open. |
| Performance and release | Scoped timing results include both gains and slower edits. Overall speed and release qualification remain open. |

### Build and use the toolchain

On Linux x86-64, use the Rust SDK identified by `upstream/bluefe-core.pin` and
the pinned TinyCC build. Set `sdk`, `tinycc`, `bluefe` and `work` to their
paths and run these commands from the Lustr checkout.

```sh
"$sdk/bin/rustc" --edition=2024 tools/build-native-toolchain.rs -o "$work/constructor"
"$work/constructor" "$PWD" "$sdk" "$tinycc" "$work/lustr" --debug \
  --proc-macro-source "$bluefe" --rustdoc-source "$bluefe"
```

Use the installed compiler and Rustdoc for regular Cargo commands in your
project. The constructor records the source and runtime dependencies in its
output directory.

```sh
export RUSTC="$work/lustr/toolchain/bin/rustc"
export RUSTDOC="$work/lustr/toolchain/bin/rustdoc"
"$work/lustr/toolchain/bin/cargo" test
```

### Source-built procedural macro support

On Linux x86-64, pass `--proc-macro-source PINNED_RUST_REPOSITORY` to
`tools/build-native-toolchain.rs` with a checkout containing the revision in
`upstream/bluefe-core.pin` and the locked dependencies available to Cargo. The
constructor builds the expansion worker and includes it in the resulting
toolchain. Ordinary Cargo commands then use the pinned Rust implementation to
build the host macro crate and Lust to check and link its consumer.

The current adapter covers directly qualified function-like calls with source
token trees and expansions containing call-site identifiers, represented
literals, punctuation and visible delimiter groups. Expansions share a host
worker for each native compilation. Source-text and filename queries, tracked inputs, and emitted
errors work through this path. General expansion output, attribute and derive
macros, additional span and hygiene operations, and performance qualification remain open.

The [completion checklist](docs/compiler-set/completion-checklist.md#execution-queue)
is the sole work register. It now includes an ordered execution queue, source
entry points, dependencies, public acceptance commands, parallel assignments and
a resume record. The [architecture plan](docs/compiler-set/architecture-plan.md)
explains the shared semantic/ABI boundaries. [Contributor instructions](AGENTS.md#continue-implementation-across-checkpoints)
require agents to continue to the next ready implementation after each verified
checkpoint, preserving existing work and exact-source evidence.

The [continuation review](docs/compiler-set/evidence/continuous-completion-20261122/README.md)
records what was inspected at `bc38dece`. Earlier qualified results retain their
own source identities: the [profile/construction correction](docs/compiler-set/evidence/compiler-review-20261122/README.md),
[executable unwind review](docs/compiler-set/evidence/executable-unwind-review-20261122/README.md),
and [original public scalar checkpoint](docs/compiler-set/native-cargo-continuation.md#verified-hosted-checkpoint).
A green prior-source job, research qualifier or refusal-only release guard cannot
qualify later source or the complete compiler. Historical previews remain evidence,
not recommended packages for the current replacement.

Represented requests use Lust's retained compiler. The installed Rust component
handles additional request options and operations identified by the native
parser or verified declaration loader. Mixed units use original Rust analysis
and linking for the complementary code. Lust checks and emits its selected
bodies once from the original source. The full Rust source stays in blueFe, with
the imported component's pinned source and dependencies recorded in the package.

The public host still uses `std` and Unix sockets, and its archive/object tools
retain a substantial LLVM shared-library dependency. Those portability and size
costs remain visible. A `no_std` core and socket-independent retained transport
are preferences that must preserve speed; they do not displace completion of
ordinary compiler behavior. The [retained-work disposition](docs/compiler-set/retained-work-disposition.md)
preserves useful earlier source and failed evidence without reviving retired routes.

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
| [Documentation disposition](docs/compiler-set/documentation-disposition.json) | Complete baseline Markdown inventory, active authority, scoped checkpoints and untouched donor/evidence boundaries. |
| [Capability result qualification](docs/compiler-set/capability-result-qualification.md) | Compares real generated output and test outcomes through the public compiler; runs installed-package contracts in CI. |
| [Retained-history result qualifier](tools/qualify-retained-results.py) | Uses pinned acceptance expectations, separate compile/runtime results and the same surviving daemon; unresolved histories remain visible. |
| [Native Cargo continuation](docs/compiler-set/native-cargo-continuation.md) | Authored public scalar-library profile and Cargo command; existing dependency/host-script integration, retained proofs and source-bound evidence. |
| [Public scalar-library contract](docs/compiler-set/public-scalar-library.md) | Real request identity, artifact publication, retained history and current package/coverage limits. |
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
Each current near-stock claim also uses paired total elapsed costs. Geometric
ratios and empirical p95 cannot hide expensive requests; means and maxima remain
visible. See the [latest corrections](docs/compiler-set/sole-goal-audit.md#follow-through-after-5cd40fd3).
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

The `preview-2026.11.09-1`, `preview-2026.11.09-2` and
`native-preview-2026.11.15-1` releases are historical coordinator or full-SDK
packages. Their downloads and installation recipes are not recommendations for
current Lustr. There is no qualified replacement release to install yet. Current
source and unfinished acceptance remain in the
[active correction record](docs/compiler-set/active-correction.md).
