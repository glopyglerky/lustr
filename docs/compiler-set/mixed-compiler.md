# Mixed compiler development candidate

lustr now has an executable mixed boundary, in addition to its existing
whole-target Lust path. A real Cargo project containing a procedural macro,
async Rust and a `no_std` library using `alloc` builds and executes through
`lustr build`. Rust supplies completeness, metadata and ordinary code; Lust
supplies a checked scalar definition in the same executable. Application source
and the Cargo graph remain unchanged.

This is a bounded development candidate. It does not establish general mixed
Rust interoperability, broad retained body reuse, or equivalent complete-work
speed. The complete imported compiler remains available for every request
outside the accelerated boundary. No independent full-language implementation
is required first.

## Imported implementation and original fast path

The complete implementation is the upstream Rust compiler at
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, including its matching LLVM backend,
driver, standard libraries and metadata. The `full-core` feature links that
compiler's existing `rustc_driver` and `rustc_codegen_llvm`; it does not fork
the backend or substitute an inspection report for executable compiler data.
Build it with that exact toolchain's `rustc-dev` and LLVM development components.
The default semantic inspector retains its separate older pin.

Lust's pristine reference remains recorded in `upstream/lust.pin`. The native
object boundary is implemented in `crates/lustr-native-core/src/mixed_unit.rs`
through the declared native integration proposal. Its ordinary parser,
resolver, checker, borrow checker, lowering, C generation and retained daemon
produce the object. The original whole-target path keeps its existing compiler
and daemon. Mixed requests use a distinct daemon namespace and a matching
client/core identity.

Relevant upstream mechanisms are Rust's
[`rustc_driver`](https://github.com/rust-lang/rust/tree/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_driver_impl),
[`CodegenBackend`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_codegen_ssa/src/traits/backend.rs),
[`CompiledModules` link serialization](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_codegen_ssa/src/lib.rs),
and the pinned
[Lust compiler](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9).
These are version-bound compiler interfaces, with their original licenses and
provenance. They are not a stable third-party plugin protocol.

## Actual boundary

For a cold mixed build, Rust performs its complete analysis. A nongeneric,
unattributed, root-level scalar function is considered only after checking its
actual Rust instance, mangled symbol, parameter/return types and effective
Rust/C argument-passing modes.
The actual HIR definition must match the source body and root item. Its paths
must resolve to local bindings or primitive types. Calls, associated items,
aliases used within the body and nested definitions require further semantic
context and stay with Rust. This prevents shadowed modules, primitive aliases
and same-named nested functions from being reinterpreted in the isolated unit.
The first target is Linux x86-64, unoptimized code with debug information
disabled. Other targets, requested code-generation
policies and unrepresented functions use the complete implementation.

The function's original body and canonical checked scalar signature enter
Lust's existing pipeline as an internal unit. Only original native IR proved
to contain total scalar operations may omit native unwind machinery. No
external runtime relocation is accepted. The generated function is renamed
to Rust's actual symbol, omitted from the original LLVM backend's codegen
unit, and linked as a genuine object by that same backend. Rust still emits
all remaining definitions and its real metadata. No rlib, rmeta, dep-info,
layout, drop behavior or aggregate ABI is fabricated.

After success, the driver retains Rust's actual version-bound link recipe and
the checked Rust objects. On another invocation it refreshes Rust expansion,
including procedural macros, and compares actual stable HIR owner/attribute
hashes and dependency crate identities.
When Rust has not precomputed HIR hashes, the bridge uses the same original
stable-hash functions as the pinned `hash_owner_nodes` implementation. It does
not enable a different Cargo incremental setting to obtain them.
Expansion is refreshed because procedural macros can observe inputs they do
not report. This does not claim a
retained Rust frontend or zero Rust work.

The first warm certificate accepts a decimal-literal change in a direct
`parameter ^ literal` scalar body, preserving its interface and source
positions. Existing Rust body queries check the new body; the original Lust
session generates the changed function. Unchanged native definitions keep
their generated objects and do not repeat their body queries or compilation.
The complete Rust analysis query and LLVM code generation are skipped only
after this certificate succeeds. Rust's
original dep-info writer still runs, followed by its original linker using
the retained Rust objects. Other edits continue through full analysis in the
same invocation. Requests are locked per cache identity, including arguments,
working directory, environment and compiler binary identities.

This small certificate is the first tested mixed edit, not a substitute for
general dependency tracking. Expanding it requires preserving diagnostics,
source locations, macro behavior, types, generics and global obligations.
Only successful requests without Rust diagnostics establish this certificate.
The bridge observes and forwards Rust's existing diagnostic-tracking callback;
it keeps the original emitter and incremental diagnostic side effects. A
request producing diagnostics uses complete analysis, preserving warnings
until version-bound diagnostic replay is connected to the retained path.

## Build and qualify

Build the native compiler with the existing TinyCC setup. Take its actual
`LUST_BUILD_HASH` from Cargo's build-script output, then build the matching
driver with the pinned complete Rust toolchain:

```text
LUSTR_NATIVE_BUILD_HASH=<actual-native-build-hash> cargo build \
  --manifest-path crates/lustr-semantic/Cargo.toml \
  --features full-core --bin lustr-mixed --locked --offline
```

Compile `tools/assemble-mixed-toolchain.rs` with Rust. Its arguments are the
existing pinned complete lustr package, built mixed driver, matching native
core and a new destination directory. It composes an explicit local development
toolchain using symlinks; keep the referenced baseline installed. It does not
modify that baseline or claim to create a standalone release archive. The
destination's `bin/lustr` provides normal Cargo commands automatically.

`tools/native-mixed-qualification.rs` tests cold and retained native object
generation with an independent Rust caller. `tools/mixed-compiler-qualification.rs`
tests the real Cargo graph, proc macro, async caller, real `core`/`alloc`
library, native edit, invalid Rust/recovery, signature change and dependency/
macro invalidation. It retains actual compiler and application bytes before
execution, per-case traces, statuses and output. The existing complete compiler
contract additionally covers native/full entry behavior, real metadata,
no_std wasm output, build scripts, doctests and panic/drop recovery.

These bounded checks passed on Linux x86-64. Runtime artifacts and raw logs
are retained separately from the public source. Broad full-project performance
and general mixed libraries, traits, aggregates, references and generic bodies
remain unqualified. The hosted compiler uses `std`, TinyCC/C, LLVM and the host
loader; its no_std output checks do not make that compiler runtime no_std or
establish whole-stack memory safety.
