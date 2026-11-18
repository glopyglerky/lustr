# Single-file full Rust check repair

The public `lustr check` command now checks valid Rust after a native refusal
without asking rustc to publish metadata to `/dev/null`. This fixes the real
`EBUSY` failure reproduced from
[upstream Lust issue 5](https://github.com/t3dotgg/lust-lang/issues/5).

Qualification on 18 November 2026 used Linux x86_64 and the pinned Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. The original source-built package was
preserved. Only its launcher was rebuilt and replaced in a separate diagnostic
package copy. The candidate is **not a fresh complete-package constructor
result**, and these checks do not establish a speed improvement or qualify
another platform.

## Failure and repair

The original single-file fallback invoked `rustc-full --emit=metadata -o
/dev/null`. The pinned compiler publishes metadata atomically; attempting that
publication at the device path failed with `Device or resource busy (os error
16)`. The issue's unchanged `local_inner_macros` reproduction passed the matching
Rust check with a real metadata path, and its built program printed `42`.

The repaired check creates an exclusive directory with mode `0700`, gives rustc
a metadata path inside it, and waits through the existing owned-child process
helper. It removes the directory after the child finishes, including a compiler
error or child signal, before returning that result. Successful cleanup disarms
the guard. If cleanup itself fails, a successful compiler result becomes an
entry error; an existing compiler failure or signal remains the primary result
and the cleanup error is reported. Guard drop supplies a best-effort retry.

The trace records `full-rust-core` / `owned-rustc-check`. Build still uses its
existing executable path; Cargo routing and the requested compiler settings
are unchanged. Check does not overwrite the caller's `-o` artifact.

## Actual Rust regression

The added
[`single_file_full_check_preserves_metadata_cleanup_and_errors`](../../tests/compiler_contract.rs)
test executes the real packaged compilers. It first checks the exact issue 5
source, then introduces an intentional type error in the same reproduction.

| Observation | Original launcher | Corrected launcher |
| --- | --- | --- |
| Valid source through public check | Exit 1, metadata publication `EBUSY` | Exit 0 through full Rust |
| Focused test result | Failed, harness exit 101 | Passed, harness exit 0 |
| Invalid source | The original test stops at the valid-source failure | Exit 1 and JSON error `E0308`, matching the Rust reference |
| Metadata cleanup | Not reached by the original assertion | Empty output-temporary directory after success and compiler rejection |
| Existing caller output | Not reached by the original assertion | Bytes preserved after both checks |

The separate [upstream reproduction corpus](upstream-repro-correctness.md)
also records the corrected package's seven cold-path cases and its explicit
warm-daemon infrastructure limit.

The test inventory grew from 16 to 17. This local run selected the new test:
**1 passed, 16 filtered out** on the candidate. It was not a rerun of all 17.
The first candidate test attempt incorrectly counted a retained mixed-cache
`request.lock` beneath `TMPDIR`; the fixture now stores the persistent mixed
cache separately. That failed attempt and the original compiler failure remain
in the private evidence bundle.

## Commands and identities

The following are the recorded commands with absolute workspace roots replaced
by named variables. `REPAIR_BUILD` is the original constructor's output root,
`REPAIR_ORIGINAL` its preserved `toolchain` directory, `REPAIR_CANDIDATE` the
separate diagnostic copy, and `REPAIR_EVIDENCE` a fresh results directory.
`REPAIR_SDK` names the matching pinned SDK; `REPAIR_CARGO_CACHE` is the populated
offline Cargo cache. The native-build receipt shown below is the original
constructor's actual stdout receipt.

```sh
export LUSTR_CORE_ROOT="$REPAIR_SDK"
export LUSTR_HOST_SDK_RECEIPT="$REPAIR_BUILD/HOST-SDK-RECEIPT.txt"
export LUSTR_LUST_ROOT="$REPAIR_ORIGINAL"
export LUSTR_NATIVE_BUILD_OUTPUT="$REPAIR_BUILD/work/native-intermediates/release/build/lust/dabdb40893bca91a/run/stdout"
export RUSTC="$REPAIR_SDK/bin/rustc"
export CARGO_HOME="$REPAIR_CARGO_CACHE"

"$REPAIR_SDK/bin/cargo" build --manifest-path "$REPAIR_REPO/Cargo.toml" \
  --locked --offline --jobs 2 \
  --target-dir "$REPAIR_BUILD/work/entry-target" \
  --config "build.build-dir=\"$REPAIR_BUILD/work/entry-intermediates\"" \
  --release --bin lustr --features source-native-runtime

# Green: the rebuilt launcher was copied into REPAIR_CANDIDATE/bin/lustr.
LUSTR_TEST_PACKAGE="$REPAIR_CANDIDATE" \
LUSTR_TEST_OUTPUT="$REPAIR_EVIDENCE/candidate-v2-green" \
"$REPAIR_SDK/bin/cargo" test --manifest-path "$REPAIR_REPO/Cargo.toml" \
  --release --features source-native-runtime --test compiler_contract \
  --locked --offline --jobs 2 \
  --target-dir "$REPAIR_BUILD/work/entry-target" \
  --config "build.build-dir=\"$REPAIR_BUILD/work/entry-intermediates\"" \
  -- --ignored --exact single_file_full_check_preserves_metadata_cleanup_and_errors \
  --nocapture --test-threads=1

# Red: the final, identical compiled test was also run against the old package.
LUSTR_TEST_PACKAGE="$REPAIR_ORIGINAL" \
LUSTR_TEST_OUTPUT="$REPAIR_EVIDENCE/original-final-red" \
"$REPAIR_BUILD/work/entry-intermediates/release/build/lustr-compiler/ae4260d69178f707/out/compiler_contract-ae4260d69178f707" \
  --ignored --exact single_file_full_check_preserves_metadata_cleanup_and_errors \
  --nocapture --test-threads=1
```

The original entry sources exactly match
[`43a7e07d32929309fc6e402d7bca6b5673e186a2`](https://github.com/glopyglerky/lustr/commit/43a7e07d32929309fc6e402d7bca6b5673e186a2).
The source hashes below identify the reviewed change independently of a later
commit or documentation edit.

| Item | SHA-256 |
| --- | --- |
| Original launcher | `7c4028ad4ed6a92e533b0227dd7772166ba9a041a6450a025a0f2891a90a9a2e` |
| Corrected launcher | `37b70ea7f0e77de79b22d47f6ff502a8ee75dabac2fe09e5099747a5a1d067a8` |
| Original `src/full.rs` | `e7015c5462da232fd817091c2749e4e776287832f1279ebaab87314f0644412e` |
| Corrected `src/full.rs` | `e8eb95bbdb2f413a6779ead7660906cea68adc49281357ffc22acdd7ac4ef86e` |
| Original `src/main.rs` | `5503c02cc66cee00a1398f5cededd0c155fcbe52a8d122bed6d314e5dbe36c1c` |
| Corrected `src/main.rs` | `1047de9580c55b0f20f5869c21a6fe89c3ad00818c64678d7699b21bcebbabba` |
| Added regression test file | `28211a790afd4daa4b97ed61c0cd55f6febb6f882c818803014b07fd2da7a9a7` |
| Exact upstream issue 5 source | `2b697b58774426fc1cdcfe222b067cd39626c89a9b8cd270224012f688eea4ea` |

## Changed-path timing

The corrected standalone fallback **misses the near-stock review budget** on
this fixture. One fixed 24-pair run compared public `lustr check` with the
matching `rustc-reference --emit=metadata`, using a normal writable metadata
file. Both checked the unchanged issue 5 source with edition 2024, the host
target, unwind panic behavior, overflow checks and debug assertions enabled.
The original failing `/dev/null` command is not a valid performance baseline.

| Command | Measured samples | Median | p95 |
| --- | ---: | ---: | ---: |
| Matching rustc metadata check | 24 | 15.46456 ms | 26.79065 ms |
| Corrected lustr standalone fallback check | 24 | 22.00132 ms | 42.00327 ms |

The paired median added time was **6.75538 ms**, with a conditional 95%
bootstrap interval of **4.52282 to 9.11705 ms**. The paired geometric lustr/rustc
duration ratio was **1.49234**, with interval **1.29434 to 1.72409**. Both the
5% relative and 0.5 ms absolute allowances fail. These are paired estimates;
the added-time estimate is not a subtraction of the two table medians.

The plan ran 58 commands: two initial controls, three excluded primes per lane,
24 alternating pairs, then two final controls. Only the 48 paired commands
contribute to the timing estimates.
Even pairs ran rustc first and odd pairs ran lustr first. All observations,
including high values, remain in the
[numeric timing evidence](evidence/full-check-metadata-2026-11-18/timing-24.json).
No further trials or optimization experiments followed this run.

`LUST_NO_DAEMON=1` was fixed for both lanes. `LUST_NO_CACHE` was unset; dedicated
`LUST_CACHE_DIR` and `LUSTR_MIXED_CACHE_DIR` paths were reused after priming,
without resets between measured commands. The source had no manifest, so the
Cargo selection cache was not involved. Timed commands had no trace or native
diagnostic recording. Separate controls before and after confirmed native
refusal followed by `full-rust-core` / `owned-rustc-check`.

Every command exited zero with empty stdout and stderr. After each command,
the caller's existing `-o` sentinel was unchanged and the metadata temporary
directory was empty. Every reference command produced a regular metadata file;
the runner removed that file after its timer stopped. Lustr's own temporary
directory creation and cleanup were inside its elapsed time. Source and binary
identities matched before and after the run.

The measurement used the exact v2 diagnostic launcher identified above on a
quiet Linux x86_64 host with nine visible CPUs and an AMD EPYC 9V45 processor.
CPU affinity, frequency and scheduling were not controlled. The intervals use
10,000 paired bootstrap resamples with fixed SplitMix64 seeds recorded in the
JSON. All 58 original command records and streams were checked, and both
intervals were recomputed from the 24 matched pairs before publication.

This is total standalone fallback latency, including native refusal, the mixed
full Rust driver, child-process handling and metadata cleanup. It does not
isolate the cost of spawning a child or measure general Cargo project
performance. It establishes a correctness repair and a remaining performance
gap on this path.

The separate private timing archive contains 133 verified files, is 51,992
bytes, and has SHA-256
`327b94ea1d8a25b9a4aef03001126b9c4b8c0894aa251e695e18908fb1f277b1`.
Public evidence includes all numeric observations, identities and validation
summaries. Original command environments, streams and source snapshots remain
private; their hashes alone do not allow replay of the omitted bytes.

## Synthetic process controls

Six additional controls replaced `rustc-full` with an explicit synthetic child
in another isolated package copy. They test process ownership and cleanup,
not Rust compilation. Each spawned child observed mode `0700`, a real metadata
pathname, and `--emit=metadata`; its stdout and stderr were preserved.

| Injected child behavior | Observed result |
| --- | --- |
| Exit 17 | Caller receives 17; temporary output removed |
| Child terminates itself with SIGTERM | Caller receives SIGTERM; temporary output removed |
| Child terminates itself with SIGINT | Caller receives SIGINT; temporary output removed |
| Exit 0 with deliberately denied cleanup permissions | Caller receives 2; cleanup failure reported |
| Exit 17 with deliberately denied cleanup permissions | Caller still receives 17; cleanup failure reported |
| Executable cannot be spawned | Caller receives 2; temporary output removed |

These signal controls target the **child**. External cancellation of the lustr
parent follows the existing immediate signal propagation path and can bypass
Rust destructors, leaving its private temporary directory. Cleanup under parent
cancellation or SIGKILL is not claimed. The harness restored deliberately denied
permissions and removed its synthetic leftovers after recording the outcomes.

[Sanitized verification data](evidence/full-check-metadata-2026-11-18/verification.json)
retains the statuses, source and component identities, and hashes of the private
original command logs. Those logs, failed first attempts and the synthetic runner
are retained privately; this public directory contains no package binaries,
private workspace logs or encrypted archives. The private archive is 62,871
bytes, SHA-256
`7310155eaa01c64269689175f7c1195711266aa78b195bb228cff988c3af988e`;
all 130 file hashes in its manifest were verified before retention.
