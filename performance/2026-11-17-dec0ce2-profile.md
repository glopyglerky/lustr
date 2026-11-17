# Mixed compiler phase attribution: 17 November 2026

Measured PR head: `dec0ce2e3a68b82cac16eae238fc51b16e534e5e`.
Actual checkout: `dbacce1d14c8df60427eeca8dc7865156c98dcb4`;
tree: `c29f3bda282c0870c163303d865abb9be81bd066`.
The [original archive](archives/2026-11-17-dec0ce2-qualification.zip) retains
`run.json`, `summary.json`, per-process traces, compiler hashes and the complete
qualification commands and outputs. The instrumented qualifier completed with
exit 0. Its `summary.json` SHA256 is
`f1103703e65f7b408597fef2a78db501687cc35e4ba7210efbbdc6156ebb1354`.

The table below is copied from the original profiler report. This document
adds provenance and interpretation; it is a derived presentation, not a new
measurement. The [paired timing campaign](2026-11-17-dec0ce2.md) used separate
uninstrumented invocations.

Instrumented execution of the existing mixed Cargo qualification. This is a separate diagnostic run, not paired speed evidence.

Each cell sums one named phase across the compiler processes for that command, in milliseconds. Spans can nest and crates can run concurrently; columns must not be added into command wall time. Dynamic loading before driver entry and final process teardown are outside these spans. All original records and qualification stdout/stderr/status/trace files remain under `lustr-mixed-profile/` in the original qualification archive.

| Command | Compiler invocations | driver | context-fingerprint | native-compile | native-symbol-rename | temporary-link-copies | original-linker | edited-native-object-sync | publish-certificate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cold | 8 | 298.878 | 0.248 | 180.306 | 5.559 | 0.000 | 0.000 | 0.000 | 0.491 |
| body | 1 | 56.787 | 0.429 | 31.939 | 1.006 | 0.141 | 15.420 | 0.177 | 0.443 |
| arithmetic-edit | 1 | 57.684 | 0.451 | 32.371 | 1.063 | 0.146 | 15.624 | 0.158 | 0.470 |
| invalid | 1 | 19.432 | 0.419 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| recovery | 1 | 56.802 | 0.407 | 32.121 | 1.063 | 0.143 | 15.631 | 0.158 | 0.455 |
| link-failure | 1 | 41.998 | 0.429 | 31.370 | 1.034 | 0.142 | 1.999 | 0.163 | 0.000 |
| link-failure-recovery | 1 | 21.667 | 0.428 | 0.000 | 0.000 | 0.144 | 14.749 | 0.000 | 0.451 |
| versioned-object | 1 | 55.514 | 0.412 | 31.567 | 1.038 | 0.143 | 15.125 | 0.153 | 0.406 |
| versioned-object-noop | 1 | 22.138 | 0.428 | 0.000 | 0.000 | 0.136 | 14.967 | 0.000 | 0.520 |
| signature | 2 | 246.682 | 0.652 | 155.070 | 5.211 | 0.000 | 0.000 | 0.000 | 0.507 |
| dependency | 3 | 260.767 | 0.636 | 156.252 | 5.567 | 0.000 | 0.000 | 0.000 | 0.496 |
| macro | 2 | 252.632 | 0.714 | 159.707 | 5.498 | 0.000 | 0.000 | 0.000 | 0.461 |
| warning-cold | 1 | 208.347 | 0.682 | 153.353 | 4.993 | 0.000 | 0.000 | 0.000 | 0.000 |
| warning-edit | 1 | 202.526 | 0.659 | 147.890 | 5.040 | 0.000 | 0.000 | 0.000 | 0.000 |
| warning-denied | 1 | 16.617 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |

The full JSON includes every named phase, per-process success/reuse/check state, exact compiler identities and record hashes. A zero in this table means no recorded duration for that phase. These observations do not isolate a before/after optimization effect or establish a compiler speedup.
## What this trace establishes

In the one-compiler body-edit command, the `native-compile` span is 31.939 ms
and the retained original linker is 15.420 ms. Temporary link copies are
0.141 ms, edited-object synchronization 0.177 ms, and certificate publication
0.443 ms. The driver span is 56.787 ms. The native span includes the request
and object-production work; it does not identify an individual internal
function as the cause. Finer attribution belongs inside that measured path.

Signature invalidation spans two compiler invocations and 155.070 ms of native
compilation; that is an aggregate phase sum, not the command wall duration.
The failed-link recovery records no native compilation and still executes the
original linker, consistent with the successful immutable-object reuse check.

This run does not compare the temporary-copy change against its predecessor
on a controlled identical host. Its small copy/synchronization spans therefore
do not prove a speedup from that change. The paired campaign still shows large
mixed compilation overhead, including body and signature edits. Removing
checking, metadata, invalidation or diagnostics is not a qualified remedy.
