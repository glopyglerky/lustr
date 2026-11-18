# Cargo overhead: completed hosted qualification, 18 November 2026

The corrected compiler-selection gate and all five unchanged-project scenarios passed in [run 44242694462](https://github.com/glopyglerky/lustr/actions/runs/44242694462). The fixed mixed Cargo campaign still has six measured regressions. This result clears the broken benchmark gate; it does not meet the overall performance requirement.

### Exact source and executed gates

- PR head: `34d847da3d90c3c2b253246a8ffdfdeb901c56b0`.
- Actual merge checkout: `480114c2dc973770fbfbbc3c2b32915b516c6716`; tree: `e1c028f15e29a72fdbe338883613c23a6c8dccec`, identical to the reviewed head tree.
- Coordinator subtree: `74f91068723cf2aa4a1dee168713281db7705a1f`.
- [Source-built job 134594250483](https://github.com/glopyglerky/lustr/actions/runs/44242694462/job/134594250483), attempt 1: complete optimized package construction, mixed graph, failed-link regression, retained metadata, library interoperability, and all 16 entry/runtime contract tests passed.

Step 36 passed the actual coordinator/direct-compiler regression, the gate unit test, 13 runner tests, three native-state tests, and 18 campaign-verifier tests. The workflow's 20 structural tests passed too. The routed regression executes a nonempty Cargo test project, covers direct and legacy selection, and rejects stale/mismatched/missing selection and `--no-run`. Its prior red result is preserved in the [implementation review](https://github.com/glopyglerky/lustr/pull/16#pullrequestreview-5951170322). Step 37 then completed the real-project campaign without changing its checks.

All ten jobs in this run have completed successfully, including Cargo compatibility on Ubuntu 24.04, macOS 26 and Windows. Separately, [CI run 44242694616](https://github.com/glopyglerky/lustr/actions/runs/44242694616) remains in progress; it is distinct from this completed Compiler set run. Source-built mixed compilation and the performance measurements below were qualified on Linux x86-64; the compatibility jobs do not establish identical mixed native coverage on every platform.

### Actual unchanged project, normal development/test profiles

Each scenario contains 12 measured pairs and two excluded warmup pairs. Ratios are matched Rust elapsed time divided by Lustr elapsed time: above 1.0 favors Lustr. These are paired geometric ratios with exploratory 95% bootstrap intervals, not ratios of the reported medians.

| Scenario | Rust / Lustr ratio | 95% interval | Observation |
| --- | ---: | --- | --- |
| Clean dependency build | 1.0037× | [0.9967, 1.0108] | Interval crosses 1.0 |
| Primed root build | 1.0023× | [0.9924, 1.0114] | Interval crosses 1.0 |
| Primed root check | 0.9790× | [0.9702, 0.9875] | Slower in this campaign |
| No-op | 1.0120× | [0.9794, 1.0607] | Interval crosses 1.0 |
| Compile and test | 0.9992× | [0.9929, 1.0045] | Interval crosses 1.0 |

Across the five scenarios, all 120 timed commands and 20 excluded warmup commands are present in balanced lane order. Each scenario has matching initial/final execution and structured E0502 rejection gates in both lanes. The test gate recorded 41 passed and nine existing explicitly ignored tests, with no failures or filtered tests. Source and lockfile guards, compiler evidence, constructor receipts, Cargo fingerprints, state resets and final shutdown receipts passed. Independent replay reproduces all five published statistical results exactly; the frozen source digest also matches the exact committed coordinator tree.

The project retains `debug=2`, which is outside current mixed-object eligibility. Native participation is not asserted. The primed operations force a root rebuild through a byte-preserving timestamp change; they are not body or signature edits. Four intervals cross 1.0, so they establish neither a measured win nor equivalence; the primed root check remains slower.

### Fixed mixed Cargo workload

This separate corpus uses its recorded development `opt0/debug0` profile and 12 pairs per mode. Original-record verification and statistical replay passed for all 36 campaign comparisons: 648 timed commands and 2,160 original output streams. Its nine Cargo outcomes are:

| Cargo operation | Rust / Lustr ratio | 95% interval | Observation |
| --- | ---: | --- | --- |
| Body edit | 1.0807× | [1.0686, 1.0967] | Faster |
| Invalid-code rejection | 1.0683× | [1.0521, 1.0848] | Faster |
| Compile and test | 1.0012× | [0.9852, 1.0161] | Inconclusive |
| No-op | 0.9462× | [0.9354, 0.9563] | Slower |
| No-op check | 0.9223× | [0.8855, 0.9565] | Slower |
| Check after body edit | 0.9540× | [0.9411, 0.9664] | Slower |
| Ordinary check | 0.9034× | [0.8896, 0.9158] | Slower |
| Cold build | 0.8845× | [0.8713, 0.8966] | Slower |
| Signature edit | 0.8496× | [0.8274, 0.8663] | Slower |

The supported body-edit and rejection advantages survive this integration. Six Cargo modes remain slower and compile-plus-test is inconclusive. These same-run comparisons cannot establish a causal improvement over an earlier hosted run. Shared-host variation, exploratory unadjusted intervals and this bounded corpus limit broader claims; no release or overall speed admission is granted.

### Retained artifacts and verification boundary

Each downloaded original ZIP matched GitHub's recorded byte count and SHA-256. Public checksums, source identities, component/build receipt consistency and statistical replays passed. The upstream public slice also replayed successfully: 288 timings and 18 comparisons, with no compiler execution during replay.

| Artifact | Exact artifact | Original ZIP SHA-256 |
| --- | --- | --- |
| Real-project public export | [14403265065](https://github.com/glopyglerky/lustr/actions/runs/44242694462/artifacts/14403265065) | `ada115375a8219243483b77794bc64dbb88535e8a022bc3a8aaa36be464be258` |
| Encrypted project originals | [14403070421](https://github.com/glopyglerky/lustr/actions/runs/44242694462/artifacts/14403070421) | `24b481f80bb4e3c93195f7bf1b1563b3c3c11e118d1e1759ab837225d111c05b` |
| Exact source / qualification | [14401914209](https://github.com/glopyglerky/lustr/actions/runs/44242694462/artifacts/14401914209) | `df0699339896d1e527cd7233f0c1e3e564df21ae682936852f20be40a6b87617` |
| Upstream public slice | [14401619234](https://github.com/glopyglerky/lustr/actions/runs/44242694462/artifacts/14401619234) | `9ebe483fce3be9c49f7ad83cacb1cdb73051b04ddf035ac1f545de6b11cbd99f` |
| Runner source provenance | [14399499202](https://github.com/glopyglerky/lustr/actions/runs/44242694462/artifacts/14399499202) | `a597b45a26de14fa068713237bba8ae7d51235dc8a5616302373ea04ccd28766` |

Encrypted retention reports all five scenarios passed and 600 original files retained. Its ciphertext SHA-256 is `2bf60d8ff0de6cad19972c90a5598dc7ba7bb0ea521d593e1cf52f784073cf5c`; the downloaded ciphertext length and digest match the receipt. This review did not decrypt the private originals. Exact private invocation paths were checked by the source-bound CI verifier before producing the public projection. Binary identities are the retained CI records; the compiled package itself is not present in these artifacts for a fresh independent binary rehash.

Earlier failed runs and reviews remain unchanged. PR #16 remains draft at the same source head. Remaining work is reducing the six fixed Cargo regressions and the ordinary-project check overhead, extending qualified native eligibility, and completing platform-specific mixed qualification and independent review before merging the preserved stack.

## Durable evidence snapshot

This evidence-only snapshot is separate from the measured compiler source. The full [fixed workload table](fixed-workload/README.md), [all fixed samples](fixed-workload/samples.json) and [timing CSV](fixed-workload/timings.csv) preserve all 36 native and Cargo comparisons. The fixed native primary entry also has cold-build and rejection regressions against the standalone Rust reference, and rejection overhead against pristine Lust. These results remain visible; preserving the implementation does not prove equal speed in every entry path.

The separate [public slice table](public-slice/README.md), [samples](public-slice/samples.json) and [CSV](public-slice/samples.csv) retain all 18 comparisons and 288 timings. This pruned/adapted slice does not represent an unchanged full upstream project or arbitrary Rust workloads. Warm native no-ops use daemons; Cargo no-ops are Fresh. The full fixed and slice projections are labeled derivatives, and do not replace the exact originals.

The six [original public project files](project-public/README.md), original [encrypted project receipt](project-encrypted/receipt.json), original [project status](project-encrypted/status.json) and ciphertext are preserved byte-for-byte. [qualification.json](qualification.json) records the completed public verification. [identity.json](identity.json) binds the source, component receipts, original artifact IDs, complete ZIP digests and replay executable identity.

All five exact original downloaded ZIPs, retrieval/job/replay records, preserved gate red/green evidence and producer/verifier sources are also retained in a new [encrypted artifact archive](original-artifacts-encrypted/private-evidence.tar.gz.cms). Its [receipt](original-artifacts-encrypted/receipt.json) and [status](original-artifacts-encrypted/status.json) describe the wrapper. Encryption used the unchanged repository helper and pinned public recipient certificate. No private key was accessed. A deterministic reconstruction of the frozen plaintext inventory matches the archive and manifest digests; the ciphertext was not independently decrypted.

The wrapper has coordinator campaign status `not-observed` because it contains an artifact snapshot, not a newly executed project campaign. The original project campaign and its own separate retention status record all five scenarios passed. The wrapper does not replace those original records. Raw paths and command output remain encrypted. Earlier failed runs are not reclassified.

Any unchanged helper receipt wording about excluding build/cache/home state describes the wrapper staging inventory only. The nested original qualification ZIPs are preserved without filtering and can contain generated application binaries, native test objects or caches. Those retained qualification outputs are distinct from the compiled toolchain package, whose binaries are not present for an independent rehash.

Original-archive plaintext SHA-256: `b483f81ec4ce447b08d3cc41ad66bc46670a6486c01a7c85796992ad7af31215`; ciphertext SHA-256: `de3b703a2ac4dd05036362afa6887cfe6a043fb8f8c33cf4410c88847cb609ba`.
