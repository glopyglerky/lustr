# Mixed compiler phase attribution

`tools/profile-mixed-compiler.py` runs the existing mixed Cargo qualification
with `LUSTR_MIXED_PROFILE_DIR` set in a fresh work directory. This diagnostic
run is separate from every paired timing campaign. Its source, compiler
identities, original JSONL records, command outputs and diagnostic traces stay
with the generated report. Instrumented durations do not establish a speedup.

## Executed status on 17 November 2026

The [v1 profile at dec0ce2](../../performance/2026-11-17-dec0ce2-profile.md)
completed. The [v2 attempt at 40d2ac3](../../performance/2026-11-17-40d2ac3-profile.md)
completed its actual mixed qualifier but failed the independent request-coverage
check because two concurrent request lines were concatenated. All corresponding
profiles exist. The original failed record is preserved. A separately labeled
one-boundary reconstruction, replayed by the unchanged recorded reporter,
validates 26 compiler profiles, 15 qualification commands and 35 successful native
responses. It is diagnostic evidence, not a new compiler qualification or a
passed original profiling campaign.

The shared writer now frames the complete record and holds an append-file lock
through `write_all`, including short-write retries. Three actual helper tests
pass; the concurrent-process regression fails the original helper. All 23 strict
profile tests pass without relaxing the validator. Fresh source-built execution
must qualify the corrected writer. Recorded symbol-inspection spans identify an
expensive child call but do not establish its cause; the original artifact lacks
the child executable identity and spawn/CPU breakdown.

## Client request measurements

The driver records `native-compile` and `native-check` around each native
request. Their `native-request-setup` child covers argument construction,
session configuration and socket-path preparation. Each
`native-daemon-roundtrip` child covers one call to the existing protocol helper,
including connection, request serialization and transmission, waiting, and
response decoding. That interval closes before profile-record inspection.
Failed connection attempts also have intervals. Daemon launch and retry sleeps
remain inside the enclosing request, outside those child intervals.

## Native object measurements

The owned mixed emitter reads the opt-in from the requesting client's
environment. It sends one structured duration record in a successful response's
existing diagnostics. It does not create files, change the protocol, set
`req.timings`, or depend on the environment of a previously started daemon.

| Phase | Work measured |
|---|---|
| `c-prepare` | Boundary validation, function and ABI selection, scalar C text and wrapper construction |
| `output-prepare` | Parent/output checks and directory creation |
| `c-write` | Writing the generated C source |
| `libtcc` | The existing single-unit `compile_units` call, including in-process TinyCC setup, compilation, object write and teardown |
| `nm` | Starting and waiting for `nm`, then checking its result for unrepresented symbols |
| `rename` | Renaming the completed object into its fresh publication path |
| `file-sync` | Opening and synchronizing the published object |
| `directory-sync` | Opening and synchronizing its parent directory |

The sequential native phases cover a successful emitter's measured total.
Errors keep their original diagnostics unchanged because the native daemon
compares failed warm/fresh responses byte for byte. Variable timing text would
create a false mismatch and discard retained state. The client still records
the actual failed response code and full request/roundtrip intervals, with an
explicit `unavailable-failed-response` state for emitter detail. Failed responses
must have no timing decoration. Record
serialization and the surrounding native frontend, code generation,
retained-state update, daemon guard and response transport are outside the
emitter interval.

## Binding and validation

Compiler profile version 2 retains the exact remote JSON string, the native
unit identity, the actual response code, and the enclosing request and
protocol interval. The ordinary diagnostic trace independently retains that
same string. The report requires matching native build and unit identities,
one remote record per successful native compilation response, the expected phase order,
consistent success/completion flags, and an exact sum of phase durations.
It also requires the remote total to fit within the actual protocol exchange.
Missing successful-response detail, duplicated, malformed or unbound detail,
or unexpected timing decoration on a failed response fails profiling qualification.
It does not change compiler success, diagnostics or the paired timing result.

Native durations use the daemon's clock. The report does not give them
fabricated positions on the driver's timeline. The difference between an
exchange duration and its emitter duration is explicitly **unattributed**.
It includes IPC and queueing, native frontend and generated-code work before
the emitter, retained-state work after it, guard handling, and response
production. It must not be labeled frontend or codegen time. Client setup and
the complete native request remain visible separately.

Version 1 archives remain readable. They lack these remote measurements and
are displayed as `not recorded`, not zero. New successful emitters require all
stages and complete flags. A failed native compilation cannot supply qualifying
emitter timings. Its emitter duration and outside-emitter residual remain null,
never zero. Reports retain total response, measured success and unavailable
failure counts; phase totals cover only the measured successes. A command with
only failed native responses displays unavailable emitter measurements. A
command without a native compilation has no emitter observation.

The default path reads no profiling clocks and writes no profiling files.
The inline emitter helper stays inside `mixed_unit.rs`, which the existing
native build identity already hashes. Protected native sources, cache keys,
admission, symbol verification and all publication synchronization operations
remain unchanged. The real failed-link/recovery qualification still runs;
local schema tests and compiler typechecks do not replace it.
