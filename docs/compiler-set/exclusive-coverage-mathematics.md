# Known upstream coverage, one compilation, and the cost of integration

The [exclusive coverage contract](exclusive-coverage-contract.md) controls this
research. This follow-through resolves the distinction between knowing Lust's
subset in advance and discovering a particular request's semantic facts. It
adds source and CI findings to the existing [Lust review](upstream-lust-review-2026-11-19.md)
and [Rust review](upstream-rust-review-2026-11-19.md). It does not propose another
compiler architecture or claim that the complementary implementation exists.

Reviewed on 19 November 2026 against published Lustr
`d1328bc2df289835b7a48d72575a103154b3c9e3`, tree
`6713a608d50bc589232fecbabb498693af768103`. The local research base had that
identical tree. Refreshed upstream Lust `next` remains
`240e08d91d4aed0299639f6f11fd522912b248f9`. Upstream Rust `main` was
`c884ecfbabb04b04fa5054dafc67ec258ecde745`; blueFe `main` remains
`d3c5da6f3d9d097600c9b461064c76cbb345d4ef`. Lustr's protected Rust source pin is
still `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. Newer research is not permission
to change that semantic baseline.

The follow-up against published `a03b1e2d9787523c72efe628a72cb02afbfaa491`
refreshed the same upstream heads. It adds the combined-overhead and total-time
accounting below, separates first release from near-stock optimization claims,
and clarifies that ownership follows reviewed capability rules rather than
observed compiler success. These additions make no new speed measurement.

## What is already known without trying a customer's compilation

Upstream provides three different kinds of coverage evidence:

| Evidence | What it establishes | What it cannot establish |
| --- | --- | --- |
| [`src/capabilities.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/capabilities.rs) | A vocabulary of supported constructs, builtin operations, contextual restrictions and deliberate rejections, derived partly from implementation tables | A complete decision procedure for every arbitrary expanded program, target and artifact request |
| [`scripts/test-capabilities.py`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/test-capabilities.py) and its fixtures | Offline witnesses linking listed capabilities to actual accepted or rejected executions | Universal proof from a finite corpus, or proof that every listed method and every contextual combination is tested |
| [`tools/lust-port`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/tools/lust-port/README.md) | A planning inventory based on parsed syntax and manually classified feature rules | Production ownership for resolved methods, expanded macros or unknown constructs |

The scanner explicitly lacks type information, does not expand macro bodies,
and treats unmatched constructs as supported in its coverage curve. Its feature
selection is syntactic and uses Cargo metadata without a dependency resolve
graph. Do not run it as a compiler-selection preflight. Its approximation can
both miss contextual gaps and classify a whole file too coarsely.

There is also a concrete schema distinction. The current `capabilities --json`
producer emits sections such as `items`, `types`, `methods` and `rejected`.
The scanner's [`apply_capabilities`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/tools/lust-port/src/features.rs)
expects a `features` array containing feature IDs and statuses. Its README
describes a future producer using those IDs. The present output is not that
input. An inventory integration must map the actual schemas explicitly rather
than claim that a pipe between these two commands supplies authoritative routing.

The durable coverage register should bind each capability to the donor pin,
implementation and contextual predicate, relevant positive and negative
witnesses, known defects, and integration status. This work happens during
development and qualification. Production uses the resulting rules inside the
ordinary compilation, not by rerunning those witnesses on user input.

## Dispatch is part of the required compilation

Let `K` be the pinned capability rules and `F(q)` the real facts needed to
compile request `q`. A supported operation can be assigned using a predicate
`owner(K, F(q), operation)`. The rules are known in advance; names, macro
expansions, concrete types, instances, target layouts and dependency identities
come from this request's normal semantic work.

The useful implementation is a typed decision at the existing semantic
boundary. Keep the facts and checked state that produced it, then continue with
the assigned operation. Neither a subprocess that attempts compilation nor a
second parse/check pass is needed merely to select an implementation. A short
table lookup still has a cost. Zero selection cost is not a measured fact.

This distinction prevents two opposite errors. A feature-name table cannot
replace required resolution and checking. Conversely, saying that contextual
facts are necessary does not justify running the complete rustc frontend before
Lust. Every shared prerequisite query has one executing owner and reusable
results. Requests for an imported service must be narrow enough to preserve that
ownership.

Use the contract's separate sets for Lust support, actual complement and
unfinished integration. A documented Lust capability with a compiler defect
requires a repair and a retained failing witness. A failure is not authority to
shrink the claimed Lust subset or silently invoke rustc. A newly discovered
contextual limitation needs explicit upstream/source evidence and a coverage
record. Unknown support is an unresolved obligation, not an automatic
complement decision, invalid-program judgment or successful compilation.

An early unsupported construct in a mixed function does not authorize restarting
that function or crate under a complete second compiler. The continuation must
preserve completed facts and assign only the missing operations to their imported
owner. This is why scalar emission after whole-crate analysis was insufficient:
emission ownership did not account for earlier duplicated semantic work.

## Costs that an integration cannot hide

For matched Lust-covered work, separate unavoidable stock work from additional
costs:

```text
T_Lustr,S = T_Lust,S + H_dispatch + H_conversion + H_transport
                       + H_publication + D_duplicate - W_saved
```

All quantities refer to equivalent requests and artifacts. `W_saved` is actual
saved work, not an assumed future optimization. If it is zero, near-stock
relative overhead `epsilon_S` requires the sum of added costs to be at most
`epsilon_S * T_Lust,S`. This is especially restrictive for a retained edit that
already takes very little time. The [performance policy](practical-performance.md)
must keep an engineering noise allowance distinct from evidence of near-stock
relative speed.

For complementary work, the corresponding condition is

```text
T_Lustr,C - T_rustc,C = H_C + D_C - W_C
H_C + D_C - W_C <= epsilon_C * T_rustc,C
```

Copying unchanged algorithms preserves an implementation starting point. It
does not establish this inequality after adding conversions, changing data
layout, repeatedly initializing a service or broadening invalidation. A fast
Lust result cannot turn a failing complementary comparison into a near-stock
claim. Overall release performance is assessed separately below.

The historical policy admitted 10% overhead **or** 0.5 ms added latency, whichever
allowance was larger. For baseline time `L`, that permits

```text
T_candidate / L <= 1 + max(0.10, 0.5 ms / L)
```

At a hypothetical `L = 0.2 ms`, `T_candidate = 0.7 ms` passes the absolute arm
while taking 3.5 times as long, a 250% slowdown. The permitted ratio grows without
bound as the baseline tends to zero. Therefore this rule cannot by itself
establish near-stock speed. These numbers illustrate the rule; they are not a
new Lustr measurement.

Keep original measurements and historical decisions intact. A new near-stock
claim needs relative evidence. Retaining the existing engineering relative
budgets means the relevant paired upper bound on candidate/reference latency
must be at most 1.10 for Lust-covered warm work and 1.05 for ordinary
complementary work, separately. These budgets remain engineering choices, not
the user's numerical specification. Absolute timing uncertainty helps explain
why an experiment is inconclusive; it does not admit an arbitrarily large
relative regression. A wide interval means unknown, with the measurement plan
revisited before drawing a speed conclusion.

For an ideal serial baseline `T_R`, let fraction `p` be replaced with speedup
`s`, and let `h` be net added time divided by `T_R`. Then

```text
T_Lustr / T_R = (1 - p) + p/s + h
speedup = 1 / ((1 - p) + p/s + h)
```

The integration beats the baseline only if `h < p * (1 - 1/s)`.
As `s` grows, the remaining work and integration costs dominate. Even unlimited
speed on the Lust portion cannot overcome an arbitrarily expensive bridge.
If a fraction `f` must still execute serially, speedup cannot exceed `1/f`
under this model. Whole-Rust analysis that remains on every request creates
such a floor. Running the entire stock compilation and adding native work
cannot produce a speedup unless some original work is actually displaced.

These are conditional accounting results, not claims about measured Lustr
performance. Real Cargo builds are dependency graphs. With `P` effective
workers, weighted work `W` and critical-path length `L`, an ideal schedule has
`T >= max(W/P, L)` before resource contention and scheduling overhead. Avoid
adding bridge barriers to the critical path. CPU-time sums and ratios of source
lines do not substitute for elapsed-time measurements.

### The two subset budgets do not guarantee overall acceleration

For the same disjoint serial baseline, let `epsilon_S` be added cost relative
to stock Lust on its work, `epsilon_C` the net relative added cost on all
remaining baseline work, and `h` residual net added cost normalized by `T_R`.
Include unchanged shared prerequisites once in that disjoint baseline. `h`
contains only additional shared, transport or publication cost beyond the
baseline, excluding costs already represented by either epsilon. A strictly
complement-only epsilon requires a separate unchanged shared fraction instead.
Do not add inclusive spans twice. Then:

```text
T_Lustr / T_R = (1-p) * (1+epsilon_C) + (p/s) * (1+epsilon_S) + h

overall speedup requires:
h + (1-p)*epsilon_C + (p/s)*epsilon_S < p*(1-1/s)
```

The right side is the time Lust saves relative to rustc. The left side is the
integration cost that consumes that saving. As an illustrative arithmetic case,
`p=0.01`, `s=100`, `epsilon_S=0.10`, `epsilon_C=0.05` and `h=0` yield a candidate
ratio of `1.03961`. Both provisional subset budgets hold, but total time is about
3.96% worse. This is not a Lustr benchmark. It demonstrates why neither separate
budget can replace the complete-command comparison.

The latest release direction accepts a correct full replacement once it is
faster overall. Matching stock Lust remains the optimization goal. Therefore
keep the overall result, the near-stock claims and correctness/ownership as
separate decisions. A missed engineering subset target does not impose a new
release hold, and a passing aggregate does not make that target achieved.

### Define overall work before measuring it

For a serial mix with frequencies `w_i` fixed before sampling:

```text
R_total = sum(w_i * T_rustc,i) / sum(w_i * T_Lustr,i)
```

Estimate uncertainty on matched total-time blocks while preserving paired
measurements. This is not the geometric mean of unrelated case speedups. For
example, changing one 1 ms operation to 0.5 ms and one 1,000 ms operation to
1,100 ms gives a geometric mean speedup of about 1.35, yet total time grows
from 1,001 ms to 1,100.5 ms. The example is mathematical, not measured evidence.

Choose representative workloads and frequencies before seeing results. Include
public-request costs and real equivalent artifacts. Preserve every class,
failed operation, slower result, latency tail and scope limit. A concurrent
Cargo workload uses its measured complete dependency-graph wall time instead
of a sum of overlapping phases. Hash-bound receipts can detect changed inputs;
they cannot by themselves prove independent execution, truthful timings or that
the workload was declared before the campaign. Those need the actual retained
execution and review records.

## Why retained facts matter more than changing the transport name

Upstream's [`driver/patch.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/patch.rs)
updates eligible bodies against retained resolver state. It falls back to
Lust's fuller resolution when the edit cannot preserve those facts. That is
valid recomputation inside the same owner, not compiler takeover. Its
[`driver/graph.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/graph.rs)
records precise semantic reads, including negative answers and stable identities.

The relevant history demonstrates both sides of this requirement:

- [Drop-check dependency repair `93e57035`](https://github.com/t3dotgg/lust-lang/commit/93e57035ed84f607e9b6402a6bf9a8370acffc0b) fixed stale acceptance. A broad layout dependency forced excessive rechecking; a dedicated drop-constraint fingerprint narrowed invalidation while retaining the needed dependency.
- [Shared answer table `1fca7814`](https://github.com/t3dotgg/lust-lang/commit/1fca781447632713a151733243259b3bbffc5972) reused answers between two dependency walks in one request while the program remains unchanged. Apply the same generation boundary across complementary services. Completing or changing definitions must invalidate any answers whose inputs changed.
- [Revert `89974f0d`](https://github.com/t3dotgg/lust-lang/commit/89974f0d41d045a1c0af73909ea2788cad39ebac) removed a plausible optimization after paired replay found no gain. [Revert `5a6f97c1`](https://github.com/t3dotgg/lust-lang/commit/5a6f97c1f5272239404ebd1c4cc1534ecc8a8e0d) repaired order-dependent instance naming found in longer retained histories.

For an edit, work should follow the affected semantic dependency closure. Missed
edges can miscompile; unnecessarily broad edges erase the speed advantage.
Rust's [incremental query design](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html)
similarly distinguishes changed inputs from changed query results. Neither
design allows a cached answer without its relevant dependencies.

The transport comparison is therefore conditional on retaining the same state:

```text
T_after - T_before = transport_change + extra_copying + extra_scheduling
                     + extra_revalidation + lost_reuse
```

Removing Unix sockets is useful when this total stays within the performance
contract. Replacing a warm worker with a fresh process can make `lost_reuse`
dominate. An in-process session or retained pipe protocol is a candidate, not
proof of improvement. `no_std` likewise describes available library facilities;
it does not itself remove parsing, allocation, I/O or semantic work. Follow the
recorded completion-first priority and measure any migration on the same retained
workload. No transport or `no_std` migration was implemented in this research.

## Minimal importing has a constrained meaning

Take the dependency graph under explicit targets, features and build options.
For chosen missing-capability roots, compute their least dependency closure,
stopping only at verified Lustr adapters. Graph traversal is linear in the
modeled vertices and edges. Choosing the best interface cut is a different
design problem: its costs include imported source, adapter maintenance,
conversions, initialization and critical-path work.

Minimizing file count alone can select an expensive adapter. Minimizing adapter
code alone can pull in a duplicate frontend. The admissible choice must first
satisfy complete semantics and exclusive ownership, then be compared using real
costs. There is no demonstrated globally optimal cut without those inputs.
Lust-supported operations and missing operations can depend on shared type or
metadata representations. Share those facts; do not import duplicate executing
implementations under the label of a dependency.

The existing consumer-metadata boundary remains the next concrete integration
obligation. Its definition, privacy, lint and emitted-instance facts must be real.
An encoder requiring those facts does not authorize unconditional analysis
success, fabricated metadata, an empty instance graph or a whole driver.

## CI history and the remaining evidence gap

The refreshed Lust workflow history contains one change,
[`21fa7e4f`](https://github.com/t3dotgg/lust-lang/commit/21fa7e4f0891f21b36ba7c937b88093bbcabc288).
The `scripts/ci.sh` history contains ten changes. These include a [cold-cache
correction](https://github.com/t3dotgg/lust-lang/commit/2511b82454f63d7a5fb0dca6a42963067ae8e9a1),
[warm/cold/Rust edit scenarios](https://github.com/t3dotgg/lust-lang/commit/9b21facf17910c45115d7c65b940e1ef69633d07),
and [macro/environment dependency invalidation](https://github.com/t3dotgg/lust-lang/commit/86ac7cbe3a0c1d7794fe6ef2d700ccf80bd87f06).
These are specific lessons to retain, not a generic instruction to trust a badge.

One qualification gap is visible in the call graph. The pinned workflow runs
[`scripts/ci.sh`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/ci.sh).
The capability probe runner is called by
[`scripts/agentkit-test.sh`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/agentkit-test.sh),
but `ci.sh` does not call either script. The probe runner checks eight named
sections and declares four exceptions; it does not prove all capability
combinations. Keep this explicit when connecting capability qualification.
The upstream [speed-row script](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/speed-row.sh)
also deliberately does not fail on slow ratios. Its successful exit is not a
near-stock performance gate.

Rust's refreshed [workflow](https://github.com/rust-lang/rust/blob/c884ecfbabb04b04fa5054dafc67ec258ecde745/.github/workflows/ci.yml)
and [job definitions](https://github.com/rust-lang/rust/blob/c884ecfbabb04b04fa5054dafc67ec258ecde745/src/ci/github-actions/jobs.yml)
distinguish pull-request, merge, try and performance builds. Ordinary try
configuration explicitly skips some component construction and tests. The
[performance process](https://rustc-dev-guide.rust-lang.org/tests/perf.html)
compares exact compiler revisions over fresh and incremental workloads.
Recent CI history also includes [macOS disk-space changes](https://github.com/rust-lang/rust/commit/5f8395784f2cfd94c643368b79143500e92c5006)
and the previously reviewed Windows ARM toolchain workaround. Infrastructure
failure and compiler regression require different evidence.

The historical Lustr [architecture allowance at `f42649f4`](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md)
permitted the full upstream dependency graph and judged imports mainly by work
executed. The same snapshot records whole-crate Rust analysis before restricted
native emission. That permission, public retry routes and acceptance centered
on compatibility allowed successful artifacts to obscure the ownership and
source-selection failures. The corrective contract removes the permission;
executable ownership, import and separate performance gates must prevent its
return.

This review inspected these named sources, source history and existing research.
It did not rerun compilers, qualify an optimized package, review all Rust history,
or prove universal language coverage. Its implementation consequence is concrete:
establish the upstream capability inventory offline, consume normal native facts
once, preserve their owner through emission, integrate only justified missing
services, and measure the two portions separately.
