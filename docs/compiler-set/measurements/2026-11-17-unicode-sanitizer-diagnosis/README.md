# Unicode sanitizer timeout diagnosis

The single local attempt failed with a LeakSanitizer fatal error. It did not
reproduce the hosted timeout and supplies no passing sanitizer result. The
[original failed Linux gate](../2026-11-17-native-linux-40d2ac3/README.md)
remains failed. No compiler, fixture, deadline or sanitizer setting was changed.

## Original failure and its actual stage

At PR head `40d2ac3f0b71abaa1e73cd366d8c43f8eb4d4581`, the ordinary differential
suite passed all 5,527 cases. The later sanitized suite passed 5,526 and failed
`unicode_props_hash` with this original diagnostic:

```text
FAIL unicode_props_hash: sanitized build: sanitized build timed out after 10s
```

The [recorded differential script](https://github.com/glopyglerky/lustr/blob/40d2ac3f0b71abaa1e73cd366d8c43f8eb4d4581/compiler/native/scripts/difftest.sh)
emits that message for status 124 from `run_to "./$name.san"`, after GCC linking
succeeded. The ten-second deadline applies to the generated program's execution.
GCC unit compilation has a separate twenty-second limit and an inherited `-O0`
fallback. The original diagnostic's word "build" does not identify the stage.

The retained CI artifact omits generated C, the sanitized executable, GCC's
identity and per-stage durations. It does not establish whether GCC used the
fallback, which runtime operation consumed the deadline, or why this execution
failed after an earlier full gate passed.

## Source comparison

[source-comparison.json](source-comparison.json) binds twelve selected files to
Git blobs, full SHA-256 hashes and byte counts in four snapshots:

| Snapshot | Native tree |
| --- | --- |
| Pinned pristine Lust `240e08d9` | `0ab1b09815802894e6078de0af59273cbdf46cb0` |
| Successful native gate at `dec0ce2` | `0c196f3b8f0803cbf3985cc2f4be687129831e26` |
| Failed native gate at `40d2ac3` | `cb797ce8ce82192b87b3835267fc223c295dba70` |
| Existing local compiler source `b4ae69f` | `cb797ce8ce82192b87b3835267fc223c295dba70` |

The fixture, `difftest.sh`, `case-pool.sh`, `ci.sh`, Unicode predicate and escape
tables, core character runtime, formatting files and core character library
are byte-identical across all four snapshots. For example:

| File | SHA-256 shared by all four snapshots |
| --- | --- |
| `tests/pass/unicode_props_hash.rs` | `f68662c528f0a470b30856def6fade326b2154b4ebda5a0e5b83c8f70da28b7d` |
| `scripts/difftest.sh` | `e6553b35a7199ad61fb59274756494725c7cc47b3be4029eb820fa73b6d9bb6f` |
| `scripts/case-pool.sh` | `f90ec0c777b72751f91efb84a8169a00f5d35a1ffd591b46718d606eaeca12fc` |
| `src/runtime/core.c` | `f0de4382f2f126f3844a34410b7383b2294bdde0c056ea6f9da255e9c214d2ee` |
| `src/runtime/unicode_table.h` | `86e693d5339c8d90bd7e72c4c84f94617a347bc8dc73755f523f0fb243ad1e52` |

The native subtree differs between `dec0ce2` and `40d2ac3` only in
`src/resolve/cargo.rs`, the library-target selector correction. The case uses
an ordinary file build. Its source exhaustively visits 1,112,064 Unicode scalar
values and checks predicates, digit conversion, trimming and debug formatting.
Those source facts do not attribute the hosted timeout.

The whole compiler is not identical to pristine Lust. The comparison also
records changed code-generation files, the three changed runtime system files,
and the owned inputs outside the native subtree. Equal selected source bytes
do not prove binary equivalence or rule out every regression.

## One local attempt

The attempt used an untouched archive of the `40d2ac3` native subtree and ran:

```bash
SANITIZE=1 bash scripts/difftest.sh unicode_props_hash
```

The script's required `target/release/lust` and `lust-core` paths pointed to the
existing **debug-profile** binaries from the earlier `b4ae69f` focused package.
The directory name does not make them release binaries. Their native subtree
matches `40d2ac3`, but the owned `mixed_unit.rs` predates its profiling supplement.
The shared native-cache input policy is identical. These are not the original
CI compiler binaries. [identity.json](identity.json) records both executable
hashes, the source distinction, exact invocation and selected environment.

The local Rust oracle was 1.99.0, commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, matching both recorded native gates.
Local GCC was Debian 14.2.0, on Linux 6.18.57 x86-64. The original failed CI host
reported Linux 7.0.0-1017-azure. These are different execution environments.

The original script disabled the native daemon. All original limits remained
in force: thirty seconds per compile, ten seconds per run, and twenty seconds
for the first GCC unit compile. ASan and UBSan remained enabled, with the
original leak checking. The Rust-result cache used a new isolated directory.
The supported filter selected one case without changing its input or the corpus.
No compiler rebuild, altered-settings retry or sanitizer bypass followed.

The unmodified script returned exit 1 and printed:

```text
FAIL unicode_props_hash: sanitized build: sanitizer: ==5771==LeakSanitizer has encountered a fatal error.
==5771==HINT: LeakSanitizer does not work under ptrace (strace, gdb, etc)
difftest: 0 passed, 1 failed
```

[stdout](stdout) and [stderr](stderr) preserve the original outer streams.
[result.json](result.json) records their hashes, exit status and total elapsed
time. That elapsed time covers the whole script, not the sanitized program
alone. The ordinary Rust/native comparisons reached the sanitizer stage; the
sanitizer result was a fatal error, not the hosted timeout. The unchanged
script's exit trap removed its per-case temporary files, so this record does
not contain a separate complete sanitizer stderr file or generated executable.

The cause of the hosted timeout remains unresolved. Fresh hosted execution of
the full native gate is the remaining matching qualification. Neither this
source comparison nor the blocked local attempt converts the original failure
into a pass or establishes a host-load explanation.

`SHA256SUMS` covers every retained file in this directory except itself. The
record contains public fixture paths, selected tool identities and original
diagnostic output. It contains no full environment dump or private project data.
