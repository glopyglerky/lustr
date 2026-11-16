# lustr compiler contract

Current direction: combine Lust's established fast compilation of its supported
subset with complete Rust compilation through lustr.

## Required result

lustr is the project and toolchain name. It is not a requirement to invent an
independent frontend or reconstruct Rust one feature at a time. Importing and
using the upstream rustc implementation is explicitly permitted, with pinned
source/toolchain identities, licenses, notices and provenance. blueFe packages
the combined toolchain.

Preserve Lust's native path, incremental behavior and speed for the code and
requests it supports. Full Rust, including no_std, may use the identified
upstream Rust implementation. There is no requirement that every full-Rust
feature match Lust's subset speed or be reimplemented before delivery.

Use the simplest working combination of the existing implementations. Reuse
prior code and qualification. Investigate an earlier failure before repeating
its approach. A shell wrapper that merely finds an external stock compiler is
not by itself the finished integration: deliver the matched components,
selection rules, runtime/sysroot, Cargo behavior and real user commands.

## Compiler and Cargo boundaries

Use one lustr-facing toolchain and Cargo's real graph, configuration, jobserver
and parallel units. Preserve source, package/target selection, features, target,
profiles, panic strategy, build scripts, proc macros and diagnostics.

Lust has no rustc-compatible rlib/rmeta format. Do not fabricate metadata,
dep-info or ABI compatibility. A unit requiring Rust metadata or unsupported
semantics can use the full Rust implementation. Whole-target Lust compilation
and mixing native objects with Rust crates are distinct contracts.

Choose engines from actual request/artifact contracts and native results, not
source keyword guesses. Unsupported native compilation may proceed through the
full compiler before execution. Compilation errors stay errors when neither
engine accepts. Never retry an already executed application or test through
another compiler; preserve crashes and signals.

The existing full compiler runs through its proper executable startup. A prior
late-loaded compiler-library experiment failed TLS initialization; importing
the implementation does not justify repeating that failed loading mechanism.

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

Report actual engines, platforms, imports and limits. Do not claim independent
Rust semantics or whole-toolchain memory safety. Keep source, failed firsts,
published PR7, rollback, evidence and ownership intact.
