# Automatic repeat with unchanged product source

The workflow automatically ran again after documentation and a separate CI
constructor repair at `ff9c0de9`. The measured product source, constructor,
selector gates and performance runners are unchanged from the fourth campaign
at `3c378a11`. Both warm modes still miss pristine Lust. This observation remains
separate from every earlier campaign; no samples are pooled or selected by speed.
It does not measure a before/after benefit for `ff9c0de9`, and it contains no
later native NoChange candidate.

## Source and original artifact

- Published revision: `ff9c0de9d2a2c89edd8df48e74e64a5c9483f232`.
- Executed merge: `c3513db6bc32a57ec4ae48bc0e0c504582d3a5b5`.
- Both full source trees: `52ab272cbed5d88b7126d35da5a7f7ff2607cc24`.
- [Automatic hosted run](https://github.com/glopyglerky/lustr/actions/runs/44328101382), job `134884750412`, artifact `14439746642`.
- Original ZIP SHA-256: `bc66c58469bf8de24fdc5544133ba4965fba26463cb84342da7c5d91e1865473`.
- Same-run public baseline: `d57f485eeb2b638d1f95a0594294d11e8001fe8e`.
- Pristine Lust source: `240e08d91d4aed0299639f6f11fd522912b248f9`; imported Rust SDK: `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.
- Host: Intel Xeon 6975P-C, four visible logical CPUs, affinity 0–3, Ubuntu 24 runner image `20261111.127.1`.

`ff9c-repeat-product-identity.json` records equal Git objects for the product
source directories, native source, pinned inputs, measured workflow, constructor
and runner closure. The changed files since the fourth source are documentation
and the separate broad-CI regression-constructor helper/test/workflow changes.

Source equality does not imply identical rebuilt executables. Public/client/SDK
launcher hashes match the fourth run, but the integrated and pristine
`lust-core` binaries and the native shared library hashes differ. Both campaigns’
integrated native receipts record build hash `be64fca1236b2d1f`; the actual binary differences are
preserved without assigning their cause. The same CPU model also does not make
two jobs a controlled before/after experiment.

The same-run baseline is an entry-only build sharing this run's candidate-built
native runtime and SDK. The original equality guard is intact. This is not a
comparison of independently built native candidates.

## Complete primary results

Elapsed command medians are milliseconds. Each cell contains 24 measured
observations from the predeclared balanced four-lane schedule.

| Operation | Pinned public baseline | Candidate public entry | Pristine Lust | Rust/Cargo |
| --- | ---: | ---: | ---: | ---: |
| Cold check | 158.604453 | 158.888040 | 130.636082 | 256.919755 |
| Warm check | 1.477528 | 1.429239 | 1.095341 | 8.807738 |
| Cold build | 290.764084 | 289.292966 | 262.362554 | 357.744295 |
| Warm build | 3.409709 | 3.410951 | 3.139076 | 8.670344 |

The candidate median remains 0.333898 ms, or 30.4835%, above pristine Lust for
warm check, and 0.271875 ms, or 8.6610%, above it for warm build.

Ratios below are paired reference/candidate geometric means. Values above one
favor the candidate. Intervals are conditional paired-bootstrap 95% estimates
with 10,000 resamples; they do not establish another host or workload's result.

| Operation | Reference | Ratio | Bootstrap 95% interval |
| --- | --- | ---: | --- |
| Cold check | Pinned public baseline | 0.996324 | [0.988122, 1.004270] |
| Warm check | Pinned public baseline | 1.046838 | [1.007543, 1.088402] |
| Cold build | Pinned public baseline | 1.003141 | [0.996962, 1.009371] |
| Warm build | Pinned public baseline | 1.013549 | [0.975266, 1.053325] |
| Cold check | Pristine Lust | 0.824926 | [0.818049, 0.832309] |
| Warm check | Pristine Lust | 0.793759 | [0.764299, 0.823680] |
| Cold build | Pristine Lust | 0.907177 | [0.901821, 0.912326] |
| Warm build | Pristine Lust | 0.913791 | [0.868481, 0.958824] |

Within this repeat, the current entry again establishes a warm-check advantage
over the earlier `d57f485e` public baseline. Warm build and both cold comparisons
against that baseline do not establish an advantage. Candidate warm-build median
is slightly higher than that baseline's median. Both cold operations remain
slower than pristine Lust.

The strict gate requires the lower paired speed-ratio bound to reach one in
both warm modes. It remains false. An interval crossing one would not establish
parity either. All 16 primary comparison rows remain in the original report and
`ff9c-repeat-statistics.json`.

All 384 primary observations and 576 separately recorded excluded primes are
retained. No outlier was removed. Candidate warm-check nearest-rank p95 is
1.486675 ms and maximum is 1.584952 ms; warm-build p95 is 3.537215 ms and maximum
is 3.538840 ms. Every lane and operation's minimum, median, p95 and maximum is
reconstructed in the statistics file, including the cold results.

## Separate residual diagnostics

This later diagnostic pass has its own 144 native process timings and 48
selector component timings. Its process schedule differs from the primary
campaign. The selector timer measures inside the helper; it is not the whole
public command. These observations must not be added to, subtracted from, or
pooled with the primary times to assign cost to a component.

| Operation | Public entry | Integrated native entry | Pristine Lust |
| --- | ---: | ---: | ---: |
| Check | 1.220543 ms | 0.982790 ms | 0.895376 ms |
| Build | 2.960650 ms | 2.685698 ms | 2.667776 ms |

Integrated/public paired ratios are 0.792874 [0.762845, 0.823260] for check and
0.904125 [0.891817, 0.916967] for build. Pristine/integrated ratios are 0.923887
[0.888247, 0.958342] for check and 0.982339 [0.961160, 1.003233] for build. The
build interval does not establish equivalence. No particular integration,
client, layout or environment input is assigned the residual difference.

The complete-selector helper medians are 0.264890 ms with validated reuse and
23.825960 ms with reuse disabled. The paired component ratio is 90.220136
[88.638647, 91.748815]. This is a selector component observation, not a
whole-command speedup.

## Qualification records and replay

The primary raw log contains 3,431 records, including 1,268 commands, 212 exact
artifact-bound runtime executions, 157 owned daemon starts and stops, 12
retained-route proofs, eight initial/final public selection hits, and eight
separate public-edit proofs. The diagnostic raw log contains 776 records,
including 347 commands, 78 runtime executions, 18 owned daemons, 12 retained
proofs and four public-hit proofs.

The selector gate passed 154 invocations and 290 assertions; the seed gate
passed 38 and 85; the compact-selection gate passed 103 and 486. Replay
reconstructs 77 complete cached/fresh selector pairs and eight exact fallback
runtime outcomes. All 16 entry-contract tests passed. The original entry log
SHA-256 is `f813bd61f8461fb2214b934ab010f48c5c910e733d0df47aee1ebf1028e5a87e`.

The installed candidate and baseline launchers each executed their own checked
initialization and produced separate nine-identity certificates. Their command,
status, stream, receipt and certificate bindings remain archived. Complete
compiler binaries and live ownership/modes are absent from the ZIP; replay does
not restamp them or claim a new live initialization.

`replay.py` validates this repeat with the byte-identical frozen fourth runners,
constructor sources and qualification closure. It also replays every earlier
campaign, retaining the initial failure and all prior target misses. The new
statistics are checked against all original observations, and rebuilt binary
hash differences are checked against both original artifact reports. No source
was merged, and this docs-only branch does not update the performance PR.
