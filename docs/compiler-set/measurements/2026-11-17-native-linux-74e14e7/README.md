# Complete Linux native gate at `74e14e7`

The Linux x86-64 job in [native CI run 44087679846](https://github.com/glopyglerky/lustr/actions/runs/44087679846/job/134065321686)
passed. This is the concurrently updated base of the combined candidate; it
does not qualify the later portability, library-selection or profiling changes.
The original three-member ZIP is retained byte for byte, with archive/member
hashes in [manifest.json](manifest.json). Its four native source-input hashes
also match the recorded Git source.

| Identity | Value |
| --- | --- |
| PR head | `74e14e7a14cb283423465f665adf7288e2088a88` |
| Actual merge checkout | `d0caf843d519763ae3b7bc8a3fdd1c26b12b307d` |
| Repository tree | `c15cdcb3481f2ee34651a8aa6c901d24df05ff8f` |
| Native tree | `9e6062a6f36047f9f6b8693026ce9a70d4a8e6ab` |
| Artifact | `14335034144` |
| Original ZIP SHA256 | `28691f2ac311cdf5d9d6d535cfee2584580e47f0d5f9357a42140a38716fc0cf` |

## Executed scope

The original log records 5,526 successful differential checks, followed by
the same case set with `SANITIZE=1`, both with zero failures. Sanitization
applies to generated C for accepted cases; these are not 11,052 unique programs
or 5,526 separately sanitized compiler binaries. The daemon suite records
363/363 successful checks and 27 explicitly unsupported versions. Native
macro-dependency, client, crate and `lust-port` suites report 13, 89, 85 and
13 passes respectively. Slot checks pass. The seeded edit fuzzer completes
1,200 edits with zero failures and zero guard firings, and its later smoke
suite passes.

The 58 macro-corpus results and 43 kernel/compatibility checks use the gate's
default Rust/Cargo reference mode. They do not establish that native Lust
supports those complete corpora. The separately named macro-dependency suite
does execute native work.

The benchmark smoke has one repetition. All its observations remain in the
original log, including slower cold and runtime cases. It tests benchmark
execution and supplies no paired speed-preservation conclusion. The independent
[fixed-fixture campaign](../../../../performance/2026-11-17-74e14e7.md) has the
matched comparisons and uncertainty. Do not combine its observations with these
smoke timings.

The native reference is Rust 1.99.0, commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM 23.1.1. Its identity differs
from the matching development SDK used in the mixed compiler campaign.

## Platform and source limits

The macOS ARM64 job was cancelled without executing any steps. GitHub reported
hosted runner acquisition/capacity failure. This Linux result cannot establish
macOS execution or an all-platform pass.

The combined candidate adds a Unix-address boundary fixture, bringing its
intended differential corpus to 5,527. This older base has 5,526; the counts
must not be copied between source revisions. Its native compiler binaries
are identified by the original SHA256 receipt but are not included in the ZIP.

```bash
sha256sum native-gate-original.zip
unzip -p native-gate-original.zip native-gate-identity.txt
unzip -p native-gate-original.zip native-gate-builds.sha256
unzip -p native-gate-original.zip native-gate.log
```
