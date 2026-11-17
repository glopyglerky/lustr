# Historical native compiler roadmap

This document preserves the superseded native-only plan. Its implementation
statuses, architectural restrictions and qualification claims describe that
plan, not the current product. Follow the [current delivery roadmap](roadmap.md)
and [compiler contract](native-contract.md) for new work.

# Native compiler implementation roadmap

The active delivery is complete native replacement of rustc under the
[native-only contract](native-contract.md#active-objective-complete-native-rust-through-cargo).
Upstream Lust with an explicit reviewed integration diff and the native Lustr
complement form the complete implementation;
Cargo's existing unit graph and parallelism compose compatible native artifacts.
One Cargo-facing integration entry can dispatch without replacing that graph.
Mixed-feature units require shared native semantic/IR interfaces and one complete
session, avoiding duplicate parse/check work merely for dispatch. rustc remains
bootstrap/oracle only. Native unit/artifact interoperability is an early gate.
Retain useful no_std/object work as this shared infrastructure: core/preludes,
optional explicit alloc, panic/entry/allocator and freestanding target closure
remain required within full coverage, not a separate campaign. Optional narrow
feature expansion is paused. Preserve input proofs while reassessing extra
frontend/replay work. Match inherited Lust and stock on equivalent successful
work before claiming speed or negligible dispatch cost.
Reuse Cargo's graph and parallel units through the single Lustr entry. Qualify
native library/metadata interoperability early; mixed-feature units must share
one complete native semantic session. Preserve the distinction between core's
prelude, explicitly imported alloc, explicitly declared std and user lookalikes.

The [native compiler contract](native-contract.md) states the project requirement.
Start with the pristine upstream reference and add Lustr-owned support for full Rust while
retaining and expanding its fast development compilation. Existing Lust support
must keep using the native path. Lustr supplies the missing native support;
blueFe delivers the combined compiler. Follow the [upstream preservation policy](upstream-lust.md);
focused shared interfaces may use a small declared integration diff, with
correctness, independent review and appropriate performance evidence. The current
diff remains empty. Guards bind reference, patch and expected integrated result;
no hidden overlays or arbitrary drift are permitted. blueFe's Rust pin is unchanged.

The earlier Cargo/rustc coordinator work is transitional compatibility tooling.
It must remain accurately identified and cannot close native compiler tasks.

## Current implementation

| Area | Status |
| --- | --- |
| Native compiler | Exact upstream Lust baseline in `compiler/native`. Owned source-library additions work through a bounded interface; full frontend, Rust language and crate/metadata coverage remain incomplete. |
| Native frontend | Existing phases plus opt-in owned checked `OsString`/`PathBuf` constraint elaboration and arena reindexing, with full native recheck. The unchanged coordinator compiles as a whole target; general language coverage remains open. |
| Native libraries and generation | Existing source libraries, adapters and C/TinyCC generation, plus owned naming constants, borrowed path conversions and OS-string/path capacity methods. Other APIs and semantic support remain native implementation work. |
| Fast edit path | Existing daemon, query reads, retained state and caches. Every expansion needs warm/cold and edit invalidation checks. |
| Cargo/crate support | Experimental structural artifact with producer resolver modules/bindings/re-exports, early independent types/region signatures, sized slice/string references, struct/enum identities and lifetime applications, symbolic type/const nominal templates and native consumer instances, typed constant records, explicit runtime/location dependencies, lifetime-only function declarations and bounds, borrowed callbacks, checked calls and function values. General library/input semantics, generic/const bodies and artifact/Cargo interoperability remain open; integration unadmitted. |
| Full Rust compatibility | Currently supplied by rustc through the separate coordinator. This is not native compiler coverage. |
| Packaged previews | `preview-2026.11.09-2` contains a rustc-based toolchain and no native Lust compiler. Native compiler packaging and default selection remain open. |

## 1. Reproduce native coverage gaps

Use unchanged public project snapshots and native compilation to identify the
first real language, library, macro and crate blockers. Keep exact source,
features, target and profile selections. A static feature scanner is navigation
evidence, not proof that a program compiles or runs correctly.

Acceptance requires reproducible native failures and corresponding rustc oracle
results. Dependency availability and incorrect target selection are separate
failures; they cannot count as language coverage observations.

## 2. Extend the native implementation

Implement missing behavior in Lustr-owned native parsing, resolution, type/borrow
checking, const evaluation, libraries, lowering and generation through explicit
interfaces to Lust and its declared integration. Any required focused Lust patch
must pass source-policy review and evidence; substantive missing support stays
Lustr-owned. Reuse existing retained sessions and native optimizing C/LLVM units
before introducing new compiler or backend projects. Keep actual cross-invocation
state and final rechecks for changed semantic inputs. Do not replace valid
application code with subset-specific rewrites or suppress required checks.

Acceptance requires failing-first regression cases, native artifacts, matching
runtime behavior and invalid-program rejection. Test programs must combine
existing Lust-supported constructs with the new features. rustc is the oracle,
not the compiler producing the candidate's accepted artifact.

## 3. Preserve fast incremental compilation

Specify positive and negative semantic reads for new operations. Track changed
trait candidates, lifetimes, types, constants, macros, generated source and
crate metadata. Retained state must remain valid after edits; uncertain reuse
must recompute native work rather than return stale answers.

Acceptance requires edit sequences comparing warm native answers with cold
native compilation and the Rust oracle. Include accepted-to-rejected and
rejected-to-accepted transitions, correct ownership/drop behavior, and recovery
after errors. Bound retained memory and keep all first failures.

The bounded retained-ABI artifact prototype has real no-op, edit, alias,
runtime-retirement, mixed Drop/unwind and Cargo graph qualification. It keeps
no transformed AST between requests. The pinned Linux prototype now validates
actual header/linker inputs and identical generated objects through a Rust
observer. Fresh optimized measurements retain substantial miss cost and a
coordinator no-op slower than stock Cargo. Qualify runtime-loader closure and a supervised
product lifecycle before treating it as general dependency reuse or native
Cargo freshness. Its bounded measurements do not
close the full-language or whole-toolchain speed requirement.

## 4. Complete crates and the Rust language contract

Handle unchanged workspace targets, dependencies, features, build scripts,
procedural macros, tests and documentation. Complete missing Rust language and
standard-library semantics. Implement the required native crate artifacts,
metadata, linkage, layouts, calling conventions and unwinding behavior.

Acceptance requires real projects and Rust compatibility tests, including
cross-crate generics, trait objects, macro-generated code, const evaluation,
unsafe/FFI boundaries, allocation ownership, panic recovery, TLS and atomics.
Native library checking is not rlib emission. A C ABI demonstration or metadata
placeholder does not establish Rust artifact compatibility.

## 5. Deliver the native compiler through blueFe

Package the native compiler, its matching libraries and required TinyCC assets
with correct notices, relocation behavior, source bindings and privacy checks.
Make the native compiler the primary product entry point as its command and
artifact contracts become usable. Preserve existing compatibility tools during
that transition and identify their use explicitly.

Acceptance requires clean installation, offline repeated compilation, actual
native execution, upgrade/cache invalidation, rollback and every advertised
host/target. The existing rustc-based preview is not this delivery.

## 6. Establish full-project speed

First compare pristine Lust with the minimal integration, then measure extension
cost when unused, then mixed/native-complement units against equivalent stock
work. Keep profiling separate from timing and supported-path regressions visible
individually. General Cargo profiles need more than fixed optimization flags.

Compare native Lust, expanded native Lustr and stock Rust on equivalent work.
Measure cold compilation, warm/no-op requests, body/signature/type/macro edits,
invalid edits, linking and compile-plus-test time separately. Bind exact
compiler identities, configuration, source and cache state.

Acceptance requires retained raw measurements, regressions, representative
unchanged projects and replayable analysis. Preserve Lust's native supported
path and measure the cost of added features. Shared-host intervals and narrow
mathematical arguments must retain their assumptions. An upstream subset ratio
does not prove full Rust speed, and a compiler-build benchmark cannot substitute
for native application compilation.

## Retained compatibility research

Earlier work remains useful evidence within its original scope:

- The [verification report](verification.md) records coordinator and explicit
  native/Cranelift qualification, including failures and runtime checks.
- The [matched CGU16/ThinLTO experiment](measurements/2026-11-09-cgu16-thinlto-v3/README.md)
  measured 108 rustc-based metadata-only root checks. Its 17.9% time reduction
  was against a matched stage2 control. Stock remained faster on all three
  workloads. These are not native Lust/Lustr measurements.
- `lustr-semantic` inspects a pinned rustc's checked information. It emits
  inspection reports, not native executable IR or full cache keys.

The project remains incomplete until the native compiler meets the language,
execution, artifact and performance requirements above. Memory-safety migration
is later work and must preserve the working implementation's behavior.
