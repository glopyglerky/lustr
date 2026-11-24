# Original public slice qualification failure

The public upstream slice campaign at candidate `6bf6e3e` failed during its
initial retained-state check, before collecting any timed samples. It has no
completed speed comparison. The fixed scalar/Cargo campaign and phase profile
completed independently in the same job.

The [original run](https://github.com/glopyglerky/lustr/actions/runs/44179667793/job/134380277750)
used checkout `5a3c5666a16d46629a4787e37ca9b20e65720ccc`, tree
`02652f033e84c0957dd2b99eb562db8171289e50`. The [manifest](manifest.json)
binds [artifact 14374800386](https://github.com/glopyglerky/lustr/actions/runs/44179667793/artifacts/14374800386)
to the byte-identical [original ZIP](original-qualification.zip), all 98 original
files, and the original report and diagnostic hashes. The adjacent
[report](report.json) and [original README](original-README.md) preserve their
exact original bytes.

## Observed failure

Command 33 was a successful pristine-Lust check of 1,797 functions. Its human
timing output reported zero rechecked bodies. The recorder then rejected it
with `ValueError: missing or repeated native diagnostic record` because the
structured statistics needed to prove the retained route were absent.

The client had `LUST_STATS=1`, but its owned daemon did not. Upstream
`compiler/native/src/driver/compile.rs::stats_on` reads the daemon process
environment; a client request cannot set that process variable.

The correction enables diagnostics on the untimed qualification daemon and
both qualification requests. Measured commands and daemons remain
uninstrumented, and replay now rejects instrumentation in those records.
The retained-route validator also accepts the exact upstream tuples
`reuse/reuse/-`, `bodies/patch/-` and `bodies/chunks/bodies`. All existing
zero-repeated-work checks remain required. In particular, an entry trace
environment change may select the valid body-patch route, whose reason is `-`.

## Correction verification and limits

The failing-first tests reproduce the environment omission and incorrect route
reason. All 27 slice-recorder tests pass after the correction. They compile and
execute the actual upstream statistics reader/renderer and verify the recorder's
orchestration and replay rules. Independent source/artifact review found no
remaining defect in this correction.

Those tests do not execute a complete native daemon campaign. The local task
environment cannot bind Unix sockets; fresh hosted execution is still required.
The original failed report remains failed with zero samples and null summaries,
and the corrected validator still rejects its original diagnostic stream.

Verify the original archive without executing archived commands:

```sh
sha256sum performance/slices/2026-11-17-6bf6e3e/original-qualification.zip
```

Expected SHA-256:
`9f62b09d95c87af2ff3ea380ce68a06263a4acd5017cc5f20ed70c7572ce2eea`.
