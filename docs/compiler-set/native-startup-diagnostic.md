# Optional native-client startup diagnostic

Status: infrastructure with synthetic local controls. No real startup
measurement, compiler speedup or causal attribution has been established.

The [fourth completed residual diagnostic](evidence/warm-entry-2026-11-18/fourth-results.md) found slower whole-command integrated
native requests than pristine requests, while its two untimed driver probes
did not establish a slower integrated compiler core. The successful warm
client's source delta adds socket-address resolution; the recorded short socket
paths take its borrowed-path branch without filesystem work. The next bounded
discriminant checks the native client's early `--version` branch, which returns
before executable-path discovery, request parsing or daemon work.

`tools/run-warm-native-startup.py` is a separate optional diagnostic, schema
`lustr-native-startup-diagnostic-v1`. It executes **96 fixed alternating pairs**
of the already built frozen-baseline native `lust` and pristine `lust`, after
exactly **three excluded primes for each**. It preserves all 192 observations,
all six primes, every command and captured failure. No compiler is rebuilt.

The baseline is the full package built from frozen PR25 source
`3c378a11d60149a3ec99fa49ea1b336e9406fca8`, with its complete constructor receipt
and native component identities. The reference is pristine Lust
`240e08d91d4aed0299639f6f11fd522912b248f9`. Both client and sibling-core hashes
must match the completed main campaign. Before and after hashes are recorded
for the binaries, manifests, build receipts and diagnostic implementation.

Expected stdout comes from each pinned source manifest's package version and
the **actual emitted Cargo `LUST_BUILD_HASH`**, never from the command result
being checked. For pristine, the runner requires exactly one native receipt in
the release build-script output tree, archives its original bytes and path,
and binds the source identity, release build log and binary hashes to the main
campaign. The baseline uses its already verified full-package native receipt.
Each command must return zero, exact expected stdout and empty stderr; signals,
timeouts, wrong build IDs and output-limit failures remain failures.

Both lanes use the same controlled environment, caller directory and held
temporary/Cargo state. The commands are exactly `[native_client, --version]`.
No daemon or socket variable is supplied, and no daemon or socket operation is
performed. The observed wall time still includes process startup, executable
image placement, stdout/stderr capture and the existing recorder's
100-microsecond exit polling. Those effects are deliberately not attributed to
a compiler phase.

The optional workflow step runs only after the primary campaign's status is
`completed`, including a completed strict-target miss. The diagnostic replays
that campaign before starting. It cannot run from incomplete main evidence,
and it never changes the primary 384 timings, lane order, raw outcomes or
strict Lust criterion. Its own failure streams are uploaded independently.
There is no diagnostic speed-admission threshold.

A persistent `--version` gap would support further investigation of startup or
measurement behavior. An absent gap would support examining request parsing
and roundtrip spans. Neither outcome alone establishes a cause of the warm
command difference. Do not subtract these values from independently measured
warm commands as an additive decomposition.

Run the local controls with
`python3 tools/test-warm-native-startup.py`. They use labeled synthetic child
results and cover a completed main target miss, source-derived stdout,
non-version requests, missing primes, changed order, ambiguous emitted receipts,
incomplete main evidence and workflow completion/failure retention. They are
not compiler execution or performance evidence.

Replay a downloaded diagnostic with:

```sh
python3 tools/run-warm-native-startup.py verify /absolute/path/to/native-startup-diagnostic
```

Replay checks the archived full baseline build, pristine source/build/binary
bindings, exact outputs, shared environment, fixed priming/sample order,
before/after identities, all stream digests and reproduced statistics/table.
Portable startup replay binds the archived primary report; it does not itself
replay that campaign's 384 raw timings. The live `run` command does perform that
full primary replay before any startup command. To independently replay the
downloaded primary evidence as well, run:

```sh
python3 tools/run-warm-native-comparison.py verify /absolute/path/to/native-warm-comparison
```
