# Lustr compiler behavior contract

The [exclusive coverage contract](exclusive-coverage-contract.md) is canonical.
Lustr is the full rustc replacement being built. Its Lust implementation owns
all work the pinned Lust supports in the requested context. Only necessary
upstream implementation supplies the complement. Both compose in the unchanged
program. The complete Rust source stays in blueFe.

## Required behavior

Preserve Lust's frontend, checked state, worker, semantic dependency reads and
fast edit path. A supported request must not run a competing complete Rust
frontend to establish native capability. Mixed requests must retain all Lust
participation required by the actual supported set. Scalar bridge limitations
remain integration defects rather than a new definition of Lust coverage.

Use real compiler facts for typed decisions before emission. Keep prepared
state alive through generation. Distinguish prepared native work, complement
required, integration incomplete, invalid Rust and implementation failure.
A failed native emitter, transport, object, linker or publication fails the
request. No stock retry or diagnostic-based routing may hide the failure.

Preserve source, Cargo graph, flags, features, dependencies, edition, SDK,
target, profiles, overflow/debug/panic choices, procedural macros and build
scripts. Never rewrite the program into the subset, fabricate metadata, relax
its requested semantics or retry an executed program/test.

## Semantics and artifacts

Use authoritative definition and instance identities, hygiene, bindings,
types, constraints, constants, layouts, ABI, symbols, ownership, allocation,
drops and unwinding. Shared services compute necessary facts once and expose
their dependencies. A second complete type-checking pass over Lust-owned work
is not a shared representation.

Bind language and library behavior to the declared Rust source independently
of Lustr's build host. Character properties, lexical Unicode and normalization
have distinct source/data dependencies and must preserve the requested version.
Keep the exact failing and repaired examples rather than changing a version
string or an oracle to conceal mismatched behavior.

A native C object is not a Rust rlib/rmeta. Reuse genuine metadata, archive,
dep-info and link machinery through the justified import boundary with valid
shared facts. Native and complementary emitters must agree on symbol/instance
ownership, and each required instance is emitted exactly once. Actual dependent
Rust crates must consume the artifacts through the normal build contract.

## Source and dependency contract

Follow [upstream preservation](upstream-lust.md) and the
[source register](sources.md). Record every admitted import's source, license,
necessity, target/features, dependency closure, adapter, semantic owner and
qualification. Broad SDK/driver research dependencies must be declared and
excluded from production admission. They are not missing from the inventory
merely because they are linked rather than copied into the repository.

Whole Rust vendoring and importing a full driver as the complement violate the
goal. Reimplementing all missing Rust semantics independently is not required.
Necessary correctness/integration changes to the native donor need an explicit
patch and applicable evidence; each patch need not create a speedup.

## Retention and failure recovery

Retain identities and semantic observations across both implementations. Include
source, generated inputs, environment reads, negative lookups, toolchain/runtime,
features, target/profile and transitive dependencies. Recompute unknown or changed
inputs with their assigned implementation. Preserve invalidation correctness
across edit, reversal, restart and failure histories.

Do not publish failed or ambiguous work as successful retained state. Preserve
last successful state and report errors, signals and stdout correctly. Keep
Cargo JSON clean, OS arguments lossless, configuration local to the invocation,
nested builds correct and process cleanup owned.

## Qualification and limits

Use unchanged supported, complementary and mixed programs, real dependencies,
macros, build scripts, invalid Rust, drop/panic/unwind behavior and actual package
entry points. A platform with only stock-Cargo compatibility is not a platform
with the required integrated compiler. Preserve meaningful upstream differential
and retained-edit checks, and test changed behavior and concrete risks.

Assess supported work against matched pristine Lust and complementary work
against matched Rust under the [performance policy](practical-performance.md).
Keep source ownership, correctness, performance and package qualification
separate. Finite tests qualify their observed scope; they do not prove every
possible Rust program. No current artifact is qualified as the full replacement.

Current source conflicts and retained fixes are listed in the
[design checkpoint](design.md) and [disposition record](retained-work-disposition.md).
The [implementation cleanup](https://github.com/glopyglerky/lustr/blob/114227e8b255e8aa49ce50019b7d614d53e39c22/docs/compiler-set/exclusive-coverage-cleanup.md) records removed
routes and verification separately from the unfinished full-compiler work.
