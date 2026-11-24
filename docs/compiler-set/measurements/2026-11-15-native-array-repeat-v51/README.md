# Native original-AST array repeats — component51

The complete original array-repeat assertion SHA256 `786203e024d5b411b8268606fb10fcb638c1cefc8bc2da87bcc98797fa38775e` compiles, links and runs natively with exact stock runtime output. Copied50 refused it with L0809; separate stock compilation/runtime succeeded. The rejected source supplies no successful native50 speed baseline. Earlier complete originals and protected Lust240e/Rust5b7b remain unchanged.

The owned formatter binds original native value/length children to exact source spans, brackets and semicolon, retaining explicit wrappers, a separate indentation box and a break after the semicolon. The anonymous-const length is printed as its value expression. Only derived panic literals change before complete native rechecking, borrowing, generation and linking. No retained transformed AST, protected printer copy, overlay, source rewrite, rustc native fallback or metadata fiction. See the [design and proof limits](../../proofs/native-assert-text.md).

Optimized direct27 and retained24 suites pass, followed by copied direct11, retained1 and host-input1 checks. Fresh actual gpt-6.1-sol HIGH/default R1 found a missing summary-engine entry (F51-R1-001). It was corrected before any campaign; a fresh MEDIUM/default R2 approved the corrected frozen8 scope. The first source and both retired review receipts remain retained. Cases cover zero/single/multiple repeats, nested arrays/tuples, calls, comments, parentheses, long values/lengths, order/once, borrowing, moves, zero/single Drop, unwind, const evaluation, invalid length/type/Copy/move/borrow, retained value/length/type/token/position/signature edits, rejection and recovery. Const-block formatting remains a separate gap.

## Equivalent successful work

SAME successful optimized51 direct/retained and stock compile the identical complete generated mixed workload. Full native/stock compile/runtime preflight passed BEFORE timing; bytes match timed sample0 cold. Nine rotated samples per phase/engine, fresh retained process per sample, native disk caches disabled before Check, OS pages untouched. Compiler opt3/CGU1/ThinLTO and application opt0/debug0/unwind/checks-on stay fixed. Compiler-build time is separate. Standalone no-op is not Cargo freshness; compile-plus-runtime is not Cargo test. Shared-host activity limits attribution.

No gain follows from comparison with rejecting50 or earlier distinct sources. Cold/edit/rejection stock regressions remain visible; a bounded no-op win does not close full-project speed.

Median milliseconds, lower is better.

| Phase | Direct51 | Retained51 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 281.715 | 430.347 | 110.064 | 282.639 | 431.306 | 110.825 |
| repeat | 253.974 | 12.639 | 100.425 | 255.016 | 13.508 | 101.091 |
| noop | 254.139 | 13.275 | 101.371 | 255.059 | 14.264 | 102.067 |
| body-edit | 285.313 | 437.051 | 112.827 | 286.330 | 438.255 | 113.468 |
| signature-edit | 281.740 | 445.136 | 110.289 | 282.745 | 446.029 | 111.093 |
| type-edit | 263.688 | 411.183 | 107.446 | 264.629 | 412.003 | 108.281 |
| macro-edit | 275.370 | 462.069 | 106.550 | 276.435 | 462.900 | 107.249 |
| rejection | 199.598 | 198.587 | 39.033 | — | — | — |
| recovery | 280.906 | 415.161 | 109.714 | 281.880 | 416.067 | 110.501 |

Raw events: 757; SHA256 `cb18bff2c3273335966124fa892578b694fb5b38700973e7abb91f87312355dd`. Summary51 SHA256 `c0d2362e8faba090d85768bf66ec002adee0ff3655f4cb5b44eaf945eb620e37`. Core `93c7d74231e38a3696c87b28c119b24f5a0dc668d4dc06ed690a80385a00c56d`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 378.078992964s` is separate from application performance. Independent raw verification checks every group, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Case | Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | --- | ---: |
| mixed | cold | compile | repeat51-direct | 189.091640, 190.147718, 198.311741, 260.121315, 281.714786, 296.004135, 304.842041, 309.335150, 321.555142 | 321.555142 |
| mixed | cold | compile-plus-runtime-suite | repeat51-direct | 189.728985, 190.926029, 199.081041, 261.072123, 282.639060, 297.012695, 305.946405, 310.346028, 322.495244 | 322.495244 |
| mixed | cold | compile | repeat51-retained | 308.109480, 314.624887, 321.705936, 399.280987, 430.346572, 442.865407, 473.240040, 503.607138, 519.617890 | 519.617890 |
| mixed | cold | compile-plus-runtime-suite | repeat51-retained | 308.835787, 315.422095, 322.545794, 400.117576, 431.305763, 443.767409, 474.244713, 504.733294, 520.974715 | 520.974715 |
| mixed | cold | compile | stock | 85.267142, 88.996422, 90.064647, 108.085319, 110.063996, 117.139694, 126.140135, 127.054013, 134.497399 | 134.497399 |
| mixed | cold | compile-plus-runtime-suite | stock | 85.854055, 89.813145, 90.656243, 108.872130, 110.825311, 117.978718, 126.932411, 127.798060, 135.579807 | 135.579807 |
| mixed | repeat | compile | repeat51-direct | 191.831605, 197.132010, 199.064319, 244.668250, 253.973687, 273.411240, 279.825135, 285.513108, 339.642605 | 339.642605 |
| mixed | repeat | compile-plus-runtime-suite | repeat51-direct | 192.595031, 197.876733, 199.835032, 245.498710, 255.016199, 274.193414, 280.813135, 286.534902, 340.650819 | 340.650819 |
| mixed | repeat | compile | repeat51-retained | 9.926118, 10.152451, 10.954308, 12.474352, 12.639357, 12.863629, 14.372058, 22.375276, 22.931338 | 22.931338 |
| mixed | repeat | compile-plus-runtime-suite | repeat51-retained | 10.696971, 11.137767, 11.731962, 13.298985, 13.507551, 13.968075, 15.299142, 23.504492, 23.849790 | 23.849790 |
| mixed | repeat | compile | stock | 83.478406, 84.748287, 86.249382, 99.119802, 100.425065, 106.734884, 116.421453, 117.276421, 128.036279 | 128.036279 |
| mixed | repeat | compile-plus-runtime-suite | stock | 84.040118, 85.337533, 86.804952, 99.747983, 101.091400, 107.397966, 117.216129, 118.106640, 128.972115 | 128.972115 |
| mixed | noop | compile | repeat51-direct | 198.414484, 199.620654, 205.269293, 251.108576, 254.138923, 267.172198, 281.867723, 296.865306, 315.605919 | 315.605919 |
| mixed | noop | compile-plus-runtime-suite | repeat51-direct | 199.055664, 200.326623, 206.065955, 252.163660, 255.058808, 268.132230, 282.786802, 298.212089, 316.583123 | 316.583123 |
| mixed | noop | compile | repeat51-retained | 9.623440, 11.083007, 11.388007, 12.850135, 13.275403, 14.588321, 16.623051, 20.534563, 22.836787 | 22.836787 |
| mixed | noop | compile-plus-runtime-suite | repeat51-retained | 10.529993, 12.034158, 12.166847, 13.797230, 14.264419, 15.324946, 17.520661, 21.571542, 23.976814 | 23.976814 |
| mixed | noop | compile | stock | 85.226357, 88.488527, 90.787728, 99.518912, 101.371381, 107.528721, 116.269043, 118.042292, 123.187249 | 123.187249 |
| mixed | noop | compile-plus-runtime-suite | stock | 85.795158, 89.063361, 91.401342, 100.210785, 102.067095, 108.187667, 117.135474, 118.831152, 124.059372 | 124.059372 |
| mixed | body-edit | compile | repeat51-direct | 191.148074, 193.843495, 209.149689, 273.438580, 285.313484, 295.488809, 320.831175, 348.782109, 402.685000 | 402.685000 |
| mixed | body-edit | compile-plus-runtime-suite | repeat51-direct | 191.920940, 194.561646, 209.922920, 274.301806, 286.329678, 296.403378, 321.735591, 349.832263, 403.837585 | 403.837585 |
| mixed | body-edit | compile | repeat51-retained | 310.534159, 315.981916, 328.870229, 418.591698, 437.051213, 452.847134, 472.616346, 484.706036, 557.960623 | 557.960623 |
| mixed | body-edit | compile-plus-runtime-suite | repeat51-retained | 311.261551, 316.756620, 329.755342, 419.485885, 438.254815, 453.890693, 473.849418, 485.776187, 559.185856 | 559.185856 |
| mixed | body-edit | compile | stock | 85.993835, 88.418649, 89.064677, 108.142649, 112.826788, 117.087429, 137.833922, 138.182237, 139.355940 | 139.355940 |
| mixed | body-edit | compile-plus-runtime-suite | stock | 86.614353, 89.068466, 89.676234, 108.851900, 113.467575, 117.864734, 138.759165, 139.190349, 140.192606 | 140.192606 |
| mixed | signature-edit | compile | repeat51-direct | 191.805396, 203.463372, 214.912860, 265.556416, 281.739540, 291.901515, 305.539326, 316.886375, 357.068710 | 357.068710 |
| mixed | signature-edit | compile-plus-runtime-suite | repeat51-direct | 192.461218, 204.223573, 215.767833, 266.424978, 282.745499, 292.800663, 306.602056, 317.982904, 358.072490 | 358.072490 |
| mixed | signature-edit | compile | repeat51-retained | 309.178738, 325.875534, 364.665881, 406.269421, 445.136050, 482.511502, 507.201905, 508.397387, 566.820652 | 566.820652 |
| mixed | signature-edit | compile-plus-runtime-suite | repeat51-retained | 309.788482, 326.712515, 365.576413, 407.313128, 446.028540, 483.553588, 508.315279, 509.609275, 567.792874 | 567.792874 |
| mixed | signature-edit | compile | stock | 86.844811, 87.469841, 99.906224, 104.238608, 110.289028, 122.670615, 125.933173, 126.211747, 136.880803 | 136.880803 |
| mixed | signature-edit | compile-plus-runtime-suite | stock | 87.450752, 88.055994, 100.686185, 104.892878, 111.093402, 123.520723, 126.816957, 127.125501, 137.712088 | 137.712088 |
| mixed | type-edit | compile | repeat51-direct | 190.175424, 202.396885, 250.981569, 261.941292, 263.687646, 264.538642, 304.811727, 332.098196, 336.057218 | 336.057218 |
| mixed | type-edit | compile-plus-runtime-suite | repeat51-direct | 190.853770, 203.181242, 251.834390, 262.913859, 264.629169, 265.527367, 305.820884, 333.084505, 337.089748 | 337.089748 |
| mixed | type-edit | compile | repeat51-retained | 297.111236, 329.239318, 394.300645, 400.661424, 411.183007, 420.175112, 460.870651, 464.037361, 536.865103 | 536.865103 |
| mixed | type-edit | compile-plus-runtime-suite | repeat51-retained | 298.131000, 329.992668, 395.172786, 401.543442, 412.002555, 421.288327, 461.762881, 465.020245, 538.062039 | 538.062039 |
| mixed | type-edit | compile | stock | 83.473533, 88.013519, 98.743667, 106.385163, 107.446311, 111.135998, 122.825779, 126.894206, 131.902020 | 131.902020 |
| mixed | type-edit | compile-plus-runtime-suite | stock | 84.054062, 88.687653, 99.432165, 107.079556, 108.280843, 111.872264, 123.603010, 127.857135, 132.820619 | 132.820619 |
| mixed | macro-edit | compile | repeat51-direct | 195.027149, 217.787512, 247.772591, 249.161603, 275.370061, 286.388090, 300.652252, 331.542516, 376.735082 | 376.735082 |
| mixed | macro-edit | compile-plus-runtime-suite | repeat51-direct | 195.744567, 218.516972, 248.717509, 249.978145, 276.434737, 287.261152, 301.754783, 332.654534, 378.027625 | 378.027625 |
| mixed | macro-edit | compile | repeat51-retained | 308.088713, 319.073747, 390.288736, 395.801849, 462.068563, 470.842166, 494.119794, 530.163771, 539.693627 | 539.693627 |
| mixed | macro-edit | compile-plus-runtime-suite | repeat51-retained | 308.819969, 320.292254, 391.060920, 396.728863, 462.899853, 471.799719, 495.170505, 531.190545, 540.791347 | 540.791347 |
| mixed | macro-edit | compile | stock | 86.079641, 88.233999, 106.071054, 106.297827, 106.550424, 115.450315, 121.643862, 127.716836, 129.518515 | 129.518515 |
| mixed | macro-edit | compile-plus-runtime-suite | stock | 86.644836, 88.831279, 106.849389, 107.015931, 107.249092, 116.170618, 122.623538, 128.527732, 130.333936 | 130.333936 |
| mixed | rejection | compile | repeat51-direct | 135.815669, 140.607312, 178.522023, 193.573281, 199.598159, 204.516842, 207.362281, 227.995454, 242.591373 | 242.591373 |
| mixed | rejection | compile | repeat51-retained | 127.995544, 147.084555, 165.488543, 195.115241, 198.586989, 202.315501, 213.836970, 229.187424, 239.636147 | 239.636147 |
| mixed | rejection | compile | stock | 30.975896, 31.520514, 35.383811, 38.967686, 39.033449, 39.706748, 42.822798, 43.290704, 44.422592 | 44.422592 |
| mixed | recovery | compile | repeat51-direct | 191.010996, 196.639503, 245.367314, 253.937816, 280.905704, 303.949781, 314.092489, 331.624428, 349.129474 | 349.129474 |
| mixed | recovery | compile-plus-runtime-suite | repeat51-direct | 191.753512, 197.352366, 246.209225, 254.865349, 281.879743, 305.126355, 315.127256, 332.853883, 350.266610 | 350.266610 |
| mixed | recovery | compile | repeat51-retained | 297.165388, 301.474271, 361.554635, 384.309889, 415.161057, 473.462620, 502.326424, 503.064039, 504.713754 | 504.713754 |
| mixed | recovery | compile-plus-runtime-suite | repeat51-retained | 297.895899, 302.486615, 362.326272, 385.671720, 416.067162, 474.537494, 503.305094, 504.043008, 505.714736 | 505.714736 |
| mixed | recovery | compile | stock | 87.516651, 87.852123, 100.658165, 109.449737, 109.714264, 118.305350, 119.103892, 123.149102, 125.439196 | 125.439196 |
| mixed | recovery | compile-plus-runtime-suite | stock | 88.123669, 88.521208, 101.354843, 110.186597, 110.501279, 119.040295, 120.086053, 124.004530, 126.269549 | 126.269549 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
