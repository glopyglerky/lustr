# Avoid unused setup materialization on a retained check

> Source-bound checkpoint. Statements about the current candidate, required
> follow-up and measured speed below refer to the named source and date. The
> [active correction](active-correction.md) identifies present implementation
> work; the [current performance policy](practical-performance.md) controls new
> claims. Historical successes, incomplete paths and failed results remain scoped
> to their original implementation.

Status: isolated source candidate. Both focused tests pass on x86_64 Linux in
ordinary mode and in a fresh process with the resolve oracle enabled. The first
test setup failure remains preserved in the [focused evidence](evidence/native-nochange-2026-11-18/README.md).
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

These ran in the native `gate` profile with one compiler job and one test
thread, then in a new process with `LUST_CHECK_RESOLVE=1` so its process-wide
OnceLock was initialized for the oracle path. Both runs used the same test
executable. The `gate` profile is for correctness; its timings are not release
performance evidence.

The first run at source `3724211b45a3f8cc2bcf8539c8d549c3c2cbae7d` built
successfully and reported one pass and one failure. The internal-state test had
used a standalone const function, which takes the chunk frontend and creates no
patch base. Its attempted patch-base inspection therefore panicked before the
arranged state or the shortcut was exercised. The fixture now adds an unrelated
const item to enter the resolver and asserts that route before arranging state.
The original source, test executable identity and failing logs remain preserved.
This is a test setup correction; the product shortcut is unchanged.

The corrected run at source `8b68fee55b01c9bff6faa9be30612910fe6bb0f7` passed
both tests in each mode. The oracle recorded 20 successful comparisons: three
full resolves, seven reused checks and ten patches. Its stderr was empty.
The executable SHA-256 is
`3b98fee1feccf3c44770cfe6859b3aa6d5e07a907e8a9b79672b523ec2a81e36`.
The first run and both corrected runs preserve 387 filtered tests; this was
focused execution only. Existing unit, differential,
runtime/panic/drop, edit and public-entry gates remain relevant to admission of
the changed native source. Failures and unsupported operations must remain
visible. The candidate source guard checks the declared exact diff only.

## Performance obligation

The [fourth completed public campaign](evidence/warm-entry-2026-11-18/fourth-results.md)'s two untimed retained-check probes each
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
