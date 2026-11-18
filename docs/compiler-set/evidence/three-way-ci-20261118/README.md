# Hosted three-compiler dashboard qualification

[PR #21](https://github.com/glopyglerky/lustr/pull/21) now has a successful
[Compiler set run](https://github.com/glopyglerky/lustr/actions/runs/44264239249).
All ten jobs passed, including the complete source-built package and the new
three-compiler reporting step. The hosted report was independently regenerated
from its original input artifacts and matched byte-for-byte.

The [complete dashboard](dashboard/README.md) has 29 rows with separate Lustr,
pristine Lust and rustc columns. It contains 73 measured cells and 14 Lust
cells marked NT because no matched measurement or support determination exists.
It does not issue N/A without an executed unsupported-feature gate bound to the
unchanged workload and exact pristine compiler invocation.

## Exact measured source

| Role | Identity |
| --- | --- |
| PR head | `28a2842635ee994db492b43f8c956eddce57a803` |
| Actual workflow checkout | `9480f81232f6c5730cd537a1f6f633fa90e79b85` |
| PR head and workflow source tree | `67fb0c88762b8d278dc7c53ab8a4858a00230673` |
| Pristine Lust revision | `240e08d91d4aed0299639f6f11fd522912b248f9` |
| Pristine Lust tree | `0ab1b09815802894e6078de0af59273cbdf46cb0` |
| Matched Rust source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |

The Rust reference is the standard rustc in the matching pinned SDK,
`1.102.0-nightly`, rather than a different stable compiler release. The fixed
workload uses its recorded development opt0/debug0 profile. The unchanged
coordinator retains its normal development/test defaults, including debug=2,
which currently takes the full Rust path.
[identity.json](identity.json) retains compiler hashes, constructor receipt
hashes, source provenance and the original GitHub artifact digests.

This is PR #21's own campaign on its PR #16-based stack. It is not a controlled
before/after comparison to PR #18 or qualification of the combined PR #17/#18
candidates.

## Correctness and independent replay

The [source-built job](https://github.com/glopyglerky/lustr/actions/runs/44264239249/job/134664876762)
built the complete optimized package and passed the existing mixed Cargo,
retained checks, fresh metadata, library interoperability and failure recovery
gates. All 16 entry/runtime contract tests passed. Cargo compatibility passed on
Ubuntu, Windows and macOS. Native and mixed package execution in this job was
Linux x86-64; Cargo compatibility does not establish native macOS qualification.

The reviewer downloaded six benchmark/provenance ZIP artifacts and checked
every ZIP against GitHub's SHA-256 digest. The unmodified reporting code at the
exact published head then regenerated the dashboard from the original fixed,
public-slice and unchanged-project evidence. All three output files matched the
hosted dashboard exactly.

- The original fixed benchmark replayer verified 648 timed commands, 36
  comparisons and 2,160 command outputs.
- The original public-slice replayer verified 288 timed commands and 18
  comparisons.
- The project verifier checked the original public export checksums, retained
  Rust replay, all 120 timed commands and all five paired comparisons. Its 20
  warmup commands remain excluded from the statistics.
- The separate output replay reproduced the Markdown and checksum manifest
  from the dashboard JSON.

No compiler or benchmark command ran during this review. Full raw-input replay
and output-only replay are different checks; output-only replay verifies the
derived report's integrity, not its source measurements.

```sh
python3 tools/report-three-way-performance.py \
  --replay docs/compiler-set/evidence/three-way-ci-20261118/dashboard
```

[qualification.json](qualification.json) records the observed jobs, steps and
replay counts. [project-public](project-public/README.md) is a byte-identical copy
of the original sanitized project export. The original hosted project ciphertext
and receipts are copied unchanged into [project-encrypted](project-encrypted/receipt.json).

## Measured performance remains below the target

The fixed Cargo campaign retains a body-edit ratio of
**1.0897 [1.0835, 1.0954]** and a rejection ratio of
**1.0493 [1.0401, 1.0625]**. These are paired geometric Rust-time/Lustr-time
ratios with bootstrap 95% intervals. Every other fixed Cargo mode has an
interval below one. All original ratios and intervals remain in
[dashboard/report.json](dashboard/report.json).

The warm public-entry gap remains substantial. Median milliseconds from the
same run are shown below; lower is faster.

| Public-slice operation | Lustr | Pristine Lust | rustc through Cargo |
| --- | ---: | ---: | ---: |
| No-op check | 32.059 | 1.305 | 8.974 |
| No-op build | 34.546 | 3.316 | 9.143 |
| Compile and run | 34.889 | 4.297 | 9.846 |

All five unchanged-project comparisons remain visible. Primed-root check has
a Rust/Lustr ratio of **0.9769 [0.9564, 0.9988]**. The other four intervals cross
one. None establishes a speedup, and crossing one does not prove equivalence.
These observations do not isolate the cost of metadata preflight or attribute
any change in speed to the reporting implementation.

## Separate macOS setup failure

The broader [CI run](https://github.com/glopyglerky/lustr/actions/runs/44264239099)
was still running at the observation recorded here. Its
[macOS ARM64 job](https://github.com/glopyglerky/lustr/actions/runs/44264239099/job/134664704136)
failed before building or testing the native compiler. The sanitizer setup
installed Homebrew Clang **23.1.6**, then its existing exact-version assertion
required **23.1.5** and returned exit 1.

That setup failure does not resolve or reproduce the older incompatible
function-pointer sanitizer failures. The older failed evidence and the native
platform qualification requirement remain in force. No sanitizer was disabled,
no version assertion was relaxed, and no unchanged timing campaign was rerun
for this review.

## Retention and review limits

[original-artifacts-encrypted/receipt.json](original-artifacts-encrypted/receipt.json)
records an authenticated encrypted archive of the six exact downloaded ZIPs,
the source-built job log, the failing macOS setup log, the fetched metadata and
the evidence staging script. The ZIPs remain intact, including any nested cache
or metadata entries present in their original hosted artifacts. The generic
retention tool's file-selection rule applies to the staged archive files; it
does not filter inside ZIPs. The exact IDs and hashes are in identity.json.

Encryption used the repository's already pinned public recipient certificate.
No private key was accessed. The plaintext archive contents and manifest were
verified before encryption, and the resulting CMS structure was checked.
Independent decryption was not performed. The retained source-built artifacts
contain source, binary identity and execution evidence; they are not a complete
runnable compiled toolchain package.

This separate evidence commit changes no compiler source, workflow, benchmark
input or existing historical report. It qualifies the dashboard's hosted
execution and data derivation. Performance admission, native-platform admission,
the unsupported-feature gate and merge admission remain open.
