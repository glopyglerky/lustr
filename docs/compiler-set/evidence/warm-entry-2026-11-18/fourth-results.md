# Fourth completed warm-entry campaign

The compact selection record and effective-access correction are source-built and exercised successfully. Warm check improves against the corrected `d57f485e` public baseline in this campaign. Warm build does not establish an improvement. Both modes still miss pristine Lust. This is a completed target miss, not an incomplete benchmark or failed correctness gate.

## Exact source and original artifact

- Published PR25 revision: `3c378a11d60149a3ec99fa49ea1b336e9406fca8`.
- Executed GitHub merge revision: `d3b14dc637b6343bbb329dfcf6d5e1b420c06009`.
- Both source trees: `0256606d5793bfb890130f05fae013d0c5c5f9e1`.
- Baseline public source: `d57f485eeb2b638d1f95a0594294d11e8001fe8e`, tree `07235def43af3ffbdc218353d40f3f9e22b8dfd9`.
- [Hosted run](https://github.com/glopyglerky/lustr/actions/runs/44313200385), job `134832391635`, artifact `14432019846`.
- Original artifact ZIP SHA-256: `1fa5cea1ac4a4165594583288393f6141fd74384608306cf6eb47627cd773c12`.
- Host: Intel Xeon 6975P-C, four visible logical CPUs, affinity 0–3, Ubuntu 24 runner image `20261111.127.1`.

The third campaign ran on AMD EPYC 9V74. Its absolute elapsed times cannot be compared with this host to infer a before/after source benefit. The pinned `d57f485e` lane in this fourth campaign supplies that controlled comparison. This is an entry-only baseline build against the byte-identical candidate-built native runtime and SDK, not a second complete baseline package. The shared native/source equality guard remains intact.

The candidate package and the baseline launcher both ran their own actual installed-executable tool initialization. Separate certificates bind each launcher and its nine live tool identities to the same receipt. The baseline initialization preserved the candidate certificate. Constructor commands, statuses, streams, certificates and frozen sources are archived. The archive contains no complete compiler binaries and does not preserve live ownership/modes: replay validates the original bindings and recorded source operation; it does not restamp absent executables or repeat initialization.

## Complete command results

All four lanes use the same public-slice workload and requested behavior. The fixed schedule has 24 paired blocks for each of four operations, totaling 384 timings. No observations are removed. Measured commands have no trace/stats instrumentation. Three per-state warm primes remain excluded and visible in the original report.

| Operation | Baseline d57 Lustr | Candidate Lustr | Pristine Lust | rustc via Cargo |
| --- | ---: | ---: | ---: | ---: |
| cold-check | 160.704519 ms | 161.954831 ms | 135.152673 ms | 264.046700 ms |
| no-op-check | 1.492171 ms | 1.450204 ms | 1.143490 ms | 8.896727 ms |
| cold-build | 293.501670 ms | 293.554183 ms | 264.280511 ms | 355.235581 ms |
| no-op-build | 3.403832 ms | 3.471401 ms | 3.121890 ms | 8.770996 ms |

Ratios are paired reference/candidate geometric means. Above one favors the candidate. The fixed paired bootstrap has 10,000 resamples; intervals are conditional exploratory estimates on this runner.

| Operation | Reference | Ratio | Conditional bootstrap 95% |
| --- | --- | ---: | --- |
| cold-check | baseline | 0.991725 | [0.975515, 1.006096] |
| cold-check | pristine | 0.828486 | [0.818744, 0.836995] |
| cold-check | cargo | 1.616395 | [1.600453, 1.632611] |
| no-op-check | baseline | 1.042678 | [1.002444, 1.085341] |
| no-op-check | pristine | 0.816775 | [0.788929, 0.846303] |
| no-op-check | cargo | 6.061840 | [5.936971, 6.182755] |
| cold-build | baseline | 1.005795 | [0.994004, 1.023099] |
| cold-build | pristine | 0.904692 | [0.895289, 0.918881] |
| cold-build | cargo | 1.213908 | [1.193351, 1.231877] |
| no-op-build | baseline | 1.013003 | [0.974849, 1.052925] |
| no-op-build | pristine | 0.919186 | [0.879199, 0.962241] |
| no-op-build | cargo | 2.592275 | [2.525297, 2.665732] |

The continuation establishes a warm-check improvement of 1.042678× against `d57f485e`, interval [1.002444, 1.085341]. The warm-build ratio is 1.013003×, interval [0.974849, 1.052925], so a gain is not established; its candidate median is 3.471401 ms versus baseline 3.403832 ms. Both cold before/after intervals include one. Do not present the combined compact/access continuation as an independently isolated effect of either source change.

Compared with pristine Lust, candidate medians remain 26.8226% slower for warm check (+0.306714 ms) and 11.1955% slower for warm build (+0.349511 ms). Both strict criteria fail: the lower paired Lust/candidate 95% bounds are 0.788929 and 0.879199, below 1.0. The benchmark exits 2 after producing the complete report.

The earlier ~16×/~8× improvement was established against PR21 in the separate third campaign. This campaign starts from that already improved public entry. Ratios from the different hosts/campaigns are not multiplied to manufacture a combined estimate.

### Sample ranges retained

| Operation | Lane | Minimum ms | Maximum ms |
| --- | --- | ---: | ---: |
| cold-check | baseline | 153.402245 | 169.547315 |
| cold-check | entry | 150.765541 | 187.389185 |
| cold-check | pristine | 127.709385 | 141.596537 |
| cold-check | cargo | 244.825071 | 286.896720 |
| no-op-check | baseline | 1.237147 | 1.867310 |
| no-op-check | entry | 1.255955 | 1.622504 |
| no-op-check | pristine | 1.082376 | 1.365087 |
| no-op-check | cargo | 8.102953 | 9.182857 |
| cold-build | baseline | 284.162589 | 342.000539 |
| cold-build | entry | 286.056698 | 340.808147 |
| cold-build | pristine | 259.199879 | 304.593398 |
| cold-build | cargo | 343.557194 | 380.467731 |
| no-op-build | baseline | 3.035282 | 3.940588 |
| no-op-build | entry | 2.886637 | 3.565089 |
| no-op-build | pristine | 2.705902 | 3.812471 |
| no-op-build | cargo | 8.264766 | 9.170623 |

## Qualification and independent replay

- Main campaign: 3,431 raw records, 1,268 commands, 384 timings, 212 exact bound runtime executions, 157 owned-daemon start/stop pairs, and 12 retained check/build proofs.
- Both public lanes provide initial/final selection hits: eight retained selection proofs. The separate actual public edit/reject/restore gate supplies eight further selection-hit proofs across four source stages.
- Selector gate: 154 commands and 290 assertions; all 77 cached/fresh pairs preserve complete Selection or exact rejection and stderr.
- Seed gate: 38 commands and 85 assertions.
- Compact/fallback gate: 103 commands and 486 assertions, including 15 malformed/old-record controls with recomputed checksums and eight exact runtime observations of actual Cargo/Rust fallback artifacts.
- Entry contract: 16 passed, zero failed.

The public edit gate keeps the same daemon while checking/building original source, a valid body edit, an invalid type edit and restoration. Invalid compilation must return 101 and contain E0308; byte-identical compiler diagnostics are not claimed. Successful builds must match the Rust reference in exact status/stdout/stderr and be bound to the current edited artifact. Signals and timeouts fail. The compact fallback gate also verifies profile/output/package/target transitions and restored valid output.

`replay.py` independently reruns the frozen primary and diagnostic verifiers, regenerates all statistics/schedules, checks every archived command stream, validates raw/source/constructor bindings, reconstructs all 77 selector pairs and the eight compact fallback runtime outcomes, and checks the complete entry log. It preserves all earlier campaigns and the original first-run gate failure. Archived assertions and frozen test source describe filesystem mutation actions; replay does not rerun the original compiler or recreate the original filesystem credentials.

## Separate residual diagnostic

This diagnostic follows the primary campaign with separate state and explicit matching native environment. It uses held daemons, 24 paired observations per operation, all six lane orders balanced, and no trace/stats in measured commands. Before/after qualification is separate. It does not replace or amend the primary result.

| Operation | Public Lustr | Same-package integrated Lust | Pristine Lust |
| --- | ---: | ---: | ---: |
| check | 1.229278 ms | 1.040106 ms | 0.896760 ms |
| build | 2.893789 ms | 2.693056 ms | 2.588664 ms |

| Operation | Reference / candidate | Ratio | Conditional bootstrap 95% |
| --- | --- | ---: | --- |
| check | integrated / entry | 0.790226 | [0.761267, 0.819692] |
| check | pristine / integrated | 0.930085 | [0.896422, 0.966283] |
| check | pristine / entry | 0.734978 | [0.711370, 0.759536] |
| build | integrated / entry | 0.918364 | [0.909801, 0.926857] |
| build | pristine / integrated | 0.959714 | [0.938275, 0.979485] |
| build | pristine / entry | 0.881367 | [0.863735, 0.897049] |

The public entry is slower than the same-package integrated Lust command. The integrated command is also slower than pristine Lust in this run. That difference does not identify which client startup, native integration, package-layout or runtime input causes it. The third run’s integrated/pristine intervals included one; that older observation must not be restated as the current result or as proof of equality.

The source-built selector helper separately measures actual `full::select`: cached median 0.2648965 ms; disabled-cache median 23.695345 ms. Its executable identity and timing boundary differ from the public command, and state is separate. These values are not an additive decomposition and cannot be subtracted from whole-command elapsed times to claim an attributable fraction.

Diagnostic replay verifies 776 raw records, 347 commands, 144 native timings, 48 selector timings, 78 exact runtime observations, 18 owned-daemon start/stop pairs, 12 retained proofs and four public selection hits.

## Remaining scope

This completes focused Linux qualification of the published compact/access continuation. It does not establish Lust parity, broader native release admission, macOS runtime qualification, or individual/combined benefits of other PR17/18 candidates. No native NoChange, canonical-parent, fd-path or JSON-move prototype is included. The draft remains unmerged.
