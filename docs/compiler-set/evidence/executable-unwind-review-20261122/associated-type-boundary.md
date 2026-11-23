# Associated-type boundary review, 22 November 2026

The lifetime-preserving associated-type experiment is useful upstream research.
It is not yet a complementary semantic service in Lustr. Keep it as an oracle
and make the next implementation consume its equivalent typed result inside
Lust's retained checker. Do not promote the probe's full compiler dependency
graph into the product.

This note reviews the archived probe sources and logs alongside PR #33 at
[`930b384ad03a26fe141d10be4e261bd82a60be76`](https://github.com/glopyglerky/lustr/tree/930b384ad03a26fe141d10be4e261bd82a60be76),
tree `0de8622237295e2607b7ffed22359749c2e4bc1a`, after
`bc8a7adb`. The probe is absent from that 13-file implementation diff. Its
observations must retain the separate source identities below; they are not
qualification of a newly integrated service at this PR head. This review did
not rerun the probes or benchmark a compiler. The existing
[architecture plan](../../architecture-plan.md#implementation-sequence) and
[completion checklist](../../completion-checklist.md#2-admit-only-necessary-upstream-complementary-code)
remain authoritative; this note supplies a concrete witness for their early
mixed-semantic boundary.

## The missing operation is a generic associated type

The fixture's dependency declares a lifetime-parameterized associated type:

```rust
#![no_std]
pub struct Codec;
pub trait Family {
    type View<'a> where Self: 'a;
}
impl Family for Codec {
    type View<'a> = &'a u32;
}
```

Its unchanged consumer is:

```rust
use schema::{Codec, Family};

fn fold(value: <Codec as Family>::View<'_>) -> u32 {
    *value ^ 7
}

fn spare(value: u32) -> u32 {
    value & 31
}

fn main() {
    let actual = 42u32;
    println!("{}:{}", fold(&actual), spare(actual));
}
```

The manifests specify edition 2024 and an ordinary path dependency named
`schema`, with no profile overrides. The archived original-compiler run exits
0 and prints `45:10`. Replacing the call with `fold(&42u64)` fails with E0308.
Changing only the dependency binding to `type View<'a> = &'a u64;` also fails:
the unchanged `fold` now returns a `u64` where it declares `u32`, and `main`
passes `&u32` where the changed signature requires `&u64`. A successful query
returning `&u64` is therefore only an intermediate result; accepting or running
that unchanged edited consumer would be incorrect. The original logs and
fixture identities are listed below.

This is a specific donor gap, not permission to replace native trait solving.
The [pinned Lust collector](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/check/generics.rs)
explicitly rejects associated-type generic parameters, and `projection_of`
rejects associated-item arguments. The same donor already has a
[native selection/normalization implementation](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/traits/solve.rs)
and [ordinary projection tests](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/tests/traits/pass/projection_infer_from_arg.rs).
Those covered operations remain Lust-owned.

The archived installed Cargo attempt stops at the dependency's unsupported
public request, including default debug information. A direct native attempt
first rejects `#![no_std]`. A separately preserved diagnostic control removes
that attribute and reaches the explicit GAT and associated-item-argument
rejections. That control helps locate the missing operation; it does not turn
the changed fixture into support for the original request. These native logs
name a predecessor audit package, so they are not an execution qualification of
`930b384a`.

## What the upstream probe actually executes

`gat-upstream-binder/query.rs` is a separate executable with
`#![feature(rustc_private)]`. It links `rustc_driver`, `rustc_interface`,
`rustc_middle`, `rustc_infer` and `rustc_trait_selection`. Its input source is
only `extern crate schema;`. It starts `rustc_driver::compiler_entrypoint`,
loads the dependency's existing metadata, and performs its query in
`after_expansion`. It overrides `typeck_root` and `analysis` with panics and
returns `Compilation::Stop` after the query.

Those traps are useful evidence that the observed query does not need those
two local analysis entry points. They do not remove the driver, parser,
expansion or resolver dependency graph, and there is no native function body
in the stub to check. This remains a strictly bounded research use of the
original compiler, excluded from product compilation.

The material query operations are preserved here as source excerpts:

```rust
let env = ty::ParamEnv::empty();
let infcx = tcx.infer_ctxt().build(ty::TypingMode::non_body_analysis());
let obligation = ObligationCtxt::new(&infcx);
let lifetime = ty::Region::new_placeholder(
    tcx, ty::PlaceholderRegion::new(infcx.create_next_universe(), ty::BoundRegion {
        var: ty::BoundVar::from_u32(0),
        kind: ty::BoundRegionKind::Anon,
    }),
);
let concrete = if self.expected == "reject" { tcx.types.u32 } else { concrete };
let cause = ObligationCause::dummy();
let alias = Ty::new_projection(
    tcx, ty::IsRigid::No, view, [concrete.into(), lifetime.into()] as [ty::GenericArg<'tcx>; 2],
);
obligation.register_obligation(Obligation::new(tcx, cause.clone(), env,
    ty::PredicateKind::Clause(ty::ClauseKind::WellFormed(alias.into()))));
let resolved = obligation.normalize(&cause, env, Unnormalized::new(alias));
let errors = obligation.evaluate_obligations_error_on_ambiguity();
```

For the two positive inputs, the probe requires no obligation or region
resolution errors, a shared reference result, the expected integer referent,
and exact equality between the output region and the newly created placeholder.
It also asserts that this region is not `'static`. This is stronger than the
earlier `gat-upstream-query` experiment, which explicitly supplied `'static`.
The newer stdout records are:

```text
loaded schema schema::Family::View
schema::Family::View -> &u32 (region preserved)
```

and the corresponding `&u64` result for the other dependency metadata. For the
negative input, it substitutes `u32` for `Codec` and requires
`!errors.no_errors()`. It then prints `missing implementation rejected`.

The negative assertion proves an obligation failure was observed, but does not
inspect its reason. A reusable evaluator must distinguish the expected absent
`Family` implementation from ambiguity, solver overflow, unrelated
well-formedness failure or a malformed setup. The positive case also exercises
only an empty parameter environment and this concrete `Codec` family; preserving
one placeholder does not establish general lifetime escape checking, higher
ranked constraints or a native borrow-checking integration.

The query is invoked as `query SOURCE SDK SCHEMA_METADATA EXPECTED`, once for
each input, with `EXPECTED` equal to `u32`, `u64` or `reject`. Its constructed
compiler arguments include `--edition=2024`, `--crate-type=lib`, `--sysroot` and
`--extern schema=...`. The inspected folders contain source, statuses and
streams, but no complete executed-command/input manifest binding each archived
run to its exact SDK, query binary and metadata bytes. The source and logs
therefore support a bounded research observation, not independently replayed
package qualification. Fresh processes observing different metadata do not
demonstrate retained invalidation.

## The next service boundary

Keep the original fixture as the integration witness. Extend the one native
parse/resolve/definition path to represent the GAT's own parameters, predicates
and projection arguments. Bind these to stable native/dependency definition
identities. Assign the missing GAT operation from those facts before emission;
do not run either compiler first to discover which one fails.

The existing native representation is a starting point, not an implemented
transfer contract. Its
[`Region` and `Binder`](https://github.com/glopyglerky/lustr/blob/930b384ad03a26fe141d10be4e261bd82a60be76/compiler/native/src/ty/mod.rs)
already distinguish static, parameter, bound, placeholder, inference and erased
regions. `Binder` represents the documented rank-one scope, and
`Region::Placeholder(u32)` does not itself carry Rust's universe structure.
Native projections currently carry an associated-item identity and trait
arguments; the collector rejects the GAT's own parameters before completing
their meaning. There is no demonstrated adapter transporting this probe's
universe, binder and returned constraints into native shared state. Preserve the
working native region machinery, then explicitly represent or faithfully adapt
the missing information before native checking and before region erasure in
lowering. A printed type, copied numeric placeholder ID or pointer to an
upstream type cannot establish that equivalence.

The researched upstream candidate is a selective adapter to the pinned
[`rustc_type_ir::Interner`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_type_ir/src/interner.rs),
inference interfaces and
[`SolverDelegate`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_next_trait_solver/src/delegate.rs).
It is not the existing probe. The
[shared-solver design](https://rustc-dev-guide.rust-lang.org/solve/sharing-crates-with-rust-analyzer.html)
shows that a frontend can provide its own representations and query database.
It does not provide the Lustr adapter or prove its completeness or performance.
The feasibility and necessary source closure of the exact selective adapter
remain open. The named crates are research boundaries, not approval to import
them wholesale or replace the native solver for already covered projections.
The actual probe uses `ObligationCtxt`; the pinned
[fulfillment engine](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_trait_selection/src/traits/engine.rs)
selects its implementation from the inference context's solver mode. The probe
does not record that mode, so it must not be described as qualification of the
generic next-solver route.

The adapter needs a typed request and response, not a formatted `&u32` string:

| Boundary | Required facts |
| --- | --- |
| Request | Associated-item and impl identities, `Self` and all generic arguments, binder depth/universes, inference state, parameter environment, typing mode and relevant compiler/target context. |
| Definitions | Real generics, predicates, associated bindings, relevant and blanket impl sets, negative lookup observations and dependency identities supplied from the same authoritative native state. |
| Result | A native-consumable normalized type, substitutions, remaining obligations, region constraints, and distinct proven/ambiguous/overflow/no-solution outcomes. |
| Retention | Stable structural keys, complete dependency reads, valid response re-instantiation, and invalidation of native checks and artifacts that consumed changed facts. |

These requirements follow the pinned upstream
[`Response` and `ExternalConstraintsData`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_type_ir/src/solve/mod.rs),
which include inference values and external constraints alongside certainty.
`SolverDelegate` also requires operations such as well-formedness goals and
leak checking. Empty answers for those operations cannot substitute for faithful
adapters. A result retaining Rust's session-local `TyCtxt` handles is not a
durable native value; canonical results must be mapped back into the receiving
native context with their constraints intact.

Connect these reads to Lust's existing
[`ReadKey`/fingerprint graph](https://github.com/glopyglerky/lustr/blob/930b384ad03a26fe141d10be4e261bd82a60be76/compiler/native/src/driver/graph.rs)
and [generic reuse](https://github.com/glopyglerky/lustr/blob/930b384ad03a26fe141d10be4e261bd82a60be76/compiler/native/src/driver/gen_reuse.rs).
The latter already distinguishes recorded impl-set reads from changes that
clear the trait-system cache. Adding a global dependency fingerprint can be
a conservative correctness measure, but it does not pass affected-body reuse.
Record the queried associated binding and applicable impl set, including an
empty set; do not infer reuse from unchanged source text or object bytes alone.

After normalization, Lust must check and emit `fold`, `spare` and `main` with
the returned facts. Its applicable region/borrow checks still run. The selective
metadata service must eventually encode the actual GAT declarations, predicates
and dependencies when producing the original Cargo library. Stock-generated
provider metadata is suitable for the independent oracle, but cannot stand in
for an unimplemented candidate provider build. Record the imported source and
feature closure and exclude duplicate native semantic stages before product
admission.

## Concrete acceptance for this witness

These are proposed acceptance cases, not results of this review. Establish
expected outcomes with the pinned original compiler before running a candidate.
Keep ordinary Cargo/debug completion moving alongside this service; it need
not wait for general solver coverage.

| Case | Required observation |
| --- | --- |
| Cold unchanged family | The original dependency and consumer compile through the declared candidate route and print `45:10`. Ownership evidence identifies the imported GAT operation and native checks/emission for `fold`, `spare` and `main`. No second frontend checks those bodies. |
| Unchanged repeat | One retained owner reuses the solved fact and native checked bodies. Record query/body activity, not only unchanged executable hashes. |
| Native body edit | Editing `fold` rechecks its affected native work without reparsing/checking the whole consumer through upstream Rust. The unrelated `spare` body remains reusable. |
| Provider `u32` to `u64` | Keep consumer bytes unchanged. In the same owner, invalidate the normalized projection, `fold` signature/body and affected call site; reject the two type mismatches. Preserve the last successful executable and do not execute it as this failed request's result. |
| Repair | Restore the binding or make an explicit valid consumer repair. Reuse correct unaffected state and match a fresh compile's output and diagnostics. |
| Missing impl and recovery | Remove the applicable impl, require the specific no-impl failure, then restore it in the same owner. An empty impl lookup must be invalidated when its candidate set changes. Ambiguity or overflow cannot count as this control's pass. |
| Lifetime constraints | Preserve a non-static lifetime in a native signature and reject an attempt to return that borrowed value as `&'static u32`. Include two distinct lifetime arguments and a case whose well-formedness depends on a nonempty outlives environment. |
| Same result, changed dependency | Change an unrelated provider definition or represent the same associated result differently. Revalidate real dependencies and retain unaffected native work when the typed answer is unchanged. |
| Stale-result control | In a private evaluator control, force reuse of the old `&u32` answer after the provider changes to `&u64`; the same acceptance evaluator must detect the incorrect acceptance or stale type. |
| Restart and failure | Fresh-process and retained results agree. Interrupted/failed service work cannot publish partial artifacts or replace the last successful semantic state. |

Measure public request time, query/adaptation costs and body activity as this
boundary lands. A pass would establish this one mixed service, not a full Rust
compiler or a speedup. If the adapter requires a complete competing frontend or
cannot carry the necessary constraints, redesign that boundary using this
witness before broad scalar widening.

## Archived identities and limitations

The following SHA-256 values were computed from the original files read during
this review. Names are relative evidence roles; original execution paths are
omitted. Hashes of raw logs include their original path text, even where the
description above uses generic names. Original files and failed predecessors
were not modified.

| Original source or stream | SHA-256 |
| --- | --- |
| `gat-upstream-binder/query.rs` | `be79de72b59c666c83d2739b7b439f09cc21c37e00ce9415ede204d690ecb1e6` |
| `gat-upstream-query/query.rs`, earlier static-lifetime probe | `00bd95f839e0a4112cf6f5af1561a5aed03c1420fce22612e5e12031b8ba31d5` |
| Both probes' `metadata_client.rs` | `605b92510e271b2b1054161cd817f625a4abd95f1ebe3eb3500da533bbb4eb90` |
| Base `schema/src/lib.rs` | `fc863a783d66413a3758479f45b82bbe64540e1d227646b4769264c640bc1ae2` |
| Edited `schema/src/lib.rs` | `711b8a34677a901b44841ee50fdbb1b672a5a1b61a250698e6c24124c0f769bc` |
| Diagnostic control `std-fixture/schema/src/lib.rs` | `da77b20568bbb3e4d9239f38dea0e58b5b67f43c4fde6afa60e5147180a6b562` |
| Base and dependency-edit `src/main.rs` | `5d6acca7951e990e30670b7aa2b17342a0bea1639dd3b955ad2d888e0127ba73` |
| Invalid-call `src/main.rs` | `736f25a29d460543373b7ae6e5abb27bb9d19e2e6ce261680b8a12f17ca8256d` |
| Consumer `Cargo.toml` | `05e6586d35e2d88604f2c2a9c16c126720cde812378bed2fb42520428e2fe539` |
| Dependency `Cargo.toml` | `afd26c15caae8544dbb764fd41146aa9919c0f79f772c5b963d8eaa49868c2bf` |
| Binder probe `u32.stdout` | `dec1ebabbb52f0431e194cb75d412c9bce0ad7526836f3b2dba40ac1d0a23293` |
| Binder probe `u64.stdout` | `86193230f712e195bff43b5ca2939e25547502122b93c9f5215b72c268720c46` |
| Binder probe `reject.stdout` | `3b843fbbd5ea2bbd55e96ff1e342e3a39533cb2a66733ed5deba2131bd56f05b` |
| Original compiler `base-original.stdout` | `0a495f7a70608b18f8a2df2127e236e9eb79a8317058267b49c0f6d37c14e613` |
| Original compiler `invalid-original.stderr` | `aa31f2e628caa901c3d4ab54180b3e0f15e18753bbb373534f861476460687d7` |
| Original compiler `dependency-edit-original.stderr` | `6f6d3c4c3c9927c9372ac23782f5f13c064670790981836da9545b5e02615b37` |
| Native `base-installed-cargo.stderr` | `08fa9d54635bbc32c8cb6a2ad235d681ae549cc1e4b0b96e6e9b458ec52bb2fe` |
| Native `base-native-frontend.stderr` | `68798075f4875c406f8f86f4e78e10b0703830da4ea9ac5cf75430cd9f0cf8f6` |
| Diagnostic control `std-native.stderr` | `a17fdb50a05883dbf6e21d1fcce5935f8cedc9867f335aa69e77243aab0465c7` |
| Earlier metadata-stub failure `gat-upstream-query/base.stderr` | `f3250adf75b4adb53d1abdaec3a148a36c6075987532d8ca54468a79d1f14e27` |

The binder probe's build, `u32`, `u64` and rejection-control status files all
contain `0\n` (SHA-256
`9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`);
their stderr streams are empty. The original compiler's invalid and
dependency-edit statuses contain `101\n` (SHA-256
`39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c`).
The earlier query's initial metadata-stub failure and the two direct native
rejections contain `1\n` (SHA-256
`4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`).
These archived statuses are evidence of the recorded experiments, not a new
hosted check or an assertion of complete source-to-binary provenance.
