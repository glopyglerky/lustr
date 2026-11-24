# Qualification checkpoint — 18 November 2026

Runs started on 18 November and were reviewed on 19 November UTC. These results
apply to the identified candidate checkouts; they do not qualify default `next` at
[`7cea76c9`](https://github.com/glopyglerky/lustr/commit/7cea76c9d3adc587b54ccff168fe07efafd40309).
The reviewed pull requests remain separate from that default revision. Follow
the [architecture plan](architecture-plan.md) and
[completion checklist](completion-checklist.md) for the required destination.
Successful fixtures do not prove correctness for all Rust programs.

## Source identities

| Candidate | Published head | Tested merge | Source tree |
| --- | --- | --- | --- |
| [PR28](https://github.com/glopyglerky/lustr/pull/28) | [0b89794c](https://github.com/glopyglerky/lustr/commit/0b89794c60a90b2a7d4a1db5e8e78abb6ecdf281) | [f0d7ac79](https://github.com/glopyglerky/lustr/commit/f0d7ac791b3814268111ca58c18525ff295ee6b5) | `0c131afe40f98599474e61ce5d319a727fcc06db` |
| [PR29](https://github.com/glopyglerky/lustr/pull/29) | [a9fd4c63](https://github.com/glopyglerky/lustr/commit/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d) | [3d23ce4d](https://github.com/glopyglerky/lustr/commit/3d23ce4d740a1022bcd3f0dae60c6c4a79853099) | `a1a28f2d9c9271a5f0d04349d5dd3657c59f04fe` |
| [PR30 continuation](https://github.com/glopyglerky/lustr/pull/30) | [b94d3713](https://github.com/glopyglerky/lustr/commit/b94d3713616804fead3f30b18a530e2e81a9510b) | [ac79476d](https://github.com/glopyglerky/lustr/commit/ac79476daf598205e1cf1d034788c770d13aa448) | `f942f18b548809616e82de8e5ed761a114b7b621` |

PR28/29 use native integration tree
`072691a23b7a5f664d3c07500315de3ce051a22f`; their hosted source-built native
component identifies itself as `b6f53a3866241ffc`. PR30 changes the native tree
to `02ca7e6710f1a1bb7dca842d30eecf7c289a3abb`. Earlier passes therefore cannot
qualify PR30's changed implementation.

## Completed on PR28 and PR29

| Gate | Observed result and limit |
| --- | --- |
| Linux native CI | Both heads passed 18 client and 400 core unit tests, 5,528 ordinary and 5,528 sanitizer differential cases, 364 daemon scenarios, and 1,200 fuzz edits without failure. |
| Source-built mixed package | Normal release constructors, real mixed Cargo execution, retained checks with fresh Rust metadata, mixed libraries, and all 17 public-entry tests passed on both heads. This qualifies the tested bridge and compatibility paths. |
| Retained error handling | Real native-output and link-recipe filesystem failures returned failure. Restoring each input recovered output `1080` with retained Rust reuse, without hiding the failure through successful fallback. |
| Schema 3 profiling | Each instrumented qualification completed 19 modes and 30 invocation streams: 36 emitted native responses and one failed response. All stream hashes and summary bindings matched. **No complementary-response runtime event occurred in this workload.** |
| Compiler-set coverage | PR29 passed all ten jobs, including macOS/Linux/Windows Cargo compatibility, semantic inspection, exact candidate-source checks, actual Lust-subset qualification and Cranelift unwinding qualification. |

Primary results: [PR28 Linux native](https://github.com/glopyglerky/lustr/actions/runs/44418360296/job/135204490428),
[PR29 Linux native](https://github.com/glopyglerky/lustr/actions/runs/44420439581/job/135211657133),
[PR28 mixed artifact](https://github.com/glopyglerky/lustr/actions/runs/44418360328/artifacts/14482996370),
[PR29 mixed artifact](https://github.com/glopyglerky/lustr/actions/runs/44420439480/artifacts/14484610457),
and [PR29 Compiler set](https://github.com/glopyglerky/lustr/actions/runs/44420439480).

The real retained failure/recovery qualifier is no longer merely compiled or
awaiting its first hosted execution on these Linux revisions. Complementary
outcome tests and the broader architecture remain separate coverage questions.
Instrumented profiles explicitly do not constitute paired speed evidence.

## PR29 macOS: setup repaired, native correctness incomplete

PR28's exact sanitizer-version mismatch and Cargo process-test failure remain
recorded failures. PR29 passed LLVM 23.1.6 selection, all four sanitizer capability
probes, and macOS Cargo compatibility. The ownership regression demonstrated a
test-directory defect; it did not prove the cause of the original hosted Cargo
failure.

PR29 native macOS passed 18 client and 391 core unit tests, 5,528 ordinary
differential cases, and 364 daemon scenarios, with 30 unsupported versions
reported separately. Its strict main-return gate passed 33 commands across four
transitions.

The [sanitized differential](https://github.com/glopyglerky/lustr/actions/runs/44420439581/job/135211656703)
then reported **5,481 passed and 47 failed**. All reported failures involve calls
through incompatible `void (*)(void *)` callback types, including concrete drop
functions and `lust_string_drop`. This is distinct from the repaired main-entry
types. The [artifact](https://github.com/glopyglerky/lustr/actions/runs/44420439581/artifacts/14486413900)
retains both the failure and passing main-transition report. The later fuzz stage
was not reached; the separate base-image-lifetime daemon check was skipped.
PR30 proposes a callback repair, whose qualification is tracked below.

## Declared SDK and frozen native oracle

The imported SDK uses Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` (1.102 nightly). The frozen native
differential contract uses Rust 1.99.0,
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`. These identities are not
interchangeable. Correcting the earlier accidental SDK-oracle substitution
restored the intended frozen comparison; it did not establish compatibility
with every behavior of the declared SDK.

The separately retained Unicode const/runtime mismatch remains a correctness
issue. Two demonstrated predicate disagreements are witnesses, not an exhaustive
inventory of semantic gaps. **Version-specific differential reconciliation is
still required** across the declared SDK, native semantics and standard-library
data. Preserve original disagreements and explain intentional version changes;
do not silently relax assertions to make the versions agree.

## Performance evidence

Fresh source-bound measurements exist for both successful mixed-package jobs:
each paired report contains 12 pairs, 1,135 records and 36 summaries. Independent
replay reproduced PR29's published tables and numerical summaries, but the runner
explicitly grants no performance admission. Its scope is a scalar native corpus
and a real mixed Cargo graph, not all Rust. Practical numerical acceptance is assessed separately in the
[completion checklist](completion-checklist.md); exact Lust parity is unnecessary.

Two dedicated comparisons still produced **no admitted timing samples**:

- [Complete-package warm comparison](https://github.com/glopyglerky/lustr/actions/runs/44418360324): the frozen baseline passed 15 client and 387 core tests, then failed `unicode_props` with Unicode 18 instead of 17; 5,526 differential cases passed and one failed.
- [Shared-native entry comparison](https://github.com/glopyglerky/lustr/actions/runs/44418360410): the applicability guard rejected changed `compiler/native` source before baseline-entry compilation. This is not a fresh candidate runtime failure.

Neither those failures nor the [historical reporting replay](https://github.com/glopyglerky/lustr/actions/runs/44418360337)
replace qualification of the final integrated compiler.

## PR30 continuation: 18 November, 21:27 MST (19 November, 04:27 UTC)

| Current evidence | Disposition |
| --- | --- |
| [Native CI 44457024805](https://github.com/glopyglerky/lustr/actions/runs/44457024805) | Both hosts failed two C-generation contract tests before the full native gate: cross-shard drop inlining and empty-frame drop glue. Linux reported 398 passed/2 failed; macOS 389/2. One failure compares the new erased callback signature with the prior typed signature. The later main-return setup failures do not establish a new runtime failure. |
| [Compiler set 44457024620](https://github.com/glopyglerky/lustr/actions/runs/44457024620) | Source-built mixed qualification was still building. The Cranelift project comparison failed unit qualification under both routes before project timing. Earlier backend controls and Cargo compatibility passed. |

The callback repair is proposed, not yet qualified by these runs. Resolve the
failed contracts without weakening their purpose, then qualify the exact changed
source. Unicode reconciliation, the broader shared semantic/runtime boundary,
public integrated routing and final package acceptance remain distinct work in
the [architecture plan](architecture-plan.md). No result here proves all-Rust
correctness or authorizes transferring an earlier revision's speed or platform
claims to a later one.

## Later PR30 observation — 19 November 2026, 05:02 UTC

The initial PR #30 source at `b94d3713616804fead3f30b18a530e2e81a9510b`
subsequently completed the
[source-built mixed job 135334423280](https://github.com/glopyglerky/lustr/actions/runs/44457024620/job/135334423280)
successfully. Its mixed Cargo/edit/rejection, retained-check metadata,
aggregate/reference/generic-caller libraries, complete entry/runtime contract,
and profiling/reporting steps all completed. The run now has nine successful
jobs; the
[Cranelift project qualification job 135334423582](https://github.com/glopyglerky/lustr/actions/runs/44457024620/job/135334423582)
remains failed at the native project qualification step. The original 04:27 UTC
observation above is retained as the earlier state of this same run.

This later success qualifies the executed mixed boundary on that initial PR #30
source. It does not repair the separate native-unit failures or qualify the
newer Unicode, standard-library, cleanup, socket or owned-request changes in the
[19 November candidate continuation](design.md).
The new job's timing artifacts have not been substituted for the independently
replayed PR #29 performance checkpoint, and no new numerical speed claim follows
from observing job completion.

## Combined continuation — 19 November 2026

The [combined native record](native-combined-qualification.md) adds exact-source
Linux results without replacing the earlier hosted observations:

| Qualification | Result and source boundary |
| --- | --- |
| Full ordinary differential corpus | 5,540 passed, zero failed at `af0a281f2`, native tree `a3f37f8a4d2364757a3bebf476e530e6ddf4735a`, against declared Rust `5b7b37aef`. |
| Full unwind corpus | 65 passed in each of TCC and GCC exception-table modes at native tree `7dbb7ac001f81c17adbbbd030573220dadfc079a`. The preceding two 64-pass/one-failure runs remain recorded; only the obsolete hashbrown panic source path changed between those binaries. |
| Declared Unicode | All 1,112,064 scalar values match the declared Rust predicates and mappings on the combined source; const/runtime witnesses and generation provenance remain in the [Unicode record](unicode-semantics.md). Lexical NFC normalization remains a separate item. |
| Partial destruction | Both unwind modes pass the six focused fixtures and 332 independently calculated finite destruction scenarios, including the final hook and double-panic abort. |
| Unix socket ownership | 20 deterministic syscall cases in each mode match Rust; 57 simulated Apple helper controls pass. Actual macOS socket execution is not claimed. |
| Owning checked request | 200 driver/codegen/type tests and seven independently repeated ownership controls pass; the prepared request carries the same checked representation into emission. This does not complete general imported semantic ownership. |

Binary, library, fixture and harness hashes are recorded beside each result.
Original failed outputs remain preserved. The callback repair, Unicode repair,
SDK alignment and owning preparation changes have no new performance claim.
The complete changed candidate still requires its hosted native sanitizer,
daemon, source-built mixed and macOS gates, plus the remaining architecture and
installable-package work. A local finite-domain proof or full fixture pass does
not establish correctness for all Rust programs.
