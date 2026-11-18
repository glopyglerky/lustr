# Compact selection experiment

The compact record passed the bounded selection and Rust fallback gates and
reduced the measured selector component. It has not established public warm
check/build parity or qualified a native compiler change.

| Revision | Role |
| --- | --- |
| 93d882632570e1b0413630abaee939dcab16fd01 | Corrected baseline selector |
| 6f59c902a0890413ff82ae4a572fac7e362919a9 | Executed compact product and helper |
| 422ce5f111d934899f3c522e3b8ed77e2986fc86 | Same product/helper bytes; adds focused test and updated proof/documentation |

The native compiler, runtime, shared host support, SDK pin and imported source
closure are unchanged. The identity.json file binds the exact Git trees, frozen
helper source closures, helper binaries and original report hashes.

## Change and trust boundary

Fresh requests still execute Cargo metadata, optional Cargo pkgid and the
original unique-selection filter. Only that successful writer can store the
complete selected package and target. Version 3 replaces the stored full input
snapshot with its SHA-256, while retaining all live observations and every
target-source binding. Tool certificates and their seeds keep their old schema.

The recorded metadata hash is provenance, not authentication. The cache trusts
its successful writer and private owner; it does not defend against the same
owner coherently forging records and recomputing checksums. No semantic source,
module or expansion validation is removed.

## Executed correctness evidence

The existing transition suite passed **140 invocations and 265 assertions**.
Its cached/fresh comparisons now include every field of the complete Selection,
including the selected package and target used by Rust fallback.

The focused gate passed **103 commands and 486 assertions**. Fifteen malformed
records receive valid recomputed checksums, so structural checks are exercised
independently from checksum corruption. Cases include the old schema, missing
or changed input digests, missing selection/package, inconsistent manifest or
name, invalid target kinds, mismatched target source, absence from the package's
target list, missing target bindings and malformed metadata provenance.

Actual production Rust fallback runs cover check/build, explicit package
selection, changed output location, changed debug-assertion settings, a second
target, invalid Rust, and restoration. Eight executions of freshly published
ordinary binaries match exact exit status, stdout and stderr; cached/fresh
artifact hashes agree. Invalid Rust returns Cargo status 101 with E0308 and
publishes no executable. This is a selector/fallback component gate, not a
retained native-daemon execution.

The independent reviewer read the product, tools and updated conditional proof
and replayed all command streams and result hashes. The original review receipt
predates the timing run and preserves that timing as pending at review time.

## Single predeclared comparison

The fixed schedule used three initial primes per lane, 16 excluded pairs and
96 measured pairs, with balanced lane ordering. Both lanes used the same
fixture and package. Cache-hit diagnostics were enabled identically, and all
excluded and measured retained requests were hits. This instrumentation and
the component seam prevent treating these numbers as whole-command latency.

| Actual selector component | Baseline | Compact |
| --- | ---: | ---: |
| Median | 435.237 µs | 383.465 µs |
| Private selection record bytes in this context | 8,458 | 1,651 |

The paired geometric speed ratio is **1.1339×**, with a paired bootstrap 95%
interval **[1.0513, 1.2166]**. All 230 raw commands, 192 measured observations,
whole-helper wall times, initial populations and the frozen executed runner
remain retained. There was no repeat to seek a favorable result. Whole-helper
wall times are auxiliary observations and are not added to or subtracted from
the separately measured selector duration.

A separate shared source concern remains outside this comparison: file stamps
and readable tool opens do not completely observe changes to effective
execution credentials. No credential-changing runtime probe was executed here.
That correctness correction must be identified separately in later results.

## Replay and reproduction

Run the portable offline replay from any directory:

    python3 PATH_TO_THIS_DIRECTORY/verify-evidence.py

It checks the complete public inventory, original-source/build bindings,
projected raw stream hashes, assertion counts, fixture bytes and the paired
bootstrap result. Public stream hashes describe the role-projected bytes;
the original stream hashes remain alongside them. Original bytes, paths,
artifacts and source/build receipts are retained with the repository's existing
recipient certificate in encrypted/. No private key was accessed.

For a fresh component execution, build the selector helper from the stated
revision with tools/run-warm-entry-comparison.py build-selector-gate, a
matching source-built package and its native build receipt. Then run:

    python3 tools/test-native-selection-cache.py --profiler HELPER \
      --package-bin PACKAGE/bin --output NEW_SELECTOR_OUTPUT
    python3 tools/test-native-compact-selection.py --profiler HELPER \
      --package-bin PACKAGE/bin --output NEW_COMPACT_OUTPUT
    python3 component/executed-runner.py --before BASELINE_HELPER \
      --after COMPACT_HELPER --package PACKAGE --fixture FIXTURE \
      --output NEW_COMPONENT_OUTPUT --pairs 96 --excluded-pairs 16

The public fixture archive contains the exact measured fixture. The helper
recipes and source closures are retained; generic path names above must be
replaced with new, owned locations. These reproduction instructions do not
promise identical elapsed times on a different machine.
