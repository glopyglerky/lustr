# Paired Lustr performance evidence

Measured checkout: `96fa69536a68e7c2084e746d31e56a453611d949`; tree: `426397dfbefec5d81dd4cfa195450eb947b4b5d1`.
Original raw JSONL SHA-256: `ccc6862401cf50420e8b2fc438f7eeeb2c4cc1b02506ef3ca8f433d24904353f`.
12 pairs per mode; all samples retained. Ratios are baseline/candidate time; >1 is faster.
Medians/p95 are milliseconds; ratio is geometric paired mean, not ratio of medians.
Exploratory shared-host intervals are not equivalence tests or full-workload speed proof.
No compiler/release admission is granted by this replay.

The native rustc lane uses the matching SDK on the identical standalone source, nonincrementally. Every no-op starts rustc again; it is not Cargo Fresh. Check emits metadata. Compile-plus-test includes rustc --test and actual execution of the one test, with both child receipts retained.

## native: pristine → integrated

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 40.637 | 39.674 | 48.831 | 47.738 | 1.042 [0.980, 1.106] | inconclusive |
| no-op | 3.412 | 3.331 | 3.546 | 3.531 | 1.007 [0.983, 1.030] | inconclusive |
| body-edit | 5.259 | 5.111 | 5.625 | 5.400 | 1.036 [1.004, 1.070] | faster-on-this-corpus |
| signature-edit | 4.859 | 4.550 | 5.584 | 5.159 | 1.059 [1.016, 1.104] | faster-on-this-corpus |
| check | 2.175 | 1.984 | 2.381 | 2.276 | 1.079 [1.025, 1.135] | faster-on-this-corpus |
| check-no-op | 1.684 | 1.579 | 1.819 | 1.783 | 1.071 [1.038, 1.105] | faster-on-this-corpus |
| check-body-edit | 1.889 | 1.687 | 2.085 | 1.804 | 1.097 [1.050, 1.146] | faster-on-this-corpus |
| rejection | 2.054 | 1.920 | 2.472 | 2.214 | 1.081 [0.985, 1.188] | inconclusive |
| compile-plus-test | 28.368 | 28.909 | 31.725 | 30.416 | 0.988 [0.954, 1.024] | inconclusive |

## native: pristine → entry

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 40.637 | 41.857 | 48.831 | 46.926 | 0.998 [0.949, 1.053] | inconclusive |
| no-op | 3.412 | 2.997 | 3.546 | 3.208 | 1.130 [1.096, 1.164] | faster-on-this-corpus |
| body-edit | 5.259 | 5.306 | 5.625 | 5.625 | 1.002 [0.989, 1.013] | inconclusive |
| signature-edit | 4.859 | 4.583 | 5.584 | 4.969 | 1.074 [1.040, 1.111] | faster-on-this-corpus |
| check | 2.175 | 1.978 | 2.381 | 2.150 | 1.111 [1.059, 1.165] | faster-on-this-corpus |
| check-no-op | 1.684 | 1.620 | 1.819 | 1.780 | 1.033 [0.996, 1.071] | inconclusive |
| check-body-edit | 1.889 | 1.764 | 2.085 | 1.823 | 1.080 [1.053, 1.109] | faster-on-this-corpus |
| rejection | 2.054 | 17.253 | 2.472 | 17.815 | 0.118 [0.112, 0.125] | slower-on-this-corpus |
| compile-plus-test | 28.368 | 29.553 | 31.725 | 32.457 | 0.970 [0.937, 1.007] | inconclusive |

## cargo: reference → mixed

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 147.509 | 166.936 | 205.467 | 195.528 | 0.902 [0.881, 0.931] | slower-on-this-corpus |
| no-op | 8.647 | 9.115 | 8.946 | 9.750 | 0.949 [0.921, 0.980] | slower-on-this-corpus |
| body-edit | 46.766 | 42.730 | 49.250 | 44.939 | 1.094 [1.085, 1.104] | faster-on-this-corpus |
| signature-edit | 49.041 | 56.765 | 51.154 | 58.637 | 0.867 [0.856, 0.877] | slower-on-this-corpus |
| check | 38.475 | 42.213 | 39.878 | 44.589 | 0.907 [0.891, 0.922] | slower-on-this-corpus |
| check-no-op | 8.591 | 8.903 | 8.751 | 9.444 | 0.946 [0.920, 0.972] | slower-on-this-corpus |
| check-body-edit | 23.018 | 24.058 | 24.532 | 25.564 | 0.963 [0.957, 0.969] | slower-on-this-corpus |
| rejection | 32.099 | 30.300 | 33.213 | 31.220 | 1.062 [1.051, 1.073] | faster-on-this-corpus |
| compile-plus-test | 70.204 | 71.408 | 73.779 | 76.336 | 0.983 [0.977, 0.989] | slower-on-this-corpus |

## native: rustc → entry

| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |
|---|---:|---:|---:|---:|---:|---|
| cold | 36.590 | 41.857 | 38.736 | 46.926 | 0.879 [0.845, 0.911] | slower-on-this-corpus |
| no-op | 36.695 | 2.997 | 37.803 | 3.208 | 12.194 [11.939, 12.465] | faster-on-this-corpus |
| body-edit | 37.145 | 5.306 | 38.437 | 5.625 | 7.066 [6.918, 7.214] | faster-on-this-corpus |
| signature-edit | 36.866 | 4.583 | 37.886 | 4.969 | 8.101 [7.919, 8.298] | faster-on-this-corpus |
| check | 11.222 | 1.978 | 11.456 | 2.150 | 5.708 [5.498, 5.934] | faster-on-this-corpus |
| check-no-op | 10.826 | 1.620 | 11.047 | 1.780 | 6.599 [6.410, 6.790] | faster-on-this-corpus |
| check-body-edit | 10.624 | 1.764 | 11.104 | 1.823 | 6.166 [6.056, 6.279] | faster-on-this-corpus |
| rejection | 15.942 | 17.253 | 16.673 | 17.815 | 0.930 [0.919, 0.942] | slower-on-this-corpus |
| compile-plus-test | 51.902 | 29.553 | 53.389 | 32.457 | 1.756 [1.711, 1.803] | faster-on-this-corpus |

