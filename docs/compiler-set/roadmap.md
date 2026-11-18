# Lustr delivery roadmap

**Governing direction:** [one integrated Lustr compiler](architecture-plan.md),
using Lust for supported work and imported upstream Rust implementation for the
complement. Coordinator and whole-program retry work cannot replace this plan.
Read the [contract](native-contract.md) and [design](design.md) before routing
changes or release claims. The user has already authorized this architecture.

## Current checkpoint

The candidate preserves useful Lust performance, real Cargo compatibility,
correctness tests and a working but narrow mixed-function bridge. The public
standalone entry still uses native-failure retry; cold mixed work still runs
whole-crate Rust analysis and represents only a limited scalar boundary.
The complete requested architecture is **not implemented or release-qualified**.

Existing speed results, independent correctness repairs and failed attempts
remain evidence for their exact sources. They do not change the destination.
The current performance snapshot is documented in the [README](../../README.md)
and [source-bound evidence](../../performance/2026-11-18-43a7e07/README.md).

## 1. Capability contract and compiler ownership

Inventory actual Lust support from the pinned compiler, including semantic,
runtime, ABI, target and profile constraints. Distinguish Lust's own support
from the smaller set represented by the current mixed bridge. Bind the
inventory to accepted/rejected fixtures and actual compiler interfaces.

Define internal admitted/complementary/invalid/failure outcomes before emission.
Specify ownership and transfer of checked state and dependency reads. The gate
is a usable compiler contract, not a keyword scan, a descriptive capability
report or another whole-program compile probe.

## 2. Shared compilation boundary

Reuse the existing binding, ABI, symbol, object, metadata and retained-state
work. Expose native preparation and emission without discarding/rechecking the
same state. Bring in upstream Rust implementation for missing services and
complementary work. Preserve a direct fast path for fully supported work.

Prove how mixed definitions share identities, layouts, calling conventions,
drops, unwinding and metadata. Do not restart independent feature-by-feature
reconstruction of all Rust. A prepare/emit seam alone is an intermediate result;
it must not be relabeled as the completed integrated compiler.

## 3. Automatic native and complementary compilation

Route work inside Lustr from the actual capability contract. Supported work
uses Lust; complementary work uses the imported implementation in the same
compilation. Replace the public error-driven whole-program retry as the normal
entry architecture. Keep Cargo adapters limited to preserving Cargo behavior.

Require real mixed programs and traces showing which definitions and phases
use each implementation. Native emitter failure must be reported as failure,
not silently converted to a successful second compilation. The current scalar
bridge is a starting point, not a redefinition of the supported subset.

## 4. Retained correctness and broad compatibility

Exercise unchanged supported, complementary and mixed projects, including
libraries, build scripts, proc macros, no_std, generic/trait interactions,
invalid code, panic/drop, dependency changes and genuine metadata consumers.

Check cold and warm equivalence across body, signature, type, trait, macro and
configuration edits. Preserve existing native differential gates and exact
platform scope. Qualify the exact source-built package, not a diagnostic mix of
replaced components. Retain failures and all component/source identities.

## 5. Performance and publication

Preserve Lust's large gains on supported work and near-Lust warm latency, with
small overhead allowed. Complementary Rust should stay close to matched stock
Rust; large gains there are not required. Apply the [practical policy](practical-performance.md)
to separately reported operations and keep regressions visible.

Publish the requested replacement only after architecture, correctness,
platform and performance evidence support that exact package. No successful
coordinator build or speed result waives the integrated-compiler requirement.
Do not chase a fractional millisecond by weakening validation or introducing
another routing architecture.

## Retained work

The [prior roadmap](roadmap-history-before-2026-11-18.md),
[older native research roadmap](native-roadmap-history.md), previous PRs and
performance campaigns remain historical evidence. Their implementation details
and obsolete completion requirements do not override this roadmap.
