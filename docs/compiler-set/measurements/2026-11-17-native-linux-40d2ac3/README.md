# Failed Linux native gate at `40d2ac3`

The Linux x86-64 job in [native CI run 44096292187](https://github.com/glopyglerky/lustr/actions/runs/44096292187/job/134084162801)
**failed**. The ordinary differential suite passed all 5,527 cases. Its later
sanitized run passed 5,526 and failed one: the sanitized generated program for
`unicode_props_hash` exceeded its 10-second execution deadline. The gate stopped there; fuzzing,
benchmark smoke and the final completion check did not run.

[native-gate-original.zip](native-gate-original.zip) is the original three-member
GitHub artifact, preserved byte for byte. [manifest.json](manifest.json) records
its API digest, archive/member hashes, source identities, reached suite counts
and the failure. Artifact retention succeeded after the gate failed; that
does not turn the gate into a pass.

| Identity | Value |
| --- | --- |
| PR head | `40d2ac3f0b71abaa1e73cd366d8c43f8eb4d4581` |
| Actual merge checkout | `3b4b0bd67401085129d26be1afbd20e96c9b9d02` |
| Repository tree | `78318c4b01f8eb6f9518ea331efe69f0c457b1ce` |
| Native tree | `cb797ce8ce82192b87b3835267fc223c295dba70` |
| Artifact | `14341930542` |
| Original ZIP bytes | 14,217 |
| Original ZIP SHA256 | `6d7b9b3c51ff10ad8523ddec5bb261057ae8fe953269d149b25cd24a01084976` |

## Executed scope

The original log records these stages in order. Counts are the individual
suites' reported checks; they are not a total of unique programs.

| Stage | Recorded result |
| --- | --- |
| Release compiler build | Completed |
| Diagnostic registry | 112 codes in use, 108 reserved, 0 failures |
| `lust-port` | 13 passed, 0 failed: 8 unit and 5 fixture tests |
| Ordinary differential suite | 5,527 passed, 0 failed |
| Macro corpus, Rust-reference mode | 58 passed, 0 failed; fixtures cover 70/70 named macros |
| Native macro-dependency suite | 13 passed, 0 failed |
| Kernel and compatibility fixtures, Rust/Cargo-reference mode | 43 passed, 0 failed |
| Client suite | 89 passed, 0 failed |
| Crate fixtures | 85 passed, 0 failed |
| Daemon edit scenarios | 363/363 successful checks; 27 unsupported versions |
| Slot identity checks | All 8 logged checks passed |
| Sanitized differential suite | **5,526 passed, 1 failed** |
| Fuzzers | **Not reached** |
| Benchmark smoke | **Not reached** |
| Final `done` / `ci: all steps passed` | **Not reached** |

The first and only explicit failure in the retained gate log is:

```text
FAIL unicode_props_hash: sanitized build: sanitized build timed out after 10s
difftest: 5526 passed, 1 failed
```

The original diagnostic says "sanitized build," but the recorded
[`scripts/difftest.sh`](https://github.com/glopyglerky/lustr/blob/3b4b0bd67401085129d26be1afbd20e96c9b9d02/compiler/native/scripts/difftest.sh)
emits it for the status returned by `run_to "./$name.san"` after successful GCC
linking. This is a **sanitized generated-program execution timeout**, not a
compiler-build timeout. Its 10-second runtime deadline is separate from
`san_cc`'s 20-second compilation limit and existing `-O0` fallback.

The retained evidence does not establish the underlying cause, a semantic
mismatch, or that a retry would pass. The corpus, deadlines and failure remain
unchanged in this record. `scripts/ci.sh` exits on failure, which explains why
its later fuzz and benchmark stages are absent.

A separate [bounded diagnosis](../2026-11-17-unicode-sanitizer-diagnosis/README.md)
compares the relevant source and preserves one unchanged-settings local attempt.
That attempt stopped with LeakSanitizer's process-tracing limitation, a different
failure. It did not reproduce this timeout or qualify the sanitized program.

`SANITIZE=1` repeats the differential case set and sanitizes generated C for
accepted cases. These are not 5,527 separately sanitized compiler binaries.
The macro-corpus and kernel suites run in their default Rust/Cargo-reference
modes; their counts do not establish native Lust support for those complete
corpora. The separately listed macro-dependency suite does execute native work.

GitHub also reports that the separate workflow step checking the actual Unix
address helper with ASan and UBSan passed. That step precedes the gate and is
recorded separately in the manifest; its output is not one of this ZIP's three
members. The compiler-identity and artifact-upload steps succeeded after the
failed gate.

## Source, toolchain and qualification limits

The PR head and actual checkout have the same repository tree. The four
native source-input hashes in `native-gate-identity.txt` match the corresponding
blobs in the recorded checkout. The native subtree also matches PR12 head
`5a3f4937d94e8f028dd85385ea65d51fcfe35a3b` exactly. This equality identifies
unchanged native source; it supplies no passing result for either revision.

The native reference is Rust 1.99.0, commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM 23.1.1, with Cargo 1.99.0.
The recorded host is Linux `7.0.0-1017-azure`, x86-64. The matching development
SDK used in the separate mixed compiler campaign is a different reference
identity. The native compiler binary hashes are retained as original receipts;
the binaries themselves are absent from this ZIP and have not been freshly
hashed during recovery.

The macOS ARM64 job was cancelled without executing any steps. There is no
macOS result, complete Linux gate pass, all-platform pass or release admission
from this run. The failed native gate is independent of the separate mixed
compiler correctness and paired speed evidence. It produced no benchmark-smoke
measurements because that stage was not reached.

To inspect the retained bytes from this directory:

```bash
sha256sum native-gate-original.zip
unzip -p native-gate-original.zip native-gate-identity.txt
unzip -p native-gate-original.zip native-gate-builds.sha256
unzip -p native-gate-original.zip native-gate.log
```
