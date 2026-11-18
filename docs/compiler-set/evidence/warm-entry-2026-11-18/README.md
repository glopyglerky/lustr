# Warm-entry campaigns: target still missed

The third campaign measured the corrected source through the complete focused
gates and all 384 primary timings. Public warm check is 2.195 ms versus pristine
Lust's 1.481 ms; warm build is 4.595 ms versus 3.979 ms. The strict performance
target still fails. The same-host residual diagnostics locate substantial
remaining cost in the public entry/selection path. See
[the third campaign's results and limits](third-results.md).

This directory preserves all three original artifact ZIPs, their published source
identities, frozen copies of the actually executed runner closures, and a
portable replay. The first failed campaign and the second complete campaign's
special-file correctness limitation remain intact. The second source's normal
timings do not qualify its later correction; the third run executes that
correction. No source was merged by these campaigns.

## Exact executed revisions and artifacts

| Campaign | Published revision | Executed merge revision | Source tree | Result |
| --- | --- | --- | --- | --- |
| First | `18fe6c3408e6d581961bcf7f2721f72ca74cfbfa` | `6c3e5a44d54d83f1bacbd2977fe2fb32e11c0621` | `b568e8bf4d9970418b7c1f4ef2984ce391ffa5f5` | Failed qualification before any timing sample |
| Second | `3091e80fce0bf8db43b7526cb68073e6c7910f35` | `82fd75aa6e8fbe2b1d53624f7d2ea1610d76cc70` | `d6212e4ad0b816bf025b3aa17297c445d50d91a2` | All 384 timing samples completed; strict target failed |
| Third | `d57f485eeb2b638d1f95a0594294d11e8001fe8e` | `f638b6cd9819f52b53b142bf688d7ef07e252906` | `07235def43af3ffbdc218353d40f3f9e22b8dfd9` | Corrected selector and public edit gates passed; 384 primary and 192 diagnostic samples completed; strict target failed |

- First: [run 44293812634](https://github.com/glopyglerky/lustr/actions/runs/44293812634), job `134765521290`, artifact `14424121317`.
- Second: [run 44298375054](https://github.com/glopyglerky/lustr/actions/runs/44298375054), job `134781261965`, artifact `14425773276`.
- Third: [run 44305235529](https://github.com/glopyglerky/lustr/actions/runs/44305235529), job `134804957788`, artifact `14429133342`.
- Candidate discussion: [draft PR #25](https://github.com/glopyglerky/lustr/pull/25).

Original ZIP SHA-256:

```text
65bcf79991f5dc265fc96c306f15ec82b1e1d504fe93f70142d1208cd116c37d  artifact-14424121317.zip
f73d4bdf646d3abd9a426bcc0282fad244097f8f600eb78787c29332f6912e9a  artifact-14425773276.zip
de6e9fa011c979b04f4c0b64a7fa97783aa9fc7b19a946b0d0092546d122a64d  artifact-14429133342.zip
```

`manifest.json` binds each ZIP and frozen runner file to its executed revision.
Runner bytes were checked against the compiler/runner hashes retained by the
original campaign. The second runner predates the later cached-edit composition
gate and residual diagnostic tool. Replaying with a newer runner would impose
evidence requirements that the original run did not execute.

## Second campaign: complete same-host results

All values below are end-to-end command medians in milliseconds, with 24 paired
observations for each lane and operation. The public request has the original
explicit manifest and binary target, offline/locked flags, and **no package
selector**. Source, target, overflow checks, debug assertions, panic mode, and
application profile are matched. All four operations and all four lanes are
retained.

| Operation | Baseline public Lustr | Candidate public Lustr | Pristine Lust | rustc via Cargo |
| --- | ---: | ---: | ---: | ---: |
| Cold check | 248.112 | 248.792 | 202.650 | 372.742 |
| Warm check | 44.963 | 2.519 | 1.834 | 12.741 |
| Cold build | 463.072 | 464.249 | 422.835 | 509.198 |
| Warm build | 46.826 | 5.343 | 4.597 | 13.061 |

The matched baseline launcher is built from PR #21 revision
`28a2842635ee994db492b43f8c956eddce57a803`. It and the candidate use the **same
candidate-built native runtime and SDK bytes**. The baseline build checks the
entire declared shared source closure, including the root host-runtime source,
before constructing that comparison. This is an entry-change comparison, not
two independent complete toolchain builds. The candidate package itself is
source-built. Pristine Lust is separately source-built from
`240e08d91d4aed0299639f6f11fd522912b248f9`, using the same optimized compiler
profile and pinned TinyCC. Rust source is pinned to
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.

### Paired estimates

Ratios are paired reference/candidate geometric means. Values above one favor
the candidate. Intervals are conditional paired-bootstrap 95% estimates using
10,000 resamples; they do not establish portability to another host or workload.

| Operation | Reference | Candidate | Ratio | Bootstrap 95% interval |
| --- | --- | --- | ---: | --- |
| Cold check | Baseline public | Candidate public | 0.996408 | [0.982808, 1.010744] |
| Warm check | Baseline public | Candidate public | 17.640942 | [17.262993, 17.987686] |
| Cold build | Baseline public | Candidate public | 0.996196 | [0.988192, 1.004537] |
| Warm build | Baseline public | Candidate public | 8.842109 | [8.625023, 9.072321] |
| Warm check | Pristine Lust | Candidate public | 0.726665 | [0.713429, 0.740703] |
| Warm build | Pristine Lust | Candidate public | 0.856118 | [0.837589, 0.875193] |
| Warm check | rustc via Cargo | Candidate public | 5.014577 | [4.931145, 5.091445] |
| Warm build | rustc via Cargo | Candidate public | 2.473785 | [2.416228, 2.534036] |

All 16 comparison rows, including the remaining cold comparisons, are present
in the original `lustr-warm-comparison/report.json` and generated README inside
the ZIP. No cold before/after speedup is established.

Relative to pristine Lust, candidate median latency remains **37.33% higher for
warm check** and **16.23% higher for warm build**, with median differences of
0.684690 ms and 0.745886 ms on this host. Both paired intervals exclude parity in
the unfavorable direction. The strict gate requires the lower paired speed-ratio
bound to reach one in both warm modes; it remains false and the measurement
step exits 2. The target was not relaxed.

### Excluded priming costs

Each new lane state receives three complete prime commands before its measured
warm command. These commands are excluded from steady-state samples and retained
individually in the raw log. The first native prime can initialize a daemon and
populate selection state; these are combined observed costs, not isolated
component measurements.

| Lane | Check prime 1 | Check prime 2 | Check prime 3 | Build prime 1 | Build prime 2 | Build prime 3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline public | 242.212 | 44.252 | 43.773 | 519.778 | 47.227 | 47.270 |
| Candidate public | 241.824 | 2.713 | 2.352 | 513.064 | 5.767 | 5.031 |
| Pristine Lust | 196.632 | 2.026 | 1.750 | 470.993 | 5.149 | 4.310 |
| rustc via Cargo | 12.827 | 12.611 | 12.508 | 13.096 | 12.961 | 12.835 |

Cold states use new owned directories with native daemon/disk reuse disabled;
OS caches are not flushed. Warm commands use distinct retained native daemons
and caches per lane; Cargo requires a fresh artifact result. Timing excludes
diagnostic instrumentation and uses the recorded balanced four-lane schedule.
The host reports four logical CPUs on an AMD EPYC 7763 and Ubuntu 24.04. Affinity
is observed, not pinned; frequency and host load are not controlled. Absolute
times must not be compared with the earlier PR #21 campaign as a source change
effect.

## Second campaign: executed checks and replay

The second run passed the complete 16-test entry contract, the production
selector gate (132 invocations, 250 assertions), and the seed fallback gate
(38 invocations, 85 assertions). The complete comparison retains:

- 3,340 raw records and 1,224 commands, with original stdout/stderr.
- 384 command timings and 576 excluded prime timings.
- 206 runtime executions bound to their successful build artifacts and exact
  agreement with Rust's exit status, stdout, and stderr.
- 12 initial/final retained check/build proofs, four explicit candidate
  selection-cache hits, and 156 owned daemon starts and clean stops.

The independent replay verified all raw stream hashes, source/runner identities,
command/environment bindings, schedules, runtime results, retained counters,
summary statistics, and the false target result.

Retained checks report zero new body/generated-code work. Retained builds report
zero rechecked bodies, dirty function units, or newly compiled objects and one
object-cache hit. Pristine retains 101,920 C bytes and candidate/baseline retain
101,999 C bytes: the launcher/support C text is reassembled before its object
cache lookup. These builds do not claim zero C assembly.

The source-built constructor's actual installed `lustr` invocation completed
successfully and its certificate is retained separately from the helper
regression's certificate. `constructor-replay.json` records the independent
archive check: exact command/status, certificate payload/key, actual launcher
identity, nine tool identities, and receipt equality with the measured package.
The archive does not contain all compiler binaries or preserve seed filesystem
ownership and modes. Live validation of those properties is the checked source
constructor operation; the archive replay does not pretend to restamp them.

## Preserved first failure

The first run stopped at the initial pristine retained-build assertion and has
**zero timing samples**. Its command 31 returned exit status zero with
`rechecked=0`, `dirty_units=0`, `units_compiled=0`, `unit_cache_hits=1`, and
`c_bytes=101920`. The harness incorrectly applied the zero-C check contract to a
build, producing `native no-op repeated body or generated-code work`.

The corrected operation-specific gate keeps the strict check rule and requires
no new body/object work for builds, with object-cache evidence for nonzero C.
The real first-run diagnostic records and counter mutations are regression
controls. The first ZIP remains unchanged; its incomplete report is still
rejected as performance evidence.

## Second campaign: qualification limits

After the measured source was published, independent review reproduced a Cargo
path changed to a FIFO: cached selection timed out while uncached selection
rejected it. The receipt reader had the same ordering risk. Therefore revision
`3091e80f` remains **correctness-unqualified**, despite its normal-workload gates
and complete performance observations. A separate reviewed correction checks
regular-file type before opening. Its local expanded selector gate and later
hosted qualification belong to their own revisions and must not be attributed
to this run.

This second campaign also did not execute the later combined actual-public
cache-hit/body-edit/type-error/restore gate or the same-host integrated-client
and selector residual diagnostics. The existing gates separately exercised
public retained output, daemon edits, and selector cache transitions; the
successor combines those checks. No remaining 0.7 ms component is assigned from
a measurement on another host. Individual benefits of seed initialization and
executable-stamp deduplication are not isolated by this complete before/after
campaign. Broader compiler, platform, and release gates remain separate.

## Portable replay

Python 3.11 or newer is sufficient; no compiler, network, or daemon is needed:

```sh
python3 docs/compiler-set/evidence/warm-entry-2026-11-18/replay.py
```

The command checks all original ZIP hashes and exact frozen runner closures,
verifies the first run's preserved failure, independently replays all 384 timings
in each complete primary campaign and the third run's 144 native plus 48 selector
diagnostic timings, and checks the archived constructor bindings. It also binds
the third selector/seed gate results and complete entry log to their retained
source and streams. Temporary extraction does not modify the evidence bundle.

## Fourth completed campaign

[Fourth results and limits](fourth-results.md) preserves the compact/access continuation at `3c378a11d`, measured against the already improved `d57f485e` public baseline. Warm check improves by paired ratio 1.042678 [1.002444, 1.085341]; warm build remains unestablished at 1.013003 [0.974849, 1.052925]. Candidate/pristine medians are 1.450204/1.143490 ms for check and 3.471401/3.121890 ms for build. Both strict Lust targets still fail. The new host is Intel Xeon 6975P-C; cross-host absolute changes are not attributed to source.

The original fourth ZIP, exact executed runners, selector and constructor source closures, and both installed-launcher seed bindings are now included in the same portable replay. It verifies all 384 primary timings, 144 native diagnostic timings, 48 selector timings, 154/290 selector gate, 38/85 seed gate, 103/486 compact gate and 16/16 entry log. Earlier original archives and failed outcomes remain unchanged.
