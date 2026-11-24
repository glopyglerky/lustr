# Native owned OsString/PathBuf expansion — component39

Both exact original delayed owned OsString -> PathBuf and reciprocal PathBuf -> OsString witnesses now compile, link and run through the native compiler. Their source hashes remain `7e60a5dcb96c65e84007b0856c614db3a2ffe45fff7bafc4612cf43cc3a32a70` and `ccf8301ecbe890192caf0238b5690e03d0cbbf17a3d2eace63c0bd865b06393f`. Native37 and38 first L0640 failures and stock runtime witnesses remain preserved. No application or protected upstream source was rewritten.

The owned classifier now accepts only checked nongeneric canonical platform ADTs as OsString/PathBuf sources, carrying their resolved symbols into the existing owned AST reindexer. Original source, ambiguity/Extend guards and complete native collect/check/borrowck/generation/link remain. Mixed arrays, aliases, closures, Drop/unwind and arena ordering match stock. Eight invalid move/borrow/mutability/competing-method/custom-Extend/shadow cases reject. Retained source and target edits, invalid transitions, recovery and no-op proofs match cold native and stock. Optimized direct9 and retained12 suites pass; copied component owned-edit2 and host-linker1 checks are recorded separately. Unchanged product34 is not freshly qualified by these passes.

The first dev run lacked the required sibling namespace file and failed before native entry. After staging, native rechecking exposed an incorrect segmented source type spelling; it was corrected to the actual resolved canonical symbol. Both failures remain preserved. The original dev tests and final same-function direct2/retained2 tests pass. The first new campaign stopped at native L0610 on the exact same-function source. At most two elaboration rounds now expose fresh checked target constraints after full native recheck, with no source-syntax guessing; a new five-file Sol recheck approved this focused change. The failed campaign and diagnostic source/logs remain preserved; the new campaign uses identical original source. Root authored the nine-file delta as gpt-6.1-sol xhigh/default through T3 Code -> Codex; a fresh frozen gpt-6.1-sol high/default static reviewer approved without findings, ran no tests, and was task_cancel-retired with actual terminal/null-active/zero-pending/no-background receipt.

## New successful native workload

Nine samples per phase/engine on shared Intel Core Ultra5 235T Linux GNU x86_64 NAS, CPUs10-13/jobs2; candidate39 direct, SAME candidate39 retained and pinned stock rustc rotate per phase/sample. Source snapshots, commands, raw events, exact compiler/runtime/probe/harness identities, statuses and runtime stdout/stderr are retained. Candidate library uses unchanged opt3/CGU1/ThinLTO/line tables; application opt0/debug0/unwind/overflow and debug checks on. Fresh retained process per sample; native disk cache disabled before Check; OS pages untouched. Each accepted artifact is ELF, and runtime/status/output agreement is required; invalid edits reject with status1. No fallback or automatic application retry.

This is a separate successful owned-conversion workload, not a speed gain against rejecting38 and not comparable to the previous PathBuf source as an optimization. Cold and edits still trail stock. Bounded verified unchanged-artifact reuse does not establish general Cargo freshness or full-project speed.

Median milliseconds, lower is better; compile-plus-runtime is the real executable suite, not Cargo test.

| Phase | Direct39 | Retained39 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 239.426 | 423.490 | 95.271 | 240.233 | 424.378 | 95.945 |
| repeat | 260.252 | 13.675 | 94.611 | 261.120 | 14.686 | 95.333 |
| noop | 252.033 | 13.068 | 93.722 | 252.899 | 13.903 | 94.529 |
| body-edit | 242.899 | 390.977 | 93.479 | 243.652 | 391.858 | 94.170 |
| signature-edit | 245.117 | 396.848 | 90.515 | 245.876 | 397.697 | 91.220 |
| type-edit | 250.932 | 398.883 | 89.767 | 251.825 | 399.788 | 90.586 |
| macro-edit | 252.495 | 392.978 | 94.537 | 253.457 | 393.938 | 95.269 |
| rejection | 170.927 | 178.440 | 34.096 | — | — | — |
| recovery | 246.661 | 384.362 | 90.622 | 247.553 | 385.238 | 91.316 |

Raw events: 757. Raw SHA256 `a0db1214bfa51be04a3de7690ec4f4946f51253f3cf5f2db92a9488527728718`; summary SHA256 `3df6b036a4d03780a76b803bb59c901dcbcbf87818c574c8db9a9f4486bfe2b3`. Compiler core `5281d52e7926af2844f0aad1ac338c49122f8e29920f0b8a531f906cbbf9aeb7`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Optimized compiler build receipt: `exit status: 0; elapsed 396.617828264s` This compiler BUILD duration is separate from application performance.

## Complete samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | owned39-direct | 230.591058, 232.183974, 235.719044, 236.162944, 239.425958, 265.181013, 274.043080, 292.351305, 315.584282 | 315.584282 |
| cold | compile-plus-runtime-suite | owned39-direct | 231.377362, 232.951256, 236.608328, 236.968343, 240.232982, 265.963053, 274.996885, 293.313563, 316.672897 | 316.672897 |
| cold | compile | owned39-retained | 348.819525, 355.643607, 389.352553, 407.265767, 423.489651, 425.632987, 441.762129, 448.533593, 457.637644 | 457.637644 |
| cold | compile-plus-runtime-suite | owned39-retained | 349.572607, 356.688087, 390.259353, 408.093677, 424.377924, 426.488676, 442.709173, 449.518631, 458.560659 | 458.560659 |
| cold | compile | stock | 82.959220, 85.975177, 91.248597, 91.879753, 95.271165, 97.174044, 102.503767, 103.900022, 105.115534 | 105.115534 |
| cold | compile-plus-runtime-suite | stock | 83.591236, 86.624684, 91.873101, 92.527124, 95.945014, 97.789271, 103.221125, 104.695586, 105.878445 | 105.878445 |
| repeat | compile | owned39-direct | 210.432511, 227.948684, 229.572497, 253.863235, 260.252404, 260.797982, 261.002351, 280.462028, 288.421157 | 288.421157 |
| repeat | compile-plus-runtime-suite | owned39-direct | 211.305969, 228.743720, 230.405844, 254.823039, 261.119833, 261.609892, 261.838866, 281.342484, 289.270195 | 289.270195 |
| repeat | compile | owned39-retained | 11.783205, 11.923687, 12.804479, 13.569057, 13.675429, 13.844821, 15.603068, 18.328765, 19.203378 | 19.203378 |
| repeat | compile-plus-runtime-suite | owned39-retained | 12.606699, 12.693197, 13.649076, 14.440075, 14.685606, 14.815524, 16.513052, 19.218172, 20.326324 | 20.326324 |
| repeat | compile | stock | 80.430499, 84.017606, 84.283710, 91.296571, 94.611331, 97.525549, 98.655158, 98.884413, 100.232031 | 100.232031 |
| repeat | compile-plus-runtime-suite | stock | 81.126734, 84.617303, 84.924805, 92.092064, 95.332786, 98.240416, 99.352014, 99.650560, 101.071864 | 101.071864 |
| noop | compile | owned39-direct | 219.161047, 234.619329, 238.775938, 239.259359, 252.032803, 253.372772, 266.355935, 274.100672, 293.394118 | 293.394118 |
| noop | compile-plus-runtime-suite | owned39-direct | 219.958233, 235.437784, 239.658350, 239.990081, 252.898645, 254.402244, 267.286520, 275.015275, 294.312868 | 294.312868 |
| noop | compile | owned39-retained | 11.297536, 11.777552, 12.460084, 12.768231, 13.067518, 17.352743, 18.690450, 19.031071, 19.553815 | 19.553815 |
| noop | compile-plus-runtime-suite | owned39-retained | 12.074922, 12.938349, 13.292647, 13.583257, 13.902953, 18.267224, 19.627678, 19.961747, 20.446910 | 20.446910 |
| noop | compile | stock | 84.558129, 86.728817, 89.429859, 92.845033, 93.722412, 93.862680, 93.928829, 97.472562, 100.064595 | 100.064595 |
| noop | compile-plus-runtime-suite | stock | 85.204740, 87.332093, 90.179940, 93.532720, 94.529149, 94.625160, 94.768537, 98.045423, 100.725676 | 100.725676 |
| body-edit | compile | owned39-direct | 219.130167, 226.645468, 231.829746, 238.480257, 242.898852, 258.902675, 266.749417, 271.285841, 293.781812 | 293.781812 |
| body-edit | compile-plus-runtime-suite | owned39-direct | 219.916858, 227.504655, 232.586667, 239.273439, 243.651554, 259.855521, 267.595125, 272.264661, 294.846207 | 294.846207 |
| body-edit | compile | owned39-retained | 370.489842, 377.026292, 384.027147, 390.639261, 390.976531, 401.315652, 413.805050, 462.190800, 546.854874 | 546.854874 |
| body-edit | compile-plus-runtime-suite | owned39-retained | 371.290504, 377.872600, 384.921949, 391.592618, 391.858336, 402.200688, 414.686026, 463.071038, 547.963502 | 547.963502 |
| body-edit | compile | stock | 85.734242, 86.687515, 91.416599, 92.228239, 93.478605, 95.658458, 97.231314, 99.280784, 100.210845 | 100.210845 |
| body-edit | compile-plus-runtime-suite | stock | 86.397600, 87.351431, 92.097734, 92.894825, 94.169705, 96.389816, 98.053909, 99.998076, 101.031555 | 101.031555 |
| signature-edit | compile | owned39-direct | 221.351653, 226.543045, 234.421701, 237.814622, 245.116985, 248.931060, 282.595345, 299.408959, 338.087970 | 338.087970 |
| signature-edit | compile-plus-runtime-suite | owned39-direct | 222.334592, 227.423318, 235.186557, 238.734254, 245.875990, 249.811728, 283.472276, 300.328862, 339.060808 | 339.060808 |
| signature-edit | compile | owned39-retained | 355.041674, 356.812607, 357.079034, 379.650623, 396.847752, 399.645293, 435.321226, 445.892776, 446.634691 | 446.634691 |
| signature-edit | compile-plus-runtime-suite | owned39-retained | 356.029183, 357.629020, 357.898342, 380.455607, 397.696794, 400.513901, 436.201536, 446.819470, 447.549455 | 447.549455 |
| signature-edit | compile | stock | 86.269057, 88.311512, 89.420359, 90.434952, 90.514680, 93.506102, 94.676685, 97.533171, 110.983582 | 110.983582 |
| signature-edit | compile-plus-runtime-suite | stock | 86.899078, 88.982944, 89.994537, 91.182851, 91.220370, 94.273734, 95.379324, 98.264663, 111.759565 | 111.759565 |
| type-edit | compile | owned39-direct | 219.060765, 229.726181, 236.103351, 245.638569, 250.931527, 255.923656, 267.846423, 282.837371, 293.666773 | 293.666773 |
| type-edit | compile-plus-runtime-suite | owned39-direct | 219.842250, 230.502729, 236.980046, 246.521841, 251.824513, 256.813179, 268.694672, 283.771650, 294.586570 | 294.586570 |
| type-edit | compile | owned39-retained | 350.385204, 377.011661, 381.880735, 396.734701, 398.883221, 409.942136, 432.291630, 439.598191, 460.423266 | 460.423266 |
| type-edit | compile-plus-runtime-suite | owned39-retained | 351.378449, 378.058841, 382.902456, 397.712467, 399.787954, 410.888098, 433.307725, 440.599282, 461.331735 | 461.331735 |
| type-edit | compile | stock | 84.069591, 87.489742, 87.855383, 89.701430, 89.766748, 93.150119, 97.843295, 101.306500, 102.565296 | 102.565296 |
| type-edit | compile-plus-runtime-suite | stock | 84.714740, 88.240949, 88.478423, 90.361861, 90.585992, 93.877049, 98.572651, 102.067684, 103.340481 | 103.340481 |
| macro-edit | compile | owned39-direct | 226.815515, 231.854301, 247.470243, 251.997881, 252.495131, 268.457676, 273.513868, 275.494129, 283.720840 | 283.720840 |
| macro-edit | compile-plus-runtime-suite | owned39-direct | 227.726550, 232.616742, 248.277765, 252.949126, 253.457251, 269.301521, 274.443879, 276.394104, 284.704318 | 284.704318 |
| macro-edit | compile | owned39-retained | 338.654471, 351.387894, 378.271759, 385.777420, 392.977867, 415.205198, 425.707571, 437.503482, 446.646961 | 446.646961 |
| macro-edit | compile-plus-runtime-suite | owned39-retained | 339.572526, 352.144951, 379.161831, 386.628481, 393.938181, 416.159495, 426.612050, 438.509458, 447.582471 | 447.582471 |
| macro-edit | compile | stock | 80.461540, 84.845415, 87.064286, 89.493866, 94.537121, 97.574536, 97.589810, 97.794611, 103.922813 | 103.922813 |
| macro-edit | compile-plus-runtime-suite | stock | 81.152624, 85.582388, 87.714811, 90.195029, 95.269135, 98.346722, 98.430427, 98.495567, 104.793645 | 104.793645 |
| rejection | compile | owned39-direct | 146.483117, 159.524907, 161.131516, 168.172642, 170.927026, 179.017397, 183.309088, 188.078589, 210.402316 | 210.402316 |
| rejection | compile | owned39-retained | 142.981902, 169.131485, 169.149703, 172.067945, 178.440020, 185.098864, 186.392043, 191.706201, 192.284435 | 192.284435 |
| rejection | compile | stock | 28.119954, 32.299958, 32.490703, 33.889292, 34.096434, 34.169372, 34.808347, 35.459168, 36.189085 | 36.189085 |
| recovery | compile | owned39-direct | 203.769889, 214.236878, 223.935129, 242.685485, 246.660788, 256.065241, 288.934998, 295.700752, 306.696906 | 306.696906 |
| recovery | compile-plus-runtime-suite | owned39-direct | 204.509545, 215.003666, 224.703776, 243.448558, 247.553119, 256.999467, 289.974466, 296.582947, 307.798341 | 307.798341 |
| recovery | compile | owned39-retained | 326.230748, 343.165771, 345.890708, 377.305113, 384.361934, 407.352326, 414.390936, 429.102227, 432.507809 | 432.507809 |
| recovery | compile-plus-runtime-suite | owned39-retained | 327.160023, 343.913736, 346.668250, 378.237178, 385.238163, 408.355652, 415.265594, 430.157859, 433.451494 | 433.451494 |
| recovery | compile | stock | 79.752248, 81.428376, 89.754808, 90.404476, 90.621807, 92.042502, 100.231324, 105.778219, 108.158494 | 108.158494 |
| recovery | compile-plus-runtime-suite | stock | 80.511545, 82.078095, 90.467705, 91.097156, 91.316053, 92.673886, 100.995476, 106.518050, 108.863407 | 108.863407 |

Stock is a standalone pinned rustc invocation for each phase; its repeated compilation is not Cargo freshness/no-op behavior. The copied-component owned check command contains two tests. Experimental component39 remains outside product34/published28. Runtime-loader closure, stable-file-during-request assumptions, general native Rust/crates/metadata/rlib/Cargo, supervised lifecycle and full-project speed remain OPEN. All seven public-release gates remain held. Upstream Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC loader dependencies remain accurately named; no all-Rust or whole-toolchain memory-safety claim.
