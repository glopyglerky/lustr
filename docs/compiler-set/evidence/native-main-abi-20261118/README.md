# Native main ABI focused evidence, 2026-11-18

The candidate fixes a demonstrated C function-pointer mismatch. It does not
qualify the native macOS platform. Every completed probe, incomplete attempt,
timeout and failed transition below is retained.

## Source and build identities

The test builds used native source from PR #16 revision
`34d847da3d90c3c2b253246a8ffdfdeb901c56b0`. Its native tree is
`12ae35ee30f81d11f55b184134f8a0c4a977a496`. The candidate changes only `cgen.rs`
in production native source. The final declared tree also includes the new
unit-main fixture and four-version edit scenario.
The candidate build's native source tree before those test additions is
`3ba77000c7b775a98c6fd39232a0aec1b4a92e7c`; the final declared native tree is
`cfc7a49a820f7fbeed2e8b725bf779ccedf641dd`.

| Identity | Baseline | Candidate |
| --- | --- | --- |
| `cgen.rs` SHA-256 compiled into the binary | `8ab4c3e87610597e5d9c339afce898973b8e7913a532aef97c7fec59638dccdd` | `1cbce6afffc8f3df96032b5588736cbd861033feba8111bec5047889472c12b1` |
| `lust` SHA-256 | `80441b744ffae7e4f225f17d2576f7e85e9a9b71eca2cd00ae12fd3d10e684b9` | `e5fbff6e3f43c2de446f7238fdabf8a42184a9208c55d0875b3a2e5c35499f7c` |
| `lust-core` SHA-256 | `eb8ecb7fe939dceb343c80f6ae62bdb6b3fb1fc5181de531734f7c1385ed509a` | `32adcf8caee9ec68c542a3b172707ad865099d9108cf28f1daf4d5fdd61743c3` |
| Reported compiler build | `8ec4a2ea` | `889774cb` |

Both compiler builds used Cargo's `gate` profile, two build jobs, Rust 1.99.1,
and pinned TinyCC `43c7708`. The host was macOS 27.0, build `26A428`, on arm64.
The hosted failure being investigated ran on a different macOS version. These
local results cannot substitute for that hosted gate. No compilation-time or
application-performance measurements were taken.

The Rust oracle executable SHA-256 was
`1ba7bb8e273bd64125f4048d2ecce530e0a6470e565ab1b3aecf0b82b532f7b1`.
The pinned Clang 23.1.5 executable SHA-256 was
`a22ab66d53cf70dfee8544eabdeb36f075bf942861cb8d59f98a3fadc7a65c26`.
Its official archive SHA-256 was
`503de30ef90311e0b143623801b19af2250adac6d69faed65ad0051170202e3b`.
The separate LLVM acquisition candidate records archive and provenance checks.
All four unchanged clean, address, undefined-behavior and leak capability
fixtures passed for that compiler/runtime configuration.

The source workspace was already patched when some preserved baseline binaries
were exercised. The raw probe's `identity.cgen_sha256` describes that execution
workspace, not the binary's compiled source. The public projection names it
`execution_workspace_cgen_sha256` and keeps the binary/source mapping above
separate. It does not relabel the original report. The earlier incomplete
baseline attempt lost its in-memory identity report; its emitted C, streams
and durable command result remain available.

## Method

The frozen [probe](probe-main-abi.py), SHA-256
`c1dea26606f27957fabcf81369f6d277e4c29d51167e6ba8b482e727641964e4`, copies
each fixture unchanged, compiles it with rustc, and builds it with the chosen
preserved native compiler. It then compiles every emitted C unit and links with:

```text
-O1 -w -Werror=incompatible-pointer-types
-fno-sanitize-recover=undefined -fsanitize=address,undefined
```

Execution uses `ASAN_OPTIONS=detect_leaks=1:allocator_may_return_null=1` and a
10-second deadline. No sanitizer is disabled. The probe compares exact numeric
exit status and stdout, rejects sanitizer diagnostics, and retains all stderr
bytes. It does not assert stderr equality. `LUST_RELEASE_C=1` exercises named
release-C declarations; it is not a release-profile performance experiment.
The optional unwind-table mode was not exercised.

The probe returns failure when a plain native run times out even if the
separately compiled sanitized C succeeds. Aggregate failures below remain
failures in [results.json](results.json).

## Minimal regression with pinned Clang 23.1.5

The new `main_unit_abi.rs` fixture has SHA-256
`421060bc0236cbd3e2c735f73312b1e3e4660bfd0492a021f1c874496b9a5b20`.
Both native compilers and rustc build and run its plain executable correctly.

| Generated-C execution | Exit | Outcome |
| --- | ---: | --- |
| Baseline | SIGABRT | UBSan rejects calling `f0` through `void (*)(void)` at `generated.u0.c:2496:3` |
| Candidate | 0 | Exact oracle stdout; no sanitizer diagnostic |

The generated definition returns `lust_Unit`. This is the focused failing-old,
passing-new check for the ordinary main call. It establishes that defect and
repair, not a count of repaired cases in the historical full suite.

## Nine unchanged fixtures, pinned Clang 23.1.5

All rustc/native builds and sanitized C compilation/linking succeeded.

| Fixture | Oracle exit | Candidate plain TinyCC | Candidate sanitized C |
| --- | ---: | --- | --- |
| `agg_inplace` | 0 | Matches | Matches |
| `adapter_closures_apart` | 0 | Timeout | Matches |
| `main_result_ok` | 0 | Matches | Matches |
| `main_result_err` | 1 | Matches | Matches |
| `exit_code_main` | 42 | Timeout | Matches |
| `exit_code_failure` | 1 | Timeout | Matches |
| `exit_code_result_ok` | 4 | Timeout | Matches |
| `exit_code_result_err` | 1 | Timeout | Matches |
| `panic_hook_abort` | 101 | Matches | Matches |

Thus nine of nine sanitized generated-C checks passed, while the combined
probe passed four rows and failed five. Each of the four ExitCode plain-runtime
timeouts also occurred on the identical fixture with the preserved baseline
compiler. The adapter timeout was already observed in the first baseline run.
The matching baseline ExitCode generated-C runs all passed with their exact
oracle statuses.

## Release C

| Fixture | Oracle exit | Candidate plain TinyCC | Candidate sanitized release C |
| --- | ---: | --- | --- |
| `main_unit_abi` | 0 | Matches | Matches |
| `main_result_ok` | 0 | Matches | Matches |
| `main_result_err` | 1 | Matches | Matches |
| `exit_code_main` | 42 | Timeout | Matches |
| `exit_code_result_err` | 1 | Timeout | Matches |
| `panic_hook_abort` | 101 | Matches | Matches |

All six release-C checks passed. This covers unit main, Result success/error,
ExitCode, and the actual hook runner without incompatible-pointer warnings or
sanitizer diagnostics. The two plain-runtime timeouts keep the complete probe
failed. No unwind-table or full panic/drop qualification follows from these six
cases.

## Earlier LLVM 22.1.8 configuration

Before the exact pinned compiler became available, the unchanged four
sanitizer capability tests passed with LLVM 22.1.8. Its compiler SHA-256 was
`5ff8c034eab4eef825e353dbb0d602b5422236e1e2db77fd45933a42c5ccffad`.

The first baseline probe reproduced the incorrect function-pointer diagnostic
on `agg_inplace`, then stopped on an adapter plain-runtime timeout before
writing a complete report. The first probe revision's source was not preserved
before the timeout-handling revision. Its generated C, partial output and
traceback are retained; no completed-campaign result is inferred.

The candidate LLVM 22.1.8 campaign completed all nine fixtures. Eight sanitized
checks passed. `agg_inplace` failed with an ASan deallocation error for 65,536
bytes, error code 22, followed by `sanitizer_common.cpp` reporting an unmap
failure. The plain-runtime results had the same five timeout cases listed
above. This campaign passed three complete rows and failed six.

The pinned LLVM 23.1.5 `agg_inplace` run later passed with empty sanitizer
stderr. These are separate compiler/runtime configurations, both retained.
The 65,536-byte allocation is consistent with the thread alternate signal-stack
allocation in `runtime/panic.c`; that correspondence does not assign the
failure's cause. This candidate makes no runtime or sanitizer-suppression
change.

## Retained return-type edits remain failed

The historical scenario runner was stopped after an original ExitCode scenario
hung. A one-second sample of the owned process contained 682 samples in
`main → lust_tls_exit → _sigtramp → lust_segv → lust_cur_thread_guard → pthread_getspecific`.
The raw sample is retained. This describes the observed stack, not a proven
root cause. Owned test processes were stopped.

Independent review also found that the historical runner can treat two
nonzero runtime results as unsupported instead of comparing the real status.
The new four-version scenario was therefore exercised with a strict runner
that builds first and compares exact exit, stdout and stderr against rustc.

| Version | Intended return | Oracle | Candidate warm / cold | Baseline warm / cold |
| --- | --- | ---: | --- | --- |
| v1 | `Result<(), &str>::Err` | 1 | SIGSEGV / timeout | Timeout / timeout |
| v2 | `()` after Result | 0 | Exact match / exact match | Exact match / exact match |
| v3 | `Result<ExitCode, &str>::Ok` | 7 | Exact match / timeout | Timeout / timeout |
| v4 | `()` after ExitCode Result | 0 | Exact match / exact match | Exact match / exact match |

Every compilation succeeded. Candidate v1's crash versus the baseline timeout
remains unassigned. These outcomes establish failure of the transition gate,
not correct retained Termination behavior.

The candidate run used runner SHA-256
`416d72cd651dd628ed9c4786ad5ca54b969dd42a8b93897c4fba3a677186e72d`.
Its retained-mode assertion incorrectly required `mode=reuse` for std-free
unit checks, which legitimately report `mode=bodies` with zero rechecked bodies.
The raw result marks all four rows failed and is unchanged. The baseline run
used corrected runner SHA-256
`1bfd44d5bc3970a67d65deea6b3e9486f3a9ac5432ab2cbed456866b1544fefc`; its raw
result passes the two unit rows and fails the two Result rows. These aggregate
counts are not a same-runner comparison. The exact runtime outcomes above do
not depend on that assertion difference.

The final checked-in runner SHA-256 is
`1f391308d2bb4671b211408282ef0ab7a446ccfc38a1450e78a477d9ff56879e`.
It additionally requires retained build statistics, preserves executable path
spelling for rustup proxies, records verbose oracle identity, writes a failed
setup report before optional file hashing, and retains partial timeout streams
before stopping an owned live process group. Cleanup failure revokes a pass.
The [six targeted local controls](runner-controls.json) pass for timeout output/status, unrelated process
ownership, reaped leaders, setup failure and cleanup failure. A proxy control
reproduces the broken resolved-`rustup` invocation and confirms preserved
`rustc` invocation spelling. These are runner controls, not native-platform
passes. The final runner has not been rerun through a native campaign locally;
hosted CI is required.

## Retention and replay

[results.json](results.json) projects all nine campaign records without private
paths or command arguments. [inventory.json](inventory.json) lists the exact
hashes and byte counts of 2,189 retained source, stream and report files.
[project-evidence.py](project-evidence.py) reproduces those projections from
the decrypted snapshot and original command receipts. It does not modify raw
outcomes.

The encrypted archive and receipt in `private-evidence/` retain the original
snapshot, command receipts, failed setup attempts, both executed strict
runners, local runner controls and proxy evidence. The inner snapshot archive
SHA-256 is `76c558bfeda7bb02024e54587dfc7fe37e8793a0d6c782b3714599fcf39e936b`.
It contains original generated C and fixture bytes. Program/compiler binaries,
object files and caches are excluded; their compiler hashes remain above and
the preserved binaries remain available on the test host.

Encryption uses the existing repository evidence-recipient certificate with
SHA-256 `69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f`.
The retention tool's coordinator-scenario fields are `not-observed` because
this is a native ABI investigation, not a coordinator benchmark campaign.

The [integration source guard](guard.json) passes in candidate mode and continues to refuse
release admission. Complete hosted source qualification, retained native
runtime correctness and performance qualification remain open.
