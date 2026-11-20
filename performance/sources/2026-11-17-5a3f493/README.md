# Exact source for the first schema-2 campaign

Candidate `5a3f4937d94e8f028dd85385ea65d51fcfe35a3b` has GitHub merge checkout
`b3393a4a2907228516546cd34f1a5c60aaf6aaaa`. Both have source tree
`2fca60e57829890b10a4a23da57bafab2581b092`. This incremental bundle preserves
the exact merge commit after GitHub's live PR ref moves. It grants no compiler
or speed qualification; the original campaign artifacts record what executed.

[source-bundle.json](source-bundle.json) binds the bytes and both parent
prerequisites. Both prerequisites are reachable from the published candidate.
All file and tree objects already belong to that candidate; the bundle adds
the merge commit and a named recovery ref. It contains no private evidence or
recipient key.

From a clone containing the candidate:

```bash
git bundle verify performance/sources/2026-11-17-5a3f493/source.bundle
git fetch performance/sources/2026-11-17-5a3f493/source.bundle refs/retained/lustr-ci-5a3f493:refs/remotes/retained/ci-5a3f493
git --no-replace-objects show --no-patch --format='%H %T' b3393a4a2907228516546cd34f1a5c60aaf6aaaa
```

Project evidence reconciliation reads the original recorded Git source without
checking it out or trusting replacement refs. The bundle does not contain the
original compiler binaries or reproduce their execution.

The [isolated restore check](restore-check.json) started from a fresh public
clone containing the two prerequisite commits. The merge object was absent
before importing the bundle. Recovery reproduced the exact tree and all 13,443
source-entry names, modes and blob IDs. That check compares Git inventories;
it does not freshly hash every source blob or execute the compiler.
