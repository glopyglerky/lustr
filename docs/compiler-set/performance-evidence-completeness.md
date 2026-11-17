# Performance execution-record completeness

The performance reporter now verifies the measured execution sequence in both
retained campaign schemas. This strengthens evidence replay. It does not change
the compiler, benchmarks, estimator or any published timing.

An audit of the [published `5a3f493` reporter](https://github.com/glopyglerky/lustr/blob/5a3f4937d94e8f028dd85385ea65d51fcfe35a3b/tools/report-mixed-performance.py)
reproduced three gaps using in-memory
copies of the original schema-2 artifact. After updating the copied raw digest
and report records, the old reporter accepted each of these changes:

- Remove all 192 measured native post-build execution records.
- Reverse every declared lane order while retaining the original command order.
- Remove all 27 daemon-shutdown records.

The committed original archive contained those records. These counterexamples
showed missing verifier requirements, not missing execution in the recorded run.

## Required records

Each sample's order declaration must precede its measured commands. Those
commands must follow the declared lane order within each of the nine phases,
and phases must follow the recorder's order. Balanced permutations alone do
not prove that the commands followed them.

Each measured cold, no-op, body-edit and signature-edit build must immediately
precede its matching runtime record. The reporter checks the lane, source
digest, executable output path, successful exit and expected stdout/stderr.
Native builds bind the recorded `-o` output. The fixed Cargo corpus binds its
declared target directory's `debug/mixed-app` and application source. These
bindings do not add full Cargo-argument validation or retroactively impose
schema-2 native flags on schema 1.

Every measured native sample must record at least three successful owned daemon
exits; each Cargo sample must record at least one. Native and mixed probe
scopes require their own receipts, and schema 2 also requires the native-Rust
reference probe receipt. Scope names must be recognized and unique; counts
must be integers meeting the actual runner's minimum. A sample must finish its
commands and close its daemons before the next order declaration.

## Verification

The regression suite covers missing, moved and duplicated records; order
metadata that remains balanced but contradicts execution; premature shutdown;
insufficient, negative or boolean exit counts; and substituted runtime artifacts,
sources and output. Synthetic fixtures explicitly contain their execution and
shutdown records. They do not claim that a compiler ran.
All 66 reporting tests passed after the correction. The added regressions
reproduce failures against the predecessor, including the three audit gaps.

From the repository root:

```sh
python3 tools/test-mixed-performance-report.py
python3 tools/check-mixed-performance-tables.py
python3 tools/check-mixed-performance-archives.py
```

All seven original archives remain unchanged. Their 3,888 timed command values,
198 comparisons and 12,768 output files replay with the additional requirements.
The CSV and Markdown reports retain their exact bytes. Replay checks recorded
evidence; it neither reruns a compiler nor grants release or speed acceptance.
