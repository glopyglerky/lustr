# Work on Lustr

## Sole product goal

Lustr's sole goal is a full rustc replacement built around Lust. Its Lust implementation
must compile every operation that the pinned Lust implementation covers in the
requested context. Import only the upstream implementation and dependencies
needed for the remaining Rust work, and integrate them into that same unchanged
program. The full upstream Rust source stays in blueFe.

Read the [exclusive coverage contract](docs/compiler-set/exclusive-coverage-contract.md)
first. It is the canonical direction, reaffirmed on 19 November 2026. Then read
the [architecture plan](docs/compiler-set/architecture-plan.md),
[design checkpoint](docs/compiler-set/design.md),
[completion checklist](docs/compiler-set/completion-checklist.md), and
[performance policy](docs/compiler-set/practical-performance.md). Earlier plans,
passing compatibility tests and historical release assets cannot change this goal.

## Required boundaries

- Preserve Lust's frontend, checked state, retained worker, precise semantic
  dependency reads, native emission and fast body-edit behavior.
- Distinguish Lust's actual support from the narrower support of an integration
  bridge. A bridge gap is unfinished integration. It is not permission to send
  Lust-owned work to a different compiler or redefine the complement.
- Assign semantic and emission work before generation from real compiler facts.
  No keyword selector, diagnostic-based routing, native-fail/rustc-retry path,
  hidden stock compiler, or complete competing frontend on supported work.
- Maintain the pinned upstream capability rules outside the request path. Apply
  contextual rules during the one normal frontend/retained-state update. Do not
  run a scanner, probe compilation, second parser or second checker to decide
  which implementation to use. Upstream capability probes belong in development
  and CI; an unprobed entry is unknown evidence, not complementary ownership.
- A full Rust driver library is not a minimal complement merely because its
  executable is absent or selected LLVM definitions are disabled. Inspect source
  dependencies and actual semantic work, not only the binary name.
- Every imported source slice needs pinned provenance, licenses, a missing
  capability or required dependency witness, feature/target closure, an actual
  caller and an explicit semantic owner. A file allowlist alone does not prove
  minimality. Exclude Rust implementation that duplicates Lust-owned work.
- Review product Cargo features and dependency closure, including defaults,
  optional, renamed, build and target dependencies. A research-only label does
  not isolate a full compiler if a product feature combination can activate it.
  Delete obsolete retrying launchers and ABI entry points; retain useful bounded
  mechanism qualifiers separately from product construction.
- Reuse upstream implementation for genuine gaps. Do not independently rebuild
  every missing Rust semantic feature as an alternative destination.
- Preserve the unchanged source, Cargo graph, features, target, profiles,
  dependencies, panic behavior, build scripts, procedural macros and real Rust
  artifacts. Do not fabricate rlib/rmeta or weaken semantics to obtain a pass.
- A compiler, transport, emitter, object, link or publication failure fails the
  request. It cannot become a capability decision or successful compiler retry.
  Never retry a program or test that has already executed.
- Derive qualification expectations from pinned source and independent evidence
  before running the candidate. Warm and cold rejection of valid Rust cannot
  create a passing "unsupported" exception. Preserve upstream tests, but add
  independent result checks where their verdicts do not establish this contract.
  Record excluded histories as unqualified scope, with source-bound reasons.

## Continue implementation across checkpoints

The user's standing instruction is to complete this compiler, not to stop after
each proof, passing test, commit or push. Use the execution queue in the
[completion checklist](docs/compiler-set/completion-checklist.md#execution-queue)
as the only work register. The numbered completion conditions below that queue
remain authoritative; a small work package is a step toward them.

1. Read the live integration head and the queue's resume record. Inspect relevant
   local work and active commands before starting another writer or build.
   Preserve dirty work, existing sessions and failed evidence. Use an isolated
   worktree when the current checkout has another owner.
2. Choose the highest-priority dependency-ready package. Continue an owned
   unfinished package first. A proven regression in affected behavior takes
   priority over widening that behavior. Claim a concrete slice with its base
   SHA, owned paths, acceptance operation and next action. Do not wait for the
   user to choose routine implementation details.
3. Implement through the public compiler path. A bounded research qualifier may
   answer a specific blocking question, but its successful result immediately
   feeds integration into the existing pipeline. Do not substitute more demos,
   plan rewrites, repeated source audits or evaluator controls for integration.
4. Run focused checks for the changed behavior and its actual dependency risks.
   Preserve independent valid/invalid expectations and original failures. Run
   the relevant broader gate when its dependency surface changed or the gate
   is required. Do not repeatedly rerun an unchanged large campaign while other
   ready implementation work waits.
5. Review the diff, update source/import declarations where required, commit
   source-bound evidence, and publish to the live integration branch with a
   fresh expected-head check. Keep failed, pending and passing checks distinct.
   Reconcile a concurrent advance without force-pushing over it.
6. Update the same queue and resume record, then immediately select the next
   ready slice. A progress report is not a request for permission to continue.
   Pending hosted checks do not prevent independent implementation. A failed
   check creates a repair task; preserve its log and diagnose the real failure.

The Compiler set workflow cancels a preceding run on the same PR when a new push
arrives. Batch related pushes or implement the next slice locally while the
needed qualification runs. Do not keep cancelling the candidate you intend to
qualify. If a new source deliberately supersedes it, record the cancellation and
qualify the new source rather than transferring the old result.

Use one integration coordinator and up to three independent workers where useful.
Assign non-overlapping source ownership; shared semantic interfaces and final
publication have one owner. A worker's completion returns work to the coordinator,
which reviews, integrates and assigns the next ready slice. An idle or blocked
worker takes independent ready work. Do not let every worker edit the same driver,
protocol, queue or source manifest concurrently.

Keep progress messages concise: what changed, what evidence establishes, and what
is being implemented next. Do not ask for another "continue" after success, an
ordinary failure, uncertainty that can be researched, or context compaction.
Checkpoint before a runtime limit with the exact worktree/head, unfinished diff,
command/session identities, original logs, last executed result, next command and
blocked dependencies. Store private machine details in the existing private
runtime checkpoint, never in public Git files. Re-read that checkpoint on resume;
do not start a second copy of a command whose outcome is still unknown.

An unattended run may pause only for missing required authority or access, an
out-of-scope irreversible action, all ready work being blocked by documented
external dependencies, a user stop, or an actual runtime/resource limit. Finish
independent authorized work first. Report the exact blocker and smallest action
needed; a hard compiler problem is work to investigate, not an approval request.
No instruction file can keep a terminated runtime executing: use the authorized
existing continuation mechanism for subsequent sessions and report its actual
state. Never claim background execution from a plan or dispatch acknowledgment.

PR integration and product release are separate decisions. A bounded corrective
PR can be reviewed under its applicable checks while the full compiler is still
unfinished. Merge only with existing authorization and protections; a merge is
not task completion. Continue from the verified merged source. Package release
still requires the full contract and measured overall speedup, plus publication
authorization. Do not invent a requirement that PR #33 remain open until all
Rust work is finished.

## Establish source and evidence before continuing

Treat governing instructions and implementation source as separate identities.
Documentation merged into `next` does not mean the compiler correction has been
merged there. At this audit baseline, `next` still contains the earlier product
implementation. While PR #33 remains unmerged, it is the sole implementation
lane: resolve its live head, inspect the checkout and preserve unique local work.
Do not use the pre-correction default, a closed predecessor PR, an old release
tag or a cached worktree merely because its documentation states the right goal.
After PR #33 merges, verify that the current integration source contains that
correction and continue from its verified live source and evidence. Do not keep
resuming a closed PR head or reopen retired routes after their correction lands.

The documentation audit starts from published PR #33 source
`ea26fe16636d390763257a19c19df3db992cdbcb`. It is a review baseline, not a permanent
head pin. The [active correction record](docs/compiler-set/active-correction.md)
tracks that lane and its eventual integration. CI checks supply source-bound evidence;
this document does not assert that branch protection or server-side release
controls enforce them.

The pre-correction default implementation was `next` at
`7cea76c9d3adc587b54ccff168fe07efafd40309`. The audited predecessor is
[PR #32](https://github.com/glopyglerky/lustr/pull/32) at
`cb0721d49be95ec3f34a113c2354bc9d2a38bbf3`. That snapshot retains prohibited
retry paths and whole-consumer Rust analysis. It is evidence and a source of
reusable fixes, not the completed architecture.

The sole active implementation correction is
[PR #33](https://github.com/glopyglerky/lustr/pull/33), targeting `next`. Its
[cleanup record](docs/compiler-set/exclusive-coverage-cleanup.md) states the
removed routes, actual verification and unfinished coverage. The
[active correction record](docs/compiler-set/active-correction.md) binds the
published source and preserved PR history. The governing contract landed through
[PR #31](https://github.com/glopyglerky/lustr/pull/31), and
[PR #34](https://github.com/glopyglerky/lustr/pull/34) placed this single lane in
the default-branch instructions. Refresh live heads before work. Do not resume
a superseded PR as a competing architecture or treat documentation, route
removal or bounded tests as full compiler qualification.

Use the [retained-work disposition](docs/compiler-set/retained-work-disposition.md)
to preserve useful source and failed evidence while removing conflicting active
routes. Keep one integration head. Review unique changes and ownership before
cleanup; do not destroy other worktrees or rewrite shared history. Work on
branches and PRs. Documentation publication does not authorize a package release
or a direct product-code merge into the default branch.

Keep public content generic. Do not add personal infrastructure, credentials,
private coordination or unrelated project composition. Preserve original licenses
and the user's other repositories.

## Documentation and historical scope

The [documentation disposition](docs/compiler-set/documentation-disposition.json)
accounts for every tracked Markdown file at the audit baseline. Active contracts
and guides follow the sole goal above. Source-bound checkpoints describe only
their named sources and executions; words such as "current" or "complete" inside
a historical record retain that original scope. They are not new instructions
or evidence for a later source.

Pinned donor documentation under `compiler/native`, upstream runtime notices,
frozen evidence and old design snapshots are reference material. Preserve their
bytes and licenses. Their compiler routing, tests or build recipes cannot
override this repository's active contract. Reproduction of a retired architecture
is a separately scoped research action, never a product implementation route.

Historical preview releases remain evidence. Do not recommend or install them
as the current Lustr replacement, regenerate their full-compiler/coordinator
packages, or revive a retired command from an old README. A new active guide
must identify its actual entry point, ownership and evidence scope. Update the
existing audit and checklist instead of opening a second plan or status register.

## Source and correctness discipline

Keep the pristine Lust reference in `upstream/lust.pin` and declared Rust identity
in `upstream/bluefe-core.pin`. Follow the [source policy](docs/compiler-set/upstream-lust.md).
Select Rust donor source by Lustr's recorded pin, not blueFe's moving `main` or
upstream Rust's latest head. An upstream refresh is research until separately
admitted; it cannot silently change Lustr's SDK, metadata format or semantics.
An explicit integration patch requires source identities, a necessity case,
applicable correctness evidence and performance characterization. A necessary
correctness repair does not itself need a speedup. Do not hide changes in an
overlay, disable a source guard or alter a frozen historical comparison.

Preserve authoritative definitions, instances, types, macro hygiene, ABI/layout,
constants, ownership, drops, unwinding, metadata and dependency state. Bind
language/library semantics to the declared SDK, independently of Lustr's build
host. Root source equality does not establish SDK equality or runtime correctness.

Cache keys and dependency reads must include all relevant source, macro,
build-script, toolchain, target/profile, feature and dependency observations.
Track negative lookups and semantic policy limits. Unknown inputs require valid
recomputation within the assigned owner. Failed work cannot replace the last
successful retained state or be hidden as a cache miss.

Keep stdout suitable for Cargo JSON and program output. Preserve OS arguments,
failures, signals, configuration precedence, nested Cargo behavior and owned
cleanup. Actual sanitizer, differential, panic/drop and retained-edit evidence
must remain visible. Re-run only changed behavior and concrete remaining risks.

## Portability preference and priority

Prefer a compiler core that can use `core` and `alloc` without `std`, with host
services and transport behind explicit adapters. Prefer retained compilation
that does not require Unix-domain sockets. These preferences support the sole
compiler goal; they do not replace it or block completion when migration would
slow delivery or compilation. This priority was clarified on 19 November 2026.

Preserve Lust's retained state and measured speed throughout any migration.
Do not replace warm compilation with fresh processes merely to remove sockets,
move the same Unix dependency into an unnamed helper, or describe a `no_std`
interface as a `no_std` compiler. The current engine still uses `std`, Unix host
facilities and native libraries. A broader migration requires actual integration
and matched performance evidence. Continue dependency-ready compiler completion
first; retain these preferences for later work that can meet those conditions.

This preference concerns the compiler implementation and its required transport.
Full Rust support still includes programs using `std` and target-specific APIs,
including Unix sockets where the declared target provides them. A compiler's
`no_std` attribute does not establish a dependency graph free of `std`, eliminate
host services, or prove a speed gain. Qualify those claims separately.

## Performance and claims

Keep Lust-covered work near matched stock Lust speed. Keep complementary work
near matched stock rustc speed. Measure both separately and preserve slower cases.
The user's release threshold is a correct full replacement that beats matched
stock rustc overall on the declared workload. Do not delay that qualified release
solely because the longer-term stock-Lust matching target remains unfinished.
Do not call an overall gain proof that either subset already meets its target.
Fix workload frequencies before measurement and compare total completed request
time; a geometric mean of unrelated case speedups cannot establish overall gain.

The [performance policy](docs/compiler-set/practical-performance.md)
requires the relative comparison for a new near-stock claim: 10% for supported
warm operations and 5% for ordinary complementary operations, per case with
paired uncertainty. These remain engineering assumptions, not the user's exact
numerical wording, mandatory release thresholds or proof that the candidate
passes. Passing both does not itself prove an overall speedup. The old 0.5 ms alternative
is historical only; small absolute overhead cannot excuse a large relative
slowdown. Keep original historical assessments unchanged.

Use paired total elapsed costs for each near-stock claim as well as the overall
workload. A geometric mean can hide an expensive request even within one case.
Keep means, maxima and distributions alongside empirical p95; a percentile that
omits a rare slow request does not establish a tail bound.

Assess incremental boundary overhead while integrating each component. Do not
defer speed work until the final package or force every narrow source/guard
change through an unrelated full benchmark. Required compiler work is performed
once; dispatch, conversions, initialization and transport are included in the
measured public request.

Preserve cold, no-op, body/signature/type/trait/macro/dependency edits, rejection,
linking, tails, memory, generated-program runtime and compile-plus-test results.
A complete replacement claim requires actual exclusive ownership, the justified
selective complement, correct mixed artifacts, exact package qualification and
matched overall speedup. Claim near-stock subset speed only when separately
measured. No current package is qualified as that full replacement.
