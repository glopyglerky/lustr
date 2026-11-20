# Compiler source register

Reviewed 19 November 2026. This register distinguishes reference source,
partial imports, research dependencies and required product behavior. Follow
the [exclusive coverage contract](exclusive-coverage-contract.md).

While [PR #33](https://github.com/glopyglerky/lustr/pull/33) remains unmerged,
resolve its live head for implementation work, recorded for this audit at
`ea26fe16636d390763257a19c19df3db992cdbcb`. Governing documentation on `next`
does not transfer that compiler source into the default branch. After merge,
verify that the current integration source contains the correction and use its
live source/evidence rather than resuming the closed PR. The older
candidate identities below remain research inputs, not alternative current heads.
The [documentation disposition](documentation-disposition.json) separates active
guidance from source-bound records and untouched donor/evidence files.

The branch observations in the table are dated research. A 20 November 2026 UTC refresh
found blueFe at
[`9267fd0d`](https://github.com/glopyglerky/blueFe/tree/9267fd0d348dc591af85efe96597c6ea774be5c0),
declaring Rust
[`4b5350e8`](https://github.com/rust-lang/rust/tree/4b5350e87e487aed08ab2f9f0160a4b2616438bc).
That moving head is no longer the declared Lustr Rust source. Lustr remains
pinned to `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`; extract and compare against
that identity. This audit does not upgrade the pin or transfer earlier tree
equality, artifact qualification or semantics to a later blueFe/Rust head.

| Source | Exact observed identity | Role and limit |
| --- | --- | --- |
| Pristine Lust | `t3dotgg/lust-lang`, `240e08d91d4aed0299639f6f11fd522912b248f9`, tree `0ab1b09815802894e6078de0af59273cbdf46cb0` | Reference compiler and speed mechanisms preserved by `upstream/lust.pin`; integration patches need separate identities. |
| Full Rust in blueFe | blueFe main `d3c5da6f3d9d097600c9b461064c76cbb345d4ef`; Rust pin `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` | Full source reference and extraction provenance. Compiler/library/src tree equality verified in the Rust review. Do not duplicate the full tree in Lustr. |
| Declared Lustr Rust SDK | `upstream/bluefe-core.pin`, source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` | Exact SDK/tool identities; a full SDK dependency is not selective complementary implementation. |
| Audited Lustr candidate | PR #32 `cb0721d49be95ec3f34a113c2354bc9d2a38bbf3` | Retained native implementation, partial source/data reuse and shared/mixed experiments. Prohibited retries and broad consumer-analysis dependency remain at this snapshot. |
| Fresh Rust research | `rust-lang/rust`, `b0b600668bb9b4a897e2299b0a44289d42276545` | Reviewed current interfaces and CI/history. Research does not update the protected donor pin or authorize mixed semantic versions. |
| Later same-day research | Rust `c884ecfbabb04b04fa5054dafc67ec258ecde745`; Lust `240e08d91d4aed0299639f6f11fd522912b248f9` | Refreshed CI history and capability call graph in the [support/cost review](exclusive-coverage-mathematics.md). Both protected Lustr pins remain unchanged. |

The [sole-goal audit](sole-goal-audit.md) records the subsequent source cleanup
against PR #33 input `d1328bc2`. Its verification repeats the blueFe/Rust
`compiler`, `library` and `src` tree comparison and binds the actual compiled
product inputs. Complete source equality does not qualify Lustr's complement.

The [Lust review](upstream-lust-review-2026-11-19.md) and
[Rust review](upstream-rust-review-2026-11-19.md) contain primary links and
bounded review scopes. Source identity is not proof of a source-built artifact,
whole-program correctness or performance.

## Import inventory requirements

The implementation's [`upstream/complement-imports.json`](../../upstream/complement-imports.json),
schema `lustr-complement-imports-v1`, identifies real partial production imports
and retained research components separately. Include Unicode/normalization source
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

The [cleanup record](exclusive-coverage-cleanup.md) describes the implementation
that adds the inventory and structural guard, its validation and its limits.
Neither establishes full complementary coverage or a globally minimal import set.
