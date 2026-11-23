# Conditional impl-associated item environments

Repair base: `d89969c03d90723aa3e7a42b542d736db1edd485`. Qualified compiler
source: `2ea271861092e2d4c191c61528f47948ab7fbc55`. The independent
audit found a valid-program rejection in the preceding predicate integration:

```rust
trait Family { type View where Self: Copy; }
impl<T> Family for T { type View where T: Copy = u32; }
fn main() {}
```

Pinned Rust `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` accepts this source,
with a deprecated-where-location warning; the preceding native candidate
rejects it with L0612. Its item-local `'static` counterpart similarly produced
a false L0617. The original paired audit, sources, arguments and streams are
preserved by the runtime owner; its full-record SHA256 is
`cb70da6d7a9250d12b0cadd2eb0c222dd3f33f8e65c21446f5af5339e1470ceb`.
This is separate from the existing post-equals where-clause parser gap.

The native collector now retains each impl-associated declaration's actual
parameters, predicates, region facts, parent/item identities and regioned
assigned value. Whole-impl obligations, assignment obligations, and comparison
of item requirements use separate environments. Item clauses do not enter the
impl-selection record or sibling environments. Assignment WF, region checks,
dependency fingerprints and retained span shifting consume the corresponding
facts. The GAT and lifetime-trait-associated-type gates remain unchanged.

This follows the declared pin's
[`compare_type_clause_entailment`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_hir_analysis/src/check/compare_impl_item.rs#L2474)
and `check_type_bounds` in that same source: compare the impl item's clauses
under impl-wide and trait-item clauses, and check its assigned type under the
impl item's own parameter environment. Existing native proof/region operations
remain the implementation owner. No new Rust compiler import, frontend, driver
or fallback is introduced by this repair.

`tools/fixtures/associated-item-environments.json` fixes 22 valid/invalid
expectations, including conditional `Copy`/outlives, assigned-type bounds,
whole-impl controls, stronger requirements, WF and sibling isolation. All 22
pinned reference expectations passed before candidate comparison. The
assigned-reference WF controls require the item-local outlives clause and
reject its use in a sibling. The Rust
qualifier `tools/test-associated-item-environments.rs` also checks actual public
fresh/retained edit, rejection, repair and unrelated-body retention histories.
The installed package passes all 22 fresh comparisons and ten retained states,
64 recorded compiler operations in total. Successful native body counts are
`3, 0, 1, 0, 0, 1, 0` for cold, no-op, type repair, condition repair,
sibling repair, unrelated body edit and final no-op. The rejected type,
condition and sibling edits fail semantically and never publish a success.
All three executable hashes match their constructor receipts. See
[`verification.json`](verification.json) for source, binary and qualifier
identities, independent outcomes, stream hashes and observed request durations.

The changed source also passes eight focused associated-item/dependency tests
and all 32 retained patch tests. Its installed default-profile scalar Cargo
check and import-generation history passes all 38 operations, including pinned
Rust reading genuine Lustr metadata, invalid-provider rejection, malformed
metadata, renamed definitions, repairs and unrelated-body retention. The linked
control separately retains its abort/debug-disabled profile; it does not qualify
default-profile linking. See
[`import-generation-verification.json`](import-generation-verification.json).

The first installed run passed the fresh semantic cases but stopped at the cold
retained counter assertion: the qualifier had omitted `LUST_STATS`, so the
required counter line was absent. Its original log is retained. Enabling the
existing observation in the qualifier yields the results above without changing
the compiler or weakening a work-count assertion. Existing historical evidence
remains bound to its original source; this record does not repair its results
retroactively.

This is a native semantic repair toward P1/P2. Production GAT service, genuine
provider metadata through ordinary Cargo, wider environments, full compiler
coverage and complete-work performance remain unfinished. Diagnostic warning
parity is not inferred from matching semantic acceptance/rejection.
