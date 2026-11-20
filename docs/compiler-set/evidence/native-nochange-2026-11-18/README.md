# Native retained-check focused evidence

Two focused native driver tests pass on x86_64 Linux. This result does not admit
the complete native compiler, any platform, a release, or a performance claim.
The ordinary public-entry comparison's identical-native guard is unchanged.

| Execution | Source | Result |
| --- | --- | --- |
| First ordinary run | `3724211b45a3f8cc2bcf8539c8d549c3c2cbae7d` | Compilation succeeded; real source/runtime transition test passed; internal-state fixture failed before reaching retained state. |
| Corrected ordinary run | `8b68fee55b01c9bff6faa9be30612910fe6bb0f7` | Both tests passed, with 387 other tests filtered. |
| Corrected oracle run | Same corrected source and executable | Both tests passed; 20 successful resolver-oracle observations; stderr empty. |

The first fixture contained only a const function and a main function. That
selects the chunk frontend, which has no retained patch base. The correction
adds an unrelated const item, then asserts that the resolver path ran. It changes
the test setup only. The production change still returns after the existing
`ConstReach.update` and `slot_update` mutations.

The real transition test exercises unchanged checks, a valid body edit, invalid
code and repeated rejection, restoration, a conflicting module file, recovery,
and ordinary warm/cold builds. It checks successful process status, exact
`180\n` stdout and empty stderr for both built programs.

The second test explicitly arranges an appended const-evaluation root and stale
owned slot state after a real check. It checks that the next unchanged request
records the root and clears the stale slot state. After blanking the appended
root, it exercises real const-body edits, rejection and restoration against cold
sessions. This is an internal state control, not a reproduced application-level
compiler defect.

The corrected oracle run enables `LUST_CHECK_RESOLVE=1` in a new process and
uses the same compiled executable. This forces the original materialized path.
The log contains three successful full resolves, seven reused checks and ten
patches. The ordinary run exercises the new eligible shortcut.

## Identities and replay

The first native tree is `94c20e3781d0197ec79d0e0bc7eda10621af1a3c`; the corrected
native tree is `e6eea62c77095e60eaa7b1baf77055f0708ad54e`. The pristine reference
and its exact integration declaration remain explicit. Candidate source guards
pass; default release admission remains refused.

Both compiler builds use the `gate` profile, one compiler job and one test
thread. The Rust SDK is nightly-2026-11-09, compiler commit
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. TinyCC is the existing pinned upstream
revision `43c7708b85681a2fd4451c8a541af4494a8919b2`, built by the repository's
unchanged PIC builder. Its actual executable/archive hashes, builder logs and
source-archive hash are retained. No dependency implementation was substituted.

The first test executable SHA-256 is
`ac2e94603437dfa52678b4c43f1964b957ca0b9435c3d625287f6521b2e18838`.
The corrected executable SHA-256 is
`3b98fee1feccf3c44770cfe6859b3aa6d5e07a907e8a9b79672b523ec2a81e36`.
The executables are retained locally as independent files; this portable bundle
contains their identities, source snapshots and raw results, not binary copies.

Run `python3 -B replay.py` in this directory. The replay checks the archive and
every contained file against the manifest, exact command results, both successful
test summaries, the initial failure, same-binary identity, and every oracle log
line. It replays evidence checks; it does not execute a compiler or rerun tests.

The first source guard was compiled before a review-note-only qualification
wording update. Its recorded assertion covers the exact initial native tree and
candidate status. The corrected and final guards are separately rebuilt against
their recorded declarations. No source guard establishes runtime correctness.

## Remaining scope

The other 387 tests, the full differential gate, complete package/runtime gates,
macOS execution and matched performance have not run for this native candidate.
A separate experiment must source-build both complete release packages, preserve
their own environments and constructor seeds, and compare against pristine Lust
and the same-run Rust/Cargo reference. The focused correctness profile cannot
supply those performance compilers.
