# Native original-AST array literals — component50

The complete original array-literal assertion SHA256 `220116ea2640bb8d49562f37ed7b879313d0bdda53e62ac8c4895d76b10fae33` compiles, links and runs natively with exact stock runtime output. Copied49 refused it with L0809; separate stock compilation/runtime succeeded. The rejected source supplies no successful native49 speed baseline. Earlier complete originals and protected Lust240e/Rust5b7b remain unchanged.

The owned formatter binds original ordered array elements and commas to exact source spans and brackets, retaining explicit wrappers and the pinned array/list indentation boxes. Empty arrays and optional trailing commas follow the same list rule. Only derived panic literals change before complete native rechecking, borrowing, generation and linking. No retained transformed AST, protected printer copy, overlay, source rewrite, rustc native fallback or metadata fiction. See the [design and proof limits](../../proofs/native-assert-text.md).

Optimized direct26 and retained23 suites pass, followed by copied direct10, retained1 and host-input1 checks. A fresh actual gpt-6.1-sol HIGH/default frozen7 review approved the source change. Cases cover empty/single/multiple and nested arrays, tuples, calls, comments, parentheses, long elements/lists, order/once, borrowing, moves, Drop, unwind, const evaluation, invalid type/move/borrow, retained element/type/token/position/signature edits, rejection and recovery. Array-repeat formatting remains a separate gap.

## Equivalent successful work

SAME successful optimized50 direct/retained and stock compile the identical complete generated mixed workload. Full native/stock compile/runtime preflight passed BEFORE timing; bytes match timed sample0 cold. Nine rotated samples per phase/engine, fresh retained process per sample, native disk caches disabled before Check, OS pages untouched. Compiler opt3/CGU1/ThinLTO and application opt0/debug0/unwind/checks-on stay fixed. Compiler-build time is separate. Standalone no-op is not Cargo freshness; compile-plus-runtime is not Cargo test. Shared-host activity limits attribution.

No gain follows from comparison with rejecting49 or earlier distinct sources. Cold/edit/rejection stock regressions remain visible; a bounded no-op win does not close full-project speed.

Median milliseconds, lower is better.

| Phase | Direct50 | Retained50 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 334.761 | 507.989 | 124.722 | 335.712 | 508.929 | 125.530 |
| repeat | 327.545 | 15.500 | 115.460 | 328.650 | 16.512 | 116.203 |
| noop | 306.886 | 13.225 | 120.491 | 308.194 | 14.210 | 121.301 |
| body-edit | 312.606 | 493.690 | 118.549 | 313.611 | 494.570 | 119.300 |
| signature-edit | 293.883 | 467.378 | 114.911 | 294.867 | 468.550 | 115.672 |
| type-edit | 286.373 | 458.539 | 111.792 | 287.227 | 459.610 | 112.512 |
| macro-edit | 303.552 | 478.982 | 111.435 | 304.522 | 479.894 | 112.197 |
| rejection | 199.705 | 204.222 | 39.507 | — | — | — |
| recovery | 327.367 | 466.931 | 121.505 | 328.409 | 467.999 | 122.299 |

Raw events: 757; SHA256 `104e5515a86513ebd2d2f19515b95849bf5a32b5dc91c5a0c53c510ab2ba70cb`. Summary51 SHA256 `c200aadfb0d5e50eb5035ea2e420cbe911d4d43e7568d1ea31f119595559c1b8`. Core `c51d804915f77ba26f2930bb2642c973ec8f26e0c4c23e150d479ed261f0b8f2`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 409.367408277s` is separate from application performance. Independent raw verification checks every group, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Case | Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | --- | ---: |
| mixed | cold | compile | array50-direct | 269.032965, 272.130168, 282.025745, 312.513759, 334.760725, 341.016570, 352.395957, 358.377682, 418.209077 | 418.209077 |
| mixed | cold | compile-plus-runtime-suite | array50-direct | 269.921548, 272.975439, 282.913378, 313.531856, 335.712304, 341.982041, 353.503177, 359.393406, 419.357317 | 419.357317 |
| mixed | cold | compile | array50-retained | 414.859227, 431.081949, 457.265508, 501.765349, 507.988677, 526.207859, 554.625574, 578.192520, 605.938795 | 605.938795 |
| mixed | cold | compile-plus-runtime-suite | array50-retained | 415.851638, 431.987550, 458.265019, 502.764403, 508.928810, 527.207020, 555.679198, 579.485295, 607.079252 | 607.079252 |
| mixed | cold | compile | stock | 107.368949, 110.237498, 110.724927, 119.009920, 124.721733, 127.275578, 127.709832, 155.944051, 159.213180 | 159.213180 |
| mixed | cold | compile-plus-runtime-suite | stock | 108.059783, 110.968791, 111.428343, 119.851014, 125.529761, 128.084123, 128.597303, 156.796668, 160.117938 | 160.117938 |
| mixed | repeat | compile | array50-direct | 239.519772, 273.213754, 295.112862, 302.886144, 327.545092, 327.565211, 339.677737, 347.161455, 369.150908 | 369.150908 |
| mixed | repeat | compile-plus-runtime-suite | array50-direct | 240.345050, 274.270188, 295.994322, 303.871288, 328.649782, 328.672054, 340.645242, 348.440227, 370.313236 | 370.313236 |
| mixed | repeat | compile | array50-retained | 12.027833, 12.425442, 14.210875, 15.239720, 15.500082, 15.820830, 18.570996, 18.676111, 24.044319 | 24.044319 |
| mixed | repeat | compile-plus-runtime-suite | array50-retained | 12.973653, 13.381556, 15.149188, 16.411494, 16.512086, 17.006485, 19.547724, 19.595630, 25.258028 | 25.258028 |
| mixed | repeat | compile | stock | 96.207038, 101.106798, 109.392960, 113.623372, 115.460331, 121.380044, 122.385779, 130.292042, 138.095902 | 138.095902 |
| mixed | repeat | compile-plus-runtime-suite | stock | 96.828537, 101.767085, 110.154102, 114.319645, 116.203250, 122.128802, 123.224217, 131.257695, 138.978777 | 138.978777 |
| mixed | noop | compile | array50-direct | 252.194450, 287.926708, 290.406911, 296.095377, 306.885764, 330.247336, 330.725421, 340.550583, 363.580715 | 363.580715 |
| mixed | noop | compile-plus-runtime-suite | array50-direct | 253.017474, 288.903801, 291.308751, 296.955253, 308.193711, 331.198548, 331.730066, 341.524330, 364.643598 | 364.643598 |
| mixed | noop | compile | array50-retained | 11.608927, 12.222594, 13.124161, 13.165286, 13.225277, 20.984323, 21.348531, 22.776757, 23.426732 | 23.426732 |
| mixed | noop | compile-plus-runtime-suite | array50-retained | 12.429059, 13.206733, 14.167128, 14.184009, 14.209858, 22.139687, 22.271809, 23.842702, 24.485362 | 24.485362 |
| mixed | noop | compile | stock | 107.281368, 108.739158, 116.580988, 119.869678, 120.491489, 121.651086, 121.874529, 127.435534, 131.176059 | 131.176059 |
| mixed | noop | compile-plus-runtime-suite | stock | 108.038454, 109.644399, 117.396446, 120.643960, 121.301343, 122.451265, 122.673488, 128.085739, 132.152859 | 132.152859 |
| mixed | body-edit | compile | array50-direct | 275.936367, 280.807889, 305.607730, 309.024468, 312.606255, 317.691058, 318.531716, 345.536674, 382.067881 | 382.067881 |
| mixed | body-edit | compile-plus-runtime-suite | array50-direct | 276.850712, 281.831104, 306.502236, 310.030380, 313.610861, 318.617536, 319.403075, 346.586510, 383.087828 | 383.087828 |
| mixed | body-edit | compile | array50-retained | 411.551428, 447.652249, 481.509305, 484.861336, 493.690149, 532.916723, 544.157984, 552.213992, 567.534925 | 567.534925 |
| mixed | body-edit | compile-plus-runtime-suite | array50-retained | 412.902696, 448.563603, 482.454299, 486.110152, 494.570046, 533.980944, 545.167669, 553.474230, 568.672780 | 568.672780 |
| mixed | body-edit | compile | stock | 109.640047, 109.979689, 112.105870, 117.295202, 118.549272, 121.371840, 125.417963, 129.743929, 140.105098 | 140.105098 |
| mixed | body-edit | compile-plus-runtime-suite | stock | 110.315294, 110.935590, 112.858658, 118.039397, 119.300069, 121.946987, 126.268257, 130.605995, 141.001672 | 141.001672 |
| mixed | signature-edit | compile | array50-direct | 271.881378, 273.349316, 274.230623, 277.299523, 293.883386, 295.506090, 343.058571, 347.727642, 411.203101 | 411.203101 |
| mixed | signature-edit | compile-plus-runtime-suite | array50-direct | 272.844478, 274.307310, 275.242817, 278.165070, 294.866861, 296.835336, 343.995386, 348.839725, 412.170883 | 412.170883 |
| mixed | signature-edit | compile | array50-retained | 406.373030, 414.580605, 428.606547, 451.300283, 467.378328, 476.970553, 504.139008, 518.228372, 563.486099 | 563.486099 |
| mixed | signature-edit | compile-plus-runtime-suite | array50-retained | 407.259616, 415.347901, 429.573671, 452.151195, 468.549792, 478.011337, 505.185232, 519.108076, 564.634907 | 564.634907 |
| mixed | signature-edit | compile | stock | 101.452601, 109.022499, 110.665264, 113.580746, 114.911218, 115.885883, 120.716004, 121.711763, 166.177680 | 166.177680 |
| mixed | signature-edit | compile-plus-runtime-suite | stock | 102.143361, 109.739154, 111.440255, 114.329483, 115.671930, 116.631792, 121.483556, 122.625120, 167.047491 | 167.047491 |
| mixed | type-edit | compile | array50-direct | 243.913135, 246.436828, 264.343245, 280.176535, 286.372782, 321.427627, 341.780353, 360.644349, 418.723304 | 418.723304 |
| mixed | type-edit | compile-plus-runtime-suite | array50-direct | 244.699227, 247.411414, 265.180311, 281.035129, 287.227363, 322.385890, 342.932766, 361.661537, 419.811589 | 419.811589 |
| mixed | type-edit | compile | array50-retained | 362.216996, 372.999790, 400.680600, 412.423060, 458.539156, 494.516871, 566.146174, 614.177597, 618.365864 | 618.365864 |
| mixed | type-edit | compile-plus-runtime-suite | array50-retained | 363.022329, 373.907792, 401.761578, 413.804497, 459.610219, 495.639145, 567.550220, 615.291316, 619.420904 | 619.420904 |
| mixed | type-edit | compile | stock | 94.607468, 94.771940, 104.195780, 107.634640, 111.791863, 126.252312, 126.643369, 130.616586, 142.574412 | 142.574412 |
| mixed | type-edit | compile-plus-runtime-suite | stock | 95.228426, 95.399325, 104.944880, 108.459418, 112.512082, 127.054037, 127.549177, 131.496265, 143.952280 | 143.952280 |
| mixed | macro-edit | compile | array50-direct | 259.016400, 274.592641, 280.027651, 282.318672, 303.552109, 312.646541, 355.164735, 398.022725, 530.619546 | 530.619546 |
| mixed | macro-edit | compile-plus-runtime-suite | array50-direct | 260.054555, 275.561876, 280.981890, 283.340683, 304.522437, 313.814306, 356.209399, 399.389950, 531.695608 | 531.695608 |
| mixed | macro-edit | compile | array50-retained | 405.335535, 420.839790, 426.598739, 454.826073, 478.981526, 496.252807, 543.334381, 557.721994, 564.664996 | 564.664996 |
| mixed | macro-edit | compile-plus-runtime-suite | array50-retained | 406.260456, 421.730333, 427.504934, 455.972800, 479.894204, 497.371433, 544.445897, 558.843340, 565.661825 | 565.661825 |
| mixed | macro-edit | compile | stock | 103.870159, 104.841418, 108.204086, 110.748025, 111.434505, 112.533968, 128.938752, 137.522173, 147.451097 | 147.451097 |
| mixed | macro-edit | compile-plus-runtime-suite | stock | 104.665855, 105.489996, 108.915622, 111.578553, 112.197283, 113.279549, 129.808222, 138.449708, 148.438955 | 148.438955 |
| mixed | rejection | compile | array50-direct | 178.717860, 180.461480, 188.569270, 195.310398, 199.704680, 213.442036, 275.454112, 284.744966, 333.243854 | 333.243854 |
| mixed | rejection | compile | array50-retained | 186.077013, 190.821206, 191.760073, 193.873611, 204.222008, 209.620872, 251.670727, 268.429508, 326.688936 | 326.688936 |
| mixed | rejection | compile | stock | 37.731037, 37.846493, 38.573487, 38.678186, 39.506913, 41.011432, 45.049902, 46.638328, 48.256283 | 48.256283 |
| mixed | recovery | compile | array50-direct | 254.467242, 277.721569, 303.143831, 314.996567, 327.367237, 329.736224, 381.247134, 392.572486, 436.106208 | 436.106208 |
| mixed | recovery | compile-plus-runtime-suite | array50-direct | 255.336574, 278.570138, 304.205857, 316.005930, 328.409320, 330.717530, 382.344335, 393.568349, 437.170700 | 437.170700 |
| mixed | recovery | compile | array50-retained | 425.331371, 447.671641, 456.570025, 458.566547, 466.930889, 516.205882, 526.362413, 582.187842, 611.290208 | 611.290208 |
| mixed | recovery | compile-plus-runtime-suite | array50-retained | 426.282737, 448.603007, 457.471615, 459.713379, 467.998857, 517.707161, 527.435462, 583.228864, 612.797921 | 612.797921 |
| mixed | recovery | compile | stock | 110.659909, 115.884552, 117.881783, 119.146078, 121.504940, 123.533305, 130.397605, 144.310221, 156.361460 | 156.361460 |
| mixed | recovery | compile-plus-runtime-suite | stock | 111.373014, 116.988607, 118.630018, 119.933392, 122.299020, 124.316145, 131.242805, 145.173878, 157.201439 | 157.201439 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
