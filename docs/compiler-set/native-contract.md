# lustr compiler contract

Required architecture: one Lustr compiler, using Lust for supported work and
imported upstream Rust implementation for its complement. The
[governing plan](architecture-plan.md), reaffirmed 18 November 2026, controls
implementation and release claims. The current candidate does not yet satisfy
the complete integrated boundary.

## Required result

lustr is the full replacement compiler being built; blueFe packages it.
Importing upstream Rust source and internal compiler services is explicitly
permitted, with pinned identities, licenses, notices and provenance. There is
no requirement to reconstruct missing Rust semantics independently.

Preserve Lust's native frontend, incremental behavior and speed for supported
work. The complementary implementation supplies what Lust lacks, including
the necessary shared semantic and artifact services. Native and complementary
work must compose inside the compilation of the same unchanged Rust program.
There is no requirement for large speed gains outside Lust's subset.

Reuse prior code and qualification, and investigate earlier failures before
repeating an approach. A coordinator, language selector or whole-program
compile/fail/retry wrapper does not satisfy this contract, even when the stock
compiler is bundled in the package. Correctness fixes to those transitional
routes do not make them the target architecture.

## Compiler and Cargo boundaries

The required boundary automatically assigns supported and complementary work
from actual compiler capabilities before emission. An eligible native function
and complementary Rust must execute in one unchanged output through real symbols, instance
identities, layouts, FnAbi and drop/unwind contracts. Keep Lust's retained
frontend and body-edit path useful without running full Rust analysis first.
Repeated rustc_driver calls create fresh contexts; a daemon alone does not
provide this reuse. Neither whole-target routing nor the existing narrow
scalar-only bridge is completion. Do not require a second complete Rust frontend
on every supported request merely to determine that Lust can compile it.

Use one lustr-facing toolchain and Cargo's real graph, configuration, jobserver
and parallel units. Preserve source, package/target selection, features, target,
profiles, panic strategy, build scripts, proc macros and diagnostics.

Lust has no rustc-compatible rlib/rmeta format. Do not fabricate metadata,
dep-info or ABI compatibility. A unit requiring Rust metadata or unsupported
semantics can use the full Rust implementation. Whole-target Lust compilation
and mixing native objects with Rust crates are distinct contracts.

Derive the capability boundary from actual request contracts, resolved compiler
representations, semantic requirements and ABI facts, not source keyword guesses.
Unsupported work is an explicit planning result before generation. A failed
native whole-program compilation must not be the normal discovery mechanism for
the complement. Preserve implementation failures, invalid Rust, crashes and
signals. Never retry an already executed application or test through another
compiler.

Imported compiler services must retain their real initialization requirements.
A prior late-loaded compiler-library experiment failed TLS initialization;
importing upstream implementation does not justify repeating that mechanism.

## Source and delivery

Preserve the pristine Lust reference and its [explicit integration policy](upstream-lust.md).
Imports of Rust belong in separately identified source/toolchain components.
The permission to import does not authorize unexplained changes to Lust,
hidden overlays, changes to blueFe's protected pin, or license/history loss.

The semantic-image and native language experiments remain retained research.
Their missing traits, generic bodies or core records are not prerequisites for
full Rust delivery through the imported implementation. Do not resume
feature-by-feature reconstruction as the assumed architecture.

## Qualification

Check unchanged full Rust projects, real Cargo dependencies, build scripts,
proc macros, tests, invalid Rust and panic/drop behavior. Exercise no_std with
the real core/alloc identities and appropriate target/runtime/link evidence.
A host-linked native example does not establish freestanding support.

Measure the integrated Lust path against the same pinned Lust on equivalent
supported work. Keep cold, no-op, edits, rejection and compile-plus-test results
separate. Preserve regressions and exact identities. Upstream's speed results
motivate this path but do not establish a ratio for an unmeasured integration.
Full-Rust behavior and Lust-path performance are separate acceptance checks.

Report actual implementations, platforms, imports, duplicate work and limits.
Acceptance needs evidence of the architecture itself: supported work remains
with Lust, complementary work uses the imported implementation, and mixed
compilation does not discover its partition by failed native generation. Tests
passing through a packaged stock compiler cannot replace that evidence.
Do not claim independent Rust semantics or whole-toolchain memory safety. Keep
source, failed firsts, published PR7, rollback, evidence and ownership intact.
