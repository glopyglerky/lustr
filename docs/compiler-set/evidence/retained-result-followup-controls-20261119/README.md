# Retained-result qualifier followup controls

These controls repair qualification mistakes exposed by the first public retained
campaign at `62da158b33740279947c85f3fc425df6d1623c9f`. That campaign remains a
failed run. Its original results are preserved in
[the separate hosted record](../retained-hosted-62da158b/README.md). The controls
here are not a new native compiler campaign and establish no speed claim.

## What the actual failures established

Four invalid Rust versions fail only when their demanded code is generated:
`cyc09_allow_nested_chunk/v4` and the `v2` versions of
`sc02_assoc_const_request_edits`, `sc03_assoc_const_plain_header_template` and
`sc04_assoc_const_template_body`. The pinned Rust compiler accepts metadata-only
emission for each but rejects no-link LLVM IR emission with source-bound
arithmetic-overflow or E0080 diagnostics. The qualifier now uses the latter
check after a failed direct Rust build. The original compiler options and source
remain unchanged; this check runs only in qualification.

`sd23_std_impl_partialeq/v2` has six E0369 errors. Two primary spans point into
standard-library macro definitions and carry their source call sites through
`expansion.span`. The qualifier follows only primary-span call-site ancestry.
Definition sites, secondary spans and notes cannot supply a missing anchor.
Every substantive error must have an existing source origin in the fixture.

The pinned `cargo01_bin_discovery/v7` overlay removes `src/bin/e.rs` while
retaining the `--bin e` request. Both pinned Cargo build and check reject that
absent target. This is an invalid request before compilation. Its single
reviewed expectation requires the exact unchanged request, absent source,
present manifest, specific Cargo diagnostic, four public selection rejections,
no compiler ownership event, no artifact execution and the original daemon
remaining alive. The following `v8` source restoration must compile and produce
the correct result normally. It remains part of the 349 required histories.
Other missing ownership traces fail with their actual public diagnostic and
exit status; they cannot use this expectation.

## Why source spans alone are insufficient

A new real process control failed during this repair: a missing SDK emits E0463
for absent `std` with a synthetic primary span in `main.rs`. That initial
misclassification remains in `first-negative-control-failure/`, with its original
runner identity. Missing `--extern` artifacts and invalid crate metadata can
also attach diagnostics to the importing source. The qualifier explicitly
refuses these setup failures, incompatible compiler/dependency versions, and
ambiguous artifact candidates. A missing linker fails the full build but
succeeds under no-link emission, so it also cannot establish an invalid-source
verdict. A valid source-error diagnostic mixed with a compiler crash or failed
custom-build command remains a failure. These checks use specific compiler and
process summaries, not a general match for words such as `panic` in source text.

## Verification and limits

[verification.json](verification.json) binds the tool, runner, test and inventory
identities and every retained output stream. The completed run passed all **55
tests**: **50 admission, synthetic-process and lifecycle controls**, plus **5
actual pinned Rust/Cargo process tests**. The latter recorded 17 compiler
commands and asserted the result of one execution of the restored Cargo oracle
program. The earlier 53-test checkpoint remains separately identified. The compiler
identity is the declared SDK source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.

The synthetic lifecycle controls reuse the unchanged donor overlay loader and
cover the complete eight-version target-discovery history. A stale `v8` program
fails after the correctly rejected `v7`. Other controls reject missing traces,
unknown target failures, wrong Cargo diagnostics, malformed diagnostic JSON,
missing SDK/dependency artifacts, direct compiler crashes and non-source errors.
The original donor's file-handle ResourceWarnings remain visible in the log.

Constructor CI runs the pinned oracle controls explicitly before the actual
public retained campaign and uploads both outputs. The separate lightweight
control job runs the 50 non-oracle controls and explicitly skips the five pinned
process tests unless their pinned tool paths are supplied.

All 349 required histories and 1,744 versions remain required. The 13 unresolved
histories and 105 versions remain unresolved. The 15 genuine public Cargo graph
refusals found by the original hosted run are product integration failures;
these qualifier repairs neither exclude nor admit them. Passing these controls
does not establish full retained coverage, full Rust coverage or release
admission. The next fresh package campaign must establish actual native results.

## Pinned source evidence

- [Donor scenario loader](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/test-daemon-scenarios.py)
- [Target discovery and deletion history](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9/tests/daemon/cargo01_bin_discovery)
- [Nested overflow history](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9/tests/daemon/cyc09_allow_nested_chunk)
- [Associated-constant demand history](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9/tests/daemon/sc02_assoc_const_request_edits)
- [Standard macro equality history](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9/tests/daemon/sd23_std_impl_partialeq)
- Rust artifact diagnostics: [inconsistent dependencies, E0460](https://doc.rust-lang.org/error_codes/E0460.html), [ambiguous candidates, E0464](https://doc.rust-lang.org/error_codes/E0464.html), [incompatible compiler version, E0514](https://doc.rust-lang.org/error_codes/E0514.html), and [invalid metadata, E0786](https://doc.rust-lang.org/error_codes/E0786.html).
