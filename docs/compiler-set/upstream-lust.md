# Preserve Lust and import only the required complement

The [exclusive coverage contract](exclusive-coverage-contract.md) controls source
policy. Lust supplies all covered compilation work. Selected upstream Rust
implementation supplies only missing work and necessary dependencies through
the integrated boundary. The full Rust source stays in blueFe.

## Reference and integration identities

`upstream/lust.pin` identifies pristine donor
`240e08d91d4aed0299639f6f11fd522912b248f9`, tree
`0ab1b09815802894e6078de0af59273cbdf46cb0`. The
[19 November upstream review](upstream-lust-review-2026-11-19.md) refreshed that
identity and its CI/history. Preserve the reference for provenance and matched
performance. Current `next` and the older upstream `main` are not interchangeable.

Default `7cea76c9` keeps `compiler/native` at the pristine reference. Later
candidate branches have explicit integration trees and patches. Keep their
identities separate and carry the matching source-guard implementation and
qualification when importing a repair. Never bypass a guard or hide changes
in an overlay, generated copy or unrecorded build-time patch.

## Necessary changes

A native integration or correctness change requires exact baseline and diff,
a concrete reproduced defect or missing interface, applicable accepted/rejected
and runtime evidence, source/dependency/license identities and performance
characterization for affected paths. Follow the [performance policy](practical-performance.md).
A necessary correctness repair is not required to beat pristine Lust speed.
A faster invalid result is not a gain.

Preserve the frontend, retained worker, checked-state lifetime, precise semantic
reads, body patching and emission reuse. Upstream history records failures when
these contracts were weakened. Integrate through small existing interfaces where
possible, without treating external file placement as evidence of low overhead.

## Rust imports

`upstream/bluefe-core.pin` binds the declared Rust SDK/source. Import only source
slices with a missing-capability or dependency witness under exact features and
targets. Record provenance, hashes, licenses, real callers, semantic owner,
adapter and qualification. Include necessary shared data and generators.
Disclose broad SDK and compiler-private research dependencies separately.

Use the pinned Rust compiler implementation for complementary behavior and
record its effective dependency graph. A compiled program should exercise
the required Lust work through the ordinary toolchain entry. Improve and
measure interfaces that repeat covered frontend work.

The [Rust review](upstream-rust-review-2026-11-19.md) documents reusable upstream
interfaces that support a selective approach. It does not authorize copying all
listed crates or claim that the general complement has been implemented.
Any separate blueFe work must inspect its own current policy and pin; this
Lustr correction changes neither.

## Evidence and history

Source identity, architecture, correctness, speed and package qualification are
separate claims. Keep frozen baselines, failures, inapplicable runs and prior
results unchanged. A repaired candidate does not change historical evidence.
Choose legitimate matched comparisons after establishing correctness.

The [prior policy](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/upstream-lust.md)
and [older default policy](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/upstream-lust.md)
remain in history. Use the current Rust pin, source policy and measured
integration results for new work.
