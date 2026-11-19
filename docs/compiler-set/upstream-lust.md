# Preserve upstream Lust and import the Rust complement

The [architecture plan](architecture-plan.md) governs: Lust supplies its supported
implementation and retained frontend; version-pinned upstream Rust supplies the
complement inside Lustr. This policy preserves provenance and working semantics
while permitting the explicit integration and correctness work needed to join
them. It does not require independently implementing the rest of Rust.

## Pristine reference and integrated candidate

[`upstream/lust.pin`](../../upstream/lust.pin) identifies pristine Lust
`240e08d91d4aed0299639f6f11fd522912b248f9`, tree
`0ab1b09815802894e6078de0af59273cbdf46cb0`. Retain this exact reference for
provenance and applicable comparisons. A reference pin is not a claim that the
upstream head has been rechecked today.

The default snapshot `7cea76c9d3adc587b54ccff168fe07efafd40309` keeps
`compiler/native` identical to that reference. Its existing guard enforces that
identity. Later candidate branches have explicit integrated-tree records and
reviewable patches. For example, the architecture-audited
[PR #29 source](https://github.com/glopyglerky/lustr/tree/a9fd4c6336b1e39c4f584f7da50a437c5c248a0d/upstream)
records native tree `072691a23b7a5f664d3c07500315de3ce051a22f`.

Keep pristine and integrated identities separate. Importing a reviewed patch
requires the matching source-guard implementation, complete diff, integration
tree and qualification record. This documentation change does not alter the
default guard or grant an undocumented source exception. Never bypass a failing
guard, hide a fork under a different name, or generate unrecorded build patches.

## What admits a necessary change

An upstream integration or correctness change needs:

1. The exact baseline, minimal proposed diff and affected interface.
2. A concrete need: a reproduced defect or missing integration capability, and
   why an external owned interface alone cannot provide it.
3. Matching acceptance, rejection and observable runtime evidence, including
   ownership, panic/drop, ABI and retained edit behavior where affected.
4. Source/patch identities, dependency provenance and preserved licenses.
5. Performance characterization where the affected path warrants it, under the
   practical tolerances in the [performance checkpoint](performance-review-2026-11-18.md).
   Keep regressions visible and preserve Lust's supported fast path.

**A necessary correctness repair is not required to produce a speedup.** The
former rule requiring every Lust change to beat pristine Lust is superseded by
the user's explicit correctness-first, near-parity requirement. Source review
and applicable qualification still apply. A fast invalid result is never a gain.

Lustr-owned integration should remain outside the native tree where an honest,
qualified interface permits it. Do not force unrelated source relocation merely
to make a patch appear external. Preserve the original compiler, libraries,
metadata, tests and relevant history. Record gitlinks, generated dependencies
and external assets separately; a tracked-tree check does not qualify them.

## Imported Rust implementation

[`upstream/bluefe-core.pin`](../../upstream/bluefe-core.pin) binds the declared
Rust source and SDK. Reuse the required upstream resolution, trait/type, layout,
metadata, code-generation and runtime services inside the integrated compiler.
Their dependency closure may include substantial upstream source. Do not assume
the missing features form a separable directory or a compiler that can simply
be cut out of rustc.

The observable contract is which work executes and owns semantic facts. Importing
source is compatible with the goal; unconditionally performing a second full
supported compilation or retrying the whole program through stock rustc is not.
Shared services must have explicit ownership and dependency tracking. Preserve
the exact Rust version's semantics and genuine crate metadata.

Any work in the separate blueFe repository must inspect its current policy and
pins. This Lustr documentation change does not modify another repository or
claim that a source-built complementary compiler has already replaced its SDK.

## Evidence and release

Source identity, architectural completion, correctness, performance and package
qualification are separate conditions. A passing source guard proves only its
documented identity checks. A scalar bridge result does not qualify all of Lust
or the full Rust replacement.

Keep failed and inapplicable historical comparisons unchanged. A candidate fix
cannot repair a frozen old baseline retroactively. Select an applicable final
comparison after correctness rather than weakening a baseline guard. Preserve
raw timing data and disclose original workloads, profiles, cache states, source
identities and uncertainty.

The previous policy remains available at
[default snapshot 7cea76c9](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/upstream-lust.md).
Its faster-results-only and independent-expansion requirements are historical,
not instructions for new work. The [completion checklist](completion-checklist.md)
records the remaining implementation and release conditions.
