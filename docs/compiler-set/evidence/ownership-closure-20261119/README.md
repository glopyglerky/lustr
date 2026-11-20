# Selective research admission and terminal inspection refusal

This follows the same PR #33 correction. The local base is `b8fd1d1`, following
the preserved `9bb0f19` continuation and the metadata inventory/CI change.
[verification.json](verification.json) binds each executed command, tool,
source input and output stream. The source hashes, rather
than the base commit alone, identify the tested follow-up.

Published arguments, environment values and streams replace only machine-specific
scratch, SDK, Cargo cache and output prefixes with named placeholders. Both
original and published stream hashes are retained, as are each original record's
hash and the unchanged source/tool identities. The placeholders are not literal
paths used by the original commands.

## Findings and corrections

The new metadata serializer and SDK host adapter were absent from the central
research inventory and CI path triggers. Their source records now remain
explicitly research-only, with no committed native-to-encoder caller. The
structural guard binds the adapted source, donor declarations, dependency
roles and actual library/package/workspace selection. It rejects a manifest
that redirects the library or workspace while leaving the reviewed source
hashes intact. CI names the intended packages explicitly.

The existing native-entry job also gains a required build/run of the actual
`native-consumer-qualification` example with `native-qualification` only. Its
17 controls, three native functions and 131,076 C ABI runtime results must be
present, and source/tool/binary identities and failed outputs have their own
required artifact upload. The original supplemental root list and upload are
unchanged. This new hosted step has not executed locally or on a hosted runner
as part of this record.

The independent Rust semantic inspector delegated non-UTF-8 invocations to
an arbitrary supplied executable before checking compiler identity. That path
performed no inspection. It now fails with status 2 before any query or
execution. Both platform delegation functions are deleted. The actual probe
test plants executable sentinels for invalid argument and executable-name
bytes and requires no invocation, stdout or report. The retained UTF-8
inspection path remains an independent original-Rust oracle, outside product
compilation and outside any Lustr coverage claim.

Current metadata descriptions also distinguish the intended caller from the
committed completed-record/C ABI qualifier. The target-modifier note no longer
claims that an unpublished joined attempt is included in its record. Original
historical stream and source hashes remain unchanged.

## Observed local results

| Check | Result |
| --- | --- |
| Current structural import and ownership guard | Passed with full Rust coverage false and release unavailable. |
| Negative structural controls | Eight altered inventories, 18 product dependency mutations, one restored retired route, nine metadata mutations and two restored inspector passthroughs all rejected. |
| Independent SDK receipt controls | All 14 passed, including failed-command retention and malformed/foreign receipt refusals. |
| Original encoding/hash controls | All four passed with no ignored or filtered tests. |
| Serializer dependency cut | The explicitly named library passed `--all-features --target x86_64-unknown-none --locked --offline`. |
| Separate host adapter | The explicitly named library passed its locked offline check. |
| Changed Rust inspector source format | Passed using the available pinned 9 November formatter. This does not replace the old inspector SDK's hosted format check. |

[final-verification.json](final-verification.json) preserves the later workflow
and research-description changes separately. It records fresh ownership controls,
the workflow's required triggers, exact package/example selection, shell syntax
and failure uploads, plus all 19 existing upload/applicability controls. The
first record's workflow identity remains its original value. Only the workflow
and central research description changed after the first executions; all tested
compiler/component inputs retain their earlier hashes. Workflow review is not
hosted execution.

The required 5 November SDK is unavailable locally. The changed complete
semantic inspector was not rebuilt here, and its new real sentinel test was
not executed locally. Its existing pinned hosted job must run that test; a
structural refusal check does not establish its runtime result.

The serializer checks qualify only this bounded dependency and encoding work.
They do not establish a joined native Rust artifact, compiler-wide `no_std`,
socket independence, full Rust coverage or a performance result.
