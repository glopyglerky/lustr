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
declared target directory's `debug/mixed-app` and application source. Schema 1
retains its original native-flag contract; the measured Cargo command checks
below apply to the shared Cargo contract in both schemas.

Every measured native sample must record at least three successful owned daemon
exits; each Cargo sample must record at least one. Native and mixed probe
scopes require their own receipts, and schema 2 also requires the native-Rust
reference probe receipt. Scope names must be recognized and unique; counts
must be integers meeting the actual runner's minimum. A sample must finish its
commands and close its daemons before the next order declaration.

## Cargo command and test execution

A second audit found that the Cargo rows lacked the command and test checks
already present for schema-2 native rows. In separate in-memory copies of the
original `5a3f493` archive, the predecessor accepted both counterexamples after
the copied record digests were updated:

- Change all 108 timed mixed Cargo commands to invoke the identified reference
  Cargo executable while retaining their mixed labels.
- Change all 24 Cargo compile-plus-test commands from `test` to `build` and
  replace their stdout/stderr with empty, correctly rehashed streams.

The original archive remains unchanged and records the actual mixed entry and
executed tests. These counterexamples identify verifier omissions, not missing
work in the original campaign.

For each measured Cargo command, the reporter now requires the exact recorded
package contract: the hash-identified `cargo` or sibling `lustr` entry, the
requested build/check/test operation, locked offline resolution, two jobs,
the `mixed-app` package and binary, and separate target/intermediate directories
for that lane and sample. The working directory must select the recorded
workspace, whose eight source/manifest/lockfile identities must be present.
Schema 2 also binds its mixed Cargo executable to the declared product entry.
Extra profile overrides, a different target, shared lane caches, `--no-run` and
test filters fail validation.

Every compile-plus-test command must retain one successful `scalar_contract`
test line, one single-test execution header, and one summary with exactly one
pass and no failures, ignored tests or filtered tests. A build-only command,
blank output or duplicate test receipt cannot qualify as compile plus test.
These checks apply to this fixed corpus, not arbitrary Cargo project logs.

## Verification

The regression suite covers missing, moved and duplicated records; order
metadata that remains balanced but contradicts execution; premature shutdown;
insufficient, negative or boolean exit counts; and substituted runtime artifacts,
sources and output. Synthetic fixtures explicitly contain their execution and
shutdown records. They do not claim that a compiler ran.
The earlier execution-record correction passed 66 reporting tests. The Cargo
correction adds regressions in both schemas for compiler substitution, missing
or ambiguous executable identities, changed commands/flags, shared caches,
unrelated working directories and absent, filtered, ignored or duplicated test
execution. Both principal Cargo regressions fail against the predecessor before
the correction and pass afterward. All 80 reporting tests pass after the Cargo
correction.

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
