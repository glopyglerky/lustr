# Exclusive ownership follow-through

Reviewed 19 November 2026 against PR #33 at
[`ec99578d`](https://github.com/glopyglerky/lustr/tree/ec99578d51c93674af3a1bcf9c182f4ad4b17d89),
tree `e12027bdacd70dbd27efc4ce23d74d9cdc5625b6`.
This continues the same correction and [canonical contract](exclusive-coverage-contract.md).
It does not create another compiler direction or claim a complete replacement.

## Remaining causes of drift

The root contract was already correct. Several lower-level directions and
acceptance paths still contradicted it.

| Finding at the reviewed source | Correction |
| --- | --- |
| The scalar bridge returned successful `Complementary` replies for runtime, type and ABI work that ordinary Lust could compile. | Return typed `IntegrationIncomplete`, status 4. Ordinary native execution establishes that division, `u128` arithmetic and unit values remain Lust-owned even where the bridge cannot compose them. |
| The product retained daemon could turn an internal panic or function-count invariant failure into a fresh compilation and return its answer. | Product requests receive one retained compiler attempt. Preserve the original failure and discard damaged state for a later request. No immediate or background second attempt repairs the current result. |
| Auto-discovered coordinator integration tests expected the retired complete-compiler routes to execute. CI avoided them with `--lib`. | Replace obsolete execution assertions with real refusal-entry controls, retain useful planning and independent reference fixtures, and run the full current crate test suite. |
| Crate READMEs still prescribed selectable rustc/Cranelift/BlueFe execution and the removed mixed executable. | Replace current instructions with the actual planning-only or inspection-only roles and immutable links to the former instructions. |
| The live mixed profiler still required the removed mixed compiler and a sibling `rustc-full`. | Retire the executable launcher while preserving the historical parsers, schemas and evidence replay. |
| The Linux sanitizer job found a 37-byte leak after a failing range check. | Stop range validation at its first panic. The later check cannot allocate a second panic payload or publish bounds after failure. |
| The macOS odd-path failure lost its case directory and daemon log during cleanup. | Preserve failed case files, daemon diagnostics and statuses in uniquely owned artifacts without changing the test outcome. |
| Cold and warm daemon fixture checks compared program output without requiring its exit status to match the independent Cargo oracle. | Require both the raw status and output to match, preserving matching nonzero outcomes and original failure evidence. |

The original sources remain reachable through the reviewed revision. Old
evidence retains its own source and scope. Fixing an acceptance requirement does
not reclassify an earlier failed run as passing.

## The bridge cannot define Lust's supported set

The previous scalar bridge represented only some native operations. Its
successful complementary reply effectively treated that represented set as
the whole of Lust's support. The corrected [native admission contract](native-admission.md)
keeps these states separate.

For the same source, the ordinary native compiler must first build and run the
supported witness. Asking the narrower object bridge to compose it must then
produce an explicit integration gap, with no object and no complementary
compiler selection. Repeated retained checks must preserve that gap. The wire
reader rejects both status-zero integration gaps and the obsolete successful
complementary tag.

This implements the existing distinction $G(q) = S(q) \setminus B(q)$. It does not
enlarge the complement $C(q) = U(q) \setminus S(q)$. No current bridge producer proves
that an obligation is absent from Lust's actual supported set.

## Acceptance must exercise the intended behavior

The [retired coordinator disposition](retired-coordinator-acceptance.md)
records the contradictory test failure and each preserved or replaced suite.
The current controls execute all three old CLI names and real Cargo extension
lookup. Configured executable witnesses detect accidental delegation. Compiler,
wrapper, engine and toolchain options cannot restore the retired route.

Independent stock-Rust fixture checks remain expected-behavior references.
They do not count as Lustr coverage. Product daemon controls are built with the
actual `exclusive-native` feature; the ordinary donor unit build cannot stand
in for those controls.

The [range-panic record](native-range-panic.md) preserves the original leak,
the actual helper and emitted-program observations, and the local sanitizer
limitation. The [failed-case retention record](native-crate-failure-evidence.md)
documents why the macOS daemon cause is still unknown and how the next failed
case will retain the missing evidence.

The cold and warm execution controls now reject identical output with a wrong
exit status. Matching nonzero exits remain valid when the independent oracle
returns the same status. Raw status mismatches, including timeout and signal
conventions, remain distinguishable in the archived evidence. Explicit
test-process exit codes exercise that handling without claiming real timeout
or signal execution.

The historical [phase-attribution record](mixed-phase-attribution.md) points to
the retired launcher at an immutable source revision. Its parser and renderer
functions and schema constants remain unchanged. Frozen replay preserves the
original failed result and labels reconstruction separately.

## Completed predecessor checks

The final hosted results for reviewed head `ec99578d` are known. The
[compiler-set workflow](https://github.com/glopyglerky/lustr/actions/runs/44493776102)
completed successfully, including fresh partial package construction, the
required real daemon entry control, selection checks and shared-service
component qualification. Those passes belong to that source.

The separate [full native workflow](https://github.com/glopyglerky/lustr/actions/runs/44493776529)
failed. Its Linux sanitizer run reported 5,546 passed and one failed fixture,
`std_declared_source_locations`, with the 37-byte leak investigated in the
range-panic record. The macOS crate run reported 84 passed and one failed
fixture, `odd-path`, whose warm daemon build returned a truncated-response
diagnostic. Neither original failure is relabeled as a pass. The new range
repair and failure capture need their own current-source hosted results.

## Combined-source verification

The [combined verification record](evidence/exclusive-followthrough-20261119/verification.json)
binds the fresh debug partial package to the complete compiled-input trees and
blobs. Its native subtree is `e0d5f908d528ec2048c717ae8ad89b6c52dd4dae`, the
canonical integration patch is `14ed7bb3cfa9c999ccefe745fd0b93d1b31e6005`, and
the native build identity is `4fbb2e5c41854f34`. Both protected upstream pins
remain unchanged. The integration qualification remains a candidate.

| Combined-source check | Observed result |
| --- | --- |
| Fresh debug partial constructor | Passed native client/core, public entry, native library, selection priming and source-library sealing. No full compiler payload is packaged. |
| Candidate source and import enforcement | Candidate guard and structural ownership check passed; seven source-drift controls and six forged-import controls passed. Full-replacement release admission correctly refused with status 1. |
| Actual constructed public entry | 56 controls passed; one real-daemon control unavailable locally because Unix sockets return `EPERM`. Hosted CI still requires that control. |
| Product-feature bridge tests | 13 passed, including ordinary native execution witnesses and explicit retained integration gaps. |
| Product-feature daemon tests | Two passed, covering one attempt, retained no-op reuse, edits, rejection, artifact preservation and next-request recovery. |
| Original range-location fixture | The completed package matches the declared Rust oracle's stdout, stderr and status. Its emitted C passes UBSan with the same output and no diagnostics. Local LSan remains unavailable. |

The first product-test build failed with local disk exhaustion before tests
executed. A further range-witness setup could not create its work directory.
Those original failures remain recorded. After removing only reproducible
generated intermediates owned by this construction, the focused test build
resumed on the same source and configuration and passed. The completed package,
receipts and original failure logs were retained. The range witness then ran
successfully for the first time. This is not a retry of a compiled user program
or a repair of a failed product request by another compiler.

Separate component records retain 39 passing ordinary coordinator tests,
2,205 range-helper cases, 11 runner/evidence controls, 36 historical-profile
tests and four replay-corruption controls. Their scope and original sources
remain in the linked records above. None establishes full Rust coverage or
near-stock compiler speed.

The independent combined review found one CI ordering defect: macOS tried the
new retention controls before installing their GNU command prerequisites. The
workflow now runs those controls after prerequisite verification, without
changing the required command or other gates. No unresolved concrete finding
remained within the changed ownership, acceptance, tooling and documentation
scope. Current-source full Linux/macOS CI and performance qualification remain
separate requirements.

## Portability priority

Prefer `no_std` throughout the compiler and retained execution without a
required Unix-domain socket. Preserve Lust's speed and complete the compiler
first. The user permits deferring a migration that slows delivery or
compilation. This priority is recorded in the canonical contract, contributor
instructions and the existing completion checklist.

No transport or `no_std` implementation is claimed by this change. The current
compiler still depends on `std`, native host libraries and a Unix socket for
its automatic warm CLI route. A caller-owned pipe using the existing retained
session is a possible future adapter. It requires real operation and matched
performance evidence; a fresh compiler process per edit is insufficient.

## Remaining implementation boundary

Continue the [genuine consumer-metadata service](shared-consumer-metadata.md)
under the single completion checklist. Native checked declarations, privacy,
lint and definition completion, and the actual emitted-instance graph must
feed the encoder. Import only the upstream implementation and dependencies
required for missing obligations. Full Rust source remains in blueFe.

This follow-through removes further contradictory code and acceptance. It does
not implement the general Rust complement, complete all Lust-supported
integration, establish near-stock speed or admit a full-replacement release.
