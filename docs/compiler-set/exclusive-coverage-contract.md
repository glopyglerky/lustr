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

The release threshold is a correct full replacement that beats matched stock
rustc overall on a workload declared before measurement. Publish that qualified
result with its exact scope and all slower cases; do not wait for perfect
stock-Lust matching. Near-stock Lust and complementary speed remain separately
measured optimization goals. Provisional percentage targets are not additional
user-imposed release blockers, and an overall gain cannot justify claiming an
unmet subset target. Full coverage and the ownership/import contract remain
mandatory regardless of the timing result.

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
required for that unchanged request. Let `K` be the reviewed capability rules
derived from pinned upstream source and offline qualification. Let `S(q)` be
the obligations those rules assign to Lust in the request's actual semantic
context. This is an ownership definition, not the set that happened to pass a
trial compilation. Correctness is a separate qualification obligation. A known
implementation defect does not remove the affected work from `S(q)`. Define:

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

### Known support without trial compilation

Maintain versioned capability rules from the pinned Lust implementation, its
supported contexts and development-time qualification. Apply those rules to
facts already produced by the normal native frontend or retained-state update.
This is part of compiling the request once. It is not a preceding pass that
compiles, parses or type-checks the request to choose a compiler and discards
the result. Preserve the resulting owner and checked state through generation.

The upstream `capabilities.rs` vocabulary and capability tests are inputs to
that evidence. The `lust-port` syntax scanner does not expand all macros or
know actual resolved types, and its unknown constructs can count as supported.
It cannot authoritatively classify production requests. Keep scanners and
capability probes in research/CI. No helper process, trial compile, diagnostic
retry, duplicate frontend or eager complementary-driver startup is permitted
merely to choose ownership.

Distinguish unexamined donor evidence from a demonstrated unsupported operation.
Unknown support does not establish membership in the complement. Resolve it
through source review and independent qualification before claiming coverage.
A defect in an advertised Lust-owned operation remains a Lust repair with a
retained failing witness. Do not shrink the supported set or transfer ownership
to excuse a compiler defect.
Normal required name resolution, macro expansion and semantic work cannot be
assumed free; assign each such operation once and retain its results. A static
feature-name table cannot replace those facts. See the
[support and cost analysis](exclusive-coverage-mathematics.md).

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

Qualification must use expectations established before candidate execution.
Matching warm and cold rejection of valid Rust cannot create an "unsupported"
exception or remove the operation from Lust's ownership. Preserve donor tests
and add independent checks for the exact result they leave unasserted. Count
excluded or unexamined histories separately from passing histories. Keep
compiler acceptance, required artifacts and generated-program exit/stdout/stderr
as distinct observations, so a compiler failure cannot stand in for an expected
runtime failure.

## Upstream implementations and dependencies

Choose missing-capability roots under an explicit target/feature configuration.
Let `D_0` contain those roots. Repeatedly add their required dependencies, except
where a verified Lustr adapter already supplies the obligation:

```text
D_(n+1) = D_n union required_dependencies(D_n, features, target, adapters)
```

The fixed point is the least closure for that declared graph and interface cut.
For a finite explicit graph, adjacency traversal costs `O(vertices + edges)`.
Use this analysis to improve the cost and size of an integration. Revisit
the closure when a changed adapter, feature or target changes its dependencies.
The pinned Rust driver can supply complete Rust operations while the
integration is developed and qualified. Record compatibility results that
still run Lust-owned operations through the Rust driver as unfinished
integration. When the artifact or runtime interface is missing, carry the
existing Lust result into that interface and qualify the actual owner and
work performed under the original Cargo settings.

Every import needs a necessity witness that identifies its missing capability
or a dependency path back to one. Record exact donor repository/revision,
paths/symbols and hashes, licenses/notices, required features and targets,
dependency edges, substituted interfaces, real callers, semantic owner/query
boundary, excluded overlapping Lust phases, and qualification status. Include
build-time generators, referenced data and transitive dependencies.

Source inventories establish provenance and the effective dependency graph.
Verify the implementation used for each operation through the actual compiler
entry. When integration duplicates work that Lust already owns, improve that
boundary and measure the effect.

Apply this rule to the effective Cargo dependency and feature graph, not only
the normal constructor command. Review defaults, optional and renamed packages,
target/build dependencies, feature unification and transitive lockfile changes.
A product-labelled library must not activate a full Rust SDK or an alternative
retrying ABI through a research feature. Remove obsolete general launchers and
retrying exported entry points; preserve useful bounded mechanism/oracle
qualifiers outside product admission. Structural package checks run before
construction and add no per-request compilation work.

The machine-readable inventory is
[`upstream/complement-imports.json`](../../upstream/complement-imports.json),
schema `lustr-complement-imports-v1`. It distinguishes actual partial source
imports from retained research components and keeps `full_rust_coverage=false`.
The [implementation cleanup](exclusive-coverage-cleanup.md) adds this inventory
and structural enforcement. Neither the inventory nor its guard establishes
full coverage, global source minimality or performance qualification.

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

The [performance policy](practical-performance.md) requires relative evidence
for future near-stock claims: 10% supported warm overhead and 5% ordinary
complementary overhead, assessed per operation with paired uncertainty. These
are engineering assumptions for those claims, not exact numbers supplied by
the user or additional release prerequisites. Their satisfaction does not
establish an overall speedup; complementary overhead can exceed the time saved
by Lust. Assess total public-request time on the fixed workload as well. The older
0.5 ms alternative remains part of its historical assessments only. For example,
adding 0.4 ms to a 0.2 ms baseline triples latency despite satisfying that old
absolute allowance. It cannot establish preserved near-stock speed.

Keep historical samples, thresholds, decisions and failures unchanged. Fix the
future assessment before measurement and report uncertain comparisons as
unqualified. Review cold work, other edits, tails and complete user operations
separately. Assess boundary overhead during integration, then qualify the exact
complete package. Do not postpone all speed assessment until coverage is done.

The paired total-time rule applies within each near-stock comparison as well
as across the complete workload. A geometric mean of repeated ratios can hide
increased elapsed cost even for one fixed request. Report means, maxima and
distributions alongside empirical p95. This changes the evidence required for
that speed claim; it adds no numerical release requirement.

## Portability preference without a competing destination

The user prefers `no_std` throughout the compiler and no required Unix-domain
socket transport, provided compilation speed is retained. The user also
explicitly prioritizes completing the compiler and permits deferring this
migration when it would slow completion or compilation. These are preferred
properties of the same compiler, not a replacement architecture or a new
release prerequisite.

Prefer `core` and `alloc` for portable compiler logic and explicit host adapters
for storage, files, processes and retained-session transport. A caller-owned
pipe or an in-process session is a possible host interface, not proof of a
completed portable compiler. Keep the same native checked state, dependency
invalidation and emission ownership across an interface change. Starting a
fresh compiler on every edit does not preserve the retained path.

The current engine still requires `std` and native host libraries, and the
automatic retained CLI route uses Unix-domain sockets. Socket-disabled cold
execution exists but does not replace retained-session performance. No full
`no_std` or socket-independent retained implementation is claimed. Any future
migration must show real supported operation and matched performance before
becoming the preferred production path. It must not delay the established
consumer-metadata and selective-complement work merely to add an unused API.

This preference concerns implementation dependencies and required compiler IPC.
It does not narrow accepted Rust programs. The full replacement must still
compile `std` and target-specific facilities, including Unix socket programs
where the declared target supports them. Removing the compiler's socket
transport is separate from removing those language/library semantics.

The [Rust Reference](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute)
defines `no_std` as changing the implicit library/prelude; dependencies can still
link `std`. A crate attribute alone cannot establish a whole-compiler claim.
Review the effective dependency graph and actual host adapters. Preserve the
same allocation, semantic-state lifetime, concurrency and error contracts when
measuring any proposed replacement.

## Completion and first release evidence

The full rustc-replacement claim requires all of these:

- Actual Lust coverage retained in pure and mixed programs, with no competing
  semantic compiler or emitter owning covered work.
- Full complementary support using the pinned Rust implementation, with
  complete dependency, ABI/runtime and artifact contracts.
- Real unchanged Cargo projects, dependencies, macros, build scripts, generic
  instances, valid and invalid programs, and genuine downstream artifacts.
- Correct const/runtime/version behavior, drops, unwinding and target/platform
  behavior for every advertised profile.
- Complete ownership and incremental invalidation evidence across the boundary.
- Exact package/source identity and matched end-to-end overall speedup over
  rustc, with uncertainty, failures, tails, slower cases and limits retained.
- Separate stock-Lust and stock-rustc comparisons for their respective work,
  recording achieved speed and remaining optimization gaps. The provisional
  near-stock budgets qualify those named claims, not first release admission.

A finite suite cannot prove all possible Rust programs correct. State the
qualified language version, targets, profiles and test scope. Neither current
source cleanup nor this contract alone delivers the full compiler.

## Conflicting directions are retired

Lustr delivers upstream Rust compatibility together with real Lust acceleration
for its supported work. Choose implementation boundaries using actual compiler
facts and preserve terminal failures. Retain the useful evidence and components
from earlier attempts. The [disposition record](retained-work-disposition.md)
tracks the current cleanup without claiming it has already landed.
