# Native compiler requirement

The user clarified the project requirement after the rustc-based preview work.
This contract supersedes the earlier plan to treat a Cargo coordinator plus a
full Rust compatibility compiler as the project's implementation.

## Active objective: complete native Rust through Cargo

The user's latest direction requires complete native replacement of rustc,
including `no_std`, with native composition and equivalent complete-work speed.
`no_std` is part of that complete coverage, not a separate delivery campaign.
Cargo must invoke one Lustr compiler entry: upstream Lust handles the code it
already supports, and Lustr implements the remainder natively. These are two
complementary native compilers behind the Cargo-facing integration. Production
compilation of user projects must not fall back to rustc. A separately identified
stock compiler may bootstrap this compiler or act as an independent oracle.
Reuse Cargo's real dependency graph and parallel compilation units. Do not
replace its scheduler or force one engine for an entire project. The Lustr
entry chooses native Lust or the native complement for qualified units. A unit
mixing language features needs one complete native semantic session and shared
qualified phase/IR interfaces. Cargo does not supply missing compiler semantics
or library metadata. Native unit/artifact interoperability is an early gate.
The admitted job limit bounds Cargo's parallel work.
Historical compatibility tools and rollback artifacts remain preserved; they
are not the required native implementation.

`no_std` selects the target's real core identity and edition prelude and removes
automatic std linking. It does not prohibit an explicit std dependency. `alloc`
is a separate explicit dependency. Attribute acceptance, a core alias to std,
or a binary linked to the host runtime does not qualify this requirement.
Acceptance includes core and optional alloc APIs and lang items, panic and entry
points, allocator policy, target ABI/runtime, typed crate dependencies, real
native library artifacts and Cargo's metadata/dependency contracts. Freestanding
artifacts must have no unintended libc, pthread or host-loader dependency.
Positive, negative, runtime and object/link-closure evidence are required.

Pause optional narrow feature expansion and isolated timing campaigns. Preserve
checks and input soundness while replacing
unnecessary rejection/recompilation/replay with qualified native phase interfaces.
Compare inherited Lust, current Lustr and stock on the same complete work and
semantic/profile inputs. Separate cold/check/build/edit/rejection/compile-plus-run
time and account for staging, CPU, copies and host-input proofs. A win over a slow
internal baseline or a standalone no-op result does not meet the speed requirement.
No zero-overhead, universal coverage or project-speed claim follows from routing.

## Required behavior

Lust is the starting compiler. Preserve its pristine pinned reference and a
small, explicit integration diff under the [source policy](upstream-lust.md).
Focused interfaces inside Lust may preserve semantic state and eliminate
duplicate parsing/rechecking; substantive missing support remains Lustr-owned.
Everything Lust already supports retains its native behavior and required speed.
blueFe delivers the combined compiler. The declared integration diff is currently
empty; no nonempty patch has been reviewed or qualified.

Admission binds the reference identity, exact patch, qualification record and
expected integrated result. It requires correctness, independent review and
appropriate performance evidence, rather than an unrelated speed win for every
necessary interface. Guards reject unexplained changes. Copied implementations,
hidden overlays and unrecorded build-time patches remain forbidden. blueFe's
separate protected Rust pin is unaffected.

The intended result accepts unchanged full Rust projects while retaining and
extending Lust's fast development compilation. Full language coverage and
equivalent-workload speed are both requirements. Neither follows from a
wrapper, a successful parse, an optimized rustc build, or a few fixtures.

The integration must compose existing Lust-supported code and Lustr's additions
in the same program. A bounded source-library interface now supplies native
additions; full frontend and crate/metadata extension remain incomplete.
Do not replace that work with source keyword guesses, a rustc wrapper, or an
unproved edit to the upstream compiler.

## Transitional components

The current `crates/lustr` coordinator's `auto` mode selects Cargo/rustc.
The published `preview-2026.11.09-2` packages contain no native Lust compiler.
Their full Rust support and performance evidence belong to the rustc
compatibility path. They do not satisfy this native compiler requirement.

Keep existing compatibility tools usable while the replacement is developed.
Their fallback must be explicit and must not count as native language coverage,
native acceleration, or project completion. rustc remains useful as a bootstrap
compiler and a differential oracle. Its optional Cranelift, frontend scheduling
and compiler-build experiments are separate research.

The native compiler has C/TinyCC development and optimizing C/LLVM machinery;
general Cargo release/profile integration still needs qualification. It has no
Rust rlib emitter, and its language and standard-library support remain
incomplete. These are implementation gaps, not permission to reduce the full
Rust requirement. A C object and an empty metadata file cannot substitute for
Rust library metadata and ABI compatibility.

## Evidence required for each expansion

- First reproduce valid Rust rejected by native Lust, or a native behavior
  differing from rustc. Preserve the source and first failure.
- Implement the expansion in Lustr-owned code. Do not change the application to
  avoid the feature or route its acceptance through rustc. Any necessary Lust
  integration patch needs the explicit source-policy review and evidence before
  admission.
- Check native compilation, emitted artifacts and runtime results against the
  oracle. Retain invalid-program rejection, ownership, panic and drop checks.
- Test mixed programs containing existing and new native features. Check warm
  daemon answers against cold native compilation after relevant edits.
- Measure native cold, warm, no-op, body, signature, type and invalid edits
  separately. Bind source, compiler, configuration and cache state. Retain
  failures and regressions. Compiler-build and rustc-wrapper timings cannot
  stand in for these measurements.

No universal speed ratio or proof of all Rust semantics is claimed by this
contract. The requirement remains open until the actual compiler and its
representative full-project evidence meet it. Memory-safety conversion remains
future work and must preserve the working compiler's behavior.
