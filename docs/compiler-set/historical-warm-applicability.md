# Stop inapplicable historical comparisons before construction

The two historical warm workflows now check applicability before installing an
SDK, building TinyCC or constructing packages. An inapplicable comparison still
**fails** and records **blocked / unqualified / zero timing samples**. This
implements the [completion checklist](completion-checklist.md)'s instruction
not to repeat known-inapplicable campaigns. It does not qualify a package,
repair an old baseline or establish a performance result.

## What each check establishes

| Historical comparison | Required condition | Current result |
| --- | --- | --- |
| Shared-native entry, baseline `d57f485eeb2b638d1f95a0594294d11e8001fe8e` | All seven shared source objects must equal the candidate's objects. | The candidate's native compiler and native-core library differ. Exit 1 records both differences before building the candidate. |
| Complete packages, baseline `3c378a11d60149a3ec99fa49ea1b336e9406fca8` | The frozen package must qualify with its prescribed constructor, SDK and native oracle. | That exact baseline and declared SDK/oracle combination already failed the unchanged Unicode differential gate. Exit 1 retains the source-bound reason and original evidence identity. |

The entry preflight uses the same seven-path constant as the original
`build-baseline` guard. The original guard still runs if preflight succeeds.
The identical-source control exits 0 with **applicable / unqualified / zero
timing samples**: construction, correctness, identity and measurement gates
remain necessary. Changed, missing or dirty source cannot reuse a prior
applicability conclusion.

The complete-package check binds the exact baseline revision and tree, its
three relevant Unicode source blobs and bytes, the declared Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, the existing Rust 1.99.0 oracle
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, and the checksum of the
[unchanged historical failure record](evidence/hosted-warm-failures-20261118/summary.json).
That record identifies the original hosted run and artifact where the frozen
package returned Unicode 18 and the prescribed oracle returned Unicode 17.
The preflight checks declared configuration; it does not install or execute an
SDK and does not describe the old failure as a fresh execution.

Missing evidence, a modified historical record, a different baseline or a
different SDK/oracle declaration also fails as unverified applicability. There
is no override. Neither workflow has a successful skip or `continue-on-error`.
The original sources, oracle selection, constructors, corpus, comparison
masks, qualification guards and failed artifacts remain unchanged.

Both workflows retain `applicability.json`, a readable summary and SHA256SUMS
with `if: always()`, and append the result to the job summary. Their original
steps remain in order for a legitimately applicable source; ordinary failure
propagation stops the expensive steps after a blocked result. A valid final
performance comparison requires the separately declared design and qualified
candidate required by the completion checklist.

## Preserve candidate correctness independently

The three selector suites previously ran only inside the historical entry
workflow. They now also run as mandatory steps of `compiler-set.yml`, using its
existing source-built package and the same production profiler constructor:

- Exact Cargo selection and cache invalidation.
- Seed verification and safe fallback.
- Complete compact-selection equivalence.

The suites and their arguments are unchanged apart from their owned output and
package directories. A failure in one suite does not suppress the other two or
the independent mixed/compiler-entry suites. It blocks every existing mixed
profiling and measurement admission. Missing, failed, skipped or cancelled
selector steps cannot count as success. The job retains each original result
and stream set, the profiler's source/build receipt, and all four selector
construction/suite outcomes even after failure.

This uses the already constructed candidate; it does not introduce another
package build or replace a performance campaign. Historical entry workflow
selector steps also remain intact for an applicable comparison.

## Verification scope

The [verification record](evidence/warm-applicability-20261119/verification.json)
separates real Git-source checks from synthetic evidence controls. The two
current historical source combinations fail with their specific reasons; the
actual identical entry baseline passes only applicability. A conflicting SDK
declaration fails without transferring historical qualification. Applying the
new workflow controls to the previous workflow files fails because their early
checks and independent selector qualification are absent.

Owned Git fixtures cover all seven shared inputs, dirty sources, altered pins,
missing and altered evidence, recomputed misleading evidence checksums,
preservation of prior receipts, the original late guard, and the absence of an
override. Existing complete-package, entry and mixed-workflow controls remain
required. These are orchestration controls. No compiler build, runtime suite,
hosted execution, timing campaign or package publication was performed for this
change.
