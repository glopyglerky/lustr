# Compiler source register

Reviewed 19 November 2026. This register distinguishes reference source,
partial imports, research dependencies and required product behavior. Follow
the [exclusive coverage contract](exclusive-coverage-contract.md).

| Source | Exact observed identity | Role and limit |
| --- | --- | --- |
| Pristine Lust | `t3dotgg/lust-lang`, `240e08d91d4aed0299639f6f11fd522912b248f9`, tree `0ab1b09815802894e6078de0af59273cbdf46cb0` | Reference compiler and speed mechanisms preserved by `upstream/lust.pin`; integration patches need separate identities. |
| Full Rust in blueFe | blueFe main `d3c5da6f3d9d097600c9b461064c76cbb345d4ef`; Rust pin `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` | Full source reference and extraction provenance. Compiler/library/src tree equality verified in the Rust review. Do not duplicate the full tree in Lustr. |
| Declared Lustr Rust SDK | `upstream/bluefe-core.pin`, source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` | Exact SDK/tool identities; a full SDK dependency is not selective complementary implementation. |
| Audited Lustr candidate | PR #32 `cb0721d49be95ec3f34a113c2354bc9d2a38bbf3` | Retained native implementation, partial source/data reuse and shared/mixed experiments. Prohibited retries and broad consumer-analysis dependency remain at this snapshot. |
| Fresh Rust research | `rust-lang/rust`, `b0b600668bb9b4a897e2299b0a44289d42276545` | Reviewed current interfaces and CI/history. Research does not update the protected donor pin or authorize mixed semantic versions. |

The [Lust review](upstream-lust-review-2026-11-19.md) and
[Rust review](upstream-rust-review-2026-11-19.md) contain primary links and
bounded review scopes. Source identity is not proof of a source-built artifact,
whole-program correctness or performance.

## Import inventory requirements

The implementation's planned `upstream/complement-imports.json`, schema
`lustr-complement-imports-v1`, must identify real production slices and retained
research components separately. Include existing Unicode/normalization source
and data extraction, their third-party dependencies, licenses, source hashes,
features and actual qualification. Do not replace this inventory with an empty
list while those dependencies exist.

Every new slice needs a missing capability or dependency witness, exact donor
paths/symbols, target/feature closure, substituted adapter, real caller, semantic
owner/query boundary and excluded duplicate Lust phases. Keep
`full_rust_coverage=false` until the actual full complement and integrated result
are qualified. A structural checker can enforce declarations and forbidden
routes, but cannot prove minimality or language equivalence.

The compiler-private driver used by `crates/lustr-semantic` includes substantial
Rust frontend/compiler services even when no whole Rust source tree is checked
into Lustr. Disclose that dependency as research; its current broad closure
cannot be presented as an admitted minimal complement.

This docs-only branch does not add or validate that implementation manifest.
Record its exact source when the corrective implementation is published.
