# Conditional impl-associated item environments

Repair base: `d89969c03d90723aa3e7a42b542d736db1edd485`. The independent
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
whole-impl controls, stronger requirements, WF and sibling isolation. The first
20 pinned reference expectations passed before candidate comparison. Two added
assigned-reference WF controls require the item-local outlives clause and
reject its use in a sibling. The new Rust
qualifier `tools/test-associated-item-environments.rs` also checks actual public
fresh/retained edit, rejection, repair and unrelated-body retention histories.
Installed-candidate execution is pending at this source checkpoint. Existing
passing evidence remains bound to its original source; this record does not
repair its historical results retroactively.

This is a native semantic repair toward P1/P2. Production GAT service, genuine
provider metadata through ordinary Cargo, wider environments, full compiler
coverage and complete-work performance remain unfinished. Diagnostic warning
parity is not inferred from matching semantic acceptance/rejection.
