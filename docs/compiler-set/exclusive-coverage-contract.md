# Exclusive Lust coverage and the Rust complement

Reaffirmed 19 November 2026. This is Lustr's canonical product contract.
The user's latest instructions control it. All current plans and implementation
claims must follow this document. Historical documents and measurements describe
their original sources; they do not supply competing instructions.

## One goal

Lustr replaces rustc for full Rust compilation. Lustr's Lust implementation is
the only compiler implementation used for work Lust covers in the requested
context. Lustr imports only the upstream code and required dependencies needed
for the remaining Rust work, and wires that support into the same compilation
of the unchanged program. The complete upstream Rust source stays in blueFe.

Preserve Lust's speed mechanisms. Keep the supported path close to matching
stock Lust speed and the complementary path close to matching stock rustc
speed. No large speed gain outside Lust's subset is required. Full coverage,
exclusive ownership, correctness and performance are separate obligations.

This goal does not authorize a stock-rustc backup, full-driver wrapper, language
selector, whole-program native-fail/rustc-retry route, or independent rebuilding
of every missing Rust feature. Moving duplicate pipelines into one executable
does not meet the goal. One owned compiler may use internal processes or a
retained worker; process count is not the criterion.

## Define the work before assigning it

A request `q` includes source and expansion results, resolved definitions,
dependency identities, edition and declared Rust semantics/SDK, target and ABI,
features, profiles, overflow/debug/panic settings, build-script observations,
proc-macro results, requested operation and required artifacts. Coverage cannot
be determined from a source keyword or file extension.

Let `U(q)` be the semantic, runtime-support, artifact and emission obligations
required for that unchanged request. Let `S(q)` be the obligations that the
pinned Lust implementation correctly covers in that context. Define:

```text
C(q) = U(q) \ S(q)
S(q) intersect C(q) = empty
S(q) union C(q) = U(q)
```

`C(q)` is the actual complement. It is not everything the current bridge fails
to carry. Let `B(q)` be the Lust obligations the bridge currently integrates,
with `B(q)` a subset of `S(q)`. Then:

```text
G(q) = S(q) \ B(q)
```

`G(q)` is unfinished integration. It remains assigned to Lust. A scalar bridge,
a missing ABI adapter or an incomplete metadata continuation must not shrink
`S(q)` or enlarge `C(q)` to claim completion. Record actual donor support,
contextual unsupported capabilities and integration gaps separately.

Each semantic result and emitted instance has one authoritative implementation
owner. Shared identifiers, representations and prerequisite queries may serve
both sets, but the work that computes them is assigned once. Calling shared
metadata machinery is allowed when needed; recomputing Lust-owned type checking
through a complete Rust frontend and renaming it a shared service is not.

The partition may occur at query, operation, item or monomorphized-instance
boundaries. Whole-file and whole-crate selection is insufficient when supported
and complementary work coexist. Generic instances, ABI, layout, drops,
unwinding, constants and cross-crate metadata must preserve their actual
semantic context.

## Admission, invalid programs and failures

Capability admission is a typed pre-emission decision using actual compiler
representations and required semantics. It must retain checked state rather
than check once for routing and then start a second compilation for generation.

| Outcome | Required meaning |
| --- | --- |
| Lust prepared | A supported obligation has checked state, identity, dependency reads and represented runtime/ABI requirements. Lust remains its owner through emission. |
| Complement required | An obligation absent from Lust has a justified imported owner in this compilation. This decision is not proof that unchecked source is valid. |
| Integration incomplete | Lust covers the obligation but the current bridge cannot compose it, or the required complementary implementation is not integrated. Report the gap honestly. |
| Invalid Rust | The authoritative checker reports the invalid program. Missing implementation cannot alone establish invalidity. |
| Implementation failure | Compiler, transport, invariant, emitter, object, link or publication failed. Fail the request. Do not reassign it to another compiler. |

Preserve diagnostics, exit status, signals, stdout and the last successful
retained state. A failed generation cannot be hidden as a cache miss. Never
retry a program or test that already ran. An independent stock oracle may run
in qualification; it cannot rescue a failed production request.

## Minimal justified imports

Choose missing-capability roots under an explicit target/feature configuration.
Let `D_0` contain those roots. Repeatedly add their required dependencies, except
where a verified Lustr adapter already supplies the obligation:

```text
D_(n+1) = D_n union required_dependencies(D_n, features, target, adapters)
```

The fixed point is the least closure for that declared graph and interface cut.
For a finite explicit graph, adjacency traversal costs `O(vertices + edges)`.
This does not prove global minimality among all possible compiler refactorings.
Changing an adapter, feature or target changes the graph and requires a fresh
closure review.

Every import needs a necessity witness that identifies its missing capability
or a dependency path back to one. Record exact donor repository/revision,
paths/symbols and hashes, licenses/notices, required features and targets,
dependency edges, substituted interfaces, real callers, semantic owner/query
boundary, excluded overlapping Lust phases, and qualification status. Include
build-time generators, referenced data and transitive dependencies.

An allowlist proves only that listed paths are present. An empty list proves
neither full coverage nor minimality. A path called `complement`, a shared
library instead of an executable, or link-time dead-code elimination does not
establish compliance. The whole Rust source tree and full `rustc_driver`
pipeline are not an acceptable shortcut for this import contract. If a chosen
root pulls in duplicate Lust-owned compiler stages, narrow the boundary or
supply a verified adapter before admitting it to production.

The contemplated machine-readable inventory is
`upstream/complement-imports.json`, schema `lustr-complement-imports-v1`.
It must distinguish actual partial source imports from retained research
components, and keep `full_rust_coverage=false` until full coverage is qualified.
The implementation change owns the file and enforcement. This docs-only change
does not assert that the manifest or guard already exists in default.

A small Unicode or normalization import is real partial reuse. It must remain
listed with its dependency and source identity. It does not establish a general
complement. Necessary compiler-private SDK dependencies used by independent bounded
mechanism/oracle qualifiers must be disclosed and excluded from product admission.
Delete obsolete whole-program alternate compiler launchers rather than retaining
them under a research-only name. Preserving source history and tests does not
require keeping a general compiler escape executable.

Rust's generic type-IR and solver interfaces provide a researched example of
selective reuse. They are a candidate interface, not a blanket instruction to
copy all shared crates. See the [Rust review](upstream-rust-review-2026-11-19.md).
Keep the full Rust source in blueFe as reference and extraction provenance.
Do not modify blueFe's pin implicitly through a Lustr change.

## Concrete metadata obligation already identified

The [recorded consumer-metadata probe](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md) shows that the genuine encoder
requires frozen definitions through Rust's `analysis(())` dependency. A native
prepared body alone does not discharge all declaration, visibility, privacy,
lint and diagnostic obligations. The next import/adapter must provide a narrow
completion service using real native checked proofs and applicable original
checks. It must preserve the actual definition and emitted-instance inventory.
Unconditional analysis success, fabricated typed-body results and empty
monomorphization graphs are prohibited. Qualification must retain traps showing
that Rust does not recheck covered consumer bodies or generate their MIR.
This existing work takes precedence over speculative imports from the upstream
research list.

## Retain the source of Lust's speed

Preserve the pinned frontend, daemon/worker, checked body state, stable
identities, precise semantic read sets, eligible body patching and affected-unit
emission. Imported complementary services must consume or supply the same
owned facts and declare their dependencies. They must not force a complete
competing frontend on Lust-covered requests.

For edit `delta`, let `A(delta)` be the affected dependency closure. Incremental
correctness requires complete invalidation; speed requires avoiding unnecessary
invalidation. Track bindings, types, impl sets, negative lookups, layout/drop
requirements, macro hygiene, included files, environment observations and policy
limits. Retain unchanged results only when all relevant inputs remain valid.
Reuse must survive cold/retained/restarted comparisons and longer edit/reversal
histories, not only the first edit. The [Lust review](upstream-lust-review-2026-11-19.md)
records upstream fixes and reverts that establish why these obligations matter.

## Performance mathematics and assessment

For equivalent supported work, write `T_Lustr,S = T_Lust + H_S`, where `H_S`
includes all added dispatch, validation, conversion, loading, transport,
duplicate work and publication costs. A relative allowance `epsilon_S` requires
`H_S <= epsilon_S * T_Lust`. Fixed overhead matters especially for fast retained
edits. Source preservation alone does not establish this inequality.

For equivalent complementary work, write
`T_Lustr,C = T_rustc + H_C - saved_work_C`.
Near-stock behavior requires `H_C - saved_work_C <= epsilon_C * T_rustc`.
This comparison must not be replaced by a gain on supported work.

For an idealized serial workload, let `p` be the fraction of baseline time
eligible for Lust, `s` its acceleration factor, and `h` added cost normalized by
the original total. Then:

```text
speedup = 1 / ((1 - p) + p / s + h)
```

`p` is a time fraction, not lines of code or syntax coverage. On parallel builds,
the dependency graph's critical path, resource contention and jobserver limits
must be measured. This equation is conditional reasoning, not a prediction or
a universal optimum. Full Rust analysis before native emission increases work
and can erase the gain even when selected objects come from Lust.

The [performance policy](practical-performance.md) retains the current budgets
without changing them after observing new results. Supported warm operations
use 10% or 0.5 ms added latency; ordinary complementary operations use 5% or
0.5 ms, whichever allowance is larger. These are engineering assessment
assumptions, not exact numbers supplied in the user's goal or evidence of a pass.
Apply them per operation with paired uncertainty. Other workloads remain
explicitly reviewed; do not invent passing results or hide regressions in means.

## Completion evidence

The full rustc-replacement claim requires all of these:

- Actual Lust coverage retained in pure and mixed programs, with no competing
  semantic compiler or emitter owning covered work.
- Full complementary support through the justified selective imports, with
  complete dependency, ABI/runtime and artifact contracts.
- Real unchanged Cargo projects, dependencies, macros, build scripts, generic
  instances, valid and invalid programs, and genuine downstream artifacts.
- Correct const/runtime/version behavior, drops, unwinding and target/platform
  behavior for every advertised profile.
- Complete ownership and incremental invalidation evidence across the boundary.
- Exact package/source identity and matched near-Lust/near-rustc results, with
  failures, tails, slower cases and limits retained.

A finite suite cannot prove all possible Rust programs correct. State the
qualified language version, targets, profiles and test scope. Neither current
source cleanup nor this contract alone delivers the full compiler.

## Conflicting directions are retired

The active direction excludes whole-Rust-source vendoring, a complete-driver
wrapper as the product, source-keyword selection, compiler-failure retry,
scalar-only ownership presented as full Lust support, independent reimplementation
of every Rust gap, and unrelated compiler/memory-safety rewrites as alternative
Lustr goals. Preserve their unique evidence in history and reuse compliant
components under this contract. The [disposition record](retained-work-disposition.md)
tracks the current cleanup without claiming it has already landed.
