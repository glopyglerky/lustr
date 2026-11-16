# Native function statement items in unsafe-containing bodies - component56

The complete original nested-function witness `53a4b5d9e98d73de94e69473138724c85004fdbc63b1255aa8d6cbfc3d3f95a1` compiles and executes natively with stock output, original assertion text and Guard/drop behavior. Component55 rejected it in the owned parser; that rejection is not a successful timing baseline.

The owned parser sends each complete unchanged local function item to the pinned native item parser. The native scan supplies only a boundary; native arena append shifts all expression, pattern, type, block, string and nested statement-item references. The item is attached to its original block through Stmt::Item, retaining native FnDef body ranges. Native collection and full checking govern block-wide visibility, forward declarations, shadowing, duplicates, capture rejection, type/borrow/move/const checking, Drop and unwind. Exact safe Rust call target, qualifier, ABI and boolean-signature obligations still qualify before collection returns. There is no copied parser/checker, application rewrite or fallback.

Fresh Sol HIGH/default R1 approved the frozen parser integration. A fresh R2 reviewed final tests, contracts and measurement mode after added same-function delayed-conversion composition. Both completed and were task_cancel-retired. One first runtime command failed in bwrap before application execution because compilation had not finished; the subsequent completed native artifact ran once with stock parity. A brief qualification overlap has no timing attribution. First failures are retained.

Optimized33 direct and30 retained tests pass. Copied artifacts pass separate nested-item and combined-conversion direct/retained gates,15 assertion tests, old/local/call retained tests and host-input invalidation. Full generated native/stock compile/runtime preflight precedes timing and equals timed sample0 cold.

## Equivalent successful work

SAME successful56 optimized direct/retained/stock; nine rotated samples per phase. Compiler opt3/CGU1/ThinLTO; applications opt0/debug0/unwind/checks-on. Native disk caches disabled before Check; fresh bounded retained process per sample; OS pages untouched. Shared-host noise limits attribution. Compiler-build cost is separate. Standalone no-op is not Cargo freshness, and compile-plus-runtime is not Cargo test. No gain is claimed against rejecting55 or distinct earlier sources. Cold/edit/rejection regressions are retained.

Median milliseconds, lower is better.

| Phase | Direct56 | Retained56 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 261.577 | 402.147 | 105.016 | 262.454 | 403.433 | 105.689 |
| repeat | 248.288 | 11.936 | 101.354 | 249.107 | 12.787 | 101.970 |
| noop | 267.560 | 14.178 | 104.911 | 268.511 | 15.155 | 105.603 |
| body-edit | 254.913 | 399.913 | 103.205 | 255.788 | 400.806 | 103.952 |
| signature-edit | 264.041 | 402.694 | 104.591 | 264.842 | 403.650 | 105.404 |
| type-edit | 260.242 | 426.105 | 105.792 | 261.176 | 427.022 | 106.486 |
| macro-edit | 256.752 | 406.363 | 101.860 | 257.687 | 407.213 | 102.462 |
| rejection | 180.827 | 184.508 | 36.570 | — | — | — |
| recovery | 254.585 | 397.430 | 105.885 | 255.451 | 398.395 | 106.673 |

Raw757 events SHA256 `cfdf8c6c8b260b55e6606ee9ae97cbc18ebe3e6360544ff7ca07544a2e656066`; summary SHA256 `87e5d2928c4e06e29c14799ac4683aee21f9c14c530f5d4354e6d6c9138f982c`; core `e6361fe3cfdbba758f0f6f1ac584bc19598a6ebc260e23439c527a4bbd23a5e8`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Independent verification checks all51 groups, unique samples0..8, sorted values, medians/maxima,81 actual direct commands,81 retained calls/18 hits and81 stock commands. Accepted compiler build: `exit status: 0; elapsed 444.932490077s`. Compiler-build cost is separate from application timing.

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | item56-direct | 226.082640, 231.540975, 243.386228, 244.829189, 261.577146, 266.456690, 288.707383, 354.025748, 416.326239 | 416.326239 |
| cold | compile-plus-runtime-suite | item56-direct | 226.845612, 232.397268, 244.252058, 245.612403, 262.454496, 267.299155, 289.568939, 355.009100, 417.317273 | 417.317273 |
| cold | compile | item56-retained | 359.950293, 365.689877, 390.005435, 402.090113, 402.147129, 409.021001, 430.944906, 548.352674, 632.893509 | 632.893509 |
| cold | compile-plus-runtime-suite | item56-retained | 361.114234, 366.871786, 390.898170, 403.253075, 403.432679, 410.012430, 431.832654, 549.348568, 634.250528 | 634.250528 |
| cold | compile | stock | 93.925452, 97.339432, 104.616046, 104.865079, 105.016055, 105.124609, 110.566642, 125.181469, 142.081844 | 142.081844 |
| cold | compile-plus-runtime-suite | stock | 94.552869, 98.088818, 105.242086, 105.555911, 105.688531, 105.917026, 111.249471, 125.927802, 142.810233 | 142.810233 |
| repeat | compile | item56-direct | 223.065725, 224.721495, 243.233579, 244.208935, 248.287772, 259.963882, 264.409158, 333.687329, 368.337673 | 368.337673 |
| repeat | compile-plus-runtime-suite | item56-direct | 223.912947, 225.486959, 243.990457, 245.023616, 249.107376, 260.844033, 265.236535, 334.778431, 369.462444 | 369.462444 |
| repeat | compile | item56-retained | 11.051029, 11.459484, 11.650950, 11.814163, 11.936172, 14.082751, 19.283222, 19.862422, 22.556425 | 22.556425 |
| repeat | compile-plus-runtime-suite | item56-retained | 11.813514, 12.314452, 12.417134, 12.695985, 12.786622, 15.098422, 20.047709, 20.706973, 23.661095 | 23.661095 |
| repeat | compile | stock | 95.656081, 96.703159, 98.109670, 100.067353, 101.353937, 107.705150, 109.721309, 121.685585, 134.937970 | 134.937970 |
| repeat | compile-plus-runtime-suite | stock | 96.350750, 97.237499, 98.744999, 100.740755, 101.970457, 108.398496, 110.555629, 122.545493, 135.923824 | 135.923824 |
| noop | compile | item56-direct | 216.938901, 221.196242, 243.097300, 250.793285, 267.559508, 270.455874, 288.378141, 390.969200, 436.947098 | 436.947098 |
| noop | compile-plus-runtime-suite | item56-direct | 217.766817, 221.979298, 243.891081, 251.676246, 268.511454, 271.283118, 289.220870, 392.056903, 438.065712 | 438.065712 |
| noop | compile | item56-retained | 10.223792, 10.467012, 11.631735, 12.349940, 14.177646, 15.495730, 18.413754, 18.981206, 21.510051 | 21.510051 |
| noop | compile-plus-runtime-suite | item56-retained | 11.071661, 11.259332, 12.486207, 13.244170, 15.155278, 16.326876, 19.385278, 20.074598, 22.622608 | 22.622608 |
| noop | compile | stock | 91.921574, 93.311891, 98.417607, 101.173296, 104.911107, 105.151314, 116.463609, 129.670161, 134.151440 | 134.151440 |
| noop | compile-plus-runtime-suite | stock | 92.565547, 94.022023, 99.022674, 101.868833, 105.603261, 105.959223, 117.222798, 130.474562, 135.350729 | 135.350729 |
| body-edit | compile | item56-direct | 217.601138, 221.799575, 243.655620, 250.232366, 254.913241, 255.055583, 258.774515, 310.584824, 337.278794 | 337.278794 |
| body-edit | compile-plus-runtime-suite | item56-direct | 218.372086, 222.580588, 244.573885, 251.030570, 255.787952, 255.895413, 259.572976, 311.601288, 338.307843 | 338.307843 |
| body-edit | compile | item56-retained | 370.098986, 387.728310, 389.636193, 396.843910, 399.913412, 402.060460, 423.155592, 477.837466, 505.500127 | 505.500127 |
| body-edit | compile-plus-runtime-suite | item56-retained | 371.335361, 388.567438, 390.902572, 397.908601, 400.806177, 402.761049, 424.235890, 478.777603, 506.534718 | 506.534718 |
| body-edit | compile | stock | 94.708595, 96.445176, 100.044598, 100.487562, 103.204714, 103.944732, 105.017908, 117.850370, 125.506430 | 125.506430 |
| body-edit | compile-plus-runtime-suite | stock | 95.326517, 97.124146, 100.682835, 101.171881, 103.951639, 104.501688, 105.701528, 118.673711, 126.378283 | 126.378283 |
| signature-edit | compile | item56-direct | 229.479370, 231.999147, 245.691143, 249.334749, 264.041478, 264.859445, 275.622486, 282.119267, 352.956181 | 352.956181 |
| signature-edit | compile-plus-runtime-suite | item56-direct | 230.305263, 232.841123, 246.628219, 250.163393, 264.842211, 265.786022, 276.550549, 283.065170, 353.826073 | 353.826073 |
| signature-edit | compile | item56-retained | 359.206347, 387.061706, 394.173221, 400.126448, 402.694396, 413.760627, 433.265318, 486.492342, 490.716858 | 490.716858 |
| signature-edit | compile-plus-runtime-suite | item56-retained | 360.012178, 388.332233, 395.127547, 401.358875, 403.650010, 414.839282, 434.223744, 487.512153, 491.721762 | 491.721762 |
| signature-edit | compile | stock | 93.677326, 101.978293, 104.032846, 104.531696, 104.590970, 105.627622, 107.695916, 121.736838, 123.970285 | 123.970285 |
| signature-edit | compile-plus-runtime-suite | stock | 94.353765, 102.659515, 104.725953, 105.304000, 105.404141, 106.307194, 108.511122, 122.512007, 124.728801 | 124.728801 |
| type-edit | compile | item56-direct | 237.152089, 238.593035, 248.625212, 249.919709, 260.241839, 267.517125, 272.043848, 278.210148, 319.971007 | 319.971007 |
| type-edit | compile-plus-runtime-suite | item56-direct | 237.995215, 239.418860, 249.449024, 250.725894, 261.176026, 268.353323, 272.997105, 279.183215, 320.849299 | 320.849299 |
| type-edit | compile | item56-retained | 367.966095, 393.905517, 396.208292, 402.146670, 426.105153, 439.732263, 445.230887, 461.359889, 490.457317 | 490.457317 |
| type-edit | compile-plus-runtime-suite | item56-retained | 368.757020, 394.758564, 397.072360, 403.383849, 427.022100, 440.934042, 446.068790, 462.306031, 491.444851 | 491.444851 |
| type-edit | compile | stock | 96.157632, 97.783695, 101.388443, 102.866544, 105.791545, 106.027683, 107.328217, 113.538369, 121.387751 | 121.387751 |
| type-edit | compile-plus-runtime-suite | stock | 96.969791, 98.486515, 102.093410, 103.568151, 106.485546, 106.625667, 108.077979, 114.376819, 122.213238 | 122.213238 |
| macro-edit | compile | item56-direct | 230.585147, 232.251885, 245.432176, 254.885466, 256.751810, 257.876392, 259.565340, 264.327963, 288.279593 | 288.279593 |
| macro-edit | compile-plus-runtime-suite | item56-direct | 231.316348, 232.999123, 246.175002, 255.749858, 257.687101, 258.966723, 260.423474, 265.179459, 289.200326 | 289.200326 |
| macro-edit | compile | item56-retained | 378.757740, 393.432857, 394.490806, 403.492506, 406.363231, 419.289212, 432.522586, 435.953710, 443.230561 | 443.230561 |
| macro-edit | compile-plus-runtime-suite | item56-retained | 379.969239, 394.327028, 395.566904, 404.524129, 407.213000, 420.344306, 433.463332, 436.952310, 444.065931 | 444.065931 |
| macro-edit | compile | stock | 97.144251, 100.971511, 101.507757, 101.555262, 101.860499, 103.264490, 104.135507, 105.767714, 110.343790 | 110.343790 |
| macro-edit | compile-plus-runtime-suite | stock | 97.757237, 101.660731, 102.162196, 102.405437, 102.461563, 103.883330, 104.870186, 106.581707, 111.039162 | 111.039162 |
| rejection | compile | item56-direct | 157.798712, 163.093776, 172.487801, 178.443516, 180.826815, 187.590974, 189.372453, 191.188422, 218.015754 | 218.015754 |
| rejection | compile | item56-retained | 166.094334, 175.066112, 177.794089, 181.211339, 184.507918, 186.025692, 190.176013, 194.702257, 227.707489 | 227.707489 |
| rejection | compile | stock | 35.610334, 35.766154, 35.903728, 36.367130, 36.569709, 37.071239, 38.363278, 40.926034, 40.929712 | 40.929712 |
| recovery | compile | item56-direct | 228.224941, 243.001437, 247.920587, 249.978789, 254.584579, 255.223353, 257.732907, 314.327048, 351.223771 | 351.223771 |
| recovery | compile-plus-runtime-suite | item56-direct | 229.002198, 243.964086, 248.847838, 250.787465, 255.450637, 256.059658, 258.555569, 315.330573, 352.222896 | 352.222896 |
| recovery | compile | item56-retained | 356.648338, 361.388479, 376.348497, 391.500778, 397.430435, 400.896406, 400.934870, 409.985226, 700.816249 | 700.816249 |
| recovery | compile-plus-runtime-suite | item56-retained | 357.604608, 362.540048, 377.512674, 392.453065, 398.394685, 401.842888, 402.223652, 410.834456, 701.881932 | 701.881932 |
| recovery | compile | stock | 94.392678, 95.593739, 100.469462, 103.660451, 105.885487, 107.169458, 107.813513, 108.055462, 163.252467 | 163.252467 |
| recovery | compile-plus-runtime-suite | stock | 95.067398, 96.203394, 101.124481, 104.343478, 106.673489, 107.838027, 108.535485, 108.698381, 164.185171 | 164.185171 |

Full Rust/unsafe semantics, crates/metadata/rlib/Cargo, runtime-loader closure, supervised lifecycle and representative unchanged-project speed remain OPEN. All seven public release gates are held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC/host linker remain dependencies. No all-Rust, whole-toolchain memory safety or universal speed claim.
