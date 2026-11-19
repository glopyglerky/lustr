# Original Linux run at 62da

The previously pending [Linux job
135566552186](https://github.com/glopyglerky/lustr/actions/runs/44525981171/job/135566552186)
completed successfully. This record preserves that original outcome without
rerunning it. The overall workflow concluded with failure because its separate
macOS job failed. [run-source.json](run-source.json) preserves those distinct
conclusions and every reported Linux step.

## Source and compiler identity

The workflow head was `62da158b33740279947c85f3fc425df6d1623c9f`. GitHub checked
out merge `a3c878f6123b4f5dc5e2eff598b52c83ab5b4ec3`, whose tree is
`390e25946a5ebcb8ccef417467a7b29d8034987d`, exactly the original published tree.
The job used Rust source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` and declared
two differential-test workers. [Compiler and build
receipts](artifact-receipts/native-gate-identity.txt) preserve the SDK and source
hashes.

The produced native binaries have these SHA256 identities, also preserved in
[the original receipt](artifact-receipts/native-gate-builds.sha256):

| Binary | SHA256 |
| --- | --- |
| `lust` | `aedc107f653d98c13b25cd353284eb1b5eb3d0164a13c1a42afd2d8539341f70` |
| `lust-core` | `b0a917d7891175b352e75750c04e05b1e6870d54ecbd99555d3746d63a7f538a` |

These are direct native-build results. Qualification of the installed public
Lustr package and the later implementation tree has separate evidence.

## Reported results

The counts below retain their test mode and scope. Repeated modes are not added
into a supposed number of unique cases. [results.json](results.json) also records
the smaller dependency, sanitizer, host and artifact-retention controls.

| Check | Original result |
| --- | --- |
| Native unit tests | 22 entry tests and 417 core tests passed |
| Host runtime and SDK | 31 runtime tests passed, 1 ignored; 6 SDK tests passed |
| Range helper | 2,205 cases, including 1,505 invalid bounds; zero failures |
| Build-script transition | Passed in both declared Cargo output layouts |
| Ordinary differential | 5,547 passed, zero failed |
| ASan/UBSan differential | 5,547 passed, zero failed |
| Macro and macro-dependency fixtures | 58 and 13 passed |
| Kernels, client fixtures, crate fixtures | 43, 89 and 85 passed |
| Original donor daemon runner | 364/364 reported OK; 29 unsupported versions retained |
| Edit fuzzing | 1,200 edits; zero failures and zero guard activations |
| Native unwind and emitted-C unwind | 65 passed in each mode; zero failed or unsupported |
| Unicode | 1,112,064 scalar values; 8 exact-output comparisons |
| Erased callbacks | 10 passed in each of ordinary and unwind-table modes |
| Main-return transitions | All 4 versions matched independent Rust in warm and cold execution |

The real-daemon base-image lifetime step and slot identity checks passed.
The donor runner's 364 histories and 29 unsupported versions retain its original
acceptance rules. They do not establish the separately qualified public
349-history campaign. The native gate also completed its one-repetition
benchmark smoke; those observations do not establish near-stock subset speed,
overall public Lustr speedup, compiler completion or release admission.

## Preserved originals

[original-job.log.gz](original-job.log.gz) is the complete decoded GitHub job log,
compressed without changing its content. The uncompressed SHA256 is
`8c2d4729bc1061a563c8d5e1f1f85dc80ae431dfed513df60e4032259de33edd`.
[file-index.json](file-index.json) records the compressed file and selected
receipt hashes.

The original GitHub artifact is `14541155848`, named
`native-gate-Linux-X64-a3c878f6123b4f5dc5e2eff598b52c83ab5b4ec3-1`.
It contains 569 members in 48,436,126 bytes. The downloaded archive matches
GitHub's published SHA256
`dff8ac3705cc56a618de144c76129a6e02805151d59fb8e91e8aa6378b137432`.
[artifact.json](artifact.json) records its API location, retention date, verified
local copy and the exact selected members preserved here. The large ZIP remains
outside Git.

This evidence update changes no source, workflow, fixture, acceptance rule or
historical result.
