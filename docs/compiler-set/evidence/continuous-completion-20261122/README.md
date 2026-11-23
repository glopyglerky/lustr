# Continuation and completion-plan review

Review date: 22 November 2026 in America/Denver. The source/CI observation below
was made on 23 November UTC. This is a source and existing-record review plus a
continuation-plan change. It is not a new compiler build, runtime rerun or speed
qualification. No compiler implementation, source pin, evaluator or CI admission
was changed by this planning increment.

## Source and current boundary

| Identity | Value |
| --- | --- |
| Repository and lane | `glopyglerky/lustr`, [PR #33](https://github.com/glopyglerky/lustr/pull/33), targeting `next` |
| Reviewed implementation | `bc38dece7a734b97c0108582b7f61f98ccb819db` |
| Root tree | `82afa54968f68dbbb5f99540fffc1e67ebc7d3d5` |
| Integrated native tree | `9f65da81325b5428f0e286425310216d8139fb49` |
| Preceding semantic implementation | `eb4d3d31f5ca3a5c6864bfab49634eadcaa43909` |
| Declared Rust donor | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Development checkout observation | Clean at the reviewed implementation; other work and running runtime processes were left intact |

`eb4d3d31` joins an upstream associated-type result to native signature resolution,
checking and emission. `bc38dece` changes the evaluator so metadata generations
come from the same Cargo dependency identity, then asserts that identity stays
fixed. This is real progress beyond the preceding query-only research.

The [qualifier](../../../../tools/native-projection-qualification.rs) loads a
genuine metadata snapshot, creates a fresh placeholder lifetime, requests
well-formedness and normalization, evaluates obligations and region constraints,
and requires a shared integer reference that preserves that lifetime. Native
signature handling substitutes the actual caller region. The native consumer
source goes only to Lust; the upstream research driver sees a metadata client
stub with local body typechecking and whole-analysis traps.

The [retained evaluator](../../../../tools/test-native-projection-retention.rs)
and inspected saved runtime records require:

| Step | Outcome | Native bodies checked | Upstream queries |
| --- | --- | ---: | ---: |
| Original provider | Accept | 3 | 1 |
| Identical provider metadata | Accept | 0 | 0 |
| Same-identity referent changes from `u32` to `u64` | Reject unchanged consumer | 2 | 1 |
| Implementation removed | Reject | 0 | 1 |
| Original provider restored | Accept; `fold` and `main` rechecked | 2 | 1 |
| Repeat repaired generation | Accept | 0 | 0 |

The recorded native executable and original Rust executable both print
`34 1`. The separate lifetime-escape case reports native `L0733` and publishes
no executable. The qualifier checks prior output bytes on rejection. The
projection identity stays fixed across the retained history, and `spare`
remains cached on repair. These observations were inspected, not rerun here.

The existing record hashes and source manifest identity are in
[review.json](review.json). The prior shared-service verification record names
`eb4d3d31`, whereas the stable-identity history's source manifest names
`bc38dece`. Do not transfer the earlier full shared-service outcome to a later
source merely because the integrated native tree is unchanged.

## What this does not establish

The mechanism is still a standalone research entry. It uses
`rustc_driver::compiler_entrypoint`, a full SDK dependency graph, a synthetic
native session, abort mode and disabled debug information. Its native entry
refuses Cargo manifests. Stock Rust makes the provider metadata. Only direct
function parameter/return projections to the proved shared integer-reference
family are represented; arbitrary projections and regions are not established.

The normal daemon still replaces its session when the imported generation is
no longer current. Production integration must validate and apply imported
generation changes within the existing Owner/Session while preserving unaffected
native state. Removing the freshness check alone would allow stale semantics.

The provider is unfinished as well. Native collection retains the GAT syntax,
but its checker rejects that declaration shape and discards the associated
item's where clause on the rejected path. The product service must validate
those predicates from the existing native parse. It cannot merely unblock the
syntax or keep a stock compiler producing the provider in the background.

These source facts lead directly to P1/P2 in the
[execution queue](../../completion-checklist.md#execution-queue). The research
proof is preserved as the regression witness for that integration.

## Default Cargo evidence and scope

The inspected separate default-profile workspace has no profile overrides.
Its saved native results are check `0`, build `101`, run `101`, test `101`.
The matching pinned Rust operations succeeded. The actual native compiler in
those logs is the earlier constructed `930b384a` package, not a newly built
`bc38dece` package. These are prior-package observations.

The current source independently confirms the remaining limitations:
`src/library_request.rs` and native request admission still refuse linked full
debug information, `src/lib.rs` does not admit `--test`, and retained transport
does not carry the donor test harness mode. Dependency-bearing unwind remains
restricted. The [execution queue](../../completion-checklist.md#execution-queue)
requires genuine debug and harness/runtime implementation before relaxing
those guards. The existing native test runner is not Cargo/libtest completion.

The [930b384a executable audit](../executable-unwind-review-20261122/README.md)
is published with this increment as previously completed, source-bound evidence.
It includes independently downloaded hosted records, extra runtime cases and
separate CFI observations. Its old next-step research discussion remains tied
to that source; the later retained proof supersedes that active next step.
Its results are not new `bc38dece` runtime qualification.

## Hosted observation

At **2026-11-23 01:57:19 UTC**, the seven PR-triggered workflows returned 21 jobs:
18 completed successfully and three were running, with no reported failures.
The pending jobs were the exclusive native/shared-service job in
[Compiler set](https://github.com/glopyglerky/lustr/actions/runs/45096398955)
and the Linux/macOS jobs in
[CI](https://github.com/glopyglerky/lustr/actions/runs/45096398931).
This is a timestamped status observation, not a claim that all checks passed.
No new-head result is inferred from it.

The Compiler set workflow cancels earlier same-PR runs on a new push. The
continuation rules therefore require batching pushes or doing the next slice
locally while collecting the needed candidate result. Cancelled evidence stays
cancelled; it cannot become a passing qualification for the successor.

## What the plan change fixes

The governing architecture already described one native compiler with selective
upstream services. Its work register lacked active ownership, dependency-ready
selection, concrete public acceptance and an exact resume action. Several active
status introductions and the PR description still presented `514dfe1e` or
`bc8a7adb` as current despite the later implementation.

The existing checklist now contains P0–P10, explicit source entry points and
acceptance, the parallel semantic/debug/test tracks, full language/artifact/
runtime/target work, matched whole-operation speed and release qualification.
AGENTS now requires advancing immediately after each verified slice. Ordinary
engineering failures, a successful push and pending CI are not global stops.
Runtime limits require a checkpoint and subsequent resumption; text instructions
alone do not establish background execution.

Task verification, bounded PR integration and full replacement release remain
separate decisions. The final release work must replace the current refusal-only
guard with independently verified positive admission, not flip a coverage flag.
No automatic merge or package release is authorized by this planning commit.
