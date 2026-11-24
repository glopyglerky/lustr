# Exact source for the Cargo overhead correction campaign

Candidate `6bf6e3e3384a99fbaeab2c66aa7c97bc73231996` has GitHub merge checkout
`5a3c5666a16d46629a4787e37ca9b20e65720ccc`. Both have source tree
`02652f033e84c0957dd2b99eb562db8171289e50`. This incremental bundle preserves
the exact merge commit after GitHub's live PR ref moves. It grants no compiler
or speed qualification; the original campaign artifacts record what executed.

[source-bundle.json](source-bundle.json) binds the bundle bytes and both parent
prerequisites. Both prerequisites are reachable from the published candidate.
The source tree matches that candidate, so the bundle adds the exact merge
commit and a named recovery ref. It contains no private evidence or recipient
key.

From a clone containing the candidate:

```bash
git bundle verify performance/sources/2026-11-17-6bf6e3e/source.bundle
git fetch performance/sources/2026-11-17-6bf6e3e/source.bundle refs/retained/lustr-ci-6bf6e3e:refs/remotes/retained/ci-6bf6e3e
git --no-replace-objects show --no-patch --format='%H %T' 5a3c5666a16d46629a4787e37ca9b20e65720ccc
git --no-replace-objects ls-tree -r -z --full-tree 5a3c5666a16d46629a4787e37ca9b20e65720ccc | sha256sum
```

The inventory command includes every source-entry name, mode and object ID in
Git's NUL-separated format. Its SHA-256 is
`9f58fdd772d9a6622e2e9c72e58bfdd75f98a0c28e8d19d0e7daa485157c1623`.
The bundle does not contain the original compiler binaries or reproduce their
execution.

The [isolated restore check](restore-check.json) started from a fresh bare
single-branch clone of the local candidate checkout, using `--no-local` and
`--filter=blob:none`. A command-scoped upload-pack setting enabled that filter;
no repository configuration was changed. The first unfiltered clone could not
read a historical object from the partial source object store. The receipt
preserves that failure and the successful filtered method.

The merge object was absent from the isolated clone's local object inventory
before importing the bundle. Recovery reproduced its exact commit bytes, tree
and all **13,521 source-entry names, modes and object IDs**. This check used no
GitHub network request or replacement refs. It compares Git inventories; it
does not freshly hash every source blob, independently authenticate remote
source, recover original compiler binaries or execute the compiler.
