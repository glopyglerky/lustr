# Retain failed native crate evidence

The macOS `odd-path` failure in
[run 44493776529](https://github.com/glopyglerky/lustr/actions/runs/44493776529/job/135455703043)
left a truncated-response diagnostic but no daemon log. The downloaded native
artifact contained 582 files, and none belonged to the failed case directory.
The crate runner's exit trap had removed that directory before upload.

`compiler/native/scripts/test-crates.sh` now accepts the optional
`LUST_CRATE_FAILURE_DIR` destination. On a failed fixture, it saves the case
directory and failure record before ordinary temporary cleanup. Both CI jobs
set that destination and include it in their existing native-gate artifact.
The full default suite remains selected. Retention does not change a fixture's
failure result.

Each capture uses a new private directory named after the fixture. It contains:

- `case.tar.gz`, with the fixture source, build artifacts, captured stdout and
  stderr, and its `.lust-cache/daemon.log` when created.
- `failure.json`, with the original reported failure and capture outcome.

The independent Cargo oracle and ordinary native program have their captured
exit codes saved alongside stdout and stderr. The daemon client and generated
program also have exit codes and stderr recorded for each cold and warm attempt.
When the caller explicitly sets `LUST_CACHE_DIR`,
the capture also includes that directory's regular, owned `daemon.log` under
`external-daemon.log`. It does not copy the caller's entire cache directory.

The archive keeps hidden fixture directories without requiring the artifact
uploader to collect unrelated hidden runner files. It preserves symlinks as
links and skips sockets, FIFOs and device files. Capture directories are
exclusive; earlier captures and caller files are never replaced or removed.
The source case and destination must be real directories owned by the current
user. A destination inside the source case is rejected.

Retention errors are printed and remain separate from the fixture failure.
They do not turn a failure into success, trigger another compiler attempt,
skip a fixture, or replace the runner's exit status. A capture is a best-effort
snapshot; a live daemon can still append log bytes afterward. It is not a
claim that every writer has stopped.

## Retained execution acceptance

Both daemon-built programs must have the same stdout and captured exit status
as the independent Cargo oracle. Previously the daemon checks compared stdout
alone, allowing a generated program with the wrong exit status to pass. The
runner now rejects that mismatch before reporting success and retains all
captured output, stderr and statuses in the failed case archive.

The comparison uses the captured status directly. Expected nonzero results,
including panic statuses, remain valid when they match the oracle. Timeout and
signal status codes are not coerced to success or a generic failure value.
A failing cold retained run stops that fixture before the warm attempt; no
executed program is retried.

## Narrow reproduction

The runner already supports a substring filter and fails if it selects no
fixtures. After building the ordinary native helpers and installing the
declared Rust oracle, run this from `compiler/native`:

```sh
LUST_CRATE_FAILURE_DIR="$RUNNER_TEMP/native-crate-failures" \
  bash scripts/test-crates.sh odd-path
```

That selects the existing fixture with its space, `#`, and non-ASCII project
path. It preserves the existing cold and warm daemon checks and failure
behavior. A targeted result is separate from a complete native-gate result.

## Verification

`python3 tools/test-native-crate-failure-retention.py` passes eleven controls.
They exercise exact binary daemon-log retention, skipped FIFOs, symlinks that
are not followed, an external daemon log, non-overwriting repeated captures,
invalid destinations, optional-off cleanup, successful runs and empty filters.
The real shell runner is invoked with compiler stubs that induce a known warm
failure. Its result remains one failure with exit status 1, while the original
daemon-client exit status 3 and full diagnostic survive in the archive. A
capture failure also leaves that original runner result unchanged.

The runtime controls execute the actual shell runner with known generated
programs. Identical stdout with mismatched exit statuses fails for both cold
and warm daemon builds, including an unexpected success where the oracle
returns 101. Matching nonzero statuses 7, 101 and 134 pass. Raw statuses 124,
137 and 143 remain distinct mismatches and survive in the archive. These last
programs return the codes explicitly; they do not claim to induce real timeouts
or signals. Before the comparison fix, the new wrong-status controls exposed
false successful fixture results.

Those stubs qualify runner acceptance and evidence handling. They do not
establish a native compiler or daemon repair. The original macOS failure still
needs its own source-bound diagnosis and actual hosted execution.
