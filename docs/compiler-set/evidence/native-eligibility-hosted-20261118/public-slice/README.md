# Public upstream slice performance

Status: **completed**.

Pinned public goport-util-small slice, upstream driver, no source edits between samples.

One public pruned/adapted slice; not unchanged full ts-rust, full Rust, fastest Rust setup, resources or release admission.

Times are medians and nearest-rank p95 in milliseconds. Ratio is paired baseline/candidate geometric mean with a conditional bootstrap 95% interval.
Below one favors the reference. These exploratory intervals do not establish equivalence. Twelve-pair p95 is the maximum.

| Scenario | Comparison | Reference median | Candidate median | Reference p95 | Candidate p95 | Ratio [95% interval] |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| cold-check | pristine → integrated | 131.451 | 132.491 | 133.356 | 135.873 | 0.988 [0.975, 1.000] |
| cold-check | pristine → entry | 131.451 | 160.549 | 133.356 | 166.100 | 0.817 [0.806, 0.828] |
| cold-check | cargo → entry | 258.758 | 160.549 | 271.484 | 166.100 | 1.630 [1.595, 1.670] |
| no-op-check | pristine → integrated | 1.151 | 1.122 | 1.359 | 1.326 | 1.010 [0.939, 1.089] |
| no-op-check | pristine → entry | 1.151 | 30.118 | 1.359 | 31.428 | 0.040 [0.038, 0.042] |
| no-op-check | cargo → entry | 8.446 | 30.118 | 9.217 | 31.428 | 0.290 [0.280, 0.301] |
| cold-build | pristine → integrated | 262.545 | 262.857 | 272.252 | 268.788 | 0.998 [0.988, 1.008] |
| cold-build | pristine → entry | 262.545 | 289.436 | 272.252 | 294.607 | 0.910 [0.904, 0.918] |
| cold-build | cargo → entry | 356.352 | 289.436 | 370.111 | 294.607 | 1.237 [1.225, 1.249] |
| no-op-build | pristine → integrated | 3.142 | 3.174 | 3.439 | 3.374 | 1.002 [0.953, 1.057] |
| no-op-build | pristine → entry | 3.142 | 31.843 | 3.439 | 33.423 | 0.098 [0.094, 0.103] |
| no-op-build | cargo → entry | 8.529 | 31.843 | 8.894 | 33.423 | 0.268 [0.262, 0.276] |
| compile-plus-run | pristine → integrated | 3.981 | 3.923 | 4.616 | 4.085 | 1.023 [0.993, 1.058] |
| compile-plus-run | pristine → entry | 3.981 | 30.894 | 4.616 | 32.421 | 0.130 [0.125, 0.136] |
| compile-plus-run | cargo → entry | 9.067 | 30.894 | 9.313 | 32.421 | 0.295 [0.288, 0.302] |
| runtime | pristine → integrated | 0.881 | 0.876 | 0.903 | 0.893 | 1.003 [0.987, 1.018] |
| runtime | pristine → entry | 0.881 | 0.877 | 0.903 | 0.915 | 0.999 [0.978, 1.019] |
| runtime | cargo → entry | 0.720 | 0.877 | 0.739 | 0.915 | 0.821 [0.806, 0.836] |

Application dev opt0/debug0/unwind/overflow+debug assertions; Cargo incremental enabled; native TinyCC dev.

New owned lane directories; native daemon/disk cache disabled; OS caches not flushed.

Retained native daemons versus Cargo Fresh; unchanged source; no body/signature-edit claim.

Command wall time with 100us wait polling; compile-plus-run sums its two child command spans, excluding bookkeeping between them.
