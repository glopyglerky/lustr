# One Lustr compiler with exclusive Lust coverage

The [exclusive coverage contract](exclusive-coverage-contract.md) controls this
plan. Lustr's only goal is full rustc replacement using Lust for all work Lust
covers and only the necessary upstream implementation for the complement.
The full Rust source stays in blueFe. A larger dependency graph is not blanket
permission to import a complete competing compiler.

## Required result

Compile an unchanged program with one semantic and artifact contract. Preserve
Lust's frontend, retained checked state and incremental speed. Assign missing
operations to justified imported components before emission, using shared or
faithfully bridged definitions, types, instances, layouts, ABI, symbols,
constants, drops, unwinding, metadata and dependency reads.

Actual Lust support is broader than the existing scalar bridge. The difference
is unfinished integration and remains Lust-owned. Neither complete Rust analysis
before a native body substitution nor whole-program native-fail/rustc-retry
completes this architecture. A missing implementation must remain visible.

## Current source state

The observed default is
[7cea76c9](https://github.com/glopyglerky/lustr/tree/7cea76c9d3adc587b54ccff168fe07efafd40309).
The audited continuation is [PR #32](https://github.com/glopyglerky/lustr/pull/32)
at [cb0721d4](https://github.com/glopyglerky/lustr/tree/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3).
It contains reusable native/correctness/metadata work but also prohibited retry
and whole-consumer analysis paths. This documentation change establishes their
required disposition. It does not remove product code or qualify a replacement.
The [design checkpoint](design.md) and [disposition](retained-work-disposition.md)
record that distinction.

## The concrete conflicting direction

The earlier [PR #31 architecture text](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md)
permitted the full upstream compiler dependency graph and treated executed work
as the only import criterion. That exception conflicts with selective source
reuse. Earlier defaults also prescribed independent implementation of the missing
Rust language, while candidates retained retries and whole-consumer analysis.
The result was progress on compatibility and narrow bridges without completing
the required compiler boundary. This correction removes those competing active
instructions. Source cleanup must delete obsolete alternate compiler launchers,
not rename them as research-only execution paths.

## Implementation order and acceptance

| Order | Work | Observable result |
| --- | --- | --- |
| 0 | Reconcile active directions and source ownership | AGENTS, README, contracts and work list agree. Existing source/evidence is captured before conflicting production routes are retired. |
| 1 | Expose actual Lust capability and checked state | Support decisions precede emission. Lust coverage, bridge gaps, complement, invalidity and implementation failures remain distinct. Checked work is retained. |
| 2 | Select and connect complementary upstream services | Every import has a pinned necessity/dependency witness and real semantic owner. One unchanged mixed program uses the integrated service without duplicating Lust-owned work. |
| 3 | Complete ABI/runtime and artifact composition | Real instances, cross-boundary calls/values, drops and unwind behavior compose with genuine metadata, libraries, archives and downstream Rust consumers. |
| 4 | Retain correctness across changes | Cold and retained behavior agree across body, signature, type, trait, macro, dependency and configuration changes. Unaffected native work remains retained. |
| 5 | Use the boundary through public commands | Standalone and Cargo requests preserve the original build contract. No user-selected language engine, stock backup or error-driven compiler retry remains. |
| 6 | Qualify exact source and package | Every advertised platform and profile passes applicable actual-native, complementary, mixed, invalid-input, runtime, package and failure controls. |
| 7 | Assess speed against the correct references | Supported work stays near matched Lust and complementary work near matched rustc, separately, with actual paths observed and losses retained. |

The [completion checklist](completion-checklist.md) is the single work register.
These steps are dependencies, not permission to call a partial interface the
completed compiler. Useful correctness repairs can proceed alongside structural
work, but cannot substitute for it.

## Immediate missing artifact capability

Continue the [existing genuine-metadata study](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md). The original encoder
requires a completed/frozen definition inventory. The current native checked
state does not yet supply every declaration, privacy and lint obligation from
Rust's analysis. Extract or adapt the narrow completion service so the encoder
uses actual native proofs and the exact emitted instance graph. Do not bypass
analysis with unconditional success or fabricate empty instance partitions.
This existing dependency failure is the next concrete interface to close; the
reusable solver research does not replace it with a new arbitrary import task.

## Importing the complement

Choose a missing operation and its smallest justified upstream interface. Compute
the source/dependency closure under explicit features and targets. Cut edges only
through verified Lustr adapters. Keep provenance, licenses, build generators,
shared data and real callers recorded. The Rust review identifies reusable
[generic type IR and solver interfaces](upstream-rust-review-2026-11-19.md);
it does not qualify a complete complement or authorize a blanket crate import.

If the closure requires duplicate Lust-owned compiler stages, improve the
interface before production admission. Do not copy the full Rust source, treat
a full-driver dynamic library as minimal, or make dead-code elimination the
source-policy proof. Conversely, do not replace necessary shared semantics with
independent speculative reimplementation merely to reduce an import count.

## Completion and publication

A qualified replacement must satisfy the full exclusive coverage contract,
correctness, actual artifact integration, target/runtime behavior and the
[performance policy](practical-performance.md). A green coordinator test,
source manifest or benchmark of scalar emission cannot establish those facts.
No current package has been qualified as the full product.

The prior [18 November plan](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md)
and [older default plan](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/native-contract.md)
remain historical. In particular, permission for the full upstream compiler
dependency graph and the independent-native-reimplementation destination are
superseded. Their recorded successes and failures retain their original scope.
