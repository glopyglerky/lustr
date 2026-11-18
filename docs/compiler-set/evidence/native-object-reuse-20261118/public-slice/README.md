# Public upstream slice performance

Status: **completed**.

Pinned public goport-util-small slice, upstream driver, no source edits between samples.

One public pruned/adapted slice; not unchanged full ts-rust, full Rust, fastest Rust setup, resources or release admission.

Times are medians and nearest-rank p95 in milliseconds. Ratio is paired baseline/candidate geometric mean with a conditional bootstrap 95% interval.
Below one favors the reference. These exploratory intervals do not establish equivalence. Twelve-pair p95 is the maximum.

| Scenario | Comparison | Reference median | Candidate median | Reference p95 | Candidate p95 | Ratio [95% interval] |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| cold-check | pristine → integrated | 135.438 | 136.541 | 137.498 | 141.301 | 0.995 [0.975, 1.013] |
| cold-check | pristine → entry | 135.438 | 163.177 | 137.498 | 168.850 | 0.826 [0.816, 0.838] |
| cold-check | cargo → entry | 263.011 | 163.177 | 280.481 | 168.850 | 1.631 [1.600, 1.668] |
| no-op-check | pristine → integrated | 1.261 | 1.248 | 1.352 | 1.314 | 1.024 [0.971, 1.077] |
| no-op-check | pristine → entry | 1.261 | 30.681 | 1.352 | 33.743 | 0.041 [0.039, 0.043] |
| no-op-check | cargo → entry | 8.622 | 30.681 | 9.774 | 33.743 | 0.286 [0.279, 0.293] |
| cold-build | pristine → integrated | 267.599 | 266.460 | 274.604 | 277.145 | 1.001 [0.991, 1.012] |
| cold-build | pristine → entry | 267.599 | 293.124 | 274.604 | 302.017 | 0.909 [0.899, 0.920] |
| cold-build | cargo → entry | 360.104 | 293.124 | 371.930 | 302.017 | 1.229 [1.212, 1.249] |
| no-op-build | pristine → integrated | 3.246 | 3.216 | 3.402 | 3.445 | 1.011 [0.983, 1.039] |
| no-op-build | pristine → entry | 3.246 | 32.096 | 3.402 | 34.340 | 0.100 [0.097, 0.103] |
| no-op-build | cargo → entry | 8.737 | 32.096 | 9.158 | 34.340 | 0.271 [0.267, 0.275] |
| compile-plus-run | pristine → integrated | 4.079 | 3.998 | 4.320 | 4.192 | 1.028 [1.006, 1.053] |
| compile-plus-run | pristine → entry | 4.079 | 31.849 | 4.320 | 35.034 | 0.127 [0.124, 0.131] |
| compile-plus-run | cargo → entry | 9.273 | 31.849 | 9.956 | 35.034 | 0.291 [0.286, 0.295] |
| runtime | pristine → integrated | 0.879 | 0.879 | 0.901 | 0.915 | 1.000 [0.984, 1.014] |
| runtime | pristine → entry | 0.879 | 0.887 | 0.901 | 1.065 | 0.974 [0.938, 1.005] |
| runtime | cargo → entry | 0.723 | 0.887 | 0.760 | 1.065 | 0.802 [0.776, 0.828] |

Application dev opt0/debug0/unwind/overflow+debug assertions; Cargo incremental enabled; native TinyCC dev.

New owned lane directories; native daemon/disk cache disabled; OS caches not flushed.

Retained native daemons versus Cargo Fresh; unchanged source; no body/signature-edit claim.

Command wall time with 100us wait polling; compile-plus-run sums its two child command spans, excluding bookkeeping between them.
