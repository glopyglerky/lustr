# Three-compiler performance dashboard

Measured source revision: `8a7af2e1e2c21743d14dad45c83283232e2bc102`.

All numerical cells are median wall-clock milliseconds, lower is faster. Measured families retain p95 and paired-ratio statistics in the JSON and original artifacts.
NT means not measured or support not established. N/A is reserved for benchmark-bound executed unsupported-feature evidence and is not currently issued. FAIL denotes a failed benchmark step or rejected evidence, not necessarily a compiler correctness failure.
Standalone rustc no-op invokes a compiler; Cargo no-op can be Fresh. These are separate workloads.

## Fixed Cargo

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| body-edit | 35.597 | NT | 38.777 |
| rejection | 25.470 | NT | 27.110 |
| compile-plus-test | 61.646 | NT | 61.025 |
| no-op | 8.551 | NT | 8.103 |
| check-no-op | 8.163 | NT | 7.811 |
| check-body-edit | 19.829 | NT | 19.284 |
| check | 34.728 | NT | 31.495 |
| cold | 141.784 | NT | 125.449 |
| signature-edit | 47.163 | NT | 40.959 |

Rust reference: rustc via Cargo.

## Native scalar

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| body-edit | 4.809 | 4.850 | 33.818 |
| rejection | 14.457 | 1.615 | 13.477 |
| compile-plus-test | 27.426 | 26.630 | 47.979 |
| no-op | 2.824 | 2.949 | 30.864 |
| check-no-op | 1.423 | 1.439 | 8.636 |
| check-body-edit | 1.520 | 1.433 | 8.530 |
| check | 1.604 | 1.630 | 8.750 |
| cold | 38.870 | 37.409 | 33.654 |
| signature-edit | 4.027 | 4.157 | 31.632 |

Rust reference: standalone rustc.

## Public slice

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| cold-check | 153.014 | 126.273 | 248.260 |
| no-op-check | 1.407 | 1.098 | 8.443 |
| cold-build | 282.161 | 255.076 | 337.885 |
| no-op-build | 3.333 | 3.127 | 8.623 |
| compile-plus-run | 4.047 | 3.767 | 9.053 |
| runtime | 0.876 | 0.872 | 0.713 |

Rust reference: rustc via Cargo.

## Unchanged project

Campaign: **MEASURED**. replayed qualified evidence

| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |
| --- | ---: | ---: | ---: |
| clean-dependency-build | 496.701 | NT | 496.987 |
| primed-root-build | 141.005 | NT | 139.917 |
| primed-root-check | 51.652 | NT | 51.566 |
| no-op | 8.954 | NT | 8.862 |
| compile-plus-test | 4608.873 | NT | 4594.786 |

Rust reference: rustc via Cargo.

This is a derived presentation of the existing benchmark families, not an independent new timing campaign or release admission.
