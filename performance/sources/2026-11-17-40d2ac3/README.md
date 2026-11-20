# Exact GitHub merge checkout source

The component evidence at candidate `40d2ac3` records GitHub merge checkout
`3b4b0bd67401085129d26be1afbd20e96c9b9d02`, with the same tree as the candidate.
This small incremental bundle preserves that exact commit object after GitHub's
live PR merge ref moves. It adds no compiler result or release qualification.

[source-bundle.json](source-bundle.json) binds the bundle and both required parent commits. Both are reachable from
the published candidate. All
file/tree objects already belong to the published prerequisite candidate;
the bundle adds the merge commit and a named ref for recovery. It contains no
private evidence or private recipient key.

From a clone containing candidate `40d2ac3f0b71abaa1e73cd366d8c43f8eb4d4581`:

```bash
git bundle verify performance/sources/2026-11-17-40d2ac3/source.bundle
git fetch performance/sources/2026-11-17-40d2ac3/source.bundle refs/retained/lustr-ci-40d2ac3:refs/remotes/retained/ci-40d2ac3
git --no-replace-objects show --no-patch --format='%H %T' 3b4b0bd67401085129d26be1afbd20e96c9b9d02
```

Evidence reconciliation loads source by the original recorded commit, without
checking out that commit or trusting replacement refs. The separate original
artifacts remain the authority for which checkout each campaign actually ran.
