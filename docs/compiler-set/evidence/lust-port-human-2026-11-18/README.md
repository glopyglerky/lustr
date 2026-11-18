# Lust-port human-report diagnosis

The hosted failure is preserved and remains unassigned. One unchanged local
run with the exact CI Rust/Cargo versions passed all 13 tool tests. One separate
logging-only diagnostic run passed the human-report test and retained its full
report. These successes do not erase or clear the hosted failure.

| Run | Source | Result |
| --- | --- | --- |
| Hosted Linux CI `44328101820`, job `134884752622` | Merge `c3513db6bc32a57ec4ae48bc0e0c504582d3a5b5`, identical full tree to `ff9c0de9d2a2c89edd8df48e74e64a5c9483f232` | Eight unit and four integration tests passed; the human-report assertion failed. Its report was not printed. |
| One original local run | Unchanged `ff9c0de9d2a2c89edd8df48e74e64a5c9483f232` | Eight unit and five integration tests passed. |
| One instrumented local run | `fc94bc2b892a7e34e3133d4c6fb1e19a2e3ee8b0` | Exact human-report test passed, four integration tests filtered; complete actual report retained. |

All local commands are bounded and retained with status/stdout/stderr. No
test was repeated until it passed. The four added diagnostic lines print the
fixture path, process status, stdout and stderr before the unchanged assertions.
The fixture, scanner and renderer are unchanged.

Both local runs use Rust 1.99.0, commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, and Cargo 1.99.0
`5f94df478`. This matches the hosted toolchain. The original Cargo.lock and
the full tool source snapshots are included, along with dependency build logs.
The instrumented report shows seven files, three findings, one parse error, and
two `attrs.unsupported` occurrences. Its stderr reports the same file/finding
counts. The tool binary hash is unchanged across the test-only modification.

`records.zip` includes the untouched hosted artifact ZIP and decoded full job
log, all local command records, source/binary identities, the complete
instrumented report, and frozen tool sources. `manifest.json` binds every
contained file. `python3 -B replay.py` checks these records without executing a
compiler or rerunning a test.

The native prefix tree changes because the diagnostic test lives under
`compiler/native`; the exact integration patch and candidate declaration were
updated. The native compiler's source files, runtime, TinyCC dependency and
NoChange candidate are untouched by this separate branch. Candidate source
checks pass and default release admission still refuses.
