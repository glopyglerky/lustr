# Native object reuse: hosted qualification and performance, 18 November 2026

**The recovered native-object reuse path passes the hosted correctness checks. The performance requirement remains unmet.** The fixed Cargo workload still has seven slower modes and two faster modes against matched Rust/Cargo. The corrected unchanged-project benchmark now completes all five scenarios. These results do not establish a before/after speedup caused by the cache change.

[PR #17](https://github.com/glopyglerky/lustr/pull/17) preserves the recovered cache work separately from PR #16 and the eligibility-guard experiment in PR #18. This evidence snapshot is published separately so the measured compiler source head remains unchanged.

## Source and qualification identity

| Role | Identity |
| --- | --- |
| Measured PR head | `e674fab486d4ee6b67a2372a5e9e135588dcb076` |
| Parent containing the corrected project gate | `34d847da3d90c3c2b253246a8ffdfdeb901c56b0` |
| Actual checked-out merge and workflow | `96fa69536a68e7c2084e746d31e56a453611d949` |
| Exact checked-out source tree | `426397dfbefec5d81dd4cfa195450eb947b4b5d1` |
| Full Rust SDK/source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Source-built workflow job | [134595232335](https://github.com/glopyglerky/lustr/actions/runs/44243040376/job/134595232335) |
| Run | [44243040376](https://github.com/glopyglerky/lustr/actions/runs/44243040376) |

The source-built Linux x86-64 job completed successfully: complete optimized constructor, native layouts, mixed Cargo graph, metadata and retained checks, mixed libraries, entry/runtime contract, fixed workload, public slice and corrected unchanged-project campaign. The macOS-26 Cargo lane was queued at the initial Linux artifact capture, then failed in [job 134595232256](https://github.com/glopyglerky/lustr/actions/runs/44243040376/job/134595232256). The complete Compiler set run therefore has nine successful jobs and one failed platform job. The Linux source-built results below remain successful; complete platform qualification is not established.

The native source and upstream pins remain unchanged. The [identity record](identity.json) binds exact component and constructor-receipt hashes, original artifact IDs and original ZIP digests to this run. The runner-source receipt and checked-out-source artifact agree.

## Recovered cache behavior

The existing source-bound qualifier exercised all nine cases below. It also checked canonical native-source and object hashes, Rust-symbol binding, actual executable output, preservation of prior successful object bytes, separate generation inodes, and preservation of the last successful certificate after link failure. Per-case trace/status/output digests are retained in [cache-qualification.json](cache-qualification.json); the original files remain in the encrypted original qualification ZIP.

| Case | Compiler exit | Native objects reused | Native compilations | Retained Rust objects |
| --- | ---: | ---: | ---: | --- |
| cold | 0 | 0 | 2 | no |
| literal | 0 | 0 | 1 | yes |
| rust-context | 0 | 2 | 0 | no |
| damaged-object | 0 | 1 | 1 | no |
| changed-abi-record | 0 | 1 | 1 | no |
| changed-source-record | 0 | 1 | 1 | no |
| failed-link | 101 | 2 | 0 | no |
| link-recovery | 0 | 2 | 0 | no |
| changed-profile | 0 | 0 | 2 | no |

The intended failed-link case contains linker exit 75, which Cargo reports as compiler failure 101; link recovery then succeeds. The literal edit exercises the existing retained Rust-work path. The Rust-context edit continues complete Rust analysis and reuses both separately validated native objects. Corruption, ABI-record and source-record changes each force the affected object to be regenerated; a changed overflow-checking profile regenerates both. Successful per-case application output was checked by the executing qualifier, but was not separately saved as application stdout files.

## Fixed Cargo workload: every mode retained

Ratios are matched reference time divided by Lustr time. Values above one favor Lustr. Each mode contains 12 pairs; brackets are the paired bootstrap 95% interval. Applications use dev opt0/debug0 with ordinary unwind, overflow and debug assertions. Compiler components use their release profiles.

| Cargo mode | Rust / Lustr ratio [95% interval] | Rust median (ms) | Lustr median (ms) |
| --- | ---: | ---: | ---: |
| Body edit | 1.0944 [1.0855, 1.1045] | 46.766 | 42.730 |
| Invalid-code rejection | 1.0617 [1.0515, 1.0732] | 32.099 | 30.300 |
| Compile and test | 0.9830 [0.9768, 0.9887] | 70.204 | 71.408 |
| No-op | 0.9494 [0.9213, 0.9799] | 8.647 | 9.115 |
| No-op check | 0.9458 [0.9197, 0.9718] | 8.591 | 8.903 |
| Check after body edit | 0.9634 [0.9574, 0.9691] | 23.018 | 24.058 |
| Check | 0.9069 [0.8913, 0.9223] | 38.475 | 42.213 |
| Cold build | 0.9018 [0.8810, 0.9312] | 147.509 | 166.936 |
| Signature edit | 0.8666 [0.8559, 0.8769] | 49.041 | 56.765 |

Body editing and rejection retain their advantage on this fixture. All seven other Cargo intervals remain below one. These same-run comparisons cannot isolate the effect of this PR from its parent; a controlled before/after campaign is still required. Absolute timings from different runners are not compared.

## Native retention and the primary entry

All native comparisons are shown here. The integrated Lust lane preserves the native implementation; the primary entry additionally includes Lustr routing. The standalone Rust comparison starts nonincremental rustc for every request, including no-op. Its no-op result is not a Cargo Fresh comparison.

| Native mode | Pristine Lust / integrated Lust | Pristine Lust / primary entry | Standalone rustc / primary entry |
| --- | ---: | ---: | ---: |
| Body edit | 1.0357 [1.0038, 1.0698] | 1.0018 [0.9893, 1.0132] | 7.0659 [6.9176, 7.2135] |
| Invalid-code rejection | 1.0814 [0.9849, 1.1884] | 0.1184 [0.1117, 0.1254] | 0.9303 [0.9195, 0.9416] |
| Compile and test | 0.9884 [0.9544, 1.0239] | 0.9696 [0.9369, 1.0067] | 1.7563 [1.7114, 1.8027] |
| No-op | 1.0072 [0.9830, 1.0304] | 1.1299 [1.0962, 1.1641] | 12.1935 [11.9391, 12.4654] |
| No-op check | 1.0712 [1.0385, 1.1055] | 1.0332 [0.9964, 1.0710] | 6.5987 [6.4097, 6.7900] |
| Check after body edit | 1.0967 [1.0499, 1.1463] | 1.0804 [1.0533, 1.1091] | 6.1664 [6.0563, 6.2787] |
| Check | 1.0791 [1.0253, 1.1350] | 1.1114 [1.0595, 1.1654] | 5.7079 [5.4985, 5.9343] |
| Cold build | 1.0424 [0.9800, 1.1058] | 0.9980 [0.9487, 1.0534] | 0.8786 [0.8449, 0.9111] |
| Signature edit | 1.0594 [1.0158, 1.1043] | 1.0739 [1.0401, 1.1106] | 8.1007 [7.9195, 8.2981] |

The primary entry has material rejection overhead versus pristine Lust, and its cold build and rejection are slower than the standalone Rust reference on this fixed native fixture. Those regressions remain visible. The reported intervals are exploratory shared-host estimates, not equivalence tests or proof of universal native-speed retention. Full tables and all timing values are in the [fixed-workload summary](fixed-workload/README.md), [samples](fixed-workload/samples.json) and [timing CSV](fixed-workload/timings.csv).

## Separate public slice

The pinned public goport-util-small slice uses the upstream driver and no source edits between samples. It is a pruned/adapted slice, not unchanged full ts-rust or full Rust coverage. There are 12 pairs per phase. Warm native lanes retain their daemons; the Cargo lane is Fresh.

| Slice phase | Pristine Lust / integrated Lust | Pristine Lust / primary entry | Cargo / primary entry |
| --- | ---: | ---: | ---: |
| cold-check | 0.9946 [0.9746, 1.0127] | 0.8265 [0.8155, 0.8380] | 1.6313 [1.6004, 1.6682] |
| no-op-check | 1.0244 [0.9714, 1.0766] | 0.0407 [0.0391, 0.0426] | 0.2864 [0.2791, 0.2930] |
| cold-build | 1.0013 [0.9910, 1.0119] | 0.9094 [0.8994, 0.9203] | 1.2294 [1.2118, 1.2492] |
| no-op-build | 1.0106 [0.9825, 1.0387] | 0.0997 [0.0970, 0.1027] | 0.2710 [0.2669, 0.2753] |
| compile-plus-run | 1.0282 [1.0065, 1.0533] | 0.1275 [0.1244, 0.1305] | 0.2908 [0.2863, 0.2950] |
| runtime | 1.0000 [0.9845, 1.0138] | 0.9744 [0.9385, 1.0053] | 0.8024 [0.7755, 0.8283] |

Integrated Lust is near pristine Lust on this slice. The primary entry has substantial warm no-op overhead relative to both native Lust and Cargo. Cold checks/builds are faster than Cargo here, while runtime is slower. This evidence identifies a route to investigate; it does not attribute the cost to metadata or establish a benefit from PR #18. The original path-free [slice summary](public-slice/README.md) is preserved exactly; [all samples/comparisons](public-slice/samples.json) are a labeled path-free projection of the original report.

## Corrected unchanged-project campaign

The actual crates/lustr coordinator keeps its ordinary dev/test defaults, including debug=2 and two Cargo jobs. This is outside mixed-object eligibility, so the campaign asserts no native acceleration. It verifies that the routed compiler and corrected executing-test gate work on the unchanged project.

| Scenario | Rust / Lustr ratio [95% interval] | Rust median (ms) | Lustr median (ms) |
| --- | ---: | ---: | ---: |
| clean-dependency-build | 0.9914 [0.9857, 0.9969] | 516.349 | 521.093 |
| primed-root-build | 0.9976 [0.9828, 1.0134] | 151.821 | 153.315 |
| primed-root-check | 0.9817 [0.9714, 0.9923] | 58.707 | 60.223 |
| no-op | 0.9981 [0.9820, 1.0084] | 10.088 | 10.045 |
| compile-plus-test | 0.9975 [0.9928, 1.0022] | 5037.076 | 5046.057 |

Every scenario contains 12 measured pairs and two excluded warmup pairs. Initial and final executing-test gates in both lanes each report 41 passed, nine existing explicitly ignored, and zero failed tests. Structured E0502 rejection gates return the expected exit 1. All other recorded commands succeed without signal, timeout or output-limit failure. Component/receipt hashes and source/lockfile hashes remain consistent across scenarios.

Clean-dependency-build and primed-root-check have intervals below one; the other three intervals cross one. No scenario establishes a speedup in this run. Primed root build/check and compile-plus-test use a byte-preserving source timestamp change; they are root rebuilds rather than body/signature edits. No-op changes no timestamps. Owned cold state is cleared, while operating-system caches remain warm.

The six original [public campaign files](project-public/README.md) are preserved byte-for-byte, including [samples](project-public/samples.json), [campaign identity](project-public/campaign.json), [replay receipt](project-public/replay.json), host facts and checksums. The original [encrypted project receipt](project-encrypted/receipt.json) and [status](project-encrypted/status.json) record all five scenarios passed and 600 original files retained. No private key was used to inspect this successful campaign.

## Independent verification and exact originals

- Fixed workload: 1,135 records, 2,160 command-output records, 648 timed commands and 36 paired comparisons verified by the unchanged evidence replay.
- Public slice: 288 timing values and 18 comparisons verified, including retained outputs, initial/final correctness, execution order and daemon shutdown.
- Unchanged project: all five comparisons reproduced exactly using a newly built local executable from the unchanged Rust statistics source. All 120 measured timing records and 20 excluded warmup records, gate counts and public-file checksums were checked. No archived compiler command was executed during review.

| Original GitHub artifact | ID | ZIP SHA-256 |
| --- | ---: | --- |
| [coordinator-project-performance-1](https://github.com/glopyglerky/lustr/actions/runs/44243040376/artifacts/14403035021) | 14403035021 | `ac273a79244b1f292fff6e72db85a794e82218fe413542e8169c94d73026e80e` |
| [coordinator-project-originals-encrypted-1](https://github.com/glopyglerky/lustr/actions/runs/44243040376/artifacts/14402870700) | 14402870700 | `14b8ff538989816887d606a25c0a5ff300ab7cfafb66df439f9170078dcbfab1` |
| [source-built-mixed-qualification-1](https://github.com/glopyglerky/lustr/actions/runs/44243040376/artifacts/14402551701) | 14402551701 | `3c0f810c93bee01de53f45c9c631fc8f624422c3cfdfa988051d71cc3626cbd5` |
| [upstream-public-slice-1](https://github.com/glopyglerky/lustr/actions/runs/44243040376/artifacts/14402500694) | 14402500694 | `cbbf5ad744f4bb1de0f106d4ce092a560a4f0d7c23d43a4be06bd337b7e9e7f9` |
| [runner-source-44243040376-1](https://github.com/glopyglerky/lustr/actions/runs/44243040376/artifacts/14400297905) | 14400297905 | `61a6a2feddc16f98cb1fe57bbc69d4a9031ab93d27e46b30138ff0b29228b83d` |

The exact bytes of all five original downloaded ZIPs are inside the new [authenticated artifact archive](original-artifacts-encrypted/private-evidence.tar.gz.cms), together with retrieval identity and producer/verifier records. Its [receipt](original-artifacts-encrypted/receipt.json) binds plaintext archive SHA-256 `33c48d8d9d499ba92a81e1602ed5f40f8c11c797c850242490c503c341ddd1a6`, ciphertext SHA-256 `fb07530b58edc742728da3106bb380ec30743b8dd9e0bca3f24c271d96fee1e0`, and 16 original files.

The original qualification ZIP also contains its generated object files, native-cache bytes and built fixture application bytes. Those nested contents remain preserved inside the unchanged original ZIP. The helper receipt's generic build/cache/home exclusion describes the absence of additional loose executor state in this wrapper; it does not exclude or rewrite build/cache/application content already inside the original ZIP.

The downloaded qualification artifact records compiler component hashes, constructor receipts and build logs, but does not contain a complete runnable compiled toolchain package. This review verifies those retained identity bindings and execution records; it does not independently rerun an archived compiler package. The generated fixture executables and objects mentioned above are a separate part of the original qualification ZIP.

The unchanged repository encryption helper used the pinned public recipient certificate with DER SHA-256 69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f. No private key was used. A deterministic reconstruction of the frozen plaintext inventory matches the original archive and manifest hashes; the ciphertext was not independently decrypted. Raw paths, commands and logs remain encrypted. Public projections are labeled and never substituted for exact original ZIP bytes.

The new artifact-wrapper status has coordinator fields marked not-observed because it contains a snapshot of ZIPs, not a newly executed project campaign. Those wrapper fields do not override the original successful project campaign and its separate encrypted-retention status.

## Later macOS compatibility failure

The `cargo_alias_can_select_release_without_enabling_cranelift` process test failed when rustc could not write a dependency-info file, with `ENOENT`. The other process test completed successfully. The original scratch allocator uses a timestamp and `create_dir_all`, which permits shared ownership if two allocations use the same timestamp. That source defect makes a collision a plausible explanation; the failed log does not record both scratch roots, so it does not prove that a collision caused this execution.

The exact failed job log, SHA-256 `0733ba10311aa4a0a5838fd4f4b2f3b7fac067e105bdd303f6c8ffcad1f61d7a`, and the exact process-test source are retained in a [separate encrypted failure attachment](macos-failure-encrypted/private-evidence.tar.gz.cms). Its [receipt](macos-failure-encrypted/receipt.json) and [status](macos-failure-encrypted/status.json) are separate from the previously sealed five-ZIP archive. That earlier archive and all Linux campaign exports remain byte-for-byte unchanged. This attachment used the same unchanged helper and pinned public certificate. Its deterministic review archive matches the receipt; no private key was used and the ciphertext was not independently decrypted. Its coordinator status is `not-observed` because it retains a failed platform job, not a new coordinator campaign.

## Remaining admission work

The cache implementation has concrete hosted correctness evidence. A controlled before/after performance comparison is still needed, together with the remaining platform qualification and independent review. Seven fixed Cargo modes and the primary-entry regressions remain unresolved. These results do not authorize a merge, claim all-workload acceleration, or combine this cache candidate with the independent eligibility-guard candidate.
