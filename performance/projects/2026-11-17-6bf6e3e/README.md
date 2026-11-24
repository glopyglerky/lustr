# Completed unchanged-project campaign at `6bf6e3e`

The actual coordinator project completed all five scenarios in
[source-built run 44179667793](https://github.com/glopyglerky/lustr/actions/runs/44179667793/job/134380277750).
The project step and encrypted retention passed. The enclosing source-built job
failed because the separate public-slice recorder failed its initial retained
check before collecting samples. That separate failure remains recorded; it
does not invalidate this completed project campaign or turn the job green.

Authenticated recovery and recorded-source reconciliation of the project's
encrypted originals passed. The original public and encrypted ZIPs, six public
files, all 120 timing values and recovery receipts are retained here.

## Complete-work results

Times are milliseconds. Each scenario contains twelve measured pairs after two
excluded warmup pairs. Ratios are paired geometric means of matched Rust time
divided by Lustr time; they are not ratios of the displayed medians. Intervals
are conditional paired-bootstrap 95% intervals with 10,000 resamples, using the
existing estimator. These exploratory comparisons are not adjusted for
multiplicity and do not establish equivalence.

| Scenario | Matched Rust median | Lustr median | Ratio [95% interval] | Observation |
| --- | ---: | ---: | --- | --- |
| Clean dependency build | 495.682 | 500.290 | 0.9905 [0.9836, 0.9973] | Slower |
| Primed root build | 146.147 | 147.562 | 0.9932 [0.9777, 1.0083] | Inconclusive |
| Primed root check | 47.891 | 49.520 | 0.9672 [0.9547, 0.9808] | Slower |
| No-op build | 8.872 | 9.904 | 0.9130 [0.8930, 0.9393] | Slower |
| Compile plus test | 4583.380 | 4653.394 | 0.9832 [0.9793, 0.9879] | Slower |

Four intervals favor matched Rust; primed root build is inconclusive. This
campaign establishes remaining default-profile overhead, without demonstrating
an ordinary Cargo speedup. The earlier
[`5a3f493` project campaign](../2026-11-17-5a3f493/README.md) and
[`40d2ac3` project campaign](../2026-11-17-40d2ac3/README.md) remain separate.
Differences between shared-runner campaigns do not isolate the effect of a
source change.

[The original public table](public/README.md) includes both p95 values.
[timings.csv](timings.csv) is a lossless projection of all 120 timed observations,
including pair order and original stream hashes. The full public sample record
also preserves warmups, command contracts and gate results. With twelve samples,
the nearest-rank p95 is the maximum observed time.

## Workload and correctness scope

The source is the unchanged `crates/lustr` package, including its library,
executables, integration tests and existing fixture builds. Both lanes preserve
normal dev/test profiles, debug=2, features and panic behavior. This profile is
outside native-object eligibility, so these measurements qualify compatibility
and overhead. Compiler components were built with the constructor's release
profile; application builds retain their normal defaults.

The three primed scenarios change only `src/lib.rs`'s timestamp. They do not
edit bodies or signatures. The project has no registry dependencies; cold state
clearing leaves operating-system page caches and dependency sources warm.
Before cold resets, the runner requires observed successful owned-daemon
shutdown. No native participation is asserted. Tests and structured E0502
rejection gates execute in both lanes before and after every scenario. Each
executing test gate reports **40 passed, 9 existing ignored, 0 failed**; ignored
tests are not counted as passes.

The [host snapshot](public/host.json) reports Intel Xeon 6975P-C, four visible
logical CPUs with affinity 0–3, 16,464,306,176 bytes of physical memory and Linux
7.0.0-1014-aws. Two Cargo jobs are used. CPU quota, memory limit and frequency
governor are unreported. No process CPU time, RSS, energy or isolated application
runtime is measured. Physical memory is not a process budget; affinity is
observed, not pinned. Host sharing and twelve pairs limit generalization.

## Source and original artifact identity

| Identity | Value |
| --- | --- |
| PR head | `6bf6e3e3384a99fbaeab2c66aa7c97bc73231996` |
| Actual merge checkout | `5a3c5666a16d46629a4787e37ca9b20e65720ccc` |
| Repository tree | `02652f033e84c0957dd2b99eb562db8171289e50` |
| Coordinator Git tree | `604de8e03b95de93f88da3ccc6a140fcc0a2953a` |
| Runner Git tree | `62c9fe321be80c576209406aaedcda24fc3eafb3` |
| Matching full-Rust revision | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Original public artifact | `14374661651` |
| Original public ZIP SHA256 | `bf3bc4f222afbce2a729788573ceb0a86b126bf0cba78d39039c9201212876f3` |
| Original encrypted artifact | `14375565377` |
| Original encrypted ZIP SHA256 | `b6afe1c1619ece2c726c545537b66956dd6abb7f8b0376754bb98729fdbe5d24` |

The exact recorded merge object and source trees were available during recovery.
The six public files and both ZIPs are byte-identical to the original artifacts.
[manifest.json](manifest.json) binds every retained file; original GitHub API
artifact metadata and the completed job observation are retained separately.
Original compiler binaries are absent from these archives.

## Actual authenticated recovery

Authenticated decryption succeeded with the dedicated recipient key. Before
extraction, the archive verifier checked the original archive and encrypted
manifest. The verified archive was extracted into a new private directory using
`tar --same-permissions`, preserving the original ordinary-file modes.

The [reconciliation result](recovery/result.json) freshly verified all **600
original files totaling 9,759,145 bytes**, including five reports and **510
command-output streams**. It verifies the frozen project against the recorded
Git source and loads the original exporter from that revision with replacement
refs disabled. All five public cases agree; samples and README reproduce byte
for byte; campaign, host and replay receipts agree; the existing Rust replay
reproduces all five comparisons. Private plaintext and the recipient key are
not committed. The original ciphertext remains inside the encrypted ZIP.

[The recovery receipt](recovery/receipt.json) identifies the actual decryption,
archive verification, extraction, reconciler source and result. The
[local replay build receipt](recovery/replay-build.json) identifies an existing
release replay build, freshly rehashed and verified to use the same runner
source subtree as this campaign. It was reused without recompilation. Its
executable hash differs from the original CI replay. The absent original
compiler, runtime, constructor and runner binaries were not freshly rehashed
or requalified. Exporter runtime identity was not recorded by campaign version
1 and remains null. Recovery preserves recorded identity consistency and CI
provenance without asserting a new binary-to-source equivalence proof.

From the repository root, the existing public checker discovers all retained
project campaigns and verifies their original artifacts and statistics:

```bash
cargo +nightly-2026-11-09 build --manifest-path crates/lustr-bench/Cargo.toml \
  --release --locked --example replay --target-dir target/project-replay
python3 tools/check-coordinator-performance-archives.py \
  --replay target/project-replay/release/examples/replay
```

The check verifies inventories, original artifact digests, public/ciphertext
bindings, exact timing CSVs and statistical replay. It does not perform a new
compiler run or a fresh private recovery. The private recovery above is an
additional completed check, identified by its retained receipts.
