# Mixed compiler development candidate

> Historical implementation and measurements. The whole-program mixed launcher,
> fallback routes and package constructors described below are removed by the
> [exclusive coverage cleanup](exclusive-coverage-cleanup.md). Commands below
> apply to their recorded source snapshots, not the current product. Useful ABI,
> metadata and retained-state components remain subject to the
> [canonical contract](exclusive-coverage-contract.md).

The [governing plan](architecture-plan.md) requires automatic native and
complementary work inside one Lustr compiler, with Lust's fast frontend and
retained path preserved. This document records the narrower implementation
that exists today. Its scalar bridge and whole-crate Rust analysis are not
the completed boundary; whole-program native-failure retry is not the target.

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

The [recorded source-built run at 5a3f493](completion-qualification.md#preceding-recorded-candidate-5a3f493)
passed its mixed correctness, full entry/runtime, original v2 profiling and
first schema-2 timing gates. This does not change the bounded acceleration
scope or qualify the separate full native gate and macOS execution. The
[performance index](../../performance/README.md) preserves seven fixed campaigns
and two unchanged coordinator campaigns, including regressions and original
failed evidence.

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
must resolve to local bindings or primitive types. Free calls, unrepresented
methods, associated paths, aliases used within the body and nested definitions require further semantic
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

The warm certificate accepts a decimal-literal change in a direct
`parameter ^ literal` or primitive wrapping-method scalar body, preserving its interface and source
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
Primitive wrapping/rotation methods use Rust's actual selected inherent
definition, the real `core` crate identity and the primitive receiver type.
Methods supplied by user traits stay with the complete compiler. Unchanged
method bodies retain their checked bindings only while the full semantic
context matches; edited bodies run the original Rust queries and binding check.

Ordinary `lustr check` Cargo requests can retain this checked context for both
binary and library targets, including the ordinary development debug profile.
The first request uses complete Rust analysis and checks selected nongeneric
scalar bodies with the original Lust checker. A certified subsequent body edit
runs Rust's body queries and Lust's retained check, leaving unchanged analysis
reusable. Lust emits no C or object for that check. Signature, dependency, macro
or diagnostic changes still require complete analysis.

Every check continues through Rust's original metadata encoder and dep-info
writer. The current source determines the real crate hash and rmeta; an old
metadata file is never relabeled. The retained path preserves the pinned Rust
analysis phase's definition-table transition before entering its encoder.
Its query results remain available lazily. The check certificate is published
only after the complete invocation and metadata generation succeed without
diagnostics. Compiler arenas and local definition IDs are not serialized.

The native host declarations use libc's actual variadic `fcntl` interface.
Fixed three-argument declarations conflicted with standard-library imports
when the compiler itself was built using the pinned Cranelift complement.
This is a host ABI correction; it does not change the Rust application language
or substitute a compiler backend.
The source-built native revision `c32b145b439c21fc` passed the mixed-project
and complete-entry contracts, 380 native unit tests, independent Rust-caller
object execution and both ordinary native unwind/drop fixtures. Qualification
of the pinned Cranelift build in CI remains separate from these LLVM SDK builds.

## Build and qualify

For a fresh local build, compile `tools/build-mixed-toolchain.rs` with Rust and
run it with the repository, matching upstream Rust SDK, built TinyCC directory
and a new output directory outside the source checkout. It checks the declared
native source identities, builds Lust, the lustr entry, Cargo integration and
mixed driver and native C ABI library, and assembles `toolchain/bin/lustr`.
The supplied TinyCC static archive must support shared linking.
`tools/build-tinycc-pic.rs` builds the pinned upstream source into a new
directory with its normal optimization and position-independent code, then
checks the complete archive through an actual shared link. Its arguments are
the TinyCC source checkout and new output directory; use the resulting
`install` directory as the toolchain constructor's TinyCC input.
Dependencies must already be
available for locked offline Cargo builds. No historical lustr package is
required. The `source-native-runtime` feature binds the entry to Cargo's actual
native build receipt; the complete Rust SDK keeps its existing exact pin.
The ordinary feature set retains the older package's binary checks.
`--current-sdk` can replace the SDK directory when `rustc` already selects the
matching pinned toolchain. The constructor checks its source identity before
building native components. CI builds this complete composition and runs the
real mixed-project qualifier through its lustr entry.

The assembled toolchain owns its expanded standard-library source copy. Its
dependency and support links retain the expected package layout, and the
expanded compiler component is a hard link to the installed native core.
After recording the native library identity, the constructor seals the
complete source inventory and checks it through the same validator used by
the runtime. Editing the installed copy cannot change the source checkout;
changes to any sealed input invalidate retained source-library state.

The constructor accepts a final `--release` option to build every owned
component with its manifest's release profile. It selects the matching release
artifacts and native build receipt; the imported Rust SDK remains unchanged.
The original four-argument command still uses development builds, and an
explicit final `--debug` selects the same behavior. For example:

```sh
rustc --edition=2024 tools/build-mixed-toolchain.rs -o /absolute/build-mixed-toolchain
/absolute/build-mixed-toolchain /absolute/lustr /absolute/rust-sdk \
  /absolute/tinycc/install /absolute/new-output --release
```

`BUILD-PROFILE.txt` records the selection before compilation starts. Each build
command and its output/status remain in `logs`, and `COMPONENTS.txt` binds the
resulting compiler binaries and profile. Optimized builds remain development
candidates until correctness, independent review and performance qualify them.
Selecting a release profile does not establish a speed result.

`lustr test FILE.rs` delegates to the installed Lust test command, preserving
its native test harness, JSON output and exit status. This command has Lust's
supported-language scope; a failed or executed test is never retried through
the full compiler. `lustr test` without a source-file argument retains Cargo's
complete Rust project tests. The runtime contract checks successful native
tests and proves that a failed test's observable side effect occurs once.

The source-built CI job uses `--release` and retains its exact source revision
and tree, component identities, native build receipt, build commands, and mixed
execution/check outputs even if a later step fails. Its artifact is named
`source-built-mixed-qualification-<attempt>`. The retained workload sources,
traces and statuses are evidence for those executed cases; they do not qualify
unmeasured projects or general mixed coverage.

### Paired integration measurements

`crates/lustr-bench` supplies a separate `lustr-mixed-bench` executable for the
current composition. Build it with the pinned SDK and run:

```sh
cargo build --manifest-path crates/lustr-bench/Cargo.toml --release --locked \
  --bin lustr-mixed-bench --target-dir /absolute/benchmark-build
/absolute/benchmark-build/release/lustr-mixed-bench \
  /absolute/pristine-lust-source /absolute/pristine-lust-build/release/lust \
  /absolute/source-built-output /absolute/new-measurement-output 12
```

The pristine checkout must be the exact pinned Lust commit and tree with no
tracked modifications. Build its `lust` and `lust-core` binaries with the same
SDK, TinyCC and release profile as the integrated native compiler. Keep that
build command and log beside the report. The runner verifies the source pin,
the matching Rust SDK and the candidate's release-profile declaration, then
records hashes of the actual compiler binaries and component records. Binary
hashes alone do not prove a source-to-binary relationship.

The current schema-2 native comparison has four lanes: pristine Lust, integrated
Lust, the Lustr entry and the matching standalone rustc. Historical schema-1
campaigns retain their original three native lanes. The runner preserves one
source across all lanes and compares native cold
compilation, repeated builds, body/signature edits, checks, repeated checks,
check body edits, rejection and actual compile-plus-test commands. The Cargo
comparison uses the same pinned complete Rust implementation and Lustr on one
unchanged dependency graph containing a procedural macro, async code and a
`no_std`/`alloc` library. It measures the corresponding nine Cargo modes.

Schema 2 adds rustc→entry to the three existing contrasts, producing 648 timed
commands and 36 summaries at twelve pairs. The [direct-reference contract](native-rustc-comparison.md)
specifies matching semantic flags, exact executable/child-command bindings,
reference correctness gates and version-aware replay. Standalone rustc is
explicitly nonincremental: its no-op starts a compiler process and is not
Cargo Fresh. Its test operation compiles and actually executes the test within
one outer wall-time measurement. Existing historical CSV/table bytes remain
unchanged. The first complete schema-2 campaign is now
[recorded at 5a3f493](../../performance/2026-11-17-5a3f493.md), actual checkout
`b3393a4a2907228516546cd34f1a5c60aaf6aaaa`. Its native cold-build and rejection
intervals favor rustc; seven other native-reference intervals favor the
packaged entry on this scalar corpus. The mixed Cargo table still records
slower body and signature edits. These workloads and comparisons remain separate.

Each of the twelve paired observations starts with independent compiler,
mixed-state, Cargo-target, intermediate and XDG runtime-cache directories.
Native cold samples disable the native daemon and disk compiler cache; Cargo
cold samples include the normal mixed daemon startup. OS page caches remain
untouched. Warm Lust measurements retain the original daemons and same-lane state;
the standalone rustc lane has no persistent compiler daemon or incremental cache.
Separate untimed probes require observed native frontend reuse and the mixed
driver's retained build/check routes. Measured commands omit tracing and
statistics instrumentation. The runner stops only its private sockets and
observes actual exits of its adopted daemon children.

`raw.jsonl` records commands, source hashes before and after, exact output
hashes, statuses and pair identities. `report.json` contains separate paired
ratios and bootstrap intervals for every mode using the existing statistics
and position-balanced ordering implementation. Ratios are baseline time divided
by candidate time. A ratio below one is slower. Missing pairs, wrong results,
unexpected success on invalid input, absent reuse or failed daemon cleanup
fail the run. A completed run reports timing regressions and inconclusive
comparisons without granting performance or release admission.

The source-built CI job runs this campaign on its optimized candidate after
the regression, mixed-library and full entry-contract checks. It retains raw
reports, failed output, fixture sources and the matched pristine build log.
This corpus does not establish full-workload performance or universal speed
preservation, and it does not substitute for independent integration review.

This local composition references the selected source libraries, SDK and TinyCC
installation. It is not a standalone release archive or a change to the host's
default compiler. Retain its toolchain and logs, then remove its disposable
`work` directory after qualification.

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

Compile `tools/native-mixed-qualification.rs` beside the selected `lust-core`;
the original wire protocol also binds the core's actual file identity.
The qualifier tests cold and retained native object
generation with an independent Rust caller. `tools/mixed-compiler-qualification.rs`
tests the real Cargo graph, proc macro, async caller, real `core`/`alloc`
library, native edit, invalid Rust/recovery, signature change and dependency/
macro invalidation. It retains actual compiler and application bytes before
execution, per-case traces, statuses and output. The existing complete compiler
contract additionally covers native/full entry behavior, real metadata,
no_std wasm output, build scripts, doctests and panic/drop recovery.
`tools/mixed-check-qualification.rs` exercises ordinary debug-profile Cargo
checks of a real proc-macro graph, a no_std core/alloc dependency, retained
application and library body edits, changed metadata consumed by the matching
original compiler, type/signature/macro rejection and recovery, and warning/
deny preservation. Both this check journey and the mixed execution regression
passed with the same mixed driver on Linux x86-64.

These bounded checks passed on Linux x86-64. The public fixed-fixture
[qualification ZIPs](../../performance/README.md#historical-completed-fixed-corpus-campaigns)
retain original commands, outputs, source/profile identities and timing data in
Git. The separate actual coordinator campaigns preserve private originals in
authenticated encrypted archives and expose reviewed public projections. They
measure normal dev/test debug=2, outside native-object eligibility; timestamp
rebuilds do not measure body or signature changes.

General performance beyond these fixed corpora and one actual coordinator
package remains unqualified, as does general native acceleration of mixed
libraries, traits, aggregates, references and generic bodies. Complete Rust
semantics outside the eligible boundary remain supplied by the imported
implementation. The hosted compiler uses `std`, TinyCC/C, LLVM and the host
loader; its no_std output checks do not make that compiler runtime no_std or
establish whole-stack memory safety.
