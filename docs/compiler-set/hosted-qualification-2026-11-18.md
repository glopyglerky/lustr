# Hosted Linux qualification at 429fe605

The executed native Linux and source-built compiler jobs passed on 18 November
2026. Both workflow conclusions remain **failure**. Each has one macOS job
cancelled before any step was recorded. No executed job or recorded step in
either workflow failed. The job metadata establishes non-execution on macOS;
it does not establish why the runners cancelled those jobs. Their log endpoints
returned HTTP 404 `BlobNotFound`.

| Workflow | Executed jobs | Job without execution | Original result |
| --- | --- | --- | --- |
| [CI 44363351365](https://github.com/glopyglerky/lustr/actions/runs/44363351365) | Source provenance and Linux native qualification passed | [macos-arm64](https://github.com/glopyglerky/lustr/actions/runs/44363351365/job/135010409182) | Failure |
| [Compiler set 44363351391](https://github.com/glopyglerky/lustr/actions/runs/44363351391) | Nine jobs passed, including the complete source-built candidate and Linux/Windows Cargo compatibility | [macos-26 Cargo compatibility](https://github.com/glopyglerky/lustr/actions/runs/44363351391/job/135010410465) | Failure |

The [sanitized receipt audit](evidence/hosted-pr28-2026-11-18/verification.json)
lists every job, exact source and compiler identities, all 33 strict transition
command outcomes, and the original artifact and log hashes. The audit checked
existing evidence without running another compiler or benchmark. These two runs
provide no reason to weaken a check or change the ABI repair.

## Exact source

Both workflows tested the same GitHub pull-request merge. The head and merge
have identical trees, verified against local immutable Git objects and the
hosted source receipts.

| Identity | Value |
| --- | --- |
| [PR 28 head](https://github.com/glopyglerky/lustr/commit/429fe6054e6119221e598b95c2661665ecdd7be2) | `429fe6054e6119221e598b95c2661665ecdd7be2` |
| Tested merge | `84257ab1e63469a3162f1ebd3b9177b3fd4a371e` |
| Source tree | `7caaeb641eba6dda5b26b9d3d6761ca3a5a79581` |
| Native tree | `d9ca06dc18dde19bf5bd2af688adf75a041b12a5` |
| PR base | `43a7e07d32929309fc6e402d7bca6b5673e186a2` |
| Attempt | `1` for both runs |

Later capability-boundary changes need qualification against their own source.
This record does not transfer these results to a later candidate.

## Native Linux results

[Linux job 135010409278](https://github.com/glopyglerky/lustr/actions/runs/44363351365/job/135010409278)
passed the full native CI script, the real-daemon base-image lifetime test, and
the independent strict main-return transition gate. It used the Rust 1.99.0
oracle at `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, Ubuntu 24.04 on x86-64,
and GCC 13.3.0 for sanitizer capability checks. The recorded host was an AMD
EPYC 9V74 with four visible logical CPUs. Differential concurrency remained two.

| Native suite | Recorded result |
| --- | --- |
| Ordinary differential | 5,528 passed, 0 failed |
| Sanitizer differential | 5,528 passed, 0 failed |
| Legacy daemon scenarios | 364 of 364 passed, with 29 unsupported versions reported |
| Macro corpus | 58 passed, 0 failed |
| Macro dependency fixtures | 13 passed, 0 failed |
| Kernel fixtures | 43 passed, 0 failed |
| Client tests | 89 passed, 0 failed |
| Crate fixtures | 85 passed, 0 failed |
| `lust-port` tests | 13 passed, 0 failed |

These are separate suite counts, not a deduplicated total. The sanitizer result
is the Linux GCC run. It does not establish a macOS Clang qualification.

The strict gate accepts every compilation, requires retained daemon recheck and
build statistics, and compares the actual warm and cold executables with rustc.
It does not count rejection, signal termination or timeout as runtime parity.
Its four consecutive source versions produced the following results.

| Version | Main return | rustc exit | Warm exit | Cold exit | stdout and stderr |
| --- | --- | ---: | ---: | ---: | --- |
| `v1.rs` | `Result<(), &str>`, error | 1 | 1 | 1 | Exact byte match |
| `v2.rs` | Unit after Result | 0 | 0 | 0 | Exact byte match |
| `v3.rs` | `Result<ExitCode, &str>`, code 7 | 7 | 7 | 7 | Exact byte match |
| `v4.rs` | Unit after ExitCode | 0 | 0 | 0 | Exact byte match |

All 33 recorded commands completed without timeout. The receipt audit checked
all 66 stdout/stderr hashes, all four fixture hashes against the pinned source,
and the retained recheck/build statistics. It also checked that the gate's
compiler hashes agree with the build receipt. The recorded cgen SHA-256 is
`1cbce6afffc8f3df96032b5588736cbd861033feba8111bec5047889472c12b1`,
the exact [reviewed main-type repair](native-main-type-repair.md). The runner
SHA-256 is
`1f391308d2bb4671b211408282ef0ab7a446ccfc38a1450e78a477d9ff56879e`.

This closes the previously pending hosted Linux daemon check for that repair
and source tree. The older failures remain retained in the repair record.

## Source-built compiler results

[Source-built job 135010409987](https://github.com/glopyglerky/lustr/actions/runs/44363351391/job/135010409987)
built the complete candidate from source with its normal constructor and release
profiles. This was a new construction, not the earlier diagnostic launcher
replacement. Its imported Rust SDK was `nightly-2026-11-09`, Rust
`1.102.0-nightly` at `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.
The host was Linux x86-64 on an Intel Xeon 6975P-C with four visible logical
CPUs. This is a separate build and host from the native gate above.

The job passed target-layout comparisons, the mixed Cargo graph and edit tests,
retained checks with freshly encoded metadata, and mixed library tests with
aggregates, references and generic callers. **All 17 public-entry contract
tests passed, with zero failures, ignored tests or filtered tests.** That
includes the new real-metadata cleanup/error regression, process cancellation,
output failures, library identity, Cargo selection and build-script environment
checks. The [full check repair](full-check-metadata.md) now has this complete
hosted entry-suite result in addition to its earlier local evidence.

The intentionally unchanged failed-link baseline still reproduced its expected
failure with exit 101. It printed the old result `1080` where `1081` was required.
The workflow required that witness and then passed the candidate's mixed
qualification. The baseline failure is retained evidence, not a failed
candidate job.

This qualification exercises the implementation present at the recorded tree,
including its transitional compatibility routes and limited mixed bridge.
It does not establish the [required integrated architecture](architecture-plan.md),
universal full-Rust coverage, a new performance claim or release admission.
Windows Cargo compatibility passed separately; that is not Windows native
compiler qualification. macOS remains unqualified in these runs.

## Original artifacts and audit limits

The downloaded ZIPs matched GitHub's reported size and SHA-256 before the audit
read them. The original ZIPs, decoded job logs and metadata are retained
privately. The public JSON is a projection without machine paths or original
raw logs. Compiler binaries were not included in the downloaded artifacts, so
their identities are the hashes recorded by the hosted build and gate.

| Original artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [Native gate 14462711425](https://github.com/glopyglerky/lustr/actions/runs/44363351365/artifacts/14462711425) | 36,485 | `8709ef8ce41d09472e6059fe9ce4ff4beb227cc5c038e7136df864e3dffcf94d` |
| [Native source receipt 14454857392](https://github.com/glopyglerky/lustr/actions/runs/44363351365/artifacts/14454857392) | 449 | `3ccae4cb6ab683369875130a9ab8e735ed786c085823c5958ea41012e5689d12` |
| [Source-built qualification 14458797002](https://github.com/glopyglerky/lustr/actions/runs/44363351391/artifacts/14458797002) | 1,358,884 | `1daad6355908d546ca7a272c824ccb77f725171dcfe27735daee541c5f5fc158` |
| [Compiler-set source receipt 14456651799](https://github.com/glopyglerky/lustr/actions/runs/44363351391/artifacts/14456651799) | 456 | `c48743fe7a3ef9d899fee4a9eed5f554d2bf6c37c3405703c9dcf3ae0b0f02e6` |

The audit verified the strict transition streams and their source bindings.
The broader suite counts are the original CI summaries, supported by successful
job/step records. No benchmark was rerun or accepted as part of this audit.
The incomplete macOS jobs and both original aggregate failure conclusions are
preserved.
