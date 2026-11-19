# Compiler design and implementation checkpoint

The [exclusive coverage contract](exclusive-coverage-contract.md) defines the
only goal. This document separates the required interfaces from observed code.
Reviewed 19 November 2026. Refresh source and CI identities before new claims.

## Ownership and representation

Lustr owns one compilation context for the unchanged program. Lust retains its
frontend and checked/incremental state. Selected upstream components provide
missing semantic operations and artifacts through explicit interfaces. Each
operation and emitted instance has one owner. The actual Lust-supported set,
current bridge coverage and true complement are distinct.

| Interface | Required invariant |
| --- | --- |
| Prepare supported work | Use actual native representations before emission; retain checked state, source/instance identities, dependencies and ABI/runtime obligations. |
| Request a complementary fact | Invoke an admitted necessary upstream component through the shared context; record inputs, result owner and dependency reads. |
| Report incomplete integration | Preserve the true Lust support inventory and expose bridge/complement gaps. Do not substitute a stock compiler. |
| Emit native or complementary work | Consume the assigned checked state once and produce validated objects/symbols. An emitter failure remains a failure. |
| Encode metadata and link | Consume genuine identities, checked facts and complete object inventory. Preserve real Rust downstream compatibility. |
| Retain/publish | Commit only complete successful artifacts and dependency state; failed work cannot overwrite the last success or become a hidden cache miss. |

These are role descriptions, not claims of a complete current API. A process or
daemon boundary is permitted when ownership and lifetime rules are explicit.
Borrowed in-process state and generation-bound worker handles must not be
confused with persistent semantic identities. Context needed for coherence,
resolution or metadata must be shared or retained without running a complete
second frontend over covered work.

## Observed snapshots

| Source | Observed implementation | Consequence |
| --- | --- | --- |
| Default [7cea76c9](https://github.com/glopyglerky/lustr/tree/7cea76c9d3adc587b54ccff168fe07efafd40309) | Native-first public entry, library retry and full-Rust compatibility. No later scalar mixed implementation. | Default has not delivered the required boundary. |
| [PR #29](https://github.com/glopyglerky/lustr/pull/29), [a9fd4c63](https://github.com/glopyglerky/lustr/tree/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d) | Typed scalar preparation, retained checked state, native object handoff and real Rust metadata. Cold mixed selection follows Rust analysis. | Valuable components and scoped evidence, with narrow coverage and duplicated analysis. |
| [PR #32](https://github.com/glopyglerky/lustr/pull/32), [cb0721d4](https://github.com/glopyglerky/lustr/tree/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3) | Later native/correctness and shared semantic work, plus the retained scalar/metadata implementation. Public compiler retries and whole-consumer Rust analysis remain. | Current audited source for corrective work. Reuse repairs; retire prohibited routes and replace broad driver dependence with admitted complementary interfaces. |
| Merged [PR #31](https://github.com/glopyglerky/lustr/pull/31), [2fbafe39](https://github.com/glopyglerky/lustr/tree/2fbafe391fdf3d0a59b8cc24e9b0f55be3570cbd) | Reconciles active requirements and source/history evidence. | Changes no product source and does not qualify a package. |
| Current [implementation cleanup](https://github.com/glopyglerky/lustr/blob/114227e8b255e8aa49ce50019b7d614d53e39c22/docs/compiler-set/exclusive-coverage-cleanup.md) | Removes conflicting public retries, alternate whole-program launchers and whole-compiler packaging; inventories partial imports and guards native package boundaries. | Exposes remaining integration gaps. Full complementary semantics, artifacts and performance qualification remain unfinished. |

Primary candidate paths:

- [Public entry](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/src/main.rs).
- [Mixed driver](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/crates/lustr-semantic/src/mixed_main.rs).
- [Native preparation](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/crates/lustr-native-core/src/mixed_unit.rs).
- [Native emission filtering and metadata integration](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/crates/lustr-semantic/src/mixed_backend.rs).

Omitting successfully emitted native instances from LLVM is useful artifact
composition. It does not remove the Rust frontend work already done on those
instances. Likewise, additional shared-definition experiments are progress only
within their actual scope, not proof that a full consumer-analysis dependency
has disappeared.

## What to reuse and what to replace

Keep the native worker and explicit source patches, strict preparation/failure
contracts, validated object/symbol handling, genuine metadata knowledge,
retained dependency observations and version/callback/runtime repairs. Preserve
original failure evidence and their exact source identities.

Remove whole-program retry and stock-compiler escape routes from production.
Delete obsolete whole-program alternate compiler launchers and their package
assembly paths. Retain only necessary compiler-private dependencies in independent
bounded mechanism/oracle qualifiers, with explicit disclosure. Renaming a general
alternate compiler as research-only does not satisfy this cleanup. Do not weaken
an ownership guard to preserve a compatibility demo. Missing full Rust support
must be reported as unfinished until the admitted complement exists.

The [disposition register](retained-work-disposition.md) is the detailed record.
The [Rust review](upstream-rust-review-2026-11-19.md) identifies existing generic
solver and representation interfaces. Every real import still requires its
own necessity, closure, owner, caller and qualification evidence.

## Immediate continuation: genuine consumer metadata

The established next artifact boundary is already recorded in the
[PR #32 metadata study](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md) and its
[original dependency probe](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/evidence/shared-metadata-dependency-20261119/verification.json). Keep this work ahead of an arbitrary
new solver import.

The genuine Rust encoder calls `LocalDefsRemapping::new`, which reaches
`TyCtxt::definitions`; that accessor demands `analysis(())` before freezing
local definitions. In the recorded probe, consumer `typeck_root` and complete
`analysis` were panic traps. The encoder reached the analysis trap, exited 101
and produced no requested rmeta. This is a useful dependency failure, not a
metadata qualification pass.

Native prepared state currently lacks a joined proof for all declaration,
attribute, visibility, privacy, lint and diagnostic obligations that original
analysis establishes. Import or adapt the narrow upstream completion service
so the real encoder consumes the validated definition inventory and actual
native checked proofs. Preserve applicable original checks. Keep unrepresented
obligations explicit. Never replace analysis with unconditional success,
fabricate typed-body results or run the complete consumer frontend to recreate
facts Lust already checked.

After completion is valid, retain the real exported-symbol query and give it
the exact native monomorphized-instance graph. An empty fabricated graph or
changed shared-generics/profile flags is not an adapter. First qualify a bounded
nongeneric native library with genuine metadata/archive and a separately built
matching-Rust caller. Preserve traps against consumer Rust body checking/MIR,
invalid and uncovered native controls, symbol ownership, output failure,
artifact preservation and recovery. That bounded result still does not deliver
the full complement or all supported Lust artifact integration.

## Evidence and acceptance

The [18 November qualification record](qualification-review-2026-11-18.md)
and [performance record](performance-review-2026-11-18.md) remain unchanged
historical evidence. Their passed gates must not be described as missing;
their failures must not be erased. A later source does not inherit a pass or
remain broken merely because its predecessor failed. Follow exact evidence.

The next complete slice must show an unchanged mixed program through retained
Lust and a justified complementary service, with observed phase/instance
ownership, real artifacts, invalid-input controls and retained-edit correctness.
It must preserve the full supported inventory rather than redefine it around
what the slice can demonstrate. Apply the [checklist](completion-checklist.md)
and [performance policy](practical-performance.md) before any release claim.

## Documentation validation boundary

PR #31's earlier docs-only source
[f42649f4](https://github.com/glopyglerky/lustr/tree/f42649f4f02d170dc899ac70fcc048db2b91da73)
retained the pristine native compiler. Its
[Linux native job](https://github.com/glopyglerky/lustr/actions/runs/44459525498/job/135342625059)
reported 5,525 sanitized passes and the `unicode_props_hash` timeout; its
[macOS native job](https://github.com/glopyglerky/lustr/actions/runs/44459525498/job/135342624876)
failed linking `_personality`. These match the observed donor failure classes
recorded in the Lust review. They do not qualify or invalidate later source
repairs automatically. The merged documentation correction changed no compiled
inputs and made no fresh compiler or performance qualification claim. The later
[implementation cleanup](https://github.com/glopyglerky/lustr/blob/114227e8b255e8aa49ce50019b7d614d53e39c22/docs/compiler-set/exclusive-coverage-cleanup.md) has its own source and
verification record. Preserve the failed native gates as evidence rather than
presenting documentation checks as compiler qualification.
