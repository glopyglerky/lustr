# Paired Lustr performance evidence

Measured checkout: `df8f55aa0430ef81f27e7f403d102256b7fc298b`; tree: `fdb6af7d1ea50c4b63fd8b6a217633979deedf22`.
Original raw JSONL SHA-256: `ffccfa65fe133260117c8960f1180d0abaa6e5eca1742db05d9c3d02eaeb75fd`.
12 pairs per mode; all samples retained. Ratios are baseline/candidate time; >1 is faster.
Medians/p95 are milliseconds; ratio is geometric paired mean, not ratio of medians.
Exploratory shared-host intervals are not equivalence tests or full-workload speed proof.
No compiler/release admission is granted by this replay.

The native rustc lane uses the matching SDK on the identical standalone source, nonincrementally. Every no-op starts rustc again; it is not Cargo Fresh. Check emits metadata. Compile-plus-test includes rustc --test and actual execution of the one test, with both child receipts retained.

## native: pristine → integrated

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 38.652 | 38.315 | 45.901 | 46.163 | 1.013 [0.959, 1.069] | inconclusive |
| no-op | 3.102 | 3.140 | 3.286 | 3.432 | 0.986 [0.964, 1.007] | inconclusive |
| body-edit | 5.269 | 5.038 | 5.485 | 5.320 | 1.042 [1.000, 1.084] | faster-on-this-corpus |
| signature-edit | 4.572 | 4.552 | 4.990 | 4.679 | 1.031 [1.005, 1.056] | faster-on-this-corpus |
| check | 2.081 | 1.875 | 2.273 | 1.988 | 1.110 [1.068, 1.157] | faster-on-this-corpus |
| check-no-op | 1.584 | 1.447 | 1.704 | 1.584 | 1.062 [1.021, 1.098] | faster-on-this-corpus |
| check-body-edit | 1.749 | 1.598 | 1.790 | 1.755 | 1.083 [1.049, 1.121] | faster-on-this-corpus |
| rejection | 1.861 | 1.776 | 2.565 | 2.089 | 1.076 [0.997, 1.174] | inconclusive |
| compile-plus-test | 28.345 | 28.529 | 31.445 | 29.559 | 1.020 [0.971, 1.071] | inconclusive |

## native: pristine → entry

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 38.652 | 41.679 | 45.901 | 46.219 | 0.950 [0.888, 1.022] | inconclusive |
| no-op | 3.102 | 2.852 | 3.286 | 2.988 | 1.093 [1.070, 1.115] | faster-on-this-corpus |
| body-edit | 5.269 | 5.175 | 5.485 | 5.429 | 1.007 [0.976, 1.036] | inconclusive |
| signature-edit | 4.572 | 4.497 | 4.990 | 5.236 | 1.032 [0.987, 1.074] | inconclusive |
| check | 2.081 | 1.799 | 2.273 | 2.339 | 1.100 [1.025, 1.177] | faster-on-this-corpus |
| check-no-op | 1.584 | 1.535 | 1.704 | 1.657 | 1.011 [0.978, 1.046] | inconclusive |
| check-body-edit | 1.749 | 1.604 | 1.790 | 1.644 | 1.088 [1.059, 1.116] | faster-on-this-corpus |
| rejection | 1.861 | 15.913 | 2.565 | 16.753 | 0.122 [0.116, 0.130] | slower-on-this-corpus |
| compile-plus-test | 28.345 | 28.136 | 31.445 | 30.083 | 1.013 [0.981, 1.048] | inconclusive |

## cargo: reference → mixed

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 143.185 | 160.697 | 147.152 | 165.298 | 0.893 [0.884, 0.903] | slower-on-this-corpus |
| no-op | 8.640 | 8.969 | 9.109 | 9.535 | 0.946 [0.917, 0.976] | slower-on-this-corpus |
| body-edit | 45.679 | 41.462 | 46.900 | 50.750 | 1.086 [1.047, 1.110] | faster-on-this-corpus |
| signature-edit | 48.044 | 55.081 | 50.208 | 57.001 | 0.870 [0.863, 0.877] | slower-on-this-corpus |
| check | 37.233 | 41.290 | 38.469 | 43.840 | 0.906 [0.891, 0.921] | slower-on-this-corpus |
| check-no-op | 8.338 | 8.831 | 8.840 | 9.166 | 0.944 [0.923, 0.964] | slower-on-this-corpus |
| check-body-edit | 22.469 | 23.306 | 23.468 | 24.288 | 0.958 [0.947, 0.967] | slower-on-this-corpus |
| rejection | 31.010 | 29.376 | 32.694 | 30.860 | 1.055 [1.049, 1.063] | faster-on-this-corpus |
| compile-plus-test | 69.378 | 69.912 | 70.695 | 71.691 | 0.988 [0.982, 0.995] | slower-on-this-corpus |

## native: rustc → entry

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 35.942 | 41.679 | 36.832 | 46.219 | 0.858 [0.815, 0.902] | slower-on-this-corpus |
| no-op | 34.490 | 2.852 | 35.937 | 2.988 | 12.056 [11.824, 12.259] | faster-on-this-corpus |
| body-edit | 36.882 | 5.175 | 37.181 | 5.429 | 7.072 [6.909, 7.236] | faster-on-this-corpus |
| signature-edit | 35.614 | 4.497 | 36.540 | 5.236 | 7.928 [7.629, 8.180] | faster-on-this-corpus |
| check | 10.672 | 1.799 | 11.011 | 2.339 | 5.686 [5.359, 5.982] | faster-on-this-corpus |
| check-no-op | 10.122 | 1.535 | 10.536 | 1.657 | 6.595 [6.431, 6.752] | faster-on-this-corpus |
| check-body-edit | 9.786 | 1.604 | 10.159 | 1.644 | 6.184 [6.038, 6.371] | faster-on-this-corpus |
| rejection | 14.959 | 15.913 | 15.837 | 16.753 | 0.945 [0.927, 0.963] | slower-on-this-corpus |
| compile-plus-test | 50.719 | 28.136 | 51.681 | 30.083 | 1.795 [1.756, 1.842] | faster-on-this-corpus |

