# Three-compiler performance dashboard

Measured source revision: `9480f81232f6c5730cd537a1f6f633fa90e79b85`.

All numerical cells are median wall-clock milliseconds, lower is faster. Measured families retain p95 and paired-ratio statistics in the JSON and original artifacts.
NT means not measured or support not established. N/A is reserved for benchmark-bound executed unsupported-feature evidence and is not currently issued. FAIL denotes a failed benchmark step or rejected evidence, not necessarily a compiler correctness failure.
Standalone rustc no-op invokes a compiler; Cargo no-op can be Fresh. These are separate workloads.

## Fixed Cargo

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| body-edit | 45.122 | NT | 48.896 |
| rejection | 32.665 | NT | 34.610 |
| compile-plus-test | 74.843 | NT | 73.990 |
| no-op | 9.330 | NT | 8.905 |
| check-no-op | 9.344 | NT | 9.065 |
| check-body-edit | 25.870 | NT | 25.059 |
| check | 45.479 | NT | 40.923 |
| cold | 169.343 | NT | 150.709 |
| signature-edit | 59.794 | NT | 52.272 |

Rust reference: rustc via Cargo.

## Native scalar

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| body-edit | 5.439 | 5.274 | 38.694 |
| rejection | 17.784 | 2.381 | 16.666 |
| compile-plus-test | 30.781 | 30.681 | 53.424 |
| no-op | 3.112 | 3.466 | 37.910 |
| check-no-op | 1.782 | 1.775 | 11.262 |
| check-body-edit | 1.807 | 2.064 | 11.241 |
| check | 2.071 | 2.248 | 11.701 |
| cold | 43.928 | 40.634 | 37.948 |
| signature-edit | 4.852 | 4.987 | 38.185 |

Rust reference: standalone rustc.

## Public slice

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| cold-check | 174.070 | 140.833 | 281.075 |
| no-op-check | 32.059 | 1.305 | 8.974 |
| cold-build | 309.228 | 276.681 | 383.300 |
| no-op-build | 34.546 | 3.316 | 9.143 |
| compile-plus-run | 34.889 | 4.297 | 9.846 |
| runtime | 0.891 | 0.889 | 0.735 |

Rust reference: rustc via Cargo.

## Unchanged project

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| clean-dependency-build | 551.621 | NT | 550.775 |
| primed-root-build | 157.415 | NT | 158.303 |
| primed-root-check | 62.259 | NT | 61.382 |
| no-op | 10.093 | NT | 10.119 |
| compile-plus-test | 5191.512 | NT | 5179.934 |

Rust reference: rustc via Cargo.

This is a derived presentation of the existing benchmark families, not an independent new timing campaign or release admission.
