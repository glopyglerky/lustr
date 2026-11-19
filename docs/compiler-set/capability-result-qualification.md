# Capability result qualification

The pinned upstream `compiler/native/scripts/test-capabilities.py` checks the
published declarations against 73 probes. Its supported cases require exit
status zero. Seventeen supported run fixtures print their computed values and
contain no assertions, so that check alone does not detect incorrect results.
The ordinary native differential suite reads different fixture directories.
Both upstream suites remain unchanged.

`tools/qualify-capability-results.py` adds a comparison through the installed
public `lustr` entry. It discovers every supported fixture from the original
headers and fails on an empty inventory or an unqualified command. For each
run fixture it compiles the unchanged program with Lustr and the independently
selected Rust oracle, then executes both artifacts. Exact stdout, stderr and
successful exits must match. Matching failures cannot pass.

The supported `attributes-test.rs` fixture uses different test harness formats.
The qualifier compares every test name and status, requires complete nonempty
summaries, and rejects missing, duplicate, filtered, failed or timed-out tests.
Its intentionally ignored test must have the same status in both harnesses.
This is a result comparison, not a raw formatting comparison or a replacement
for the production runner's failure controls.

The report retains source, fixture, runner, compiler and generated-artifact
hashes, every command, exit status and output stream. The oracle's source must
equal `upstream/bluefe-core.pin`. Compiler and fixture identities must remain
unchanged throughout the run. Each attempt requires a new evidence directory;
timeouts, compiler failures and mismatches are retained without retry.

```sh
python3 tools/test-capability-results.py -v
python3 tools/qualify-capability-results.py \
  --entry /absolute/package/bin/lustr \
  --rustc /absolute/declared-sdk/bin/rustc \
  --output /absolute/new-evidence-directory
```

The evaluator's separate controls use actual small child processes and planted
test-result records. They establish that incorrect outputs, failed exits,
signals and incomplete summaries are refused. They are not compiler tests;
the qualification command must still compile and execute the real artifacts.

The constructor workflow also runs every root crate test target and explicitly
runs the five ignored installed-package contracts. Those cover single test
execution, large argument lists, library selection/rejection/recovery, output
failure and ambiguous daemon replies. A socket-capable host must run the daemon
case; lack of a socket cannot become a passing skip. The unrelated ignored
unit-test child fixture is not launched as an independent package test.

This remains finite, cold, functional qualification of these fixtures. It does
not establish full Rust coverage, every combination of declared capabilities,
retained incremental behavior, optimized-package speed, or other platforms.
The independent oracle runs only during qualification and is never available as
a rescue compiler for a failed product request.
