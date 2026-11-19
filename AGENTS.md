# Work on Lustr

## Sole product goal

Lustr is a full rustc replacement built around Lust. Lustr's Lust implementation
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
- A full Rust driver library is not a minimal complement merely because its
  executable is absent or selected LLVM definitions are disabled. Inspect source
  dependencies and actual semantic work, not only the binary name.
- Every imported source slice needs pinned provenance, licenses, a missing
  capability or required dependency witness, feature/target closure, an actual
  caller and an explicit semantic owner. A file allowlist alone does not prove
  minimality. Exclude Rust implementation that duplicates Lust-owned work.
- Reuse upstream implementation for genuine gaps. Do not independently rebuild
  every missing Rust semantic feature as an alternative destination.
- Preserve the unchanged source, Cargo graph, features, target, profiles,
  dependencies, panic behavior, build scripts, procedural macros and real Rust
  artifacts. Do not fabricate rlib/rmeta or weaken semantics to obtain a pass.
- A compiler, transport, emitter, object, link or publication failure fails the
  request. It cannot become a capability decision or successful compiler retry.
  Never retry a program or test that has already executed.

## Establish source and evidence before continuing

The pre-correction default implementation was `next` at
`7cea76c9d3adc587b54ccff168fe07efafd40309`. The audited predecessor is
[PR #32](https://github.com/glopyglerky/lustr/pull/32) at
`cb0721d49be95ec3f34a113c2354bc9d2a38bbf3`. The latter retains prohibited public
retry paths and a mixed path with whole-consumer Rust analysis. It is evidence
and a source of reusable fixes, not the completed architecture.

The sole active implementation correction is
[PR #33](https://github.com/glopyglerky/lustr/pull/33), targeting `next`. Follow
the [active correction record](docs/compiler-set/active-correction.md) for its
exact source, validation, historical PR disposition and remaining work. The
governing contract landed through [PR #31](https://github.com/glopyglerky/lustr/pull/31).
This documentation update records the implementation lane; it does not merge
product code or qualify a package. Refresh live heads before work and do not
resume a superseded PR as a competing architecture.

Use the [retained-work disposition](docs/compiler-set/retained-work-disposition.md)
to preserve useful source and failed evidence while removing conflicting active
routes. Keep one integration head. Review unique changes and ownership before
cleanup; do not destroy other worktrees or rewrite shared history. Work on
branches and PRs. Documentation publication does not authorize a package release
or a direct product-code merge into the default branch.

Keep public content generic. Do not add personal infrastructure, credentials,
private coordination or unrelated project composition. Preserve original licenses
and the user's other repositories.

## Source and correctness discipline

Keep the pristine Lust reference in `upstream/lust.pin` and declared Rust identity
in `upstream/bluefe-core.pin`. Follow the [source policy](docs/compiler-set/upstream-lust.md).
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

## Performance and claims

Keep Lust-covered work near matched stock Lust speed. Keep complementary work
near matched stock rustc speed. These are separate requirements; a gain on one
cannot conceal a loss on the other. The [performance policy](docs/compiler-set/practical-performance.md)
retains existing engineering review budgets: 10% or 0.5 ms for supported warm
operations, and 5% or 0.5 ms for ordinary complementary operations, per case.
They are assumptions for assessment, not the user's exact numerical wording or
proof that the current candidate passes.

Preserve cold, no-op, body/signature/type/trait/macro/dependency edits, rejection,
linking, tails, memory, generated-program runtime and compile-plus-test results.
A complete replacement claim requires actual exclusive ownership, the justified
selective complement, correct mixed artifacts, exact package qualification and
matched performance. No current package is qualified as that full replacement.
