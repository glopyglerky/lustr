# Public upstream slice performance

Status: **completed**.

Pinned public goport-util-small slice, upstream driver, no source edits between samples.

One public pruned/adapted slice; not unchanged full ts-rust, full Rust, fastest Rust setup, resources or release admission.

Times are medians and nearest-rank p95 in milliseconds. Ratio is paired baseline/candidate geometric mean with a conditional bootstrap 95% interval.
Below one favors the reference. These exploratory intervals do not establish equivalence. Twelve-pair p95 is the maximum.

| Scenario | Comparison | Reference median | Candidate median | Reference p95 | Candidate p95 | Ratio [95% interval] |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| cold-check | pristine → integrated | 201.455 | 203.988 | 218.107 | 209.770 | 0.986 [0.968, 1.006] |
| cold-check | pristine → entry | 201.455 | 240.365 | 218.107 | 269.262 | 0.820 [0.805, 0.833] |
| cold-check | cargo → entry | 376.037 | 240.365 | 427.106 | 269.262 | 1.532 [1.508, 1.555] |
| no-op-check | pristine → integrated | 1.850 | 1.836 | 1.940 | 2.090 | 1.001 [0.963, 1.041] |
| no-op-check | pristine → entry | 1.850 | 42.584 | 1.940 | 45.204 | 0.043 [0.042, 0.044] |
| no-op-check | cargo → entry | 12.439 | 42.584 | 13.370 | 45.204 | 0.293 [0.288, 0.298] |
| cold-build | pristine → integrated | 408.807 | 414.919 | 422.626 | 437.969 | 0.990 [0.977, 1.002] |
| cold-build | pristine → entry | 408.807 | 456.673 | 422.626 | 466.235 | 0.901 [0.892, 0.911] |
| cold-build | cargo → entry | 498.500 | 456.673 | 513.001 | 466.235 | 1.092 [1.082, 1.103] |
| no-op-build | pristine → integrated | 4.696 | 4.601 | 6.206 | 5.050 | 1.061 [1.009, 1.125] |
| no-op-build | pristine → entry | 4.696 | 46.151 | 6.206 | 48.898 | 0.105 [0.099, 0.113] |
| no-op-build | cargo → entry | 12.765 | 46.151 | 13.603 | 48.898 | 0.276 [0.272, 0.280] |
| compile-plus-run | pristine → integrated | 6.058 | 5.973 | 6.715 | 6.731 | 1.010 [0.975, 1.039] |
| compile-plus-run | pristine → entry | 6.058 | 47.708 | 6.715 | 51.461 | 0.127 [0.124, 0.130] |
| compile-plus-run | cargo → entry | 14.035 | 47.708 | 14.864 | 51.461 | 0.294 [0.289, 0.300] |
| runtime | pristine → integrated | 1.417 | 1.420 | 1.590 | 1.521 | 1.013 [0.978, 1.050] |
| runtime | pristine → entry | 1.417 | 1.420 | 1.590 | 1.623 | 1.014 [0.974, 1.049] |
| runtime | cargo → entry | 1.066 | 1.420 | 1.128 | 1.623 | 0.747 [0.721, 0.769] |

Application dev opt0/debug0/unwind/overflow+debug assertions; Cargo incremental enabled; native TinyCC dev.

New owned lane directories; native daemon/disk cache disabled; OS caches not flushed.

Retained native daemons versus Cargo Fresh; unchanged source; no body/signature-edit claim.

Command wall time with 100us wait polling; compile-plus-run sums its two child command spans, excluding bookkeeping between them.
