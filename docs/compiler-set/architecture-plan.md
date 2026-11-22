# One Lustr compiler with exclusive Lust coverage

The [exclusive coverage contract](exclusive-coverage-contract.md) controls this
plan. Lustr's only goal is full rustc replacement using Lust for all work Lust
covers and only the necessary upstream implementation for the complement.
The full Rust source stays in blueFe. A larger dependency graph is not blanket
permission to import a complete competing compiler.

## Required result

Compile an unchanged program with one semantic and artifact contract. Preserve
Lust's frontend, retained checked state and incremental speed. Assign missing
operations to justified imported components before emission, using shared or
faithfully bridged definitions, types, instances, layouts, ABI, symbols,
constants, drops, unwinding, metadata and dependency reads.

Actual Lust support is broader than the existing scalar bridge. The difference
is unfinished integration and remains Lust-owned. Neither complete Rust analysis
before a native body substitution nor whole-program native-fail/rustc-retry
completes this architecture. A missing implementation must remain visible.

## Current source and continuation

Reviewed 22 November 2026 at
[`514dfe1ef8b49160bf5c09458981768fbf4f2386`](https://github.com/glopyglerky/lustr/tree/514dfe1ef8b49160bf5c09458981768fbf4f2386)
on the still-unmerged [PR #33](https://github.com/glopyglerky/lustr/pull/33).
The [public scalar hosted checkpoint](native-cargo-continuation.md#verified-hosted-checkpoint)
passed on its exact source tree. It includes genuine scalar libraries,
metadata-only checks, bounded dependency composition and scalar executables.
Unwind-mode leaf libraries require debug information disabled. The broader
Linux/macOS workflow matrix was pending when observed, and no matched speed
result qualifies this candidate.

The first scalar metadata join is implemented. The earlier
[PR #32 metadata dependency failure](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md)
remains useful historical evidence, not an instruction to redo that completed
slice. Wider definition completion and metadata still need their actual proofs.
The [design checkpoint](design.md) explains the current interfaces, and the
[completion checklist](completion-checklist.md) remains the only work register.

The [audit record](evidence/compiler-review-20261122/README.md) separately
qualifies the overflow/default-option and SDK-construction corrections: a full
rebuild, 24 paired profile cases, the unchanged public campaign and actual DWARF
walks with corruption controls. Preserve these gates and resolve demonstrated
correctness failures before widening admission. Start ordinary Cargo/debug
completion and the first general mixed semantic
service in parallel. Establish that semantic boundary before another broad
scalar-widening campaign; the sequence below makes this dependency explicit.

## The concrete conflicting direction

The earlier [PR #31 architecture text](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md)
permitted the full upstream compiler dependency graph and treated executed work
as the only import criterion. That exception conflicts with selective source
reuse. Earlier defaults also prescribed independent implementation of the missing
Rust language, while candidates retained retries and whole-consumer analysis.
The result was progress on compatibility and narrow bridges without completing
the required compiler boundary. This correction removes those competing active
instructions. Source cleanup must delete obsolete alternate compiler launchers,
not rename them as research-only execution paths.

## One retained semantic database

Extend the existing Lust state into one authoritative logical database for the
compilation. This does not prescribe a new storage engine or a rewrite of the
working daemon. It requires stable definition/type/instance identities, actual
source and expansion locations, checked facts and dependency reads that every
admitted service can consume without reconstructing a second frontend.

| Boundary | Required input and result |
| --- | --- |
| Cargo request and host services | Actual selected units, arguments, feature/profile/target settings, source/dependency identities, build-script observations and required outputs. |
| Expansion and resolution | One definition and hygiene inventory, including generated items, imports and negative name lookups. |
| Native and imported semantic queries | Contextual capability decision, authoritative typed result or invalid-program diagnostic, explicit owner and complete dependency reads. |
| Checked body and instance completion | Actual type/region/const/layout/drop facts and the exact monomorphized instance graph needed by emission and metadata. |
| Emission and metadata | Each assigned instance emitted once; genuine symbol/ABI/metadata identity and all required native or complementary objects. |
| Publication and retained state | A complete successful output set or an explicit terminal failure, with the last successful state protected. |

Assign work during normal compilation from resolved facts. The source scanner
and independent stock oracle remain qualification tools. A native compiler bug
does not make the operation complementary, and a missing bridge does not shrink
Lust's supported set. Shared prerequisites have one authoritative result even
when native and complementary consumers both read it.

This is a necessary architecture, not a proof of full coverage. The first
general mixed program must demonstrate the interfaces. Do not spend all future
work extending isolated scalar fixtures while leaving this dependency unresolved.

## Implementation sequence

The sequence below maps to the existing checklist numbers. It adds concrete
dependencies and acceptance examples without opening another status register.
Preserve usable implementations and exact predecessor evidence at every step.

| Sequence | Checklist items | Work and completion witness |
| --- | --- | --- |
| Preserve corrected behavior | 0, 6 | Keep the qualified profile/default and construction repairs, extend coverage to retained profile transitions, and resolve reproduced semantic regressions. Check explicit and omitted flags against the pinned compiler and actual program behavior. |
| Start ordinary dev completion | 3, 5, 6 | Build, run, check and test a small unchanged Cargo workspace with normal dev settings. Finish executable unwind and Rust source debugging; do not edit the user's manifest to obtain a pass. Run this work alongside the next row. |
| Prove the general semantic boundary early | 2, 3, 4 | Before broad scalar widening, choose a concrete missing capability from an unchanged mixed fixture and integrate its upstream service into the native state. Demonstrate valid and invalid trait/projection/generic/borrow cases, retained reuse and precise cross-boundary invalidation. |
| Expand the existing native bridge | 1, 3 | Once the mixed boundary passes, carry donor-supported integers, aggregates, enums, references, methods, generics and traits through that same checked state, real metadata and downstream libraries. Implement narrow bridge prerequisites for the early fixture as needed. Document unsupported ABI/profile cells separately. |
| Close the complete Rust/build graph | 1, 2, 3, 5 | Integrate remaining macro, const, region, unsafe/FFI, async, standard-library and Cargo obligations through justified imports. Preserve real dependencies, proc macros, build scripts and all required crate types. |
| Qualify retained behavior and targets | 4, 6 | Run applicable cold/warm/restart/edit/failure histories and the complete declared host/target/profile matrix with genuine packages and independent results. |
| Qualify and publish the replacement | 7 | Demonstrate matched overall speedup on the predeclared full workload and exact package. Publish all slower cases and continue optimizing toward stock-Lust speed. |

Measure each material integration boundary as it lands. Complete-package speed
qualification is the final consolidation, not the first performance experiment.
Ordinary Cargo/debug work need not wait for the first mixed semantic service,
and that service must not wait for a broad scalar campaign to finish. Further
independent services or targets may proceed in parallel after their shared
interfaces and owners are fixed. Work must preserve correctness dependencies
and one public compiler route.

## Default Cargo behavior, debugging and unwinding

Start with the real requests already captured by the public evaluator, then
remove each current semantic limitation through implementation. Normal
[Cargo dev settings](https://doc.rust-lang.org/cargo/reference/profiles.html)
include debug information, unwind and incremental compilation. A debug-disabled
leaf-library pass does not qualify a default executable or test build.

Separate three obligations that need different execution evidence:

1. Stack walking must describe real code ranges and frame changes, including
   prologue/epilogue and changed objects after a retained edit. One GDB backtrace
   is useful but cannot establish all unwinding behavior.
2. Panic handling must preserve propagation across native/imported boundaries,
   destructor order, catch behavior, thread behavior and terminal failures.
   Ordinary tests must use the requested strategy rather than silently abort.
3. Rust debug information must describe the original source, line locations,
   types, parameters and locals. Exercise breakpoints and stepping with actual
   debuggers after edits. Generated-C locations or function names alone do not
   discharge this obligation.

The [`Rust` ABI permits unwinding](https://doc.rust-lang.org/reference/items/functions.html#extern-function-qualifier).
Compatibility is therefore more than emitting `.eh_frame`. Use the pinned
upstream ABI/runtime implementation where the native compiler lacks a required
operation. Keep emission, actual runtime cleanup and debugger observations as
separate results.

## Wider artifacts and semantic composition

Extend the completed scalar definition/metadata service, rather than returning
to the predecessor's complete-consumer analysis route. Native proofs must cover
applicable declarations, attributes, visibility, privacy, lints and diagnostics.
Metadata generation consumes those real proofs and the emitted-instance graph.
Never manufacture unconditional analysis success, typed-body results or an
empty instance inventory.

The [upstream metadata contract](https://rustc-dev-guide.rust-lang.org/backend/libs-and-metadata.html)
includes compiler and dependency identities, exported types/traits/macros and
MIR where downstream code generation needs it. Advance beyond scalar signatures
with generic and inline functions consumed by another crate, trait methods,
layout-sensitive values and dependencies edited while the owner stays alive.
Check both directions with independent pinned Rust consumers and Lustr consumers.

Semantic services cannot be divided by syntax alone. A simple native body can
depend on an associated type solved by the complement; a new impl can invalidate
a prior negative lookup; a const can change an array type and ABI; a proc macro
can create any of those definitions. Use one set of identities and record these
dependencies. This is the central feasibility test for the full compiler.

The first imported semantic fixture should be small enough to diagnose and
complete enough to cross that boundary. It must include an unchanged dependency,
native-owned work, a genuinely complementary operation, invalid-program controls
and an edit that changes the imported fact's consumers. Select the missing
operation from that fixture's actual failure, not an arbitrary list of crates.
The boundary passes only when a native-owned body consumes a genuinely imported
semantic fact, the complete unchanged program executes correctly, invalid
controls are rejected for the right reason, and an edit to that fact invalidates
its consumers while preserving unrelated work. Record the actual import closure
and phase costs. If this requires duplicate native stages, fabricated checked
facts or whole-program analysis, stop broad widening and redesign the failing
identity/query/typed-body adapter. Re-run that same fixture and its controls
before admission. Do not resolve a failed boundary with a hidden compiler
fallback or another scalar-only result.

## Importing the complement

Choose a missing operation and its smallest justified upstream interface. Compute
the source/dependency closure under explicit features and targets. Cut edges only
through verified Lustr adapters. Keep provenance, licenses, build generators,
shared data and real callers recorded. The Rust review identifies reusable
[generic type IR and solver interfaces](upstream-rust-review-2026-11-19.md);
it does not qualify a complete complement or authorize a blanket crate import.

If the closure requires duplicate Lust-owned compiler stages, improve the
interface before production admission. Do not copy the full Rust source, treat
a full-driver dynamic library as minimal, or make dead-code elimination the
source-policy proof. Conversely, do not replace necessary shared semantics with
independent speculative reimplementation merely to reduce an import count.

The pinned Rust [`Interner` interface](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_type_ir/src/interner.rs)
and the [shared-solver design](https://rustc-dev-guide.rust-lang.org/solve/sharing-crates-with-rust-analyzer.html)
provide a real model for adapting upstream algorithms to frontend-owned types
and queries. Reusing that interface still requires complete adapters, recorded
dependencies and preserved native ownership. It is not a ready-made full
complement or permission to import every compiler crate.

For borrow/region gaps, preserve Lust's applicable native checker and adapt the
necessary upstream operation to faithful typed control flow. Rust's
[MIR borrow checker](https://rustc-dev-guide.rust-lang.org/borrow-check.html)
uses move dataflow, region constraints and a MIR type check. Checking a C object
after emission cannot replace those proofs. Require use-after-move, aliasing,
partial initialization and escaping-reference rejection, including failures
introduced by changes in another definition.

[Constant evaluation](https://rustc-dev-guide.rust-lang.org/const-eval.html)
depends on checked MIR and the actual instance/environment. Preserve target
integer semantics, allocations, type-level constants and const/runtime agreement.
Treat async state, closures, drops and unsafe/FFI obligations in the same shared
context rather than giving each feature an independent unchecked translation.

Proc macros and build scripts are host programs even when their generated
program targets another platform. Preserve the actual Cargo graph and native
dependencies; adapters standing in for arbitrary crates are not full coverage.
[Proc macros can read files and process resources](https://doc.rust-lang.org/reference/procedural-macros.html),
and [build scripts supply compiler/linker inputs](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
Record the relevant executable, expansion, environment and file identities.
Reevaluate when dependencies cannot be proved unchanged. Source text equality
alone cannot justify reuse of arbitrary host execution.

## Backend choices and feasibility

Preserve the current emitter while measuring alternatives behind the same
checked-body/instance interface. A backend experiment must report construction
cost, per-request code generation, metadata/link time, generated-program speed,
debug/unwind behavior, targets and package closure. Do not replace a working
native route on the assumption that a Rust-written backend must be faster.

The reviewed [Cranelift backend](https://github.com/rust-lang/rustc_codegen_cranelift/blob/a69afc0967fddad0587b24fcd4c5534a5a7e60b9/Readme.md)
is a rustc code generator with stated SIMD and unwind limitations. It is a
candidate source for emission and runtime/debug techniques, not a substitute
frontend. Rust's [2026 performance goal](https://goals.rust-lang.org/2026/improve-cg_clif-performance.html)
reports modest whole-build improvements in its stated February workloads and
proposes retained per-function code caches. That evidence does not predict a
Lustr speedup. Qualify any imported implementation under Lustr's own contract.

The reviewed [rustd prototype](https://github.com/DioxusLabs/rustd/blob/ebcd234a02f769e380875b8b80e18164bba1b353/README.md)
offers retained analysis and function caches, but explicitly lacks a borrow
checker and genuine `.rlib`/`.rmeta` production. Use it for focused upstream
research, not as correctness or performance evidence for a full replacement.
An optimized release-profile producer is also required; handing the entire
program to stock rustc cannot complete that obligation.

The major uncertainty is semantic integration, not whether a scalar archive
can be linked. The common query interfaces must carry the full required facts
without duplicate analysis or excessive conversion. If a proposed service
cannot meet that condition, revise its boundary using the failing witness and
measure the replacement. Do not quietly change the goal, delete working donor
capabilities or accumulate alternative public compiler modes.

## Retention, platforms and performance during implementation

For each new operation, compare cold and retained results through body,
signature, type, trait, macro, configuration and dependency edits. Include
invalid edits and repair, deletion/reintroduction, restarts, evictions and
publication failures. Track changes in actual semantic reads, not only source
hashes or the number of objects rebuilt. Preserve the last successful state
when a request fails.

Use the [host/target/profile qualification section](completion-checklist.md#host-target-and-profile-matrix)
as the matrix register. Compiler host and output target are separate dimensions.
An ELF x86-64 library pass cannot qualify Mach-O, COFF, another ABI, a different
panic runtime, or cross compilation. The matrix must account for every target
in the declared SDK and mark unimplemented or untested cells explicitly. The
preferred `no_std` core and socket-independent host adapter may follow when
they preserve the same retained state and measured speed; they are not extra
blockers ahead of a correct compiler.

Declare the Rust language revision and supported editions for each package.
Treat an upstream-version change as an integration change: update the necessary
source slices, SDK, ABI/metadata identities and semantic expectations together,
then rerun the affected valid/invalid, cross-crate and retained histories.
Preserve both source compatibility and correct version-mismatch refusals.
Passing against one pinned compiler cannot silently qualify a later Rust release.

Apply the existing [performance policy](practical-performance.md) at each
material boundary. Use pristine pinned Lust for equivalent supported work,
stock rustc at the compatible pin, and a separately identified current Rust
baseline when the pin is older. A Cranelift comparison is useful only where it
implements the same required semantics. Match actual features, debug/panic
settings, output artifacts, thread counts and warm/cold state. The
[nightly frontend-thread change](https://blog.rust-lang.org/2026/11/16/enabling-parallel-frontend-on-nightly/)
is a reason to record the actual baseline settings rather than assume an old
serial compiler.

Measure complete public check/build/run/test requests, including startup,
transport, semantic work, artifact writing, linking and generated execution.
Keep phase observations for diagnosis, and do not sum overlapping spans as
whole-build time. Fix workload frequencies and sampling before measurement.
Preserve means, tails, maxima, memory, errors and all slower cases. Zero or one
rebuilt object is mechanism evidence; it is not a measured speedup.

## Completion and publication

A qualified replacement must satisfy the full exclusive coverage contract,
correctness, actual artifact integration, target/runtime behavior and the
[performance policy](practical-performance.md). A green coordinator test,
source manifest or benchmark of scalar emission cannot establish those facts.
No current package has been qualified as the full product.

The first qualified release must beat matched stock rustc overall. Do not hold
that release for perfect stock-Lust parity or the provisional 10%/5% subset
targets. Those remain separate optimization goals and named claims that require
their own evidence. Workload selection and frequencies must be fixed before
measurement; selective fast cases cannot establish the overall result.

The prior [18 November plan](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md)
and [older default plan](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/native-contract.md)
remain historical. In particular, permission for the full upstream compiler
dependency graph and the independent-native-reimplementation destination are
superseded. Their recorded successes and failures retain their original scope.
