# Upstream Rust reproduction checks

The November 18 diagnostic passed all seven selected upstream valid-Rust
fixtures through public `lustr check`, public `lustr build`, and executable
comparison with the package's pinned `rustc-reference`. It also found and
reproduced a real single-file check defect before testing its focused fix.
The [numeric summary and source hashes](upstream-repro-correctness.json)
record the bounded result without publishing private raw command logs.

## Package and scope

The base package was built from PR #26 revision
`43a7e07d32929309fc6e402d7bca6b5673e186a2`, with upstream Lust pinned to
`240e08d91d4aed0299639f6f11fd522912b248f9`. The passing diagnostic replaced
only the public launcher with a separately built metadata-check fix. Native
executables, the native library, the full-Rust compiler, and the reference
compiler remained unchanged. This composition is **not a fresh complete
package constructor result**; the base constructor receipts belong to the
original package.

Every public command in the passing run set `LUST_NO_DAEMON=1` and
`LUST_NO_CACHE=1`. Source replacements therefore exercised separate cold
process invocations. There is no warm daemon, timing, or speedup claim in
this diagnostic.

## Cases and observed behavior

| Case | Original source | Reference stdout | Public result |
| --- | --- | --- | --- |
| Local inner macros | [Issue #5](https://github.com/t3dotgg/lust-lang/issues/5), first Rust reproduction | `42` | Check, build, and exact runtime parity passed |
| Trait method bound | [Issue #6](https://github.com/t3dotgg/lust-lang/issues/6), first Rust reproduction | `3` | Passed |
| Associated type bound | Issue #6's documented `Self::S` variant | `3` | Passed |
| Nested iterator closures | [Issue #7](https://github.com/t3dotgg/lust-lang/issues/7), first Rust reproduction | `[(1, 7)]` | Passed |
| Vector/slice equality | [Issue #8](https://github.com/t3dotgg/lust-lang/issues/8), first Rust reproduction | `ok` | Passed |
| `BTreeMap` indexing through `&&u64` | [Issue #9](https://github.com/t3dotgg/lust-lang/issues/9), first Rust reproduction | `10` | Passed |
| Clone/Copy bounds and `Reverse` | [PR #12's exact test](https://github.com/t3dotgg/lust-lang/blob/02030d86135e0444d504f3adf72765b169ebcd41/tests/pass/clone_copy_impl_bounds.rs) at `02030d8` | Full collections and all three `R::clone` side effects | Passed |

All seven valid fixtures first received a native refusal and then completed
through full Rust. The original native diagnostics and route events were
retained. Every executable returned zero with empty stderr; stdout matched
the pinned reference byte for byte, including newlines. The PR #12 source
was Git blob `2422a74c73b725078ee249d5b95bcac20bd13598`; the proposed
upstream compiler patch was not imported for this test.

For each case, the runner first established the reference result, then
tested the public command on the exact upstream source. It subsequently
replaced the same source path with a supported native control, the original
repro, an invalid return-type function appended to the repro, the restored
repro, and the restored supported control. All source transitions were cold
invocations. The native controls used `lust / native-library` directly;
the repros and invalid variants entered full Rust after native refusal.

The passing run contained **154 commands**: 112 compiler invocations and
42 executable invocations. Compiler results were 84 successes and 28
expected status-1 rejections. Seven reference executions established the
outputs; 21 public repro executions matched them exactly, and 14 native
control executions matched their fixed expected output. Invalid builds
published no fresh executable. Full-Rust checks recorded
`full-rust-core / owned-rustc-check`; full-Rust builds recorded
`full-rust-core / exec`.

An independent data check verified all 154 archived source snapshots, all
616 stdout/stderr/trace/native-diagnostic stream hashes, the frozen runner,
the component replacement receipt, and unchanged component identities
after execution. No compiler failure was retried within a run.

## Preserved failures and correction

The initial oracle incorrectly requested metadata output at `/dev/null`.
Pinned rustc rejected the device with `Device or resource busy (os error
16)`. After correcting the oracle to use a regular metadata file, the
reference checked and built the issue #5 source and printed `42`.
Original public `lustr check` still failed: its full-Rust route independently
used `/dev/null` for metadata output. This second attempt established a
product defect, and the runner stopped immediately.

The [focused launcher fix](full-check-metadata.md) gives standalone full-Rust checks a private
temporary metadata directory, waits for the owned compiler process,
removes the directory, and preserves the compiler's failure or signal
status. The associated actual-package contract test covers successful
check, invalid-source `E0308`/status-1 rejection, cleanup, and preservation
of an existing caller output file. The fix does not change the native
compiler, Cargo path, or successful standalone build route.

The corrected warm attempt passed cold check/build/runtime for issue #5,
then stopped because the environment denied the native daemon's Unix
socket bind with `Operation not permitted (os error 1)`. That daemon
exited 2 and no process was left running. The failed attempt is retained
separately; it does not become a warm pass through the subsequent explicit
cold-only run. Remote platform qualification must cover native daemon state.

These seven single-file cases establish specific compatibility and
rejection behavior. They do not prove arbitrary Rust programs, Cargo
workspaces, targets, or platforms, and they do not replace qualification
of the exact package eventually delivered.
