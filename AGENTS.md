# Work on Lustr

## Required architecture

Lustr is one integrated compiler. Lust handles everything it supports; imported,
version-pinned upstream Rust implementation supplies the complementary support
inside the same compilation. Both participate in an unchanged mixed program.
Preserve Lust's frontend, checked state, daemon and fast incremental path.

Read these documents before planning or changing compiler work:

1. [Architecture plan](docs/compiler-set/architecture-plan.md): the governing
   implementation contract and the meaning of the complementary subset.
2. [Completion checklist](docs/compiler-set/completion-checklist.md): remaining
   work, dependencies and observable completion criteria.
3. [Native compiler contract](docs/compiler-set/native-contract.md) and
   [design checkpoint](docs/compiler-set/design.md): required behavior and the
   distinction between default-branch code and reviewed candidate code.

The user's latest instructions govern. These documents supersede the older plan
to independently implement all missing Rust behavior in native Lust. Imported
Rust services and their necessary source/dependency closure are allowed and
expected. A complete upstream source checkout is not evidence that a complete
extra compiler must execute for every request.

Do not replace this architecture with a coordinator, source-keyword selector,
whole-program native attempt followed by a Rust retry, or a full Rust frontend
that always runs before the supported Lust path. Moving those operations into
one executable does not complete integration. Normal compiler decisions use
resolved capabilities and semantic dependencies before emission. They must not
derive support from an error code, diagnostic text or failed code generation.

The actual Lust-supported subset is larger than the current scalar object
bridge. Do not redefine the subset to fit that bridge. Complete the bridge and
shared semantic ownership. Keep genuine Rust metadata and the useful existing
native worker; neither a new metadata format nor a single-process rewrite is
required by the product contract.

## Establish the source baseline first

The default branch reviewed for PR #31 was `next` at
`7cea76c9d3adc587b54ccff168fe07efafd40309`. It predates the mixed compiler work.
The architecture audit covers [PR #29](https://github.com/glopyglerky/lustr/pull/29)
at `a9fd4c6336b1e39c4f584f7da50a437c5c248a0d`; subsequent work and exact CI
status are recorded in the [qualification checkpoint](docs/compiler-set/qualification-review-2026-11-18.md).
PR #31 did not import those implementations into default.

This reconciliation carries PR #31's governing contract onto implementation
candidate `af0a281f2d1fae04e8ebfd5aa2c4872b20245c04`. Its source-bound progress is
recorded in the
[design continuation](docs/compiler-set/design.md#candidate-continuation--19-november-2026)
and [completion checklist](docs/compiler-set/completion-checklist.md).
Earlier default, PR #29 and PR #30 results retain their original source scope.

Before continuing implementation, inspect current PR heads, ancestry, changed
files and source-bound evidence. Select and record one integration head. Preserve
unique repairs from parallel branches and identify equivalent fixes; do not
reimplement a known repair or transplant obsolete native pins. The completion
checklist records known omissions and the callback continuation. Do not treat a
later timestamp, PR title or passing predecessor as proof of current behavior.

Keep changes reviewable on branches and PRs. Do not merge code into the default
branch, discard other worktrees, rewrite shared history or publish a package as
an incidental part of review. Preserve source history, upstream licensing and
the user's other repositories.

## Upstream preservation and correctness

Maintain a pristine Lust reference at `upstream/lust.pin` and the exact imported
Rust identity at `upstream/bluefe-core.pin`. Follow the
[upstream preservation policy](docs/compiler-set/upstream-lust.md). Necessary
integration or correctness patches require explicit source diffs, identities,
regressions and applicable source-guard support. Do not conceal edits in a
renamed copy, generated overlay or build-time patch, or disable a failing guard.
Correctness repairs do not have to make Lust faster to be admissible.

- Preserve the original program, Cargo flags, profiles, target, features,
  dependencies, panic behavior, build scripts and procedural macros.
- Keep authoritative definition/type/instance identities, ABI/layout, ownership,
  drop/unwind behavior and semantic dependency reads consistent across the
  boundary. Each definition or instance is emitted once by its assigned
  implementation. Use imported Rust's genuine metadata machinery.
- Distinguish complementary requirements, invalid Rust, and implementation
  failures. Failed emission, transport, object generation or linking must fail
  the request and preserve the prior successful retained state. No compiler
  retry can turn those failures into acceptance; never retry an executed program.
- Compare with the declared matching Rust version. A passing historical native
  oracle is not proof of a newer SDK's semantics. Keep frozen comparison sources,
  failing samples and capability guards intact.
- Bind language and standard-library semantics to the declared Rust source, independently of the compiler build host. Preserve the [shared Unicode repair and exhaustive gate](docs/compiler-set/unicode-semantics.md): constant evaluation and runtime must use matching pinned data. A changed version string cannot repair a semantic mismatch. Keep failed frozen comparisons intact, and qualify the rest of the native standard library against the same declared Rust source.
- Preserve actual sanitizer, differential, panic/drop, daemon, invalidation and
  package gates. Test changed behavior and concrete remaining risks; do not
  repeatedly run unrelated suites because a stale document says pending.
- Keep stdout usable for Cargo JSON and program output. Pass OS arguments without
  a shell or lossy decoding; preserve failures, signals and owned cleanup.
- Scope backend configuration to the Cargo invocation. Retain configuration
  precedence and nested-Cargo checks; environment overrides must not leak into
  programs. Qualify the actual backend and flags, including unwind support.
- Bind caches to compiler/runtime identities, all semantic inputs, target,
  profile/features, build-script observations and transitive dependencies. An
  untracked dependency requires recomputation, not optimistic reuse.

Preserve the [first native admission contract](docs/compiler-set/native-admission.md)
and [owned checked-request continuation](docs/compiler-set/checked-request-ownership.md).
The owning API is an implemented component, not the general imported-service
boundary. A native rejection still does not prove invalid Rust or authorize
whole-program retry.

Keep `crates/lustr`, the independent compatibility coordinator, buildable without
TinyCC, nightly Rust or registry dependencies. Its successful build is not
integrated-compiler qualification. The upstream `DESIGN.md`, `docs/ARCH.md`,
`docs/PLAN.md` and archived design snapshots remain historical instructions.

## Performance and release claims

The goal is close-to-Lust speed on supported work, useful end-to-end gains over
rustc where Lust participates, and ordinary complementary Rust near stock rustc.
Exact Lust parity and large gains outside its subset are not requirements.
Correctness takes priority over shaving a few tenths of a millisecond.

Retain the existing practical review budgets documented in the
[performance checkpoint](docs/compiler-set/performance-review-2026-11-18.md):
supported warm check/build may add 10% or 0.5 ms, whichever is larger; ordinary
complementary work has a 5% or 0.5 ms review budget. These are engineering
criteria, not a promise that every program is faster. Report paired uncertainty
and the scope of each conclusion. Do not silently relax the criteria after
seeing results or demand new speed gains from every necessary correctness patch.

Measure cold, warm/no-op, body/signature edits, rejection, linking and
compile-plus-test separately. Retain raw samples, exact source/binary identities,
profiles and slower cases. Instrumented traces, reporting replays, historical
timings and new performance qualification are different evidence. Do not assign
a shared 0.3 ms difference to a cause without a controlled observation.

A complete replacement claim requires the integrated architecture, correctness
for the declared Rust version/targets, actual package qualification and the
documented practical performance results. No finite suite proves all possible
Rust programs correct. State the tested coverage and unresolved gaps honestly.
Memory-safety migration remains separate later work.
