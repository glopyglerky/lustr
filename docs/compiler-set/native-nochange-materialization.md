# Avoid unused setup materialization on a retained check

Status: isolated source candidate. The new focused tests have not executed yet.
No compiler speedup, parity, runtime qualification or release admission is
claimed. This candidate is based on PR25 revision
`3c378a11d60149a3ec99fa49ea1b336e9406fca8`; its public selector and its existing
shared-native comparison guard are unchanged.

## Change and preserved state

The production resolver already reuses a successful check response when the
complete existing check fingerprint agrees and `find_edits` proves zero changed
files. Previously it first materialized a `Patched` value, then discarded its
source map, function-line and body-hash vectors, std vectors and table reference.

`PatchOutcome::ReuseCheck` now avoids that materialization. It returns only after
the original `apply` prefix, including `ConstReach.update` and `slot_update`, has
executed and the patch base has been restored. It carries the same function count
used by the original statistics. The unchanged response-rendering code is shared
by the materialized and minimal paths.

| Boundary | Requirement kept |
| --- | --- |
| Successful prior response | Existing `check_fingerprint`: same request/client/session; checks with C output and all non-check commands remain excluded. |
| Current source/context | Original root read and route, Cargo graph/unit selection, edition/lint gates, build-script observations, cfg and patch input fingerprint still run. |
| Retained patch validity | Original AST-compaction test and all `find_edits` conditions remain. |
| `find_edits` mutations | Joining/storing the local-name job and refreshing unchanged source stamps remain; the function is not entirely read only. |
| Const observations | `ConstReach.update(front, empty_edits)` runs during this request. It records nodes appended since its prior observation and advances the seen arena boundary. |
| Slot bookkeeping | `slot_update` still updates stale hashes and clears validity, output and read-set state where required. |
| Kept programs | Empty edits leave the original kept programs, kept-body history and edited-file history unchanged. |
| Oracle | `LUST_CHECK_RESOLVE` forces the original materialized path and its original oracle call. The item oracle's outer driver checks remain unchanged. |
| Response | Same cached response restoration, fresh visits, logging, statistics and timings. |
| Edits and builds | All edited requests, all builds and every ineligible check use the original materialized path. |

Deferring const observation until a later edit is not equivalent: that later edit
can blank nodes before their const roots have joined the retained reach sets.
The source preserves the observation before returning the minimal outcome.
This is why the shortcut is after the original empty-edit mutation steps rather
than immediately after `find_edits`.

The source-state argument assumes the original single-request retained-session
ownership and input-observation rules. It neither weakens those rules nor proves
that all existing native incremental behavior is correct.

## Focused qualification added

Two production-driver tests live in `driver::patch::tests`:

- `no_change_check_preserves_edit_error_restore_and_build` checks a retained
  response, a valid body edit, an invalid body edit and repeated rejection,
  restoration, a newly conflicting module file, recovery, and an ordinary build.
  Warm/cold diagnostics agree; both built programs must exit successfully with
  exact `180\n` stdout and empty stderr.
- `no_change_check_retains_new_const_observations_and_slot_updates` directly
  arranges a newly appended const-evaluation root and stale owned slot state
  after a real successful check. The next unchanged check must record the const
  root and perform the original slot cleanup. The test then blanks that appended
  root and exercises real const-body edits, rejection and restoration against
  cold sessions. The appended node and stale slot are explicit internal state
  controls, not a claimed reproduced source-level compiler defect.

Run these in the native `gate` profile with one compiler job and one test thread,
then repeat in a new process with `LUST_CHECK_RESOLVE=1` so its process-wide
OnceLock is initialized for the oracle path. The `gate` profile is for
correctness; its timings must not be used as release performance evidence.

The new tests are pending actual execution. Existing unit, differential,
runtime/panic/drop, edit and public-entry gates remain relevant to admission of
the changed native source. Failures and unsupported operations must remain
visible. The candidate source guard checks the declared exact diff only.

## Performance obligation

The fourth completed public campaign's two untimed retained-check probes each
spent 0.09 ms in the entire `reuse last answer` span, while the whole public check
median exceeded pristine Lust by about 0.307 ms. Those two probe values are not a
statistical attribution or an expected saving. They show why this small candidate
cannot promise parity by itself.

The current PR25 comparison composes two public launchers with identical native
runtime source. This candidate changes native code, so that comparison cannot
qualify it. A separately named comparison must build both the baseline and the
candidate complete packages from their respective source revisions, with matched
release profiles, SDK and TinyCC identities, package-specific environments and
daemons, actual constructor seeds, and the retained public runtime/edit gates.
Pristine Lust and Rust/Cargo references, raw samples, unsuccessful outcomes and
the strict Lust parity criterion remain required. No existing guard is relaxed.
