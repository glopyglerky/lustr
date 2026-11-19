# Governing plan: one integrated Lustr compiler

This is the fixed delivery plan, based on the user's clarification on 18 November
2026 and reaffirmation that it must survive later sessions. It governs the
[compiler contract](native-contract.md), [design](design.md),
[roadmap](roadmap.md), and [completion checklist](completion-checklist.md).
Change this destination only when the user explicitly changes it. A benchmark
result, implementation shortcut, old document, or successful compatibility build
does not change the contract.

This document establishes the required architecture. It does not claim that the
default branch or a development candidate already implements it. The dated
[qualification review](qualification-review-2026-11-18.md) records exact source
and test status separately.

## Required result

Lustr owns one compilation of the user's unchanged Rust program. Lust handles
work it supports. Imported upstream Rust implementation handles complementary
work that Lust does not support. Both implementations must compose in the same
program, with consistent semantics, runtime behavior, linkage, and genuine Rust
metadata.

Preserve Lust's frontend, retained worker, dependency reads, and fast body-edit
path. Fully supported work must not pay for a complete extra Rust frontend just
to establish that Lust can compile it. The worker may remain a daemon or an
internal process. One compiler means one owned semantic compilation and artifact
contract; it does not require one operating-system process.

Importing upstream Rust source and internal services is explicitly allowed.
Their necessary dependencies may include the full upstream compiler dependency
graph. Reimplementing every missing Rust feature independently is not required.
The relevant distinction is the work executed during a request. Imported
services must support the shared compilation and complement, rather than repeat
the entire supported compilation or provide a whole-program backup after Lust
fails.

The performance goal is to retain large gains on Lust-supported work, stay
reasonably close to Lust's own speeds, and keep ordinary complementary Rust
close to the matching stock Rust compiler. Exact Lust parity and large gains
outside Lust's subset are not required. Correctness takes priority over a
fraction of a millisecond.

## What the sets of work mean

| Term | Meaning | How to use it |
| --- | --- | --- |
| Lust-supported work | Work the pinned Lust implementation can correctly represent and compile under the relevant semantic, target, profile, runtime, and dependency conditions | Preserve native participation for this work and bind coverage to actual implementation and fixtures |
| Integrated bridge coverage | The part of Lust support whose facts and artifacts the current shared boundary can carry correctly | Report this separately; it is currently much smaller than Lust's own support |
| Rust complement | Work requiring upstream Rust capabilities absent from Lust, plus necessary shared upstream semantic services | Import the implementation and execute it inside the owned compilation |
| Unintegrated Lust support | Work Lust can handle but the current bridge cannot yet compose | Record an implementation gap; do not redefine it as the permanent Rust complement to claim completion |

The existing capability report is a useful inventory. It is not executable
admission for an arbitrary program. Admission depends on resolved definitions,
types, instances, layouts, runtime requirements, and recorded dependencies.

## Rules that must survive implementation changes

- Assign work inside the compiler from actual representations and checked
  semantic contracts before generation. A whole-program native failure is not
  a capability test.
- Do not introduce a source-keyword language selector. Macros, traits, generics,
  overload resolution, targets, and dependency changes affect the meaning of
  the same spelling. These features overlap within functions and types.
- Keep supported work on Lust when it appears alongside complementary work.
  Sending a mixed program entirely to stock rustc does not close the integration
  gap.
- Preserve the native frontend and retained worker. Always running complete
  Rust analysis before selecting native code generation does not meet this
  requirement, even within one executable.
- Share or faithfully bridge definition and instance identities, types,
  layouts, calling conventions, generic obligations, symbols, drops, allocation,
  unwinding, metadata, and dependency state. Reuse upstream machinery where
  appropriate.
- A missing capability is an explicit internal result before emission. Invalid
  Rust, compiler faults, ambiguous transport, invalid objects, and failed links
  remain failures. Never convert an admitted emitter failure into successful
  compilation by another implementation.
- Preserve application source, features, dependencies, profiles, target,
  debug information, overflow checks, panic strategy, Cargo arguments, build
  scripts, and procedural macros. Do not execute a program as a hidden probe
  or retry a program or test that has already run.
- Preserve source pins, licenses, notices, failures, and historical measurements.
  Necessary correctness and integration changes require explicit source identity
  and validation. A correctness repair need not itself make compilation faster.

Cargo may continue to schedule its real dependency graph, build scripts,
procedural macros, and target artifacts. Command adapters may preserve that
behavior. Cargo package/target lookup is not language capability dispatch, and
optimizing that lookup does not implement the shared compiler boundary.

## Default code and audited candidates are different snapshots

The base of this documentation change is default-branch commit
[`7cea76c9d3adc587b54ccff168fe07efafd40309`](https://github.com/glopyglerky/lustr/commit/7cea76c9d3adc587b54ccff168fe07efafd40309).
Candidate findings below are not claims about code merged into that default.

| Snapshot | Actual source state | What remains |
| --- | --- | --- |
| Default `7cea76c9` | [`src/main.rs`](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/src/main.rs) has the native-first public entry, diagnostic-guided library retry, and full-Rust compatibility path. Its semantic driver is an inspection experiment. This tree has no `mixed_main.rs` or `mixed_unit.rs`. | The default does not contain the later scalar composition or admission work |
| Architecture-audited PR29 head `a9fd4c6336b1e39c4f584f7da50a437c5c248a0d` | The [mixed driver](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/crates/lustr-semantic/src/mixed_main.rs) uses actual Rust HIR and ABI queries. Its [native boundary](https://github.com/glopyglerky/lustr/blob/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/crates/lustr-native-core/src/mixed_unit.rs) retains checked state through scalar admission and emission. Its backend composes native objects with genuine Rust metadata. | Cold mixed compilation still runs whole-crate Rust analysis and reparses native projections. Bridge coverage remains narrow, and the standalone retry remains |
| PR30 head `b94d3713616804fead3f30b18a530e2e81a9510b` | A [correctness continuation](https://github.com/glopyglerky/lustr/commit/b94d3713616804fead3f30b18a530e2e81a9510b) proposes a callback ABI correction and adds exhaustive arithmetic checks with updated source identities. It does not add a wider shared compiler boundary. | Its exact-source validation status belongs in the qualification review; a proposed repair is not a qualifying pass |

PR29's linked mixed path is restricted to represented Linux x86_64 requests and
nongeneric root scalar functions. It is not the full Lust subset. Its typed
admission and failure handling are completed components to reuse, rather than a
reason to restart from the older default implementation. Integrating candidate
code requires its own review and source guard changes; this documentation change
does not merge it or change compiler pins.

## Implementation order and acceptance

The [checklist](completion-checklist.md) records evidence and status for each
step. These steps describe dependency order, not permission to stop after an
interface or scalar demonstration.

### 1. Establish the complete capability and ownership contract

Extend the executable native preparation boundary from represented scalar work
to the supported coverage being claimed. Keep native checked state, source
identities, and dependency reads alive. Identify which component owns every
semantic fact required across the boundary. Use the
[interface obligations](design.md#required-internal-interfaces) below.

Acceptance requires distinct admitted, complementary, invalid, and failed
results before emission. The inventory must distinguish actual Lust support
from unintegrated bridge coverage. A passing parser, descriptive capability
report, or repeated native check followed by a fresh build is insufficient.

### 2. Implement the shared semantic continuation

Connect imported upstream services to the retained compilation context for
resolution, types, obligations, instances, layouts, and other missing facts.
Reuse those facts between implementations. The next implementation must compile
a real mixed program through that continuation, not merely produce a plan.

Acceptance requires a fully supported cold request to retain the native frontend
without a complete extra Rust frontend. A mixed request must preserve native
participation while imported services supply its complementary work. Record
actual phases and definition ownership so duplicate analysis cannot hide behind
an engine label or an in-process call.

### 3. Extend ABI, runtime, and metadata composition

Reuse the existing candidate's genuine symbol, object, Rust metadata, and linker
work. Extend represented calls and instances to the required types, generics,
closures, drop/allocator ownership, and panic/unwind contracts. The imported
metadata encoder must consume valid shared facts; do not invent an rlib format
or empty metadata.

Acceptance requires unchanged mixed programs and downstream Rust crates to use
real artifacts. Each definition or instance has one code-generation owner.
Cross-boundary calls and ownership preserve Rust behavior under the requested
configuration. Target support must be demonstrated for each claimed platform.

### 4. Preserve incremental correctness across the boundary

Connect native read sets to complementary semantic dependencies. Extend reuse
beyond unchanged bodies and the candidate's restricted literal substitutions.

Acceptance requires cold and retained equivalence after body, signature, type,
trait, macro, dependency, build-script, and configuration changes. Unrelated
native work stays retained when its inputs remain valid. Failed generation or
publication cannot replace the last successful certificate or become a cache
miss that hides the failure.

### 5. Make this the normal public compilation path

Connect standalone and Cargo requests to the shared context. Remove ordinary
whole-program native-fail/full-Rust retry from the default execution path.
Keep Cargo adapters responsible for the real build contract.

Acceptance requires observable native and complementary work in one unchanged
program, through the public command. Users must not manually select a language
engine. A valid complement decision and a failed compiler attempt must remain
visibly different outcomes.

### 6. Qualify the delivered compiler and its performance

Qualify the exact source-built package on unchanged supported, complementary,
and mixed projects. Cover runtime behavior, invalid Rust, Cargo dependencies,
real metadata consumers, procedural macros, build scripts, panic/drop behavior,
and the target platforms being claimed. Close relevant semantic and runtime
blockers before admitting affected work.

Then measure complete commands against matched Lust and Rust references. Keep
cold, warm, no-op, edit, rejection, linking, and compile-plus-run/test results
separate. Apply the practical tolerances in the [checklist](completion-checklist.md)
without weakening correctness or hiding slower operations. A green compatibility
job or a scalar benchmark does not establish the preceding architecture steps.

## Continue from evidence, without changing the destination

Read the checklist and dated qualification review at the start of later work.
Update status only from source-bound evidence, with default, candidate, tested
merge, and package identities distinguished. Correctness repairs may proceed
alongside the structural work, but they do not replace it. No universal claim
about all Rust programs follows from a finite test suite.

Earlier instructions that demanded independent native reimplementation of every
Rust gap or faster results for every necessary Lust modification are superseded.
Their original text remains available at the immutable default snapshot:
[contract](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/native-contract.md),
[design](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/design.md),
and [roadmap](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/roadmap.md).
Historical implementation choices and measurements retain their original scope.
