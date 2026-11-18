# Separate the lust-port test fixtures

`human_output_groups_findings` and `unknown_package_is_an_error` both called
`fixture("human")`. That helper constructs a directory from the process ID and
label, removes any existing directory at that path, and recreates the fixture.
Cargo runs these tests in threads within one process. The duplicate label lets
one test delete the other test's live source tree.

The correction changes only the unknown-package test's label to
`unknown-package`. The scanner, report renderer, test assertions, and fixture
contents are unchanged. The four earlier report-logging lines remain. This is
a test-isolation correction; it does not change compiler performance.

## Bounded interleaving diagnostic

`tools/diagnose-lust-port-fixture-race.py` exports the preserved diagnosis
baseline `8191c173d1f5036c1cbaf84a50cb2dddbc88076a` and the candidate revision.
It rejects any tool-source difference except this one fixture label. Its
scanner pause and test driver are applied only to exported diagnostic copies.
They are absent from the ordinary source and ordinary final test run.

The diagnostic runs the two actual test functions in named threads in one
process, preserving their common PID. The human scanner executes real Cargo
metadata and target-root discovery, then pauses immediately before
`walk::load`. The sibling test waits for those roots, performs the original
fixture helper's removal, records its path, and waits for the human test to
finish before recreating its fixture. The scanner resumes while that path is
absent. The existing walker omits a missing root; this makes a successful
zero-findings human report reachable under the duplicate fixture label.

There is one fixed baseline interleaving and one fixed candidate interleaving,
followed by one full ordinary candidate tool-test run with the overlay removed.
The diagnostic scanner must be byte-identical across both interleavings. The
baseline must fail the original `attrs.unsupported` assertion after a successful
empty scan, and the distinct-label candidate must retain the seven-file report
and pass both original tests. Timeouts, different failures, missing event
records, and changed scanner bytes make the diagnostic incomplete.

The commands use the same Rust 1.99.0 commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4` and Cargo 1.99.0 build
`5f94df478` as the prior hosted failure and preserved local diagnosis. All
source snapshots, actual arguments, controlled environments, scanner output,
barrier events, exit status, and executable identities are retained. The
diagnostic does not repeatedly run natural races until one fails.

The exact one-line correction and diagnostic source have independent review.
Execution of the fixed schedule is pending at this source revision. The earlier hosted failure and both earlier passing local runs remain
unchanged in [the original evidence](evidence/lust-port-human-2026-11-18/README.md).
Even a successful forced reproduction establishes a reachable failure mechanism,
not the historical runner's unobserved scheduling. Broader native/platform and
release qualification remain separate.

Example invocation after the candidate source is committed:

```sh
python3 -B tools/diagnose-lust-port-fixture-race.py run \
  --repository /path/to/repository --candidate FULL_COMMIT \
  --output /new/evidence/directory --sdk /path/to/rust-1.99.0 \
  --cargo-home /path/to/cargo-cache
python3 -B tools/diagnose-lust-port-fixture-race.py verify /new/evidence/directory
```
