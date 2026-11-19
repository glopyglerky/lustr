# Lustr completion checklist

Review date: 18 November 2026, America/Denver. This is the ordered work list for
the [architecture plan](architecture-plan.md). It supersedes the previous
independent-native-expansion roadmap. The scope is the user's full replacement:
Lust handles its actual supported work and imported upstream Rust supplies the
complement inside the same compilation.

The architecture audit covers PR #29 at
`a9fd4c6336b1e39c4f584f7da50a437c5c248a0d`. Default `next` at
`7cea76c9d3adc587b54ccff168fe07efafd40309` is older. PR #30 at
`b94d3713616804fead3f30b18a530e2e81a9510b` adds a proposed callback repair;
it does not close the general integration gap. Use the source-bound
[design](design.md), [qualification](qualification-review-2026-11-18.md) and
[performance](performance-review-2026-11-18.md) checkpoints for evidence.
Refresh a source identity before claiming a later continuation is qualified.

## Reconciled candidate source — 19 November 2026

PR #31's checklist is retained on implementation candidate
`af0a281f2d1fae04e8ebfd5aa2c4872b20245c04`. The
[design continuation](design.md#candidate-continuation--19-november-2026)
records its owned preparation API, widened scalar identity/ABI checks, Unicode
and runtime repairs, and the carried PR #27 scanner fixture correction. This
is candidate progress against the numbered criteria below, not a declaration
that the general semantic boundary or a release is complete.

The historical PR #29/initial PR #30 findings below keep their source scope.
Consult the [later PR #30 job observation](qualification-review-2026-11-18.md#later-pr30-observation--19-november-2026-0502-utc)
before treating its source-built mixed job as pending. Neither that job nor the
earlier frozen failures qualifies this newer candidate. Update completion status
from the applicable source-bound records; do not repeat an already integrated
repair because the historical review described its predecessor.

## 0. Make the contract authoritative and consolidate existing work

**Status: corrected contract and existing repairs consolidated on the candidate;
default publication remains open.**

The candidate incorporates PR #31's corrected contract without merging default
`next`. PR #19's process-directory repair was already equivalent in the base.
PR #27's scanner-fixture name is now integrated. PR #23's reproducible installer,
SDK selection and exact LLVM 23.1.5 pin are restored; the
[restoration record](macos-toolchain.md) explains the deliberate version choice
and preserves every sanitizer gate. PR #30's callback repair and corrected unit
expectations are integrated, with fresh
[ordinary and unwind results](native-combined-qualification.md).
The native source declaration records tree
`7dbb7ac001f81c17adbbbd030573220dadfc079a` and its exact diff from pristine Lust.
These dispositions supersede the outstanding status of the corresponding
historical audit items below; they do not qualify the entire compiler or default.

- Land the reviewed documentation correction on `next` so its AGENTS, README,
  contract, design and roadmap stop prescribing independent Rust reimplementation
  and a faster-than-Lust result from every patch.
- Record one implementation integration head and its complete/native trees.
  Inventory open PRs and working continuations before taking another fix.
  Preserve unique source changes, exact evidence and original failure records.
- Keep PR #29's equivalent [PR #19](https://github.com/glopyglerky/lustr/pull/19)
  process-directory ownership repair. Do not implement or merge it a third time.
- Carry [PR #27](https://github.com/glopyglerky/lustr/pull/27)'s distinct
  scanner-fixture name into the current native tree and its current integration
  patch/guard.
- Reuse [PR #23](https://github.com/glopyglerky/lustr/pull/23)'s reproducible LLVM
  acquisition and SDK selection mechanism with a deliberate qualified pin.
  PR #29's floating Homebrew installation plus version assertion remains
  vulnerable to catalog changes. Preserve the full sanitizer gate.
- Continue the proposed [PR #30](https://github.com/glopyglerky/lustr/pull/30)
  callback correction and its qualification. Do not duplicate the repair merely
  because PR #29 still records the original failures.

**Done when:** new work starts from one identified integration source, the
authoritative default instructions match the user contract, and each earlier
repair is recorded as integrated, equivalent/superseded, or still required.
A documentation PR does not merge the compiler stack.

## 1. Implement the actual Lust capability and checked-state boundary

**Depends on:** the recorded integration source.

The current mixed bridge accepts a limited family of root, nongeneric scalar
functions. That is not the full Lust-supported subset. Extend the existing
checked preparation interface to cover Lust's real supported semantics,
dependencies, layouts, runtime requirements, targets and profiles.

Capabilities must be resolved from the actual compiler representation. Macro
expansion, method/trait resolution, generic instantiation, target settings and
runtime dependencies can change what an instance needs. A keyword scanner,
diagnostic match or attempted code generation cannot define the boundary.
Capability determination belongs in ordinary compilation and must not introduce
another whole-program preflight.

Keep these outcomes distinct: supported by Lust; needs imported complementary
work; temporarily outside the implemented bridge; invalid Rust; compiler failure.
A missing bridge is an implementation task, not a reason to relabel a supported
Lust feature as permanently complementary.

**Done when:** supported work is prepared from authoritative facts before
emission; the emitter consumes that exact checked state; dependency reads are
retained; mixed programs keep their supported Lust work. Real implementation
failures remain failures. No throwaway whole-program native attempt is required
to discover the complement.

## 2. Join semantic ownership with imported upstream Rust

**Depends on:** a concrete checked-state contract from step 1.

This is the main remaining architecture task. The audited cold mixed path first
runs whole-crate Rust analysis, writes a separate `kernel.rs` projection and
checks that projection with Lust. Scalar admission removed an extra native
check/build round trip, but it did not remove this duplicated frontend work.

Implement the continuation through shared authoritative definition and instance
identities, resolved bindings, types, trait obligations, source/hygiene context,
layouts and semantic dependency reads. Import the needed upstream services and
their dependency closure. Do not independently recreate the missing Rust
compiler or assume its complement is a detachable directory.

The first implementation slice should use an unchanged mixed program that
carries a checked non-scalar Lust-supported value or definition through an
imported complementary service and back. Reuse a genuine upstream-supported
fixture to establish what Lust owns. Carry actual checked state and dependencies,
not generated source that starts an independent compilation.

**Done when:**

- A wholly supported cold request retains Lust's frontend without a mandatory
  complete Rust frontend first.
- A mixed request uses both implementations within one semantic compilation.
- Necessary shared facts have one authoritative owner and explicit dependency
  tracking; already checked supported work is reused.
- No discarded native compilation or whole-program Rust retry discovers support.
- A narrowly passing first slice is recorded as progress, not the completed
  general boundary.

Imported Rust may supply shared global services where the program requires
them. The requirement is to avoid redundant complete compilation, not to forbid
all upstream code on a supported request or delete useful internal workers.

## 3. Complete ABI, runtime and genuine Rust artifact composition

**Depends on:** the semantic boundary; reuse the existing object/metadata work.

Generalize from isolated scalars to Lust's actual supported aggregates,
references, calls and instances. Join calling conventions/layouts, generic
instances, closures, vtables, allocation/deallocation, ownership, drop glue and
unwinding where values or control cross the boundary. Unsupported operations
use imported upstream implementations within that same compilation.

Keep the existing real Rust metadata, rlib/rmeta, dep-info, native object member
and linker machinery. Feed it authoritative shared facts. Do not fabricate
metadata or require a second full frontend solely to recreate facts.

**Done when:** unchanged mixed programs and dependent crates consume the real
artifacts, each definition/instance has one emission owner, observable runtime
behavior matches the declared Rust reference, and invalid programs are rejected.
Include actual cross-boundary calls, values, drops and panic recovery; compiling
two unrelated examples separately is insufficient.

## 4. Retain and invalidate state across the joined boundary

**Depends on:** steps 2 and 3.

Preserve Lust's daemon, fast unchanged requests and current successful scalar
reuse. Extend dependency tracking beyond the limited literal/body-edit
certificate to signatures, types, traits, macros, generated source, build scripts,
dependencies, target/profile/features and compiler/runtime identity.

**Done when:** warm and cold outcomes agree through valid-to-invalid and
invalid-to-valid edit sequences, changed semantic inputs invalidate affected
work, and uncertainty causes correct recomputation. Failed transport, emission,
object publication or linking fails the request while preserving the previous
successful retained state. Existing real I/O/link recovery tests remain covered.

## 5. Use the integrated compiler through every normal public entry

**Depends on:** the working boundary, artifacts and retention.

Connect standalone build/check and Cargo's compiler invocation to the same
integrated compilation. Remove the standalone status-1 and diagnostic-driven
whole-program retry. Cargo adapters retain command, package/target, signal and
artifact behavior; they do not become a product language selector.

**Done when:** public traces demonstrate Lust and complementary participation
for unchanged mixed requests, supported requests preserve the native fast path,
and unsupported work does not move the entire program to stock rustc.
CLI compatibility must preserve original flags, dependencies, build scripts,
procedural macros, features, panic strategy, profiles and requested artifacts.

Keep transitional compatibility paths accurately documented until replacement.
A passing transitional path cannot close this item.

## 6. Resolve concrete correctness gaps and qualify the exact package

**Can proceed alongside steps 1–5; release depends on all of them.**

| Required work | Observable completion |
| --- | --- |
| Unicode and Rust-version alignment | Native constants, runtime tables and reported character semantics agree with the declared SDK from reproducibly generated, pinned data. Version text alone is insufficient. Reconcile all known native-vs-SDK differential failures. |
| Lexical Unicode behavior | Identifier/NFC behavior matches its own pinned Rust lexical dependencies. Do not blindly set lexical tables to the core character version. Address the separate non-NFC rejection versus normalization gap. |
| Erased callback ABI | PR #30 or its reviewed successor fixes incompatible C callback calls without casting away the mismatch. Resolve the two new unit failures and complete the full native sanitizer gate on that exact source. |
| Broader mixed outcomes | Exercise an actual complementary outcome alongside a native peer in a real unchanged mixed program. The PR #29 schema-3 workload covered emitted and failed responses, not this outcome. |
| Full Rust compatibility | Version-matched compile/run/rejection and dependency tests cover unchanged supported, complementary and mixed projects, including macros, generics/traits, unsafe/FFI, allocation, drop and panic behavior. |
| Advertised platforms | Every claimed platform has the integrated native/complementary path, matching runtime/ABI/linking and its actual gates. Cargo compatibility alone is insufficient. |
| Installable package | Exact source/binary manifests, matching tools/sysroot, clean installation, relocation, repeated offline builds, cache invalidation, required assets and licenses all work. |

PR #29's Linux native and source-built mixed gates already passed within their
recorded scope. Do not keep treating those executions as missing. They cannot
automatically qualify changed source or erase the declared-SDK mismatch. PR #30's
initial CI fails two native unit checks before full native qualification; the
[qualification checkpoint](qualification-review-2026-11-18.md) records specifics.

The audited Darwin package routes through the matching upstream compiler rather
than an integrated Mach-O mixed bridge. Complete that platform integration before
advertising it as the same full Lustr product. Other targets need equally honest
evidence. The contract does not require claiming every platform before the first
qualified release, but every advertised platform must satisfy the product.

**Done when:** one identified installable candidate satisfies the architecture
and all applicable correctness gates for its declared Rust version and supported
targets, with no unresolved known acceptance/runtime mismatch hidden by fallback.
No finite suite proves all possible programs; publish the actual qualification
coverage and remaining unsupported targets explicitly.

## 7. Apply practical performance criteria once, then publish the result

**Depends on:** the semantically correct, integrated candidate.

Use a bounded, preregistered comparison of that exact package, pristine Lust and
the matching Rust reference. Keep source, profile, workload, cache state and
timing boundaries equivalent. The existing complete-package assessor requires
24 matched pairs plus applicable qualification. A different campaign must state
its own justified design before measurement; neither historical 12-pair data nor
instrumented profiles can silently replace the acceptance result.

Retain the existing engineering allowances:

| Work | Practical objective |
| --- | --- |
| Lust-supported warm check/build | Within 10% or 0.5 ms of Lust, whichever allowance is larger, with a useful demonstrated gain over rustc. Apply the documented paired uncertainty criteria. |
| Ordinary complementary Rust | About stock rustc; 5% or 0.5 ms is the existing review budget. Large speedups are unnecessary. |
| Mixed workflows | Show the useful end-to-end gain where Lust participates and disclose cold/edit/check/rejection/link/test regressions. Do not hide losses in a blended headline or demand that every individual mode win. |

The older roughly 0.3 ms gap is not a mandatory blocker. Investigate only a
measured, reproducible cost, keep low-risk fixes with matching correctness
evidence, and stop when the practical goal is met. New PR #29 data has a larger
unresolved warm-build gap and several meaningful mixed losses; the
[performance checkpoint](performance-review-2026-11-18.md) reports them.
Cross-host differences do not establish a source-caused regression.

Do not repeatedly run the two known-inapplicable historical campaigns without
changing their applicability. Preserve the frozen baseline's Unicode failure
and shared-native identity refusal. Select legitimate comparisons for the final
source rather than editing historical baselines or bypassing guards.

**Done when:** the candidate has the qualified practical results, and the README
shows a clear three-compiler table with exact source/binary versions, commands,
host/profile/cache settings, intervals, tails, slower cases and raw-data/replay
links. Keep historical and current results separate. Then publish that qualified
package and its matching release record. The current documentation review does
not itself authorize claiming compiler completion or supply release evidence.

## Review controls that prevent another regression

Every compiler PR should answer these questions in its description:

1. Which completion item does this change advance, and what observable behavior
   establishes that progress?
2. Which supported work still uses Lust, which imported services supply the
   complement, and who owns the shared facts?
3. Does the change add whole-program retry, duplicated supported analysis, or
   redefine Lust's subset to fit a narrower bridge?
4. What exact source and meaningful runtime/edit/rejection evidence support
   correctness? Which limits remain?
5. Does a performance claim use equivalent actual work and disclose regressions?
   If no speed claim is made, is the necessary correctness/integration reason clear?

Do not answer these questions with a planner trace alone. Demonstrate actual
compiler participation. Reviewers must reject architectural substitutions even
when a compatibility demo passes. Keep a single current checklist rather than
letting a new branch's local plan silently replace this one.
