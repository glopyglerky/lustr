# Earlier native eligibility guards: hosted qualification and performance, 18 November 2026

**All ten jobs in this candidate's Compiler set run passed. The performance requirement remains unmet.** The unchanged fixed Cargo workload has two faster modes and seven slower modes against matched Rust/Cargo. The corrected unchanged-project campaign completes all five scenarios, but none establishes a speedup. These results do not establish a controlled before/after benefit caused by the guards.

[PR #18](https://github.com/glopyglerky/lustr/pull/18) contains only the three independently reviewed guard files. It keeps the signature/arity rejection before expensive body/ABI work and avoids codegen-unit symbol scanning when no native objects are installed. Admitted definitions retain the existing body, binding and actual ABI checks. Zero-native trace publication remains intact. This candidate is separate from the recovered object-cache work in PR #17.

This evidence snapshot is published on a separate branch. The compiler source head remains unchanged.

## Exact source and package identity

| Role | Identity |
| --- | --- |
| Measured PR head | `be27c3f32725dd80e623af89034b15eeebde7e9a` |
| Parent containing the corrected project gate | `34d847da3d90c3c2b253246a8ffdfdeb901c56b0` |
| Actual checked-out merge and workflow | `df8f55aa0430ef81f27e7f403d102256b7fc298b` |
| Exact checked-out source tree | `fdb6af7d1ea50c4b63fd8b6a217633979deedf22` |
| Full Rust SDK/source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Source-built job | [134603437673](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437673) |
| Compiler set run | [44245528716](https://github.com/glopyglerky/lustr/actions/runs/44245528716) |

The runner-source receipt, constructor source record and all three benchmark campaigns agree on the checkout and tree. The [identity record](identity.json) binds the three reviewed source blobs, protected native source and upstream pins, exact component hashes and constructor-receipt hashes. The fixed, slice and project records agree on the package components they each record. The project and fixed campaigns both record the Cargo coordinator; the slice does not separately record that binary.

The local reviewed patch was commit `992fe781180e0ecbdd7ba9053f02b1b788816996`, tree `d9ed7b6694a5517801fe4b2be2061bbda6173262`. Its three guard blobs were published unchanged on top of the corrected gate parent. The complete hosted tree above has its own source identity; the older local full tree is not relabeled as this hosted build.

## Hosted correctness and platform scope

| Compiler set job | Result |
| --- | --- |
| [Runner source provenance](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437437) | success |
| [Checked rustc semantic inspection](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437529) | success |
| [Coordinator minimum Rust](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437531) | success |
| [Cargo compatibility (ubuntu-24.04)](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437568) | success |
| [Cranelift with unwinding qualification](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437603) | success |
| [Pinned Lust reference and declared integration](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437621) | success |
| [Actual Lust subset qualification](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437664) | success |
| [Source-built mixed Rust and Lust](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437673) | success |
| [Cargo compatibility (windows-latest)](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603437773) | success |
| [Cargo compatibility (macos-26)](https://github.com/glopyglerky/lustr/actions/runs/44245528716/job/134603438045) | success |

The Linux source-built job constructed the complete optimized package, including the native shared library. Its existing mixed Cargo, body/signature edit, invalidation, warning, retained-check, metadata, library interoperability, linking and runtime assertions passed. The entry contract reports **16 passed, zero failed and zero ignored**. The cold mixed-project trace contains both `native-definitions=0` and `native-definitions=5` from separate compiler invocations. The current empty-object route and accelerated scalar route therefore both have concrete hosted execution evidence.

The [qualification projection](qualification.json) retains each recorded mixed/check/library command status, native-count trace observations and original file digests, including expected rejection and failed-link cases. It also binds the 16 runtime test names and original contract log. The original qualification ZIP preserves the full command outputs and target-layout evidence. Cross-target layout comparison is not native runtime execution on those target operating systems.

Windows and macOS Cargo compatibility passed in this run. Complete native/mixed runtime qualification here remains Linux x86-64. The [job observation](compiler-set-jobs.json) describes this Compiler set run; separate broader CI and historical platform failures retain their own identities and status.

## Fixed Cargo workload: all nine modes

Ratios are reference time divided by Lustr time, using the geometric mean of paired ratios. Values above one favor Lustr. Each mode retains all 12 pairs; brackets are paired bootstrap 95% intervals. Ratios are not computed by dividing the displayed medians. Applications use dev opt0/debug0 with ordinary unwind, overflow and debug assertions; compiler components use their unchanged release profiles.

| Cargo mode | Rust / Lustr ratio [95% interval] | Rust median (ms) | Lustr median (ms) |
| --- | ---: | ---: | ---: |
| Body edit | 1.0860 [1.0467, 1.1098] | 45.679 | 41.462 |
| Invalid-code rejection | 1.0555 [1.0488, 1.0629] | 31.010 | 29.376 |
| Compile and test | 0.9882 [0.9818, 0.9949] | 69.378 | 69.912 |
| No-op | 0.9456 [0.9171, 0.9755] | 8.640 | 8.969 |
| No-op check | 0.9436 [0.9233, 0.9638] | 8.338 | 8.831 |
| Check after body edit | 0.9576 [0.9474, 0.9673] | 22.469 | 23.306 |
| Check | 0.9061 [0.8914, 0.9210] | 37.233 | 41.290 |
| Cold build | 0.8927 [0.8837, 0.9028] | 143.185 | 160.697 |
| Signature edit | 0.8701 [0.8635, 0.8771] | 48.044 | 55.081 |

Body editing and rejection retain their advantage on this fixture. The other seven intervals remain below one. These same-run comparisons do not isolate the guards from their parent. Absolute timings from separate runners are not treated as a controlled before/after experiment.

## Retained native performance and the primary entry

All 27 native comparisons are retained below. The integrated Lust lane uses the retained native implementation; the primary entry includes Lustr routing. The standalone Rust reference starts nonincremental rustc for every request, including no-op. Its no-op result is not a Cargo Fresh comparison.

| Native mode | Pristine Lust / integrated Lust | Pristine Lust / primary entry | Standalone rustc / primary entry |
| --- | ---: | ---: | ---: |
| Body edit | 1.0418 [1.0003, 1.0839] | 1.0070 [0.9757, 1.0360] | 7.0725 [6.9089, 7.2363] |
| Invalid-code rejection | 1.0760 [0.9966, 1.1738] | 0.1219 [0.1158, 0.1302] | 0.9446 [0.9268, 0.9628] |
| Compile and test | 1.0196 [0.9715, 1.0705] | 1.0125 [0.9810, 1.0482] | 1.7954 [1.7559, 1.8419] |
| No-op | 0.9858 [0.9644, 1.0071] | 1.0929 [1.0702, 1.1149] | 12.0557 [11.8243, 12.2591] |
| No-op check | 1.0616 [1.0214, 1.0977] | 1.0113 [0.9776, 1.0456] | 6.5946 [6.4314, 6.7523] |
| Check after body edit | 1.0835 [1.0493, 1.1209] | 1.0880 [1.0593, 1.1162] | 6.1841 [6.0379, 6.3712] |
| Check | 1.1100 [1.0680, 1.1565] | 1.1004 [1.0253, 1.1766] | 5.6861 [5.3592, 5.9819] |
| Cold build | 1.0131 [0.9589, 1.0693] | 0.9500 [0.8884, 1.0220] | 0.8580 [0.8153, 0.9022] |
| Signature edit | 1.0308 [1.0050, 1.0557] | 1.0315 [0.9872, 1.0736] | 7.9276 [7.6291, 8.1799] |

The primary entry still has substantial rejection overhead versus pristine Lust. Its cold build and rejection are slower than the standalone Rust reference on this native fixture. Those outcomes remain visible. Intervals that cross one do not prove equivalence, and these measurements do not establish universal native-speed retention.

The [fixed-workload report](fixed-workload/README.md), [all timing samples and comparisons](fixed-workload/samples.json) and [timing CSV](fixed-workload/timings.csv) retain every measured value, including medians, p95 values and paired statistics.

## Separate public slice

The pinned public goport-util-small slice uses the existing upstream driver with no source edits between samples. It is a pruned/adapted slice, not unchanged full ts-rust or full Rust coverage. There are 12 pairs per phase. Warm native lanes retain their daemons; the Cargo lane is Fresh.

| Slice phase | Pristine Lust / integrated Lust | Pristine Lust / primary entry | Cargo / primary entry |
| --- | ---: | ---: | ---: |
| cold-check | 0.9877 [0.9747, 1.0003] | 0.8170 [0.8063, 0.8279] | 1.6299 [1.5949, 1.6698] |
| no-op-check | 1.0100 [0.9386, 1.0886] | 0.0399 [0.0377, 0.0423] | 0.2901 [0.2802, 0.3006] |
| cold-build | 0.9982 [0.9879, 1.0078] | 0.9103 [0.9037, 0.9177] | 1.2370 [1.2253, 1.2494] |
| no-op-build | 1.0020 [0.9534, 1.0565] | 0.0983 [0.0942, 0.1026] | 0.2684 [0.2616, 0.2757] |
| compile-plus-run | 1.0233 [0.9926, 1.0579] | 0.1299 [0.1251, 0.1364] | 0.2950 [0.2883, 0.3025] |
| runtime | 1.0029 [0.9867, 1.0176] | 0.9991 [0.9784, 1.0194] | 0.8213 [0.8060, 0.8365] |

Integrated Lust remains near pristine Lust on this slice. The primary entry has substantial warm no-op overhead relative to native Lust and Cargo. Cold checks/builds are faster than Cargo here, while runtime is slower. This slice does not establish that the new mixed-compiler guards improve performance, and it does not measure which entry phase causes the warm overhead.

The original path-free [slice report](public-slice/README.md) and [CSV](public-slice/samples.csv) are preserved exactly. The [complete samples and comparisons](public-slice/samples.json) are explicitly labeled as a path-free projection of the original report.

## Corrected unchanged-project campaign

The actual `crates/lustr` coordinator keeps its ordinary dev/test defaults, including debug=2, default features and two Cargo jobs. This profile is outside current mixed-object eligibility. These measurements cover the routed compiler and corrected executing-test gate on the unchanged project; they do not establish a benefit from native-object selection work.

| Scenario | Rust / Lustr ratio [95% interval] | Rust median (ms) | Lustr median (ms) |
| --- | ---: | ---: | ---: |
| clean-dependency-build | 0.9918 [0.9843, 0.9987] | 518.860 | 522.133 |
| primed-root-build | 1.0008 [0.9906, 1.0127] | 147.217 | 146.981 |
| primed-root-check | 0.9861 [0.9666, 1.0057] | 56.018 | 57.071 |
| no-op | 0.9617 [0.9304, 0.9924] | 9.987 | 9.989 |
| compile-plus-test | 0.9941 [0.9884, 1.0003] | 4775.826 | 4801.302 |

Each scenario contains 12 measured pairs and two excluded warmup pairs. All 255 recorded commands were checked, including 120 measured timings and 20 excluded warmup timings. Initial and final executing-test gates in both lanes each report 41 passed, nine existing explicitly ignored and zero failed tests. Structured E0502 rejection gates return the expected exit 1. All other commands succeed without signal, timeout or output-limit failure. Component, constructor, source-tree and lockfile hashes remain consistent across scenarios.

Clean-dependency-build and no-op have bootstrap intervals below one. The other three intervals cross one. No scenario establishes a speedup in this run. The primed root and compile-plus-test cases use a byte-preserving source timestamp change rather than body/signature edits. No-op changes no timestamps. Owned cold state is cleared; operating-system caches remain warm.

The six original [public project files](project-public/README.md) are preserved byte-for-byte, including samples, campaign identity, statistical replay, host facts and checksums. The original [encrypted project receipt](project-encrypted/receipt.json) and [status](project-encrypted/status.json) record all five scenarios passed and 600 original files retained. No private key was used to review the public campaign.

## Verification and exact original artifacts

- Fixed workload: the unchanged evidence replayer verified 1,135 records, 2,160 command-output records, 648 timed commands and all 36 comparisons.
- Public slice: the unchanged data-only replay verified 288 timings and all 18 comparisons, including outputs, initial/final gates, execution order and daemon shutdown.
- Unchanged project: a separately built data-only executable from the unchanged Rust statistics source reproduced the complete five-comparison receipt exactly. It did not execute any compiler or benchmark command during review.

All **59 comparisons** remain visible across the three campaigns. The intervals are exploratory shared-host observations, not equivalence tests or release admission.

| Original GitHub artifact | ID | ZIP SHA-256 |
| --- | ---: | --- |
| [coordinator-project-performance-1](https://github.com/glopyglerky/lustr/actions/runs/44245528716/artifacts/14404145161) | 14404145161 | `3556752a84e1ae1e2274e4d8e448c53b56c0f75e2a18d89c9b0db9d546aaecdc` |
| [upstream-public-slice-1](https://github.com/glopyglerky/lustr/actions/runs/44245528716/artifacts/14403575513) | 14403575513 | `a953f14aa1a02b9fb51569be7e23ec0ae250bfb9d624c977941e1dc65e756a48` |
| [coordinator-project-originals-encrypted-1](https://github.com/glopyglerky/lustr/actions/runs/44245528716/artifacts/14403068539) | 14403068539 | `1bf092a1ed29aafaf55b77a190535c08531474b999274bda9de3f25c090b1d4c` |
| [source-built-mixed-qualification-1](https://github.com/glopyglerky/lustr/actions/runs/44245528716/artifacts/14402694819) | 14402694819 | `0ca2aa7c22d4395003d5242141ce8bb65127ff849c2a5910d6a481cfd2d91828` |
| [runner-source-44245528716-1](https://github.com/glopyglerky/lustr/actions/runs/44245528716/artifacts/14401287779) | 14401287779 | `83acf285bd05b70d533a212a30e769754a35cb9ae74c943e57650c3c676e76cf` |

The exact bytes of all five downloaded ZIPs are retained in the new [authenticated original-artifact archive](original-artifacts-encrypted/private-evidence.tar.gz.cms), together with retrieval identity and evidence producer/verifier records. The [receipt](original-artifacts-encrypted/receipt.json) binds 28 original files, plaintext archive SHA-256 `1f251b17053aa058336a74286a196a63e0d3ac5880c9e3c501232af59237fb68` and ciphertext SHA-256 `5d635660868bb5cad678553b0072e7ce38ec8d726299fa0a7eda3e00bad139b2`.

The original qualification ZIP includes 12 native-cache `.bin` entries and ten `.rmeta` files. Those nested original bytes are preserved without filtering. No additional loose executor build/cache/home state was added. The helper receipt's generic exclusion wording describes that wrapper inventory; it does not exclude or rewrite contents already inside an original ZIP.

The qualification artifact records compiler component hashes, constructor receipts and build logs, but does not contain a complete runnable compiled toolchain package. Review verifies those recorded identity bindings and execution records; it does not independently rerun an archived compiler package.

The unchanged repository encryption helper used the pinned public certificate with DER SHA-256 `69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f`. No private key was used. A deterministic reconstruction from the frozen snapshot matches the original plaintext and manifest hashes. The ciphertext was not independently decrypted. Raw paths, commands and logs remain encrypted; public projections are never substituted for original bytes.

The new wrapper's coordinator status is `not-observed` because it retains artifact ZIPs, not a newly executed coordinator campaign. This does not override the original project's successful campaign and its own encrypted-retention status.

## Earlier local record and remaining admission work

The [earlier local evidence](https://github.com/glopyglerky/lustr/blob/cbb5d3994f801cec903cf383187ee8dbc1961700/docs/compiler-set/evidence/native-eligibility-20261118/README.md) remains unchanged. It records unsuccessful local constructor/shared-library attempts terminated by SIGKILL, whose cause was not established, and a later native qualification blocked by AF_UNIX socket creation returning EPERM. No local timing campaign was run. The successful hosted execution above has its own exact source, package and run identity.

The guards now have hosted correctness evidence. A controlled before/after performance comparison remains missing, seven fixed Cargo modes remain slower, and the public-entry overhead remains unresolved. The PR stays a separate draft candidate; this snapshot does not grant merge, broad performance or complete native-platform admission.
