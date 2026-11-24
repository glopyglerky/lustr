# Preserved Unicode sanitizer contention diagnostic

This historical experiment establishes contention sensitivity for one pristine
Lust specimen on its recorded host. It does not establish the cause of a later
full-suite failure or qualify the current compiler. The original diagnostic
job remains failed because its concurrent ten-second executions failed.

## Original source and retained evidence

- [Source and recorder](https://github.com/glopyglerky/lustr/commit/ddc633295fbc0eca012087d1c48932224810077a), based on `82d77539680fec9cf822ca8231d1abb985069c48`.
- [Original run and complete log](https://github.com/glopyglerky/lustr/actions/runs/42757628508/job/129635586260).
- [Original artifact 13737065633](https://github.com/glopyglerky/lustr/actions/runs/42757628508/artifacts/13737065633).
- [Byte-identical original ZIP](original-qualification.zip), SHA-256 `a16ddebc98a9d35a09c871e79780975ceb2de83d9b160fff46e4d6524b3cdd00`.
- [Source and archive identity](identity.json), [original summary](summary.json), [host/tool metadata](metadata.json), and [original upstream-script results](upstream-results.json).

The ZIP retains all 211 original files, including generated C, actual sanitizer
objects and executable, exact commands, exit statuses, streams, wall times,
GNU time CPU/RSS observations and the fixture source. The adjacent JSON files
are copied byte for byte from the archive. The archive remains available after
the original Actions retention expires.

The source guard identified pristine Lust `240e08d91d4aed0299639f6f11fd522912b248f9`,
tree `0ab1b09815802894e6078de0af59273cbdf46cb0`. The fixture SHA-256 is
`f68662c528f0a470b30856def6fade326b2154b4ebda5a0e5b83c8f70da28b7d`.
The Rust oracle was 1.99.0. The recorded host was Linux x86_64 on AMD EPYC 7763,
with four logical CPUs on two physical cores and two SMT threads per core.

## Observed outcomes

All sanitizer checks remained enabled. The original run deadline was ten
seconds, GCC's initial O1 limit was twenty seconds, and no O0 fallback occurred.

| Execution | Outcome |
| --- | --- |
| Unchanged upstream script, exact fixture, one worker | Passed |
| Unchanged upstream script, `unicode_` subset, default four-worker pool | Passed |
| Isolated execution 1 | Passed in 6.529712 seconds |
| Isolated execution 2 | Passed in 6.581628 seconds |
| Isolated execution 3 | Passed in 6.680013 seconds |
| Three groups of four concurrent copies | All twelve timed out, exit 124, about 10.04 seconds each |
| Separate exploratory isolated run with a sixty-second limit | Passed in 6.630292 seconds; diagnostic only |

The concurrent runs each consumed approximately 9.2–9.5 seconds of user CPU
before timeout. [The original ten-second results](qualification-10s-before-exploration.json)
were saved before the exploratory run. That longer run changes no earlier
failure or deadline. The [complete execution records](runtime-10s-results.json)
retain every outcome.

The unchanged upstream script removes its own temporary files. The retained
C/object/executable specimen comes from the separately recorded reproduction,
not those deleted upstream runs. Concurrent identical copies test runtime
contention, not the full pool's mixture of compilation, execution and object
cache locking. The successful `unicode_` subset cannot rule out full-suite
effects. CPU topology and worker count alone do not attribute the later timeout.

## Use in subsequent qualification

This experiment supports testing an explicit worker limit with the complete
current gate. Such a run must retain its actual host, worker count, full case
counts, unchanged sanitizers and deadlines, and original failures. A changed
schedule is a new qualification condition, not a retrospective pass or proof
that the compiler was repaired. An isolated success cannot replace the full
gate.

To inspect the retained results without executing archived programs:

```sh
sha256sum docs/compiler-set/measurements/2026-11-10-unicode-contention/original-qualification.zip
python3 -m json.tool docs/compiler-set/measurements/2026-11-10-unicode-contention/summary.json
```

No timed compiler comparisons are added by this preservation work. Its runtime
diagnostic observations are separate from the fixed-corpus paired campaigns.
