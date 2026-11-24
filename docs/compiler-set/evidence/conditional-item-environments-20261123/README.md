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

The first 22 cases in the initial version of
`tools/fixtures/associated-item-environments.json` fix valid/invalid
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

## Regioned compatibility follow-up

Five additional predeclared cases compare regioned trait arguments and an
impl-wide outlives control. The preceding package passes the trait-item-clause
pair, but incorrectly accepts the stricter item condition here:

```rust
trait Gate<A> {}
trait Family<A> { type View; }
impl<'a, T: Gate<&'a u8>> Family<&'a u8> for T {
    type View where T: Gate<&'static u8> = u32;
}
fn main() {}
```

Pinned Rust rejects it with E0478; the source-bound candidate above returns
success. The original paired streams and failed evaluator result are retained.
The original operation record SHA256 is
`6e27c4c77bc06c9d2437d442a5c87b7b1f23f3e987a7bebadf02167c79871d64`.
The corresponding `&'a u8` condition is valid, as is the stricter spelling when
the impl actually declares `'a: 'static`.

The follow-up uses the existing native region proof engine with explicit
optional frames: required impl-item clauses use the impl namespace, and allowed
trait-item assumptions use the instantiated trait frame. Compound outlives
assumptions contribute their actual region/type components. Ordinary supertrait
checks retain their existing trait-side frame. This is native correctness work,
with no upstream import or GAT gate change. Nine focused tests pass, and all
27 pinned reference expectations pass before candidate comparison. The public
qualifier adds six retained region-condition edit/rejection/repair states.
The installed follow-up at compiler source
`11a456378317334a11aa29278e83c4575335f4f8` passes all 27 fresh comparisons
and 16 retained states, 86 compiler operations. The new six-state sequence
checks `2, 0, rejected, 0, 1, 0` bodies for its initial source, no-op,
stricter condition, repair, body edit and final no-op. Rejection still occurs
when the relevant bodies would otherwise be reused. Its 38-operation scalar
Cargo/import regression also passes on the same package. All executable hashes
match the construction receipts. See
[`region-verification.json`](region-verification.json) and
[`region-import-verification.json`](region-import-verification.json).
The preceding package remains the failure witness and a scoped rollback; its
22-case evidence is unchanged. GAT service/admission and metadata integration
remain unfinished.

## P1 declaration boundary

The next native collection slice retains identity arguments in the written
lifetime/type/const order and region-preserving bounds on the full associated
application. Bound resolution includes the item's actual parameter scope,
and the declaration fingerprint includes these facts. Native parameter indices
remain distinct from Rust metadata indices. This is a typed input boundary for
P1/P2, without a second parser, checker or admitted GAT service.

Ten focused tests pass. The new collector check includes a lifetime, a type
parameter, a const parameter, `Gate<&'a T>`, an associated result outlives bound,
where predicates and a lifetime fact. The existing L0621 admission gate is
explicitly asserted. No installed execution or new service qualification is
transferred from the preceding compiler source to this slice.

Compiler source `7569bf35cb74b9335372fe940e3c92a97c9e7816` has now been
constructed and checked separately. It passes ten focused and 32 retained
patch tests, all 27 ordinary fresh differential cases and 16 retained states
(86 operations), plus the 38-operation default-profile scalar Cargo/import
regression. All three executable hashes match their construction receipts.
See [`declaration-verification.json`](declaration-verification.json) and
[`declaration-import-verification.json`](declaration-import-verification.json).

The broader GAT declaration used by the new collection test is independently
accepted by pinned Rust and still rejected by installed Lustr. L0621 remains,
along with legacy L0612/L0203 diagnostics on this unintegrated declaration.
This is an explicit valid-program coverage frontier, without a supported-GAT
claim or a passing rejection exception. Its original source and streams are
retained; their hashes are in
[`declaration-frontier.json`](declaration-frontier.json). The selective pinned
normalization/WF/region caller and genuine GAT provider metadata still need
implementation and joint installed qualification.
