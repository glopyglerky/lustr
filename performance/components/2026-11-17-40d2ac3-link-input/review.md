# Linker-input component experiment review

**Result: evidence checks passed; component-only measurement.** This experiment compares the historical copy-and-file-sync policy from `048d824` with the production disposable-copy helper. It does not compile an application, measure a complete link, or establish compiler performance. The pre-merge branch already omitted these disposable flushes, so this is not evidence of an additional gain caused by the merge.

## Provenance and integrity

[Run 44096292401](https://github.com/glopyglerky/lustr/actions/runs/44096292401), [successful job 134084163148](https://github.com/glopyglerky/lustr/actions/runs/44096292401/job/134084163148), [artifact 14332549284](https://github.com/glopyglerky/lustr/actions/runs/44096292401/artifacts/14332549284).

Head: `40d2ac3f0b71abaa1e73cd366d8c43f8eb4d4581`. Actual GitHub merge checkout: `3b4b0bd67401085129d26be1afbd20e96c9b9d02`. Both have tree `78318c4b01f8eb6f9518ea331efe69f0c457b1ce`; GitHub's commit metadata confirms the head is a parent of that checkout.

The unchanged [original ZIP](original.zip) is 2,850 bytes, SHA256 `c485d971009cfe72ff1f193c1fc3499710b85ad313b3794f91012f24b05d346e`, matching GitHub's artifact metadata. All three member hashes and the three recorded source-file hashes were independently verified. The recorded optimized probe SHA256 is `dd049177234d4c552ffafd4b282690aece3cc5a6c8cb106abf630b270e0d2511`; that executable is not in the artifact, so its recorded hash was not independently recomputed from executable bytes.

The eight named tests in the original log exactly match the production helper's eight tests. All passed with zero failures or ignored tests, including independent inodes, linker mutation/cleanup isolation, failed-copy handling and abandoned-link reconstruction.

## Replay and observations

All 24 positive-duration pairs, the scope record and completion record are present. Replaying the source's 64-bit shuffle from seed `7815279672343619951` reproduced every recorded order: 12 predecessor-first and 12 candidate-first pairs. No sample was removed, including the 14.839 ms predecessor outlier.

Each lane stages **16 copies of the same generated 64 KiB source** (1 MiB total). The predecessor synchronizes each copied file; the candidate calls `mixed_link_input::stage`. Output-byte verification and cleanup occur after timing. Page caches were not flushed.

| Statistic | Observed value |
|---|---:|
| Copy-and-sync median | 7.866133 ms |
| Disposable-copy median | 0.729382 ms |
| Median paired saving | 7.137087 ms |
| Geometric mean of paired predecessor/candidate ratios | 10.897278 |
| Median paired predecessor/candidate ratio | 10.385139 |
| Paired ratio range | 8.855731–20.684072 |
| Pairs favoring disposable copy | 24/24 |

The geometric ratio is `exp(mean(log(predecessor_ns / candidate_ns)))`; the median paired saving is the median of each pair's duration difference. These are descriptive statistics for this component experiment, not a confidence statement or compiler-speed admission. The full derived values and every paired ratio are in [review.json](review.json); original nanosecond samples remain inside the ZIP.

Host: Linux `7.0.0-1017-azure`, x86-64, reported AMD EPYC 7763 model with four visible logical CPUs, Microsoft hypervisor. Compiler: Rust 1.99.1, commit `58cf8f9c35b355199727627d07ae34a4e2846d67`, LLVM 23.1.1; the workflow builds the probe with optimization level 3.

**No measurement defect was found in this scope.** These fixed-size repeated copies do not represent a real object-size distribution, an end-to-end build, cold page caches, or durable cache publication. Retained-generation synchronization remains required and unchanged. The source-built compiler campaign must independently qualify correctness, recovery and complete-work performance.
