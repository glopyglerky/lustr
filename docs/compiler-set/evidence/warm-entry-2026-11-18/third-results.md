# Corrected public entry: full measurements still miss Lust

The corrected source passes the focused Cargo selector, seed, complete entry and
actual public cached-edit gates. It completes all 384 primary timing samples,
plus the separate residual diagnostics. **The warm performance target remains
unmet:** public check is 48.15% slower than pristine Lust by median, and public
build is 15.48% slower. Both paired intervals remain below the required speed
ratio of one.

The comparison isolates the public-entry changes against the pinned PR #21
launcher while sharing the same native runtime and SDK. It does not qualify all
compiler features or platforms. The original failed campaign and the second
campaign's FIFO-unqualified source remain preserved in [the campaign index](README.md).

## Published and executed source

- [Run 44305235529](https://github.com/glopyglerky/lustr/actions/runs/44305235529),
  [job 134804957788](https://github.com/glopyglerky/lustr/actions/runs/44305235529/job/134804957788).
- Published PR #25 head: `d57f485eeb2b638d1f95a0594294d11e8001fe8e`.
- Executed merge revision: `f638b6cd9819f52b53b142bf688d7ef07e252906`.
- Both source trees: `07235def43af3ffbdc218353d40f3f9e22b8dfd9`.
- Original artifact: `14429133342`, `warm-entry-comparison-1`, 1,723,385 bytes.
- ZIP SHA-256: `de6e9fa011c979b04f4c0b64a7fa97783aa9fc7b19a946b0d0092546d122a64d`.

The original ZIP, exact executed runner closure, constructor source and
certificate evidence, selector source closure, and qualification sources are
included here. `manifest.json` records their hashes. The main measurement step
ends with exit 2 because its completed result misses the strict target; the
subsequent diagnostic step succeeds. Neither the failure nor the target was
suppressed to run diagnostics.

## Primary comparison

These are full command medians in milliseconds, 24 paired observations per
operation and lane. The workload, source bytes, application flags, explicit
manifest/binary selection, offline/locked mode and absence of a package selector
are unchanged. The balanced order and every original observation are retained.

| Operation | PR #21 public launcher | Candidate public Lustr | Pristine Lust | rustc via Cargo |
| --- | ---: | ---: | ---: | ---: |
| Cold check | 196.057884 | 195.155119 | 159.552060 | 298.984321 |
| Warm check | 35.658003 | 2.194758 | 1.481393 | 10.325296 |
| Cold build | 349.940006 | 348.935676 | 314.103698 | 397.814362 |
| Warm build | 38.414503 | 4.595174 | 3.979305 | 10.697621 |

The baseline public launcher is source-built from
`28a2842635ee994db492b43f8c956eddce57a803`. Its source composition guard requires
the entire shared native closure to match before using the candidate-built
native runtime and SDK. The candidate package is completely source-built;
the baseline column is explicitly a launcher comparison using that identical
native package, not an independently rebuilt complete baseline package.
Pristine Lust is separately source-built from
`240e08d91d4aed0299639f6f11fd522912b248f9`, with the same optimized profile and
TinyCC pin. The reference Rust source remains
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.

Ratios below are paired reference/candidate geometric means. Values above one
favor the candidate. Intervals use 10,000 paired bootstrap resamples and are
conditional on this host and campaign.

| Operation | Reference | Candidate | Ratio | Bootstrap 95% interval |
| --- | --- | --- | ---: | --- |
| Cold check | PR #21 public | Candidate public | 0.944264 | [0.848393, 1.050265] |
| Warm check | PR #21 public | Candidate public | 16.396292 | [16.129595, 16.667412] |
| Cold build | PR #21 public | Candidate public | 1.021289 | [0.986454, 1.074483] |
| Warm build | PR #21 public | Candidate public | 7.868930 | [6.860726, 8.483257] |
| Warm check | Pristine Lust | Candidate public | 0.689529 | [0.678245, 0.701218] |
| Warm build | Pristine Lust | Candidate public | 0.812185 | [0.708997, 0.875667] |
| Warm check | rustc via Cargo | Candidate public | 4.739503 | [4.671019, 4.805859] |
| Warm build | rustc via Cargo | Candidate public | 2.256744 | [2.087379, 2.364686] |

No cold before/after benefit is established. All 16 comparison rows remain in
the archived report, including the other cold comparisons. A 21.615877 ms
candidate warm-build observation and a 21.495116 ms Cargo warm-build observation
remain in the original data; the sample set was not trimmed. The wide intervals
are reported as observed.

Relative to pristine Lust, the warm median differences are **0.713364 ms for
check** and **0.615869 ms for build**. The gate requires the lower paired
reference/candidate interval bound to reach one in both warm modes. It remains
false. An interval that overlaps one would not establish parity either.

### First-use and excluded prime costs

Each new warm state receives three complete prime commands before its timed
no-op. All 576 excluded prime durations remain in the raw record. Their medians
are shown separately; the first native prime combines daemon initialization,
initial compilation and selection population rather than isolating those costs.

| Lane | Check prime 1 | Check prime 2 | Check prime 3 | Build prime 1 | Build prime 2 | Build prime 3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| PR #21 public | 185.306421 | 36.048663 | 35.212222 | 378.912549 | 39.410039 | 37.839023 |
| Candidate public | 185.567069 | 2.443016 | 1.977019 | 378.158292 | 4.926883 | 4.130615 |
| Pristine Lust | 150.257369 | 1.798892 | 1.458235 | 341.914009 | 4.309214 | 3.537938 |
| rustc via Cargo | 10.470174 | 10.023855 | 9.897681 | 10.683652 | 10.356759 | 10.224077 |

Cold commands have fresh owned state with native daemon and disk reuse disabled.
OS caches are not flushed. Native warm lanes own distinct daemons and caches;
Cargo warm requests must report fresh artifacts. Measured commands have no
stats, entry trace or selection diagnostics.

## Same-host residual diagnostics

These diagnostics follow the primary campaign. They keep their own native
daemons across 24 paired rounds per operation, after three primes. All six lane
orders are balanced. The integrated lane uses `bin/lust` from the candidate's
same source-built package. All three native lanes explicitly set
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1`, matching the public parser's
native request injection. The primary 384 rows and strict target are unchanged.

| Operation | Public Lustr ms | Same-package integrated Lust ms | Pristine Lust ms |
| --- | ---: | ---: | ---: |
| Warm check | 1.952106 | 1.373917 | 1.301418 |
| Warm build | 4.200103 | 3.613401 | 3.617593 |

| Operation | Reference | Candidate | Paired ratio | Bootstrap 95% interval |
| --- | --- | --- | ---: | --- |
| Check | Integrated Lust | Public Lustr | 0.696389 | [0.678361, 0.716286] |
| Check | Pristine Lust | Integrated Lust | 0.981614 | [0.958703, 1.005040] |
| Check | Pristine Lust | Public Lustr | 0.683585 | [0.666829, 0.700895] |
| Build | Integrated Lust | Public Lustr | 0.871131 | [0.859483, 0.882718] |
| Build | Pristine Lust | Integrated Lust | 0.997367 | [0.981304, 1.013160] |
| Build | Pristine Lust | Public Lustr | 0.868837 | [0.852309, 0.886203] |

The separate production `full::select` component measures **0.554694 ms cached**
and **33.124445 ms with selection caching disabled**. Its disabled/cached paired
ratio is 59.857468, with interval [58.710557, 61.054366]. This is 24 paired helper
invocations per mode, bound to the actual production selector source and build
receipt, with independent before/after hit proofs. Whole-helper process times
are also retained, while the quoted component time excludes parsing and process
startup.

The public commands remain materially slower than the same-package integrated
client in both operations. The same-host component result supports the inference
that selection/entry work accounts for much of the residual. It is **not** an
additive decomposition: helper executable identity and state differ, and its
median must not be subtracted from another group's command median to assign an
exact fraction. The pristine/integrated intervals include one and do not prove
equivalence. These diagnostics do not replace the primary target result.

## Executed correctness and raw replay

The corrected production selector gate passed **140 invocations and 265
assertions**, including the Cargo and receipt special-file regressions. The seed
gate passed **38 invocations and 85 assertions**, and the complete entry contract
passed **16 of 16 tests**. All recorded gate stdout/stderr hashes were checked;
the selector helper's source and binary receipt matches the campaign revision.

The new public cache composition gate retains one owned daemon and holds the
public command/environment constant while changing the actual source. Every
public check/build probe reports a selection-cache hit:

| Source stage | Cargo check/build exits | Public check/build exits | Successful runtime result |
| --- | --- | --- | --- |
| Original | 0 / 0 | 0 / 0 | Exact Rust exit, stdout and stderr |
| Valid body edit | 0 / 0 | 0 / 0 | Exact changed Rust output, including the edit marker |
| Invalid type edit | 101 / 101 | 101 / 101 | No runtime; matching rejection status and E0308 required |
| Restored original | 0 / 0 | 0 / 0 | Exact original Rust output restored |

The invalid type edit first fails the native daemon and then follows the normal
full-Rust rejection route. The eight cache hits therefore do not imply reusing
an accepted semantic result. Successful builds remove the previous output first;
runtime checks are bound to each stage's newly built artifact. Source snapshots,
route traces and rejection streams are preserved. This is correctness evidence,
not an edit-speed benchmark. Compiler rejection streams are retained and checked
for E0308; this gate does not require byte-identical compiler diagnostics.

The independently replayed primary campaign contains:

- 3,403 raw records, 1,252 commands, and 384 timing rows.
- 576 excluded primes and 48 completed sample groups.
- 517 build identities and 212 artifact-bound exact runtime executions.
- 157 owned daemon starts and clean stops.
- 12 initial/final retained proofs, four initial/final public cache-hit proofs,
  and eight additional public cache-hit/edit probes.

The separate diagnostic contains 776 raw records, 347 commands, 144 native
timing rows and 48 selector timing rows. It preserves 78 exact runtime
executions, 18 owned daemon starts/stops, 12 retained proofs, four public
route/cache-hit proofs and two selector-helper hit proofs.

Retained checks still require zero generated C. Retained builds require zero
rechecked bodies, dirty function units and newly compiled objects, with cached
object evidence when launcher/support C is assembled. All counters are retained.
No successful runtime is inferred from a compiler exit alone.

The actual installed `lustr --lustr-prime-selection-tools` constructor operation
also succeeded. `third-constructor-replay.json` binds its original command,
status, output, actual-launcher certificate, nine tool identities and exact
measured receipt. That certificate is distinct from the helper regression's
certificate. The archive lacks complete compiler binaries and original seed
filesystem ownership/modes; the replay verifies archived bindings and source,
not fresh live-tool stamps.

## Limits and next work

This runner reports an AMD EPYC 9V74, four logical CPUs, observed affinity 0–3,
16,760,348,672 visible physical-memory bytes, and Ubuntu 24 image
`20261111.370.1`. CPU frequency and host load are not controlled. The second
campaign ran on an EPYC 7763, so absolute changes between those campaigns cannot
be attributed to the corrected source. Every before/after claim here comes
from the paired lanes within this third run.

The measured source includes the regular-file correction and new cached-edit
qualification. It excludes the later compact-selection and native NoChange
prototypes. The current before/after campaign does not isolate the individual
benefits of constructor seeding and executable-stamp deduplication. Combined
prototype effects, other workloads, broader native/platform qualification and
release admission remain separate. No parity or release claim follows from
these results.

The next performance work should target the remaining foreground selection
cost, with the same command-level target retained. The evidence does not justify
skipping current semantic checks or final-link input validation.

## Portable replay

From the repository root, with Python 3.11 or newer:

```sh
python3 docs/compiler-set/evidence/warm-entry-2026-11-18/replay.py
```

This replays the original failed campaign, both complete primary campaigns,
the third residual diagnostics, constructor bindings and retained gate records
from immutable ZIPs using their own frozen executed runners. No compiler,
daemon, network request or new timing measurement is involved.
