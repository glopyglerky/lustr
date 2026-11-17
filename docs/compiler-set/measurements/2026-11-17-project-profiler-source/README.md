# Reviewed project campaign and phase profiler source

This incremental Git bundle preserves the exact original commits referenced by
independent source review, including the profiler predecessor whose variable
error diagnostics were corrected. The published integration has identical
corresponding file content but different outer commit identities.

[source-bundle.json](source-bundle.json) records the bundle hash, prerequisite
and reviewed revisions. It contains source and tests, with no private recipient
key, private campaign output or measured compiler result. Source review and
helper tests do not establish source-built daemon qualification.

From a clone containing the published prerequisite:

```bash
git bundle verify docs/compiler-set/measurements/2026-11-17-project-profiler-source/source.bundle
git fetch docs/compiler-set/measurements/2026-11-17-project-profiler-source/source.bundle refs/heads/lustr/real-project-performance-20261117:refs/remotes/retained/project-profiler-source
git show fe18584021d9a9dbea02a6ea1f19aa5c295dbddd
```

The final correction leaves failed native diagnostics unchanged. Successful
responses carry opt-in emitter durations; failed responses explicitly have no
emitter measurement. The [phase-attribution contract](../../mixed-phase-attribution.md)
describes the clock domains, binding and unavailable measurements.
