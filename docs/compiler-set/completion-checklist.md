# Completion checklist

Reviewed 22 November 2026. This is the single work register for the
[exclusive coverage contract](exclusive-coverage-contract.md). Lustr's sole goal
is full rustc replacement with exclusive Lust ownership of covered work and
only necessary upstream complementary implementation. Full Rust stays in blueFe.

Planning baseline: [PR #33](https://github.com/glopyglerky/lustr/pull/33),
`bc38dece7a734b97c0108582b7f61f98ccb819db`, tree
`82afa54968f68dbbb5f99540fffc1e67ebc7d3d5`. The PR is unmerged at this review.
The [continuation review](evidence/continuous-completion-20261122/README.md)
separates the latest retained projection proof, earlier public executable
qualification and still-failing default Cargo operations. At 01:57 UTC on
23 November, the latest head had 18 successful jobs and three running jobs;
those observations do not qualify this planning commit or the full product.
Refresh the live source and its checks on every continuation.

Historical baselines remain `next` at
`7cea76c9d3adc587b54ccff168fe07efafd40309`, PR #32 at
`cb0721d49be95ec3f34a113c2354bc9d2a38bbf3`, and the PR #31 governing documentation
at `2fbafe391fdf3d0a59b8cc24e9b0f55be3570cbd`. The
[cleanup record](exclusive-coverage-cleanup.md) preserves their relationship to
the active implementation correction. The [architecture plan](architecture-plan.md)
orders the work below without introducing another register.

## Execution queue

This queue makes sections 0–7 executable; it does not replace their completion
conditions. Status means **READY**, **ACTIVE**, **WAITING**, **VERIFYING** or
**DONE**. READY means useful implementation can start, not that every acceptance
dependency is already implemented. WAITING rows name the specific prerequisite
for their next integration step. DONE requires the stated public behavior and
source-bound evidence. Research, implementation, installed-path qualification
and release qualification are distinct states inside each package.

The immediate product milestone is one unchanged small Cargo workspace whose
provider and consumer are compiled by installed Lustr, with ordinary `cargo
check`, `build`, `run` and `test`, correct Rust debugging, valid and invalid
semantic cases, and retained edits. It is an integration milestone, not the
definition of full Rust coverage. Preserve the original manifest, source,
dependencies and profiles. `debug=0`, abort-only execution, prebuilt stock
provider metadata and standalone qualifiers remain separately scoped controls.

| Priority / package | State at this planning baseline | Dependency and next implementation action | Completion sections |
| --- | --- | --- | --- |
| P0. Repair demonstrated regressions | READY when a concrete regression is observed | Preserve the failing source/input/result; repair the actual owner and rerun the affected witness. Keep existing profile, API, artifact and unwind gates. Otherwise select P1/P3/P4. | 0, 6 |
| P1. Admit the retained projection service | READY | Replace the proof's full-SDK query with a justified pinned upstream service over native-owned facts. Retain the proven lifetime relationship and stable identities. | 1, 2, 4 |
| P2. Use the service through ordinary Cargo | WAITING for P1's typed service interface; narrow provider metadata work may start now | Produce the provider's genuine metadata with Lustr; connect public import refresh to the retained Owner/Session. Do not wait for P1's final DONE: its service is admitted with P2's first installed Cargo check. Then close linked build/run after P3 and the necessary P5 ABI/unwind work. | 2, 3, 4, 5 |
| P3. Finish Rust debugging and default builds | READY, parallel with P1 | Replace boolean debug admission with represented levels; carry original Rust source/type/location facts into real object debug data. Use the existing default-profile failure as the first acceptance fixture. | 3, 5, 6 |
| P4. Implement Cargo test artifacts and harness semantics | READY for harness work; final default test acceptance needs P3 and the relevant P5 runtime contract | Carry `--test` through the public request, retained protocol and native preparation; implement genuine harness behavior and test selection. | 3, 5, 6 |
| P5. Complete native artifacts and the shared runtime boundary | READY for P2/P4 prerequisites; broad widening follows P2's integrated check | Extend real completed declarations, metadata, instances and ABI beyond flat integers; implement cross-boundary panic/drop/allocation and downstream interoperability. | 1, 3 |
| P6. Close remaining Rust and Cargo capabilities | WAITING for the first integrated service to establish the reusable boundary; inventory and specific dependency research may proceed | Take the first real missing operation in the coverage matrix; import its necessary upstream algorithm, wire its public caller and close the whole feature path. | 1, 2, 3, 5 |
| P7. Qualify retained correctness across the coverage matrix | READY for every implemented row; final closure depends on P1–P6 | Pair fresh and retained outcomes for real semantic edits, invalidation, repair, failure and restart; preserve unaffected work. | 4, 6 |
| P8. Qualify platforms, options and exact packages | READY for implemented cells; final closure depends on P1–P7 | Bind a package to the pinned SDK and declared host/target/edition/profile matrix; close each missing or untested cell without narrowing the agreed compiler goal. | 5, 6 |
| P9. Measure and improve complete work | READY for measurement infrastructure and each working operation; final claim depends on P1–P8 | Freeze workload frequencies and matched baselines, measure full public requests, identify dominant costs, then optimize the owning stage. | 7 |
| P10. Complete replacement qualification and publication handoff | WAITING for P1–P9 | Independently review exact release candidate, every remaining coverage cell and matched overall speed; prepare the concrete package and release record. Publish only with authorization. | 0–7 |

### P1. Retained upstream projection service

Keep the completed proof. Its important pieces are
[`Definitions`](../../compiler/native/src/driver/imports.rs),
[`prepare_projection_import`](../../compiler/native/src/driver/compile.rs),
the native imported projection in
[`generics.rs`](../../compiler/native/src/check/generics.rs), and the
[retained qualifier](../../tools/test-native-projection-retention.rs).
The full-driver implementation in
[`native-projection-qualification.rs`](../../tools/native-projection-qualification.rs)
remains an independent research/reference path, not a product dependency.

The first admitted contract may stay narrow: a concrete nongeneric receiver, a
trait with only `Self`, one associated lifetime parameter and an empty parameter
environment, with all declared well-formedness/outlives predicates validated.
Prove that every supplied lifetime maps to a shared reference to the represented
integer. Preserve the caller's actual region. Wider environments and results
remain unintegrated until their facts and obligations are represented.
Separate stable crate/definition identity from the generation of its observed
facts; do not use fresh identity to disguise invalidation. Represent success,
missing impl, unsupported shape, invalid constraints and implementation failure
separately. Generalization must carry real type/const/region arguments,
environment, universes, obligations and diagnostics; it cannot replace these
with a boolean, a string or a fabricated successful check.

The provider is part of this work: native parsing already retains associated
item generics, but the native declaration checker still rejects the GAT shape.
Expose the normally collected trait/impl/associated-type declarations and their
binders/predicates to this same semantic service. Do not finish only imported
consumer projections while leaving the provider compiled by stock Rust.
In particular, the current associated-item collection discards `at_where` while
rejecting GAT declarations. Removing that rejection without carrying the actual
`where Self: 'a` and other predicates into validation would be incorrect.

Trace the pinned normalization, well-formedness, candidate and region operations
to the smallest viable upstream interface. Evaluate the existing generic type
IR/solver adapters against this exact operation; they are a seam to verify,
not a promised drop-in. Record every retained source/dependency edge, license,
feature/target mode, native adapter and actual caller in the existing import
inventory. If a proposed cut requires duplicate native frontend stages, change
the adapter and retry the same semantic witness. This decision does not require
another user approval or a new architecture project.

Pass when the admitted service preserves the current valid, changed-referent,
missing-impl, lifetime-escape, repair and no-op outcomes without the research
driver in product construction. Add controls for changed impl candidates,
negative lookups, malformed/stale metadata, binder identity and unsupported
shapes. Valid shapes outside the admitted representation remain open coverage;
their explicit integration-incomplete refusal verifies only that boundary.
Measure query/setup/conversion work, including failures. Then continue
directly to P2. Its metadata work can already proceed against the typed service
interface; do not wait for P1's final DONE. P1's production admission and P2's
initial public integration close together at the first installed Cargo check.
Passing another isolated query does not close that integration.

### P2. Ordinary Cargo provider and consumer

Start with the existing `schema`/`Codec`/`Family::View<'a>` consumer, then rename
the crate and definitions and add an unrelated second dependency so fixed names
cannot substitute for resolution. The first complete slice is metadata-only
checking; generic MIR serialization and all library formats are not prerequisites
for this slice.

Extend the real native completed-definition inventory in
[`native_completion.rs`](../../crates/lustr-native-core/src/native_completion.rs)
for the exact struct, trait, associated type, impl, generics and predicates that
this provider requires. Connect their truthful records to
[`lustr-metadata-encoder`](../../crates/lustr-metadata-encoder/src/lib.rs) and its
external metadata/schema readers. Keep declaration, visibility and validity
proofs real. The same installed Lustr must build the provider metadata; supplying
stock-built metadata only proves consumption and cannot close this package.
Require pinned Rust to read the Lustr-produced provider metadata and check its
unchanged consumer as part of this first metadata-only gate. Lustr's encoder
and reader agreeing with each other is not an independent artifact proof.
Include an independently established invalid provider declaration or impl, such
as an associated-type assignment violating its declared bound. Require rejection
before metadata publication; consumer-only errors do not prove provider validity.

Wire the existing
[`public_library_import`](../../crates/lustr-native-core/src/public_library_import.rs),
[`public_executable_import`](../../crates/lustr-native-core/src/public_executable_import.rs),
[`Owner`](../../crates/lustr-native-core/src/public_library.rs) and native
[`Session`](../../compiler/native/src/driver/session.rs). The product daemon
currently replaces a session when imported input changes. Introduce a validated
import-generation update in that existing path: capture exact dependency bytes,
refresh affected facts/signatures, invalidate actual readers and retain unrelated
bodies. Do not simply remove `import_is_current` checks. Negative impl facts,
SDK/configuration identities, failure and publication state must participate.

Pass first through the installed `RUSTC` entry and ordinary pinned Cargo `check`
with unchanged default profiles. Then pass `build` and `run` through that same
route as P3 and the required P5 work land. Require cold/no-op/provider-type edit,
missing impl, lifetime escape, repair and unrelated-body edit. The fully unchanged
source, request, context and imported generation do no upstream normalization
or native body checking; a native body edit must still check the affected body.
changed facts recheck real dependents and leave `spare` cached. Compare actual
results and diagnostic causes against pinned Rust; preserve the last valid
output on failure. Trace candidate children and product source closure so a
stock compiler or whole-driver library cannot satisfy the test invisibly.
Distinguish Cargo not invoking the compiler from a real unchanged compiler
request reusing its owner. Exercise both; an absent invocation cannot prove the
retained compiler's zero-query/zero-body behavior.

The public acceptance commands, run from the unchanged fixture with an actual
constructed candidate at `PACKAGE`, are:

```sh
RUSTC="$PACKAGE/bin/rustc" "$PACKAGE/bin/cargo" check --workspace --offline
RUSTC="$PACKAGE/bin/rustc" "$PACKAGE/bin/cargo" build --workspace --offline
RUSTC="$PACKAGE/bin/rustc" "$PACKAGE/bin/cargo" run --offline
RUSTC="$PACKAGE/bin/rustc" "$PACKAGE/bin/cargo" test --workspace --no-run --offline
RUSTC="$PACKAGE/bin/rustc" "$PACKAGE/bin/cargo" test --workspace --offline
```

These are required future product outcomes, not commands that pass at this
baseline. Record actual tool/SDK identities and effective configuration, remove
unintended compiler wrappers in the controlled fixture, and keep a separate
matched pinned-Rust reference. A prewarmed package, altered profile or changed
source must be labeled as a different case. Do not claim a command ran from a
plan, synthetic argv test or compiler child that never executed.

### P3. Rust debugging and ordinary default builds

Keep panic propagation, stack walking and Rust source debugging as three
separate obligations. Existing `.eh_frame` and caught-panic results do not
provide Rust line tables, lexical scopes, variables or types.

Start at [`library_request.rs`](../../src/library_request.rs),
[`lib.rs`](../../src/lib.rs),
[`library_contract.rs`](../../src/library_contract.rs), native request/protocol,
source maps, checked IR and TinyCC/object emission. Preserve requested debug
levels, split-debug behavior, path remapping and option precedence. Carry source
files, original spans, scopes, types, parameters and local-location ranges through
retained emission. `#line` alone is insufficient. Correctly key cached objects
and debug artifacts by the facts they actually use.

First test whether the current emitter can expose the required code addresses
and variable locations. Keep a small comparison behind the same native checked
body/instance boundary if a necessary debug facility is missing. Select a backend
or narrow imported emission component using correct default build/debug behavior,
construction and compile/link cost, generated runtime and target coverage.
Do not replace or duplicate the native frontend; extend its source/IR debug facts
as required. Do not silently drop requested debug data. A justified
codegen change is allowed; repeated line-table-only demos are not completion.

Pass an unchanged default-profile binary and a path-dependency workspace under
`cargo build` and `cargo run`. Debug the original Rust: source breakpoint,
step/next/finish, parameter and local inspection, reference/type display and
backtrace across the relevant boundaries. Repeat after body and signature edits,
module movement and output relocation; reject stale debug generations. Keep the
existing corrupted-CFI controls. Run the declared GDB/LLDB and platform cells as
they become implemented; do not label one Linux debugger pass universal support.

### P4. Cargo test mode

Translate `--test` into an actual retained artifact mode through public argument
handling, the protocol, session identity and native preparation. The existing
`lust test` runner is not the Cargo test interface. Its one-child-per-test
behavior cannot stand in for the requested harness. Preserve `cfg(test)`, unit
and integration targets, generated entry, `harness=false`, test listing/filtering,
ignored tests, `should_panic` and `Result`/`Termination` behavior. Use necessary
pinned upstream harness/runtime implementation through the same native-owned
body and instance pipeline; do not delegate test compilation to rustc.

Verify named tests, counts, output and exit status against the pinned harness,
including an ordinary failing assertion, wrong/missing expected panic, panic
payload, destructor order, nested catch and a test returning failure. An earlier
compile error cannot satisfy an expected runtime failure. Cover parallel test
execution, flags after `--`, dependencies and doctests/rustdoc in their proper
tooling rows. Never rerun an already executed test as compiler fallback.

Pass `cargo test --no-run`, then actual `cargo test` on the unchanged default
workspace. Add and remove a test while retained state lives; verify discovery,
body invalidation and absence of stale tests. Continue to P5/P6 when this works.

### P5–P8. Full language, artifacts, retention and platforms

Expand using one capability matrix, linked from this checklist, whose rows bind
the pinned language/library operation, actual native support, bridge support,
complement owner if needed, public caller, valid and invalid witnesses, retained
history, artifact/runtime obligations and supported target/profile cells.
Populate it from pinned Lust and Rust sources and tests before candidate runs.
Unknown, excluded, filtered and untested cases remain visible work. Do not turn
the current scalar grammar or a handpicked successful crate corpus into the
definition of Rust.

Work in dependency order, taking the narrow P2/P4 prerequisites first:

1. Native declaration/ABI and metadata: remaining integer widths, aggregates,
   enums, references/slices, methods, generics, traits, exported/inline bodies,
   associated items and real monomorphized instances. Preserve privacy, lints,
   layouts, symbols and crate/dependency identity. Validate independent pinned
   Rust consumers of Lustr artifacts and Lustr consumers of Rust artifacts.
2. Shared runtime: real standard-library linkage, allocations and deallocation,
   drop flags/glue, partial initialization, foreign calls, native/imported panic
   propagation, threads and terminal double panic. The current native pending
   panic mechanism requires an explicit ABI/runtime bridge; CFI alone cannot
   make it compatible with a separately linked Rust unwinder.
3. Complementary semantics: expansion/hygiene and proc macros; general trait
   selection/projections/coherence; typed control flow, move/borrow/region facts;
   const evaluation and type-level constants; closures/async/coroutines; unsafe,
   intrinsics, SIMD/assembly and target-specific requirements. Reuse the justified
   upstream implementation. Each row includes declaration, checking, lowering,
   emission, metadata, runtime and invalidation, not just syntax acceptance.
4. Cargo/tool integration: real workspace graphs, features, build scripts,
   host/target separation, environment/generated inputs, path/registry/git
   dependencies, custom profiles, optimization/LTO, diagnostics/JSON/dep-info,
   `rlib`/`staticlib`/`cdylib`/`dylib`/proc-macro outputs and rustdoc/doctests.
5. Retained and package qualification: fresh/retained equivalence for every
   integrated row, positive and negative reads, impl/const/layout/drop changes,
   macro and environment changes, undo/repair, eviction/restart, concurrent
   requests and failed object/link/publication. Bind installed bytes, SDK,
   source/import manifests, licenses, relocation and offline reconstruction to
   actual compiler-host/output-target/profile/edition cells in section 6.

Use upstream UI/run/run-make/incremental tests, native regression histories and
unchanged real projects. Freeze expected behavior and test selection independently
of the candidate. Repair failures in their owner and keep taking the next row.
The coverage matrix and operational queue are parts of this register, not new
roadmaps or separate competing task systems. A package can advance in small
commits; its DONE state cannot precede its public integration evidence.

### P9–P10. Complete-work speed and release qualification

Establish measurements as soon as an operation works. Keep pristine Lust,
integrated Lust, packaged Lustr and matching pinned stock rustc comparisons
separate. Freeze representative workloads, operation frequencies, cache states,
profile/debug/panic/target settings and sampling before looking at results.
Measure startup, transport, all semantic work, conversion, emission, metadata,
linking and requested program/test execution together. Also retain phase data,
memory/daemon growth, tails, maxima, errors, source identities and slower cases.

For operation counts `n_i`, the observed workload speed ratio is
`sum(n_i * mean(T_rustc_i)) / sum(n_i * mean(T_lustr_i))`, with a paired
uncertainty assessment and all required successful work included. Ratios from
unrelated microbenchmarks and zero/one checked-body counts do not prove this
result. Failed or unsupported workload operations cannot be discarded to obtain
a gain. Follow section 7 and the performance policy for the exact assessment.

Profile the dominant excess cost, make the smallest owning-stage correction,
preserve a failing/slower predecessor, then repeat the affected matched
comparison. Do not postpone all speed work until coverage is complete, and do
not demand an unrelated whole-compiler benchmark for every documentation change.
Near-stock native/complement targets are separate optimization claims. Full
correctness plus matched overall speedup qualifies the first replacement; perfect
subset parity, a `no_std` rewrite and socket removal are not extra release holds.

The current source gate deliberately admits only an unavailable release with
`full_rust_coverage=false`. P10 must replace that temporary refusal-only policy
with independently checked positive admission bound to the actual package,
coverage and performance evidence. A green refusal test or a flipped inventory
flag is not completion.

When sections 0–7 close, independently review the exact installable candidate,
publishable evidence and unresolved inventory, prepare the release and request
only any publication authority that is still missing. PR #33 may be integrated
under its applicable review/checks before that product milestone; its merge
does not close the compiler task or disable continuation.

### Coordinator and resume record

Current P0/P1 repair, base `d89969c0`: the P1/P2 worker owns conditional impl
associated-item environments in `check/generics.rs`, native region validation,
their dependency fingerprints and focused/public qualification. Independent
pinned-Rust evidence accepts item-local `T: Copy` and `T: 'static` assignments
that the preceding candidate rejects. Retain the item's real conditions and
validate assignment and compatibility in that item's scope, preserving impl-wide
requirements and sibling isolation. Repair this before widening admission.
P3 debug spans/emission has a separate active owner and worktree; shared source
manifests and publication require serial integration. No prior timed handoff
is an implementation stopping condition for the resumed worker.
The repair retains impl-associated predicates and regioned values separately,
checks assignments and compatibility in their actual environments, and includes
those facts in dependency fingerprints and retained span shifting. Qualified
compiler source `2ea27186` passes 22 independent pinned/native fresh cases,
ten retained edit/failure/repair states, eight focused native tests and all
32 retained patch tests. Its installed scalar Cargo/import history passes
38 operations, including independent metadata reading. The first qualifier's
missing-counter observation failure remains preserved. See the
[source-bound repair record](evidence/conditional-item-environments-20261123/README.md).
The demonstrated conditional-environment regression is repaired. P1/P2 remain
ACTIVE: the next source action is the selective pinned service over the retained
trait/impl-associated declarations and genuine provider metadata through the
default Cargo path. GAT admission remains gated until that integration is checked.

The next compatibility control demonstrated a related region-erasure defect on
the same package: an item's `T: Gate<&'static u8>` was incorrectly accepted
under an impl-wide `T: Gate<&'a u8>` assumption. Pinned Rust rejects it. The
native region proof now distinguishes impl-side goals from trait-item allowed
assumptions and preserves their frames; no item condition becomes a whole-impl
fact. Compiler source `11a45637` passes nine focused and 32 retained patch tests,
27 fresh pinned/native cases, 16 retained states (86 operations), and the
38-operation installed scalar Cargo/import regression. Original failed and
passing records remain source-bound in the same repair evidence. Next, carry
every associated binder and regioned bound into the typed P1/P2 declaration
boundary; keep provider admission gated until the selective service and metadata
consumer join are actually checked.

P1 declaration work now retains the associated item's identity arguments in
written lifetime/type/const order and region-preserving bounds on its full
application. A generic bound no longer resolves solely in the parent's scope.
These facts enter declaration fingerprints; they are inputs for the semantic
service and metadata producer, not a successful GAT provider proof. Ten focused
tests pass, including the normal collector's retained `Gate<&'a T>` bound,
`Self: 'a`, `T: 'a` and lifetime fact. Source `7569bf35` also passes all 32 native
patch tests, the installed 27-case/16-state ordinary regression and 38-operation
scalar Cargo/import history. The broader GAT declaration is independently valid
in pinned Rust but remains rejected by Lustr (L0621 plus legacy declaration
diagnostics), explicitly unqualified language scope. The next P1 source action is
the pinned early-argument/region instantiation and obligation service consuming
these actual declarations; its successful result must feed P2 metadata, and
substitution alone cannot close admission. Keep the GAT gate until that join.

23 November implementation continuation: P2's imported-generation update is
ACTIVE on the `3507c74d` base. The public owner validates replacement metadata
and archives before refreshing native declarations, replays actual dependency
reads and retains unrelated bodies. Exact input bytes participate in that
generation; disk restoration remains disabled. The installed scalar
default-profile Cargo check/edit history is the first acceptance operation.
The new Rust qualifier is `tools/test-public-import-generation.rs`. Initial
execution passed default Cargo check, independent pinned-Rust metadata reading,
provider body/type edits, type repair and renamed-definition failure/repair,
then exposed a malformed-input recovery regression: status 3 reset a session
even though validation had not entered native preparation. Its repair keeps
input failures separate from failures after compilation starts. Qualification
of that repair passed on `3eae63d837195cf454e9ad8b611bc805bf11a47c`:
the complete scalar history, including malformed-input repair, passed with
zero unrelated-body checks. This is a P2 prerequisite, not admission
of the P1 GAT service or completion of P2. Next: construct this exact candidate,
run the valid/type-change/malformed-input/repair histories and pinned Rust
metadata consumer, then continue the native declaration/predicate service.

P0/P1 now also owns the demonstrated associated-item where-clause gap:
the native checker accepted a non-Copy provider for an associated type requiring
`Self: Copy`, while pinned Rust rejected it with E0277. Collection now retains
typed associated-item parameters, universal lifetimes, region facts and where
predicates separately from trait-wide bounds. Ordinary provider assignments
consume those predicates in native impl validation. GATs remain unadmitted;
their original rejection is preserved while the service's necessary facts are
retained. Focused predicate and fingerprint controls are being run. The next
slice preserves regioned associated-item predicates and checks their actual
impl-header lifetimes, avoiding erased-reference false acceptance/rejection.
Named-lifetime traits with associated types retain their separate preexisting
gate; they remain open coverage. The next
integration step remains the selective upstream normalization/WF/region service
and genuine provider metadata; these prerequisites do not close P1 or P2.

The next owned regression slice also checks an associated item's native trait
predicates when a generic consumer writes its projection. Pinned Rust rejected
the missing `Copy` condition with E0277 while the installed preceding source
accepted it. Associated bounds now retain their collection provenance and are
checked in the item's own environment; its conditions do not become supertrait
bounds. On compiler source `d2edc3a3d2e504c52577d9dbdae5eb4795ff2065`, all six
focused tests, 32 retained patch tests and the installed scalar history pass.
Four valid/invalid predicate pairs match pinned Rust through the public CLI.
The separate same-workspace abort/debug-disabled/non-incremental link control
changes actual output from `63` to `127` on a provider edit with zero consumer
body checks. See [verification](evidence/import-generation-20261123/verification.json).
The broader traced campaign is unrun here because strace is unavailable.
P1 remains ACTIVE; P2's scalar prerequisite is verified, while production GAT
service/provider metadata and default build/run/debug/test remain unfinished.
Next: consume the retained native item binders, bounds and predicates in the
selective pinned normalization/WF/region service, then extend genuine provider
metadata through ordinary default-profile Cargo check. Do not remove the GAT
gate or substitute the independent full-SDK proof for that integration.

The installed compiler at `49ee309d` checks the unchanged shared-reference
associated-type workspace through ordinary default-profile Cargo. Lustr
validates the provider and emits its Rust metadata. Lustr and pinned Rust both
accept the initial and repaired consumer, reject a changed referent and a
missing implementation, and preserve the original metadata bytes after
repair. Cargo skips compiler requests on a no-op. The consumer checks two of
three bodies after repair. The separate retained consumer qualification also
rejects a lifetime escape and preserves the valid outputs across malformed
metadata. The regression suite exercises renamed crate and definition
identities and different trait and impl lifetime names.

The Linux x86-64 linked path now builds and runs the represented provider and
consumer through ordinary Cargo in both default and release profiles. The
provider can export native functions alongside the lifetime-generic associated
type. Pinned Rust independently reads the native metadata, links the native
archive and runs the unchanged consumer. A provider function edit relinks the
consumer without checking its bodies again. A changed associated type still
invalidates affected work and preserves the last valid executable on failure.

Source `ef93bc2f` also preserves shared `str` and integer-slice referents in
the native and metadata representations. Pinned Rust and Lustr agree on their
runtime results and reject an escaping lifetime. Continue with wider
predicates, receivers, associated results and library signatures.

Source `78cb4906` builds ordinary Cargo unit tests for a represented
`#![no_std]` library. Its test uses the core prelude and an explicit,
configuration-enabled `extern crate std` for hosted output. Both compilers
reject a disabled import, an unqualified std-only macro and an unqualified
`Vec`. The installed package still lacks Rustdoc, so `cargo test --doc`
fails when Cargo launches it. Freestanding executable compilation and the
broader no-std target and runtime matrix remain unfinished.

Source `d5772a23` carries build-script configuration into the actual Cargo
unit. Pinned Rust and Lustr agree on Boolean flags, multiple values of one
flag, inactive declarations, changes that enable a different function and
rejection that preserves the previous executable. Source `25f75c32` also
accepts these represented options during Cargo target discovery and native
compilation when supplied through `CARGO_ENCODED_RUSTFLAGS`.

At `f8353d49`, Cargo and direct Rust requests can link a native application
to the represented genuine Rust archive using its embedded metadata. The
independent SDK built the `#![no_std]` provider. Both compilers ran the
application and produced 42. Lustr preserved the previous executable when
the archive was replaced by a stub-only or corrupt file. Direct requests
preserve the selected output path and accept either a source file or stdin.
The stdin tests also preserve Rustdoc source coordinates when supplied.

The coordinator owns this register, shared-interface decisions and final
integration. Worker A starts P1/P2; worker B starts P3; worker C starts P4 and
the runtime prerequisites it exposes. Use isolated worktrees and non-overlapping
path ownership. Agree a shared protocol/IR change before parallel edits. Finished
workers review or implement the next dependency-ready slice; CI running on one
slice does not idle all workers. Use the [AGENTS continuation loop](../../AGENTS.md#continue-implementation-across-checkpoints).
Serialize coordinator invocations through the existing authorized continuation
and its private active-session claim before dispatching workers. A Git expected-head
check protects publication but does not prevent duplicate builds. Reconcile an
unknown command/session outcome before launching a replacement.

At every handoff update the following record here, preserving original logs in
the source-bound evidence path and private runtime command/session details in
the existing local checkpoint. A missing active worker means unclaimed work,
not completed work. A dispatch acknowledgment means submitted, not executed.

Each active slice adds a short row with: package and owner; base and current SHA;
owned paths; exact unchanged acceptance operation; state and actual last result;
evidence link; next action/command; dependency or successor. Keep command/session
IDs and private paths in the private checkpoint. A worker must not mark a package
DONE because only one research or implementation substep passed.

| Resume field | State when this plan was prepared |
| --- | --- |
| Integration source | PR #33 live branch; planning baseline `bc38dece7a734b97c0108582b7f61f98ccb819db`. Resolve the current head before editing. |
| Last actual semantic result | Retained projection history `[3,0,2,0,2,0]` checked bodies and `[1,0,1,1,1,0]` upstream queries; standalone research entry; no product promotion. |
| Active implementation claim | None made by this planning change. Claim the next slice before modifying shared paths. |
| First next action | P1: inspect the exact query closure and implement its native-owned service adapter; in parallel P3: carry requested debug levels and original Rust debug facts through the public request/IR. |
| First next public witness | P2 metadata-only default Cargo provider/consumer check; P3 default-profile build/run; P4 default test compile and real harness outcomes. |
| Pending validation | Refresh exact-head hosted checks. Existing prior-source passes remain attached to their own sources. |
| External blocker | None demonstrated for writing the next code slice. Selective solver integration and missing debug/test behavior are engineering work, not reasons to ask for another continue. |
| End condition | Sections 0–7 qualified, complete-work speed established, concrete release handoff complete; actual publication follows its authorization. |

After implementing and validating a slice, update its state and next action and
continue in the same session. If a runtime ends, save the unfinished diff,
immutable head, test identity, exact next command and running-command identity
before yielding. The authorized continuation resumes this record, reconciles
live work and proceeds. It must not merely watch CI or issue another summary.

## 0. Correct active direction and reconcile existing work

Status: the documentation correction is merged. The implementation correction
removes conflicting entry, helper and package routes, adds the partial import
inventory and preserves useful components. Its focused validation and remaining
limits are recorded in the [cleanup record](exclusive-coverage-cleanup.md).

- Keep AGENTS, README, architecture, contract, design, roadmap and performance
  policy consistent. Supersede permission for a full compiler import graph,
  whole-compiler fallback and independent reimplementation of every Rust gap.
- Record one integration head, ancestry, full/native trees and artifact evidence.
  Preserve unique fixes and failed evidence before removing active routes.
- Use the [disposition record](retained-work-disposition.md) to retire conflicting
  production code and packaging claims without losing useful components.
- Preserve already integrated process ownership, scanner-fixture, SDK acquisition,
  callback, Unicode, normalization, native runtime and retained-state repairs.
  Verify whether a fix is present at the chosen head before reimplementing it.
- Correct demonstrated behavior before widening admission. The 22 November
  audit found that an omitted overflow-checks option did not inherit an explicit
  `debug-assertions=no` setting as the pinned Rust compiler does. Both entry
  adapters are corrected; all 24 cold profile cases now pass. The SDK identity
  probe's separate loader defect is also fixed and the complete constructor
  passes against the unchanged affected SDK. The
  [audit record](evidence/compiler-review-20261122/README.md) binds these results
  to exact source, package and evaluator identities. Retained profile-option
  transitions remain an additional qualification obligation; the cold profile
  cases do not establish that history.
- Keep the public status and import/profile descriptions consistent with actual
  source and qualified scope. A historical failed scalar join must not send the
  next contributor back to rebuilding an already implemented service.

Done when current instructions agree and every code disposition has an exact
implementation change. A documentation merge alone does not close code cleanup.

## 1. Cover Lust's real supported set

Inventory support from the pinned native implementation, its contexts and actual
fixtures. Expose prepared checked state and dependency reads before emission.
Keep native preparation, genuine complement requirements, unintegrated supported
work, invalid input and implementation failure distinct.

Derive and qualify support rules in development/CI, then apply them inside the
one normal frontend or retained-state update. A scanner, separate parser/checker,
probe compilation or diagnostic retry cannot select the production owner.
Unprobed/unknown donor evidence is not proof of a complementary obligation.
Capability test coverage must state its exceptions; a general CI pass is not
proof that the capability tests ran.

Advance the public artifact bridge through the existing donor-supported types
and operations: remaining integer ABIs, aggregates, references/slices, enums,
methods, generics, traits, modules and their runtime requirements. Preserve the
original donor and its explicit integration patch. For each expansion, require
an unchanged public request, independently checked result and precise retained
edit behavior. Keep general native-language support separate from the smaller
set currently represented in genuine Rust library metadata.

Done when pure and mixed programs retain Lust ownership for the complete covered
set being claimed. The current scalar bridge cannot define that set. Failure
in a represented native emission must fail the request without reassignment.

## 2. Admit only necessary upstream complementary code

Choose a demonstrated missing capability and a real upstream interface. Record
source revision, paths/symbols/hashes, licenses, feature/target dependencies,
verified adapters, necessity witnesses, actual callers and semantic ownership.
Include existing partial Unicode/normalization imports and their dependencies.
Disclose full SDK/compiler-private research components separately.

Check effective product dependency and feature closure, including defaults,
optional, renamed, target and build dependencies and transitive lock changes.
No product feature combination may activate a retrying experimental ABI or a
full competing SDK. Remove obsolete general launchers and retain only useful
bounded research qualifiers outside production admission.

Use one retained semantic database and authoritative definitions/types/instances
across the native and imported services. The pinned
[`Interner` interface](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_type_ir/src/interner.rs)
is a researched upstream boundary, not a complete implemented complement.
Select the next service from an unchanged mixed fixture's demonstrated missing
operation. Require native-owned consumers of the imported fact, valid and
invalid cases, and an edit that invalidates those consumers without starting a
second frontend. Preserve the completed scalar service while testing this
general semantic boundary early. Start this work alongside ordinary Cargo/debug
completion after correcting demonstrated behavior, before another broad scalar
expansion. Narrow bridge prerequisites for the mixed fixture may proceed.

Require the unchanged mixed program to execute, its invalid controls to fail
for the expected reason, and an edit to the imported fact to invalidate native
consumers while preserving unrelated work. Record the import closure and phase
costs. If this needs duplicate native stages, fabricated facts or whole-program
analysis, redesign the failing identity/query/typed-body adapter and re-run the
same fixture before widening admission. A compiler fallback or unrelated scalar
pass cannot close the failed boundary.

The [associated-type qualification](../../tools/test-native-projection-retention.rs)
builds genuine dependency metadata with pinned Cargo and Rust. Lust uses the
upstream projection result in direct function parameter and return types and
compiles the original consumer into a runnable native executable. Only the
dependency metadata goes to the upstream query. The test rejects a changed
referent type, a missing impl and a lifetime escape. The same native session
rechecks only the function and caller affected by the dependency type edit and
repair. The unchanged request invokes neither upstream normalization nor
native body checking. Rejected requests preserve the last executable.

This is a standalone research entry for one lifetime-generic associated type.
It uses a pinned SDK to load metadata and prove a shared integer reference
whose lifetime is the supplied argument. It does not admit arbitrary
projections or enable the mixed operation through the ordinary Cargo driver.
The qualifier records query and native request durations. The results do not
qualify complete-work performance or establish that the SDK closure is minimal.

Complete the remaining macro/proc-macro, trait/projection, region/move,
const-evaluation, unsafe/FFI and async/coroutine obligations through justified
upstream reuse. A feature is not complete until its declarations, checking,
lowering, runtime, artifacts and dependency reads work in the same program.
An adapter must expose complete typed control flow to borrowed borrow-checking
operations; a generated object cannot stand in for an invalid-program proof.

Done when admitted imports implement their claimed complementary obligations
inside the owned compilation and the closure contains no unjustified duplicate
Lust-owned stages. Neither an allowlist nor a dependency count proves global
minimality. Full-driver linking and whole-source vendoring cannot close this item.
A structural guard does not establish full Rust semantics or runtime correctness.

## 3. Compose semantic state, runtime and real artifacts

Share or bridge authoritative definitions, hygiene, types, obligations,
monomorphized instances, layouts, ABI, constants, symbols, allocation ownership,
drops and unwinding. Reuse necessary genuine metadata/archive/link machinery.
Avoid running complete consumer analysis merely to choose native instances.

For completion beyond the represented scalar path, the existing
[consumer-metadata completion study](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md)
records the original encoder's frozen-definition boundary through `analysis(())`.
Native prepared state does not yet provide all wider declaration/privacy/lint
obligations. Supply the necessary completion using actual native proofs. Preserve
original applicable checks, real emitted instances and the exported-symbol
query. Unconditional analysis success, fabricated typed results or empty
instance graphs are forbidden. The original scalar join is now complete within
its qualified scope. Extend that service for wider native declarations and
connect the first general semantic service through those same identities;
neither task may recreate the retired whole-consumer analysis route.

The [bounded native metadata caller](../../crates/lustr-native-metadata-qualification/README.md)
joins actual completed scalar declarations and emitted objects to the selective
serializer. Its [exact-source hosted campaign](evidence/native-metadata-join-20261120/README.md)
passed on tree `948074da4b3b23ed25fa789b17a924555f24e4ca`, with 31 named controls,
262,152 runtime assertions and eight complete candidate traces. This fixed
scalar-library evidence remains separate from this item's general completion.
The later retained research campaign at `fab59023` passed its 29 transitions and
25 consumers with 6,815,952 runtime assertions. Its
[original evidence](evidence/retained-metadata-correction-20261120/runtime-success-fab59023.json)
remains tied to that source.

The [public scalar-library candidate](public-scalar-library.md) now connects the
installed `RUSTC` entry and retained owner to genuine metadata/archive output
under the [declared profile and actual Cargo command](native-cargo-continuation.md#authored-public-scalar-library-path).
The [independent public evaluator](../../tools/test-public-native-library.py)
passed the retained library link and metadata-only campaigns on the published
check implementation. The direct-dependency follow-up `fb2ec542` passed the
local traced Cargo campaign for a two-crate `no_std` scalar project. It reused
the consumer's metadata on an unchanged request, rejected a changed provider
signature, and emitted metadata that the original Rust compiler consumed in
a third crate. Hosted evidence is bound separately to its exact source.
The local dependent-library build passes cold linking, unchanged archive reuse
and an edited provider. The independent executable completed 131,080 runtime
assertions against the genuine native archives. The ordinary `cargo run` path
also passes on the Linux x64 scalar project, including the unmodified cold
build, an unchanged request, body edit, type error, repair and provider edit.
The body edit visits one body and changes two of six native object
generations. The unchanged request reuses the executable with zero body
checks. The matching `cargo check --bin` passes the same six cases with
no object, archive, link or execution work. Both commands also pass when
the edited body lives in an out-of-line module and the root file stays
unchanged. These preceding local and historical statements retain their named
scope. The [hosted checkpoint at 514dfe1e](native-cargo-continuation.md#verified-hosted-checkpoint)
adds independently checked public campaign evidence for the current source.
It includes unwind-mode scalar leaf libraries with debug information disabled.
The local public campaign also builds and runs a dependency-free unwind
executable with `debug=0`. Its destructor order and caught panic match pinned
Rust. An unchanged request checks zero bodies, an edit checks one, and an
invalid edit leaves the prior executable intact. TinyCC's final link retains
the executable's unwind ranges.
Wider Rust and Cargo coverage remain open.

The remaining runtime/artifact work is concrete:

- Finish ordinary dev-profile executable linking and test builds while preserving
  the requested panic strategy, debug information and option precedence.
- Exercise actual cross-boundary panic propagation, destructor order, caught
  panics, thread panics and terminal failure cases. Keep these results separate
  from the existing GDB frame-walking result.
- Supply Rust source line/type/parameter/local information and test actual
  breakpoints and stepping after retained edits. A generated-C location or
  `.eh_frame` section is insufficient.
- Encode and consume genuine generic/inline bodies, exported traits/macros,
  layouts, crate/dependency identities and required instance graphs. Link and
  execute independent consumers in both directions at the pinned SDK revision.
- Integrate real standard-library and dependency/runtime behavior without
  substituting adapter crates to count arbitrary Cargo packages as supported.
- Preserve allocation ownership, drop flags, constants, FFI and unwind rules
  through every native/imported boundary and each declared crate type.

Done when unchanged mixed programs and real downstream Cargo crates consume
valid artifacts. Every obligation has one owner and every required instance is
emitted exactly once. Unsupported interfaces remain visible as integration gaps.

## 4. Retain correct work across changes

The [native Cargo continuation](native-cargo-continuation.md) records the current
unit-plan and host-script integration, narrow reuse checks, negative module
lookups and generated-input leases. The new public scalar candidate adds retained
artifact and inactive-history ownership with source-bound edit, rejection,
repair and eviction controls. The passing public checkpoint covers the declared
campaign, not the wider completion conditions below.

Connect precise native and complementary semantic dependencies. Cover body,
signature, type, trait, macro, included-file, dependency, environment/build-script,
target/profile and policy changes. Track negative lookups and real const/runtime
inputs. Test edit/reversal histories, restart and failed publication.

Include newly added impls and negative lookups, macro expansion/hygiene, generic
instance changes, layout/drop changes, removed and reintroduced definitions,
cache eviction and concurrent workspace requests. Host-script and proc-macro
reuse needs the executable and observed file/environment inputs; when those
inputs cannot be proved unchanged, perform valid recomputation within the
assigned owner. Include host-versus-target changes and generated source.

Done when fresh and retained results agree, relevant changes invalidate consumers,
unaffected native work stays retained and failed/ambiguous work cannot overwrite
the last successful state. Preserve independent oracles in qualification.

## 5. Replace conflicting public execution paths

Status: the retired retry/full-driver routes remain excluded from product
construction. Bounded public Cargo requests now work through the native owner.
Ordinary complete Cargo usage remains unfinished and must use the same pipeline.

Remove ordinary whole-program native-fail/rustc-retry, diagnostic-based routing,
user-selectable language engines and hidden stock/full-driver escape paths from
public compilation and package builders. Preserve useful Cargo graph/argument,
configuration, artifact and process handling. Disable invalid product claims
until the actual complement is delivered.

Complete these user operations on unchanged projects with their existing
dependencies and settings:

| Operation family | Required completion evidence |
| --- | --- |
| Direct `rustc` and Cargo discovery | Correct represented command/query/options, diagnostics, exit status, dep-info, environment and target/profile precedence. Account for unsupported flags explicitly until implemented. |
| `cargo check`, `build`, `run` | Normal workspace/package/target/feature selections, real dependencies, build scripts and proc macros; cold and retained results through the public entry. |
| `cargo test` | Unit/integration tests and applicable test harness behavior, unwind/drop semantics and expected invalid-program/runtime failures. |
| Release and custom profiles | Requested optimization, debug, panic, codegen/LTO and linker behavior without whole-program delegation to stock rustc. |
| Libraries and tool interoperability | Applicable `rlib`, `staticlib`, `cdylib`, `dylib` and proc-macro artifacts, downstream linking, rustdoc/doctest integration and Cargo JSON/diagnostic consumers. |

Start with a small workspace that exercises the complete public operation,
then qualify unchanged real projects with the same mechanisms. Keep a feature
refusal visible until implemented. Changing source, dropping features or
disabling debug/unwind is a separate fixture, never the original project's pass.

Done when production entry points use the exclusive boundary, a supported case
cannot invoke the competing compiler, and incomplete coverage fails honestly.
Failing unsupported work is a direction-enforcement milestone, not full Rust
replacement or successful complement implementation.

## 6. Qualify exact source, semantics and packages

Reconcile current evidence before rerunning gates. Historical
[qualification](qualification-review-2026-11-18.md) is preserved unchanged.
Do not declare a predecessor's failure fixed without evidence or assume a later
source still has a repaired defect.

Qualification must cover version-bound character/lexical semantics and NFC,
callback ABI, valid/invalid programs, actual complementary outcomes, full-language
constructs, macros/build scripts, dependencies, real metadata, allocation/drop,
panic/unwind, runtime and the integrated path on every advertised target/profile.
Inspect matching tools/sysroot, source/binary manifests, relocation, offline
rebuilds, invalidation, required assets and notices for the exact package.

Map the declared matrix to the applicable pinned upstream test mechanisms:
Lust's native/differential/retained-state suite and Rust's UI acceptance and
rejection cases, explicit execution-result checks, run-make artifact and
cross-crate checks, and incremental revision histories. Preserve original
fixtures and expected behavior. Record which source, compiler artifact, route,
target/profile, cases and assertions actually ran. A required case that is
skipped, cancelled, filtered or absent remains an open qualification gap.
Constructor success and helper tests cannot substitute for the public compiler
or its generated programs. A passing exit alone does not validate printed
results. Keep evaluator failure controls separate from compiler evidence.

Fix acceptance expectations from pinned source and independent evidence before
running the candidate. A donor harness that calls two failed native compiles
"unsupported" does not prove preserved coverage. Qualify known supported edit
histories independently; retain excluded histories as unqualified. Separate
compilation, artifact production and program execution so the wrong failure
stage cannot pass a negative runtime test.

Done when the package satisfies architecture and applicable correctness with
no fallback hiding a known gap. Cargo-only compatibility does not qualify an
integrated native platform. No current artifact is qualified as the full product.

### Host, target and profile matrix

Keep host execution, output target and requested profile separate. Inventory
the exact declared SDK's target list and classify every target; a family row
below is not a claim that all its members work. Record the target triple,
compiler host, SDK and linker identities, crate/operation/profile settings,
source/package hashes, expected semantics and execution evidence for every
advertised cell. Unsupported, unimplemented, untested and failed remain distinct.

| Matrix area | Current boundary and required qualification |
| --- | --- |
| Linux GNU x86-64 host/target | The exact-source public scalar campaign passed. Broader native Linux workflows were pending at the review checkpoint. Full default dev/test/release and general Rust artifacts remain open. |
| Other Linux ABIs/architectures | No scalar GNU x86-64 result transfers to musl, AArch64, RISC-V or another target. Qualify target facts, object format, layout/ABI, runtime, linker and generated results independently. |
| macOS x86-64 and arm64 | The current broad macOS workflow result was pending when observed. Qualify the actual native route, Mach-O linking/signing, runtime, debug information and retained edits for each host/target combination. |
| Windows MSVC/GNU and other hosted targets | Qualify applicable PE/COFF or other object format, process/argument/transport behavior, linker/runtime, unwind/debug format and ordinary Cargo operations. No Linux pass supplies this evidence. |
| WebAssembly and bare-metal targets | Preserve the declared target's actual `core`/`alloc`/`std`, ABI, panic and execution environment. Use appropriate execution evidence; a host-only library check does not establish target operation. |
| Remaining SDK targets | Keep each exact triple visible in the declared inventory with its implementation and evidence status. Do not silently exclude it while advertising universal rustc replacement. |
| Cross compilation | Build scripts/proc macros execute for the host while program artifacts target the selected triple. Qualify both paths, dependency identities and output behavior together. |

For every applicable cell cover default dev/test, release and supported custom
profiles; declared Rust editions, features and configuration; panic abort/unwind
where the target provides them; debug levels and
stripping; incremental on/off; and required crate types. A setting unsupported
by the target itself is not a compiler defect, but its expected rejection or
target-defined behavior must match the declared Rust SDK. Required skipped or
cancelled executions remain unqualified. This matrix records completion scope;
it does not impose an unrelated portability rewrite before the existing pipeline
works.

Keep the compiler language version, imported semantic source, SDK and metadata
version coherent. An upstream upgrade needs explicit source/dependency changes,
new or changed acceptance/rejection expectations, cross-crate version checks and
retained invalidation evidence. The previous pin's passing result does not
qualify the new revision.

## 7. Establish overall speedup, report subset targets, then release

Use the [performance policy](practical-performance.md). Keep all historical
budgets and decisions immutable. For new assessments, match source versions, requested
work, flags, targets, thread counts, cache/lifecycle state, workload inputs and
actual artifacts. The existing complete-package assessor requires 24 matched
pairs plus applicable qualification. Historical 12-pair characterization is
not that prospective assessment.

Compare pristine Lust on work it actually supports and stock rustc at the
artifact-compatible pin. If that pin is older, identify a current-stock-Rust
comparison separately. A Cranelift baseline is useful where it provides the
same required semantics, but it cannot replace the matching stock comparison.
Record the actual frontend/backend concurrency and Cargo jobserver settings.
Choose unchanged small and multi-crate projects, generic/trait-heavy code,
macro/build-script workloads and real edit sequences before sampling. Preserve
the actual source, features and requested artifacts in each comparison.

The first complete replacement must beat matched stock rustc overall on the
predeclared workload and frequencies, with paired uncertainty for total elapsed
time. A geometric mean of case speedups does not prove reduced total time. For
parallel Cargo builds, measure the complete graph's wall time directly.

For separate near-stock claims, supported warm check/build uses the 10% relative
bound and ordinary complementary operations the 5% bound. These are engineering
targets, not extra mandatory release prerequisites. Missing either target remains
visible as optimization work; meeting both does not itself prove overall gain.
Absolute added latency remains visible, but the historical 0.5 ms alternative
cannot admit a new near-stock claim.

Use paired total-time ratios within each near-stock case. Geometric ratios are
diagnostic only, because a rare slow request can erase the elapsed-time saving.
Retain means and maxima alongside p95 without inventing an extra release gate.

Assess the overhead of each material integration boundary during development.
This final package gate consolidates those results; it is not permission to
postpone all speed checks until implementation is complete. Include the whole
public request and compare equivalent work, with additional artifacts reported
separately when stock Lust cannot produce them.

Keep cold/no-op, body/signature/type/trait/macro/dependency edits, rejection,
priming, link, tails, memory, generated-program execution and compile-plus-test
results separate. Do not describe an overall gain as proof that every class
improved, or weaken correctness to meet a timing number. Keep known-inapplicable
historical campaigns unchanged rather than bypassing their guards.

Done when one exact integrated package satisfies full coverage, exclusive
ownership, selective imports and applicable correctness, demonstrates overall
acceleration, and its README reports actual source/tool identities, complete commands, confidence
limits, failures, losses and raw evidence. Only then describe and publish it as
the qualified replacement for its stated scope. Continue reducing the gap to
stock Lust after that release; do not turn perfect parity into a completion hold.

## Preferred portability follow-up

Prefer `no_std` with `alloc` for compiler logic and retained compilation without
required Unix sockets. Preserve the existing Lust engine and speed. The user
permits deferring this migration when it would slow completion or compilation;
it is not an additional gate ahead of the work above.

Current scope remains `std` with native host dependencies. The automatic warm
CLI path, including the authored public library route, still uses a Unix socket.
The library artifact tools also require the pinned 208,283,952-byte LLVM shared
library, about 208 MB. This remains a package-size and portability limitation;
the selective encoder's `no_std` boundary does not remove it. A future adapter
must preserve the same retained session, semantic environment, invalidation,
failure and ownership contracts, with actual no-socket operation and matched timing
evidence. A
protocol-only `no_std` crate or fresh per-request compilation does not complete
that migration. Continue correctness repairs and dependency-ready compiler
completion first, including the wider genuine metadata boundary.

Keep these implementation preferences separate from Rust program support.
Programs using `std` or target-specific Unix sockets remain part of the full
Rust contract. Inspect all dependencies before claiming the compiler is free
of `std`; an attribute on a small interface cannot establish that result.

## Required PR explanation

Every compiler change must identify the item it advances; the supported work
that remains with Lust; the justified complementary imports; their shared fact
owners; any remaining duplicate analysis or gap; and exact source-bound
correctness/performance evidence. A requested-engine label or planner trace is
not proof of actual ownership. Reject architecture substitutions even when a
compatibility demonstration passes.
