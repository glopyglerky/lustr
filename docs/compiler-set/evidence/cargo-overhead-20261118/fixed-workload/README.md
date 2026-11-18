# Paired Lustr performance evidence

Measured checkout: `480114c2dc973770fbfbbc3c2b32915b516c6716`; tree: `e1c028f15e29a72fdbe338883613c23a6c8dccec`.
Original raw JSONL SHA-256: `859c36f26f39f58d688c48e97ef43d5f4bd46703bf0f5709c0d8394e38a6eca4`.
12 pairs per mode; all samples retained. Ratios are baseline/candidate time; >1 is faster.
Medians/p95 are milliseconds; ratio is geometric paired mean, not ratio of medians.
Exploratory shared-host intervals are not equivalence tests or full-workload speed proof.
No compiler/release admission is granted by this replay.

The native rustc lane uses the matching SDK on the identical standalone source, nonincrementally. Every no-op starts rustc again; it is not Cargo Fresh. Check emits metadata. Compile-plus-test includes rustc --test and actual execution of the one test, with both child receipts retained.

## native: pristine → integrated

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 59.498 | 60.687 | 66.704 | 66.520 | 0.998 [0.956, 1.044] | inconclusive |
| no-op | 4.648 | 4.842 | 5.090 | 6.747 | 0.944 [0.884, 0.999] | slower-on-this-corpus |
| body-edit | 7.100 | 7.227 | 8.033 | 8.129 | 0.998 [0.968, 1.029] | inconclusive |
| signature-edit | 6.267 | 6.300 | 6.553 | 7.422 | 0.983 [0.946, 1.018] | inconclusive |
| check | 2.872 | 2.661 | 3.058 | 3.471 | 1.026 [0.970, 1.081] | inconclusive |
| check-no-op | 2.417 | 2.408 | 2.965 | 2.751 | 1.039 [0.995, 1.096] | inconclusive |
| check-body-edit | 2.712 | 2.670 | 3.022 | 2.975 | 1.009 [0.973, 1.049] | inconclusive |
| rejection | 3.080 | 3.082 | 4.274 | 3.890 | 1.013 [0.890, 1.153] | inconclusive |
| compile-plus-test | 32.631 | 31.566 | 35.651 | 36.680 | 1.028 [0.986, 1.072] | inconclusive |

## native: pristine → entry

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 59.498 | 58.391 | 66.704 | 67.853 | 1.008 [0.951, 1.067] | inconclusive |
| no-op | 4.648 | 4.231 | 5.090 | 4.508 | 1.106 [1.073, 1.144] | faster-on-this-corpus |
| body-edit | 7.100 | 7.231 | 8.033 | 7.623 | 0.996 [0.973, 1.020] | inconclusive |
| signature-edit | 6.267 | 6.323 | 6.553 | 6.634 | 0.985 [0.968, 1.000] | inconclusive |
| check | 2.872 | 2.808 | 3.058 | 3.059 | 1.020 [0.979, 1.067] | inconclusive |
| check-no-op | 2.417 | 2.593 | 2.965 | 2.936 | 0.971 [0.932, 1.015] | inconclusive |
| check-body-edit | 2.712 | 2.721 | 3.022 | 2.833 | 1.005 [0.961, 1.048] | inconclusive |
| rejection | 3.080 | 25.232 | 4.274 | 26.152 | 0.129 [0.121, 0.138] | slower-on-this-corpus |
| compile-plus-test | 32.631 | 33.119 | 35.651 | 37.246 | 0.975 [0.938, 1.011] | inconclusive |

## cargo: reference → mixed

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 208.095 | 235.682 | 214.215 | 240.616 | 0.885 [0.871, 0.897] | slower-on-this-corpus |
| no-op | 12.485 | 13.338 | 14.015 | 14.441 | 0.946 [0.935, 0.956] | slower-on-this-corpus |
| body-edit | 68.743 | 63.893 | 72.740 | 66.219 | 1.081 [1.069, 1.097] | faster-on-this-corpus |
| signature-edit | 72.827 | 84.071 | 74.232 | 97.136 | 0.850 [0.827, 0.866] | slower-on-this-corpus |
| check | 56.212 | 62.265 | 57.385 | 64.638 | 0.903 [0.890, 0.916] | slower-on-this-corpus |
| check-no-op | 12.781 | 13.615 | 13.608 | 16.665 | 0.922 [0.885, 0.957] | slower-on-this-corpus |
| check-body-edit | 35.630 | 37.148 | 36.146 | 37.967 | 0.954 [0.941, 0.966] | slower-on-this-corpus |
| rejection | 47.932 | 44.873 | 49.251 | 46.117 | 1.068 [1.052, 1.085] | faster-on-this-corpus |
| compile-plus-test | 101.876 | 101.511 | 105.853 | 104.144 | 1.001 [0.985, 1.016] | inconclusive |

## native: rustc → entry

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 49.622 | 58.391 | 51.339 | 67.853 | 0.821 [0.784, 0.856] | slower-on-this-corpus |
| no-op | 50.029 | 4.231 | 52.348 | 4.508 | 11.757 [11.481, 12.028] | faster-on-this-corpus |
| body-edit | 50.220 | 7.231 | 54.732 | 7.623 | 7.007 [6.874, 7.155] | faster-on-this-corpus |
| signature-edit | 49.891 | 6.323 | 51.700 | 6.634 | 7.877 [7.714, 8.039] | faster-on-this-corpus |
| check | 14.945 | 2.808 | 15.906 | 3.059 | 5.378 [5.158, 5.623] | faster-on-this-corpus |
| check-no-op | 15.002 | 2.593 | 15.881 | 2.936 | 5.826 [5.647, 6.011] | faster-on-this-corpus |
| check-body-edit | 15.135 | 2.721 | 15.688 | 2.833 | 5.667 [5.516, 5.820] | faster-on-this-corpus |
| rejection | 22.856 | 25.232 | 24.672 | 26.152 | 0.913 [0.884, 0.946] | slower-on-this-corpus |
| compile-plus-test | 69.911 | 33.119 | 72.488 | 37.246 | 2.089 [2.034, 2.141] | faster-on-this-corpus |

