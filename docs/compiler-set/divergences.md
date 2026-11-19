# Source ownership and divergence

The [exclusive coverage contract](exclusive-coverage-contract.md) is the current
direction. Lustr preserves Lust's implementation for covered work and imports
only justified upstream complementary source. The complete Rust source remains
in blueFe. Read the [source register](sources.md), [upstream policy](upstream-lust.md)
and [retained-work disposition](retained-work-disposition.md).

The observed default at
[7cea76c9](https://github.com/glopyglerky/lustr/tree/7cea76c9d3adc587b54ccff168fe07efafd40309)
and audited [PR #32 cb0721d4](https://github.com/glopyglerky/lustr/tree/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3)
contain different integration work. Neither is the completed replacement.
A source-preservation gate establishes only its documented identity checks,
not full coverage, SDK identity, runtime behavior or speed.

Necessary native integration/correctness patches require explicit source and
patch identities, a concrete need, applicable validation and measured effects
where relevant. A correctness repair need not create a speedup. Hidden overlays,
renamed donor copies, unrecorded build patches and disabled guards are prohibited.

Current TinyCC, C ABI and upstream code-generation dependencies must remain
visible. Changing languages or reconstructing the entire toolchain is not an
alternative goal for this work. Preserve the current Lust speed mechanisms and
replace only what the full-compilation boundary demonstrably requires.

Earlier source layouts, compatibility coordinators, source-movement decisions,
compiler tuning and proposed native rewrites are preserved in the
[prior divergence record](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/divergences.md).
They retain historical evidence but do not instruct new implementation. Product
routes that conflict with the exclusive contract must be retired through a
reviewable implementation change; this docs-only correction does not claim
that removal already occurred.
