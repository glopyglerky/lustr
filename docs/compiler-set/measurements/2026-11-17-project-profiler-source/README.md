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

## Recovery supplement

The separate [recovery source manifest](recovery-source-bundle.json) and
`recovery-source.bundle` preserve the later read-only reconciliation helper,
its tests and the predecessor corrected during review. The finding allowed
local Git replacement refs to substitute an exporter while reporting the
recorded revision. The correction makes every helper Git read ignore
replacement objects; the retained regression installs a real replacement ref.

After restoring the first bundle above:

```bash
git bundle verify docs/compiler-set/measurements/2026-11-17-project-profiler-source/recovery-source.bundle
git fetch docs/compiler-set/measurements/2026-11-17-project-profiler-source/recovery-source.bundle refs/heads/lustr/real-project-performance-20261117:refs/remotes/retained/recovery-source
git show 3a5b3192e6a33b4ae54f916c9a1441caa8f963dd
```

This supplement carries no private originals or recipient key. The helper's
focused tests are separate from an actual campaign decryption and reconciliation
receipt.
