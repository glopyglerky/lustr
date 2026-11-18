# Mixed compiler phase attribution

`tools/profile-mixed-compiler.py` runs the existing mixed Cargo qualification
with `LUSTR_MIXED_PROFILE_DIR` set in a fresh work directory. This diagnostic
run is separate from every paired timing campaign. Its source, compiler
identities, original JSONL records, command outputs and diagnostic traces stay
with the generated report. Instrumented durations do not establish a speedup.

## Executed status on 17 November 2026

The [original v2 profile at 5a3f493](../../performance/2026-11-17-5a3f493-profile.md)
passed its complete qualification and strict profile validation. Actual checkout
`b3393a4a2907228516546cd34f1a5c60aaf6aaaa` produced **26 compiler profiles across
15 qualification commands and 35 successful native responses**. The original
run records `completed: true` and qualifier exit zero. Its summary and Markdown
replay byte for byte with the recorded profiler, without reconstruction. The
[verification receipt](../../performance/profiles/2026-11-17-5a3f493/verification.json)
binds all 132 original files from the profiling directory to the retained
qualification ZIP. Recorded
package binary identities agree with the paired run; absent binaries were not
freshly hashed during recovery.

The body and signature observations below sum named phases across their actual
requests. The signature column sums five requests; it is not a per-request
median. Units are milliseconds, and nested spans must not be added together.

| Phase | Body, 1 request | Signature, 5 requests |
| --- | ---: | ---: |
| Complete native request | 37.681 | 176.701 |
| Client setup | 0.120 | 0.377 |
| Daemon roundtrip | 37.551 | 176.283 |
| Remote emitter | 35.258 | 162.980 |
| libtcc | 0.198 | 2.016 |
| `nm` | 34.545 | 158.645 |
| Emitter object synchronization | 0.409 | 1.888 |
| Emitter directory synchronization | 0.009 | 0.039 |
| Roundtrip outside emitter, unattributed | 2.293 | 13.303 |

Across all 35 measured responses, the `nm` span ranges from 30.808 to 34.545 ms,
with a median of 31.857 ms. This identifies the measured complete process call,
not its underlying cause: the archive lacks the resolved child binary identity,
object bytes and child CPU/startup breakdown. These instrumented observations
are separate from the paired performance campaign and establish no optimization
effect. The complete report retains every qualification command; its JSON
preserves every recorded phase.

### Earlier profiles remain unchanged

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
profile tests pass without relaxing the validator. The fresh `5a3f493`
source-built execution above qualified the corrected writer. It does not change
the failed status, values or reconstruction limits of the original `40d2ac3`
profile.

## Client request measurements

The driver records `native-compile` and `native-check` around each native
request. Their `native-request-setup` child covers argument construction,
session configuration and socket-path preparation. Each
`native-daemon-roundtrip` child covers one call to the existing protocol helper,
including connection, request serialization and transmission, waiting, and
response decoding. That interval closes before profile-record inspection.
Failed connection attempts also have intervals. Daemon launch and retry sleeps
remain inside the enclosing request, outside those child intervals.

## Historical native object measurements (original v2 campaign)

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

## Historical v2 binding and validation

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

The original v2 instrumentation left native admission and cache identity
policy unchanged. Its default path read no profiling clocks and wrote no
profiling files. The inline emitter helper stayed inside `mixed_unit.rs`, which
the native build identity already hashed. Those statements describe the
original instrumentation change; the later admission work below has a different
source and contract.


## Current schema 3: explicit native outcomes

The [native admission boundary](native-admission.md) now decides scalar request,
IR and ABI eligibility before C generation, retaining the checked program into
emission. Profile schema 3 records that actual typed outcome alongside the
response code. It does not treat a zero status as evidence of emitted code.

| Native outcome | Required profile evidence |
| --- | --- |
| Emitted | Status zero and exactly one complete successful emitter record |
| Complementary | Status zero, `unavailable-complementary`, no emitter record |
| Failed | Nonzero status, `unavailable-failed-response`, no emitter record; the enclosing compiler must fail |

A compile response marked checked, unclassified or inconsistent fails profile
validation. Complementary and failed counts remain separate. Their absent
emitter duration and unattributed residual remain null; a complementary-only
row says `not emitted`. Genuine emitted responses retain the existing strict
identity, timing and trace bindings.

The current emitter's `c-prepare` span constructs C and the wrapper from an
already admitted unit, with invariant checks on the generated helper closure.
It no longer measures capability admission or ABI selection. The existing
owned ELF reader is timed as `symbol-check`; the historical table's `nm` span
continues to mean the original process call at that source. Native preparation,
frontend work and C generation still happen outside the emitter interval.

The current qualifier has **19 commands**. Four new operations exercise actual
retained C-output and link-recipe filesystem failures, then restoration and
retained recovery. The validator requires those operations and their actual
outcomes for schema 3. Historical schemas 1 and 2 retain their original
15-command scope and remain readable with the original meanings. A current
profile cannot claim the shorter historical qualification.

All 36 local profile controls passed; those use synthetic records to test the
validator. The updated real qualifier compiled locally and awaits hosted daemon
execution. These results are not compiler-speed measurements or an end-to-end
pass. Ordinary profiling-disabled runs still read no profiling clocks and write
no profiling files. The admission, certificate-schema and mixed transport
changes require their own exact-source qualification.
