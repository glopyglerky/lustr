# Completed unchanged-project campaign at `5a3f493`

The actual coordinator project completed all five scenarios in
[source-built run 44111326777](https://github.com/glopyglerky/lustr/actions/runs/44111326777/job/134137398335).
The source-built job passed, including the separate repaired profile and expanded
fixed-corpus campaign. Eight non-macOS jobs passed; macOS compatibility was
cancelled without executing steps. This is not an all-platform or release pass.

Authenticated recovery and recorded-source reconciliation of this project's
encrypted originals also passed. The original public and encrypted artifact
ZIPs, six public files, 120 timing values and recovery receipts are retained here.

## Complete-work results

Times are milliseconds. Each scenario contains twelve measured pairs after two
excluded warmup pairs. Ratios are paired geometric means of matched Rust time
divided by Lustr time; they are not ratios of the displayed medians. Intervals
are conditional paired-bootstrap 95% intervals, using the existing estimator.

| Scenario | Matched Rust median | Lustr median | Ratio [95% interval] | Observation |
| --- | ---: | ---: | --- | --- |
| Clean dependency build | 536.577 | 537.211 | 0.9974 [0.9911, 1.0048] | Inconclusive |
| Primed root build | 159.417 | 163.345 | 0.9775 [0.9653, 0.9900] | Slower |
| Primed root check | 61.293 | 63.611 | 0.9533 [0.9415, 0.9667] | Slower |
| No-op build | 10.062 | 11.098 | 0.9319 [0.9109, 0.9560] | Slower |
| Compile plus test | 5099.683 | 5191.029 | 0.9814 [0.9768, 0.9861] | Slower |

Four intervals favor matched Rust; clean build is inconclusive. Median overhead
is approximately 2.5% for primed root build, 3.8% for primed check, 10.3% for
no-op and 1.8% for compile plus test. These are descriptive percentages from
the medians, without a pooled speed claim. The earlier
[`40d2ac3` project campaign](../2026-11-17-40d2ac3/README.md) remains separate;
differences between shared-runner campaigns do not isolate a source-change effect.

[The original public table](public/README.md) includes both p95 values.
[timings.csv](timings.csv) is a lossless projection of all 120 timed observations,
including pair order and original stream hashes. The full public sample record
also preserves warmups, command contracts and gate results.

## Workload and correctness scope

The source is the unchanged `crates/lustr` package, including its library,
executables, integration tests and existing fixture builds. Both lanes preserve
normal dev/test profiles, debug=2, features and panic behavior. This profile is
outside native-object eligibility, so the campaign measures compatibility and
overhead rather than larger native acceleration.

The three primed scenarios change only `src/lib.rs`'s timestamp. They do not
edit bodies or signatures. The project has no registry dependencies; cold state
clearing leaves operating-system page caches and dependency sources warm.
Before cold resets, the runner requires observed successful owned-daemon
shutdown. The recorded shutdowns observed zero native children, without
claiming native work. Tests and structured E0502 rejection gates execute in both
lanes before and after every scenario. The test gate reports **40 passed,
9 existing ignored, 0 failed**; ignored tests are not counted as passes.

The [host snapshot](public/host.json) reports Intel Xeon 6975P-C, four visible
logical CPUs with affinity 0–3, 16,464,297,984 bytes of physical memory and Linux
7.0.0-1014-aws. Two Cargo jobs are used. CPU quota, memory limit and frequency
governor are unreported. No process CPU time, RSS, energy or isolated application
runtime is measured. Physical memory is not a process budget; affinity is
observed, not pinned. Host sharing and twelve pairs limit generalization.

## Source and original artifact identity

| Identity | Value |
| --- | --- |
| PR head | `5a3f4937d94e8f028dd85385ea65d51fcfe35a3b` |
| Actual merge checkout | `b3393a4a2907228516546cd34f1a5c60aaf6aaaa` |
| Repository tree | `2fca60e57829890b10a4a23da57bafab2581b092` |
| Coordinator Git tree | `3f72ccd2f1c5f83f952e2a43564a046a32d75305` |
| Runner Git tree | `62c9fe321be80c576209406aaedcda24fc3eafb3` |
| Matching full-Rust revision | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Original public artifact | `14344745989` |
| Original public ZIP SHA256 | `6ab9dfb03080c48a406745ec90bf2ac79bca12db3c76f7553011f984deb107b5` |
| Original encrypted artifact | `14342529953` |
| Original encrypted ZIP SHA256 | `169470e0ca04a023ebb745b5f695aa4f559527b68004d2530655883c387797f5` |

The [exact source bundle](../../sources/2026-11-17-5a3f493/README.md) preserves
the merge object and its prerequisite identities after the live PR ref moves.
The six public files and both ZIPs are byte-identical to the original artifacts.
[manifest.json](manifest.json) binds every retained file; API metadata and the
completed job observation are retained separately. Original compiler binaries
are absent from these archives.

## Actual authenticated recovery

Authenticated decryption succeeded with the dedicated recipient key. Before
extraction, the archive verifier checked the original archive and encrypted
manifest. The verified archive was extracted into a new private directory using
`tar --same-permissions`, preserving the original ordinary-file modes.

The [reconciliation result](recovery/result.json) freshly verified all **600
original files totaling 9,757,639 bytes**, including five reports and **510
command-output streams**. It verifies the frozen project against the recorded
Git source and loads the original exporter from that revision with replacement
refs disabled. All five public cases agree; samples and README reproduce byte
for byte; campaign, host and replay receipts agree; the existing Rust replay
reproduces all five comparisons. Private plaintext and the recipient key are
not committed. The original ciphertext remains inside the encrypted ZIP.

[The recovery receipt](recovery/receipt.json) identifies the actual decryption,
archive verification, extraction, reconciler source and result. The
[local replay build receipt](recovery/replay-build.json) binds a fresh build of
the existing replay from the matching `5a3f493` runner subtree and Rust SDK.
Its executable hash differs from the original CI replay. The absent original
compiler, runtime, constructor and runner binaries were not freshly rehashed
or requalified. Exporter runtime identity was not recorded by campaign version
1 and remains null. Recovery preserves recorded identity consistency and CI
provenance without asserting a new binary-to-source equivalence proof.

From the repository root, the existing public checker verifies both retained
project campaigns without decrypting private originals:

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
