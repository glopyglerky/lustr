# Lustr compiler contract

The [governing architecture plan](architecture-plan.md) records the user's fixed
requirement. It supersedes the former requirement to implement every Rust gap
independently as a native extension. Follow the [design](design.md),
[completion checklist](completion-checklist.md), and dated
[qualification review](qualification-review-2026-11-18.md) when implementing or
claiming delivery.

## Required behavior

Lustr compiles the user's unchanged Rust program as one owned compilation.
Lust handles the work it supports. Imported upstream Rust implementation handles
the complement. Both must compose in the same output with genuine Rust linkage,
metadata, and runtime behavior. blueFe may deliver the resulting toolchain;
packaging does not replace the compiler work.

Preserve Lust's frontend, retained worker, precise dependency reads, and fast
body-edit behavior. Do not make every supported request run a complete Rust
frontend before native admission. The architecture may use internal workers or
daemons while retaining one compilation contract.

Actual Lust support and current bridge coverage are different sets. A bridge
that only represents scalar functions must not redefine Lust's supported subset.
Work that Lust supports but the bridge cannot carry remains an integration gap.
Do not route all of a mixed program to stock rustc and count that as completed
native participation.

## Upstream implementation and source ownership

Importing upstream Rust source, internal compiler services, metadata machinery,
and their necessary dependency graph is allowed. There is no requirement to
reconstruct every missing Rust semantic rule independently. Importing the full
upstream dependency graph is different from executing a complete second
compiler pipeline for every supported request.

Keep the pristine Lust reference and declared Rust source pin identifiable.
Record necessary integration or correctness changes through explicit source
identities, reviewable patches, notices, and the [upstream policy](upstream-lust.md).
Do not conceal divergence in a renamed copy, generated overlay, or unrecorded
build-time patch. This documentation does not modify a pin or admit an unreviewed
source change.

A necessary correctness repair requires a reproducible failure and validation
of the changed behavior. It does not need to make compilation faster to be
legitimate. Measure material performance effects and preserve regressions.
Small overhead within the agreed practical tolerance is acceptable; correctness
must not be weakened to recover a fractional millisecond.

## Capability and failure contract

Use actual compiler representations and resolved semantic facts for internal
work assignment before emission. Keywords, successful parsing, or a native
compiler's nonzero exit are not sufficient evidence of capability.

The internal boundary must distinguish:

- Admitted native work, with retained checked state and explicit semantic,
  runtime, ABI, target, and dependency obligations.
- Work requiring imported Rust capability or shared semantic services. This is
  a continuation of the same compilation, not a certificate that unchecked
  source is valid.
- Invalid Rust, reported through the authoritative checking implementation.
- An implementation, transport, emitter, object, publication, or linker failure.

A failed admitted native emission must fail the request. It cannot silently
become successful LLVM generation or a new whole-program compiler attempt.
Preserve errors, signals, diagnostics, and stdout. Do not use diagnostic text to
select a compiler. Never retry a program or test that has already executed.
Independent correctness oracles may compare results during qualification; they
must not hide the requested compilation's failure.

## Shared program and build contract

Preserve definition and generic-instance identities, macro hygiene, bindings,
types, trait obligations, layouts, calling conventions, constants, symbols,
drops, allocation ownership, unwinding, and dependency state across the boundary.
Use imported upstream services where those facts are missing from Lust.

Bind language and library semantics to the declared Rust source, independently
of the compiler used to build Lustr. Unicode data illustrates the requirement.
Core character operations and the compiler's lexical dependencies have separate
pins; do not infer that all tables must share a version or fix a disagreement
by changing a reported version. The qualification review records known gaps.

Lust's standalone C object is not a Rust rlib or rmeta. Reuse the real Rust
metadata encoder, archive writer, dependency information, and linker with valid
shared facts. Do not fabricate metadata or require independent reconstruction
of Rust's private format. A downstream Rust crate must consume the actual
artifacts through its normal compiler contract.

Preserve the user's source, Cargo graph, feature selection, dependency versions,
target, profile, debug information, overflow checks, panic strategy, build scripts,
procedural macros, compiler arguments, and configuration precedence. A command
adapter may preserve Cargo behavior. A coordinator or language selector cannot
substitute for the integrated compiler.

## Current implementations are not completion

The [plan's snapshot table](architecture-plan.md#default-code-and-audited-candidates-are-different-snapshots)
distinguishes the default branch from later candidates. The default at
`7cea76c9d3adc587b54ccff168fe07efafd40309` contains the native-first compatibility
entry and optional semantic inspection. It does not contain the later scalar
mixed bridge.

The architecture-audited PR29 candidate implements real scalar object
composition and typed pre-emission native admission. Cold mixed requests still
run whole-crate Rust analysis; the standalone retry and general semantic/runtime
ownership gaps remain. PR30 proposes further correctness repairs, with exact
validation tracked separately. Neither a proposed patch nor a passing bounded
gate establishes full architecture completion.

Retain useful compatibility tools and published experiments with their actual
scope. Their forwarding behavior, Cranelift results, rustc build tuning, and
successful full-Rust commands are not substitutes for the required native and
complementary work assignment.

The [19 November candidate continuation](design.md#candidate-continuation--19-november-2026)
adds an owning native preparation interface and further correctness repairs.
Those components retain their scoped evidence; the required shared semantic
continuation, general mixed coverage and public integration remain incomplete.

## Qualification and stopping criteria

Architecture, correctness, platform support, and performance are separate release
conditions. The [checklist](completion-checklist.md) must identify evidence for
all applicable conditions on the exact delivered source and components.

Require unchanged supported, complementary, and mixed programs; valid and invalid
Rust; real Cargo dependencies and metadata consumers; build scripts and
procedural macros; runtime output and exit status; panic/drop behavior; and cold
versus retained equivalence after relevant edits. A success status alone cannot
establish correct code or the implementation that produced it.

Keep large gains on Lust-supported work and accept modest integration overhead
relative to Lust. Keep ordinary full Rust near the matching stock compiler;
large gains there are not required. Measure each relevant operation separately,
with matched settings and complete commands. Preserve raw samples, source
identities, prior strict results, failures, and regressions. Do not average a
slow operation away or substitute an easier profile.

A finite suite does not prove every Rust program correct. Completion requires
the implemented compiler boundary and representative full-project evidence for
the stated scope. Migration toward memory-safe implementation remains separate
future work and must preserve semantics.

The superseded contract remains accessible at its
[immutable pre-plan source](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/native-contract.md).
Its independent-reimplementation and per-change faster-results requirements are
not current instructions.
