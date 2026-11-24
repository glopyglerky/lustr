# Native shared Path expansion — component40

The exact original delayed `&Path -> PathBuf` source, SHA256 `e9fb10c22b6d7475be87b73ab21007c567aeed45a3cc7f6615fb73d6e7de8976`, now compiles, links and runs natively with stdout `borrowed-native-path`. Copied39's first L0640 rejection and stock compile/runtime witnesses remain preserved. Source bytes and protected upstream compiler trees remain unchanged.

The owned classifier admits only a checked shared reference to the nongeneric canonical platform Path ADT. The resolved canonical symbol becomes a shared reference in the qualified Into call; the original receiver expression is preserved. Existing ambiguity/custom-Extend guards, arena reindexing, at most two checked elaboration rounds and complete native collect/check/borrowck/codegen/link remain. Mixed aliases, arrays, closures, mutation after copying and Drop/unwind match stock. Eight invalid owner-move/live-borrow mutation/moved-result/immutable-vector/competing-method/custom-Extend/shadow/mutable-reference guards reject. Retained source and target edits, invalid transitions and recovery match cold native and stock.

Fresh optimized direct11 passes. All 13 retained cases pass across the original run's 11 successes and a focused rerun of the two host-linker cases after making the private linker-script overlay writable. Their original permission-denied failures remain recorded; no other successful case was replayed. Copied40 borrowed direct2/retained1 plus host-linker1 checks are separate receipts. Unchanged product34 and published28 remain historical artifacts; these component passes do not qualify a new product. Root gpt-6.1-sol xhigh/default authored the expansion and Rust helpers through T3 Code -> Codex. A fresh frozen seven-file gpt-6.1-sol high/default static review APPROVE/no findings was retired with actual terminal/null-active/zero-pending/no-background evidence; review ran no tests. First pre-entry namespace omission and mistyped retained harness path, plus read-only private overlay setup failures, are retained as failed invocations, corrected before acceptance; no compiler failure was turned into success.

## Successful equivalent workload

Nine samples per phase/engine on the shared Linux GNU x86_64 NAS, CPUs10-13/jobs2. Candidate40 direct, SAME candidate40 retained and pinned stock rustc rotate each phase/sample. Compiler opt3/CGU1/ThinLTO/line tables is unchanged; application opt0/debug0/unwind/overflow and debug checks on. Each sample uses a fresh retained process. Native disk caches are disabled before Check; OS pages are untouched. Commands, compiler/runtime/probe/harness identities, 81 exact source snapshots, statuses and executable output are retained. Accepted artifacts must be ELF with matching runtime/status/stdout/stderr, and invalid edits must reject status1.

This is a language expansion with measured costs, not a speed gain against rejecting39 or a comparison to a distinct earlier source. Cold/edits and rejection still trail stock; scoped unchanged-artifact proof costs less than repeated direct compilation. Stock recompiles a standalone file each phase; its repeat/noop is not Cargo freshness. Compile-plus-runtime measures actual executed programs, not Cargo test. Full-project speed remains OPEN.

Median milliseconds, lower is better.

| Phase | Direct40 | Retained40 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 268.148 | 400.555 | 100.977 | 269.009 | 401.536 | 101.698 |
| repeat | 270.690 | 14.514 | 103.166 | 271.605 | 15.619 | 103.917 |
| noop | 247.171 | 12.721 | 100.078 | 248.151 | 13.712 | 100.867 |
| body-edit | 247.658 | 404.456 | 100.121 | 248.626 | 405.402 | 100.899 |
| signature-edit | 260.426 | 405.320 | 99.557 | 261.300 | 406.199 | 100.260 |
| type-edit | 257.882 | 418.914 | 100.513 | 258.811 | 419.883 | 101.275 |
| macro-edit | 290.921 | 431.599 | 103.658 | 291.781 | 432.568 | 104.431 |
| rejection | 198.054 | 188.479 | 35.127 | — | — | — |
| recovery | 258.097 | 400.318 | 101.559 | 258.962 | 401.221 | 102.285 |

Raw757 SHA256 `7b43c3d4b7eb638ed2ff76c2027c8a5acf17fb393985240fb74b6008d54461f1`; summary51 SHA256 `44e5048d6260af5da849731e8807520b1a1ba2c90a8ecffa50a0c43828b03323`. Core `2d77b24333bceae131fb4e7b28e2076a4f367f650b4fb91bf4bba40846b81dde`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Optimized compiler BUILD receipt `exit status: 0; elapsed 512.954324241s` is separate from application performance.

## Complete samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | borrowed40-direct | 226.160865, 242.499231, 248.148575, 252.597552, 268.148312, 269.357137, 280.566111, 306.965133, 338.922235 | 338.922235 |
| cold | compile-plus-runtime-suite | borrowed40-direct | 226.949555, 243.374916, 249.024065, 253.458863, 269.009320, 270.305545, 281.589395, 307.962460, 340.005348 | 340.005348 |
| cold | compile | borrowed40-retained | 369.661301, 370.836936, 390.946210, 391.619728, 400.555142, 401.586507, 438.555710, 474.888100, 551.401441 | 551.401441 |
| cold | compile-plus-runtime-suite | borrowed40-retained | 370.476692, 371.713161, 392.305488, 392.472070, 401.536420, 402.473266, 439.652203, 475.850710, 552.440300 | 552.440300 |
| cold | compile | stock | 96.169633, 96.222435, 96.331493, 99.264243, 100.977174, 102.771472, 102.788220, 110.937305, 111.585734 | 111.585734 |
| cold | compile-plus-runtime-suite | stock | 96.847013, 96.985326, 97.045235, 99.991151, 101.698390, 103.493279, 103.527939, 111.724423, 112.286113 | 112.286113 |
| repeat | compile | borrowed40-direct | 215.586606, 234.902132, 241.207124, 244.183284, 270.689576, 277.451040, 280.505672, 281.854282, 300.452832 | 300.452832 |
| repeat | compile-plus-runtime-suite | borrowed40-direct | 216.364613, 235.775771, 242.092108, 245.096771, 271.604969, 278.450079, 281.468381, 282.932671, 301.381301 | 301.381301 |
| repeat | compile | borrowed40-retained | 12.199433, 12.435318, 12.481806, 13.117868, 14.514311, 14.709079, 16.572253, 18.072403, 19.114159 | 19.114159 |
| repeat | compile-plus-runtime-suite | borrowed40-retained | 13.055188, 13.309416, 13.419435, 14.039590, 15.619005, 16.117909, 17.641118, 19.073940, 20.122923 | 20.122923 |
| repeat | compile | stock | 87.648985, 91.750223, 95.757337, 98.899099, 103.166097, 103.351100, 106.221302, 106.826684, 107.814343 | 107.814343 |
| repeat | compile-plus-runtime-suite | stock | 88.282108, 92.477261, 96.362452, 99.706188, 103.917431, 104.087173, 106.982703, 107.734579, 108.591888 | 108.591888 |
| noop | compile | borrowed40-direct | 219.627887, 232.465640, 239.365656, 245.643493, 247.170651, 254.064055, 255.369318, 288.262062, 303.892942 | 303.892942 |
| noop | compile-plus-runtime-suite | borrowed40-direct | 220.568614, 233.426450, 240.365129, 246.502103, 248.150842, 254.863749, 256.252335, 289.221168, 304.678946 | 304.678946 |
| noop | compile | borrowed40-retained | 11.982011, 12.140580, 12.293362, 12.452143, 12.721194, 13.572714, 18.677890, 19.841792, 20.074559 | 20.074559 |
| noop | compile-plus-runtime-suite | borrowed40-retained | 12.928701, 13.026350, 13.186769, 13.320191, 13.711994, 14.544786, 19.503158, 20.730668, 21.099894 | 21.099894 |
| noop | compile | stock | 91.505743, 91.786939, 94.069930, 99.242571, 100.078240, 101.232683, 102.338340, 104.291449, 112.001648 | 112.001648 |
| noop | compile-plus-runtime-suite | stock | 92.141936, 92.520263, 94.770651, 100.123225, 100.867098, 101.962435, 103.007144, 107.760725, 112.822497 | 112.822497 |
| body-edit | compile | borrowed40-direct | 227.307014, 228.263105, 238.226975, 246.335760, 247.658223, 254.377199, 264.208001, 289.142684, 348.534743 | 348.534743 |
| body-edit | compile-plus-runtime-suite | borrowed40-direct | 227.946987, 229.155032, 239.229443, 247.240582, 248.625843, 255.380028, 265.129031, 290.030045, 349.595295 | 349.595295 |
| body-edit | compile | borrowed40-retained | 353.406432, 387.672339, 400.539081, 402.669806, 404.456259, 410.255612, 414.050883, 476.844124, 487.949291 | 487.949291 |
| body-edit | compile-plus-runtime-suite | borrowed40-retained | 354.456084, 388.589715, 401.625589, 403.641236, 405.402240, 411.383666, 415.160910, 477.987809, 489.025346 | 489.025346 |
| body-edit | compile | stock | 89.301184, 89.378133, 93.766596, 98.637307, 100.121047, 102.834540, 104.748908, 105.218071, 120.357100 | 120.357100 |
| body-edit | compile-plus-runtime-suite | stock | 89.927676, 90.073869, 94.443941, 99.397952, 100.899186, 103.746351, 105.475215, 105.929005, 121.127983 | 121.127983 |
| signature-edit | compile | borrowed40-direct | 216.376976, 243.531015, 249.794939, 250.328100, 260.425651, 262.514936, 265.391851, 332.400341, 354.101056 | 354.101056 |
| signature-edit | compile-plus-runtime-suite | borrowed40-direct | 217.251572, 244.344482, 250.679119, 251.187000, 261.299886, 263.623223, 266.410038, 333.444425, 355.190609 | 355.190609 |
| signature-edit | compile | borrowed40-retained | 372.687274, 372.865328, 376.501270, 390.011483, 405.320092, 461.012112, 511.167396, 546.057580, 555.202251 | 555.202251 |
| signature-edit | compile-plus-runtime-suite | borrowed40-retained | 373.959386, 374.038338, 377.690863, 390.910212, 406.199003, 461.953613, 512.028480, 547.292541, 556.799500 | 556.799500 |
| signature-edit | compile | stock | 88.206612, 92.395271, 94.506690, 98.806206, 99.556644, 106.138535, 109.490053, 123.281841, 132.518215 | 132.518215 |
| signature-edit | compile-plus-runtime-suite | stock | 88.914123, 93.083505, 95.246999, 99.524789, 100.259687, 106.876238, 110.245724, 124.286591, 133.418013 | 133.418013 |
| type-edit | compile | borrowed40-direct | 222.097096, 228.226791, 244.528641, 245.810383, 257.881895, 269.642792, 290.499625, 358.476605, 509.373735 | 509.373735 |
| type-edit | compile-plus-runtime-suite | borrowed40-direct | 222.874918, 228.974210, 245.398666, 246.694178, 258.810823, 270.592067, 291.591137, 359.631508, 510.847280 | 510.847280 |
| type-edit | compile | borrowed40-retained | 366.386946, 380.988877, 390.917286, 399.680666, 418.913780, 444.198033, 452.005634, 548.044073, 597.820001 | 597.820001 |
| type-edit | compile-plus-runtime-suite | borrowed40-retained | 367.358018, 382.035423, 391.755643, 400.755576, 419.882684, 445.353907, 453.015402, 549.646822, 599.102987 | 599.102987 |
| type-edit | compile | stock | 90.754310, 95.991714, 97.917906, 99.031520, 100.512825, 104.623724, 113.918031, 119.932668, 128.337640 | 128.337640 |
| type-edit | compile-plus-runtime-suite | stock | 91.350442, 96.743723, 98.661260, 99.876798, 101.274645, 105.350401, 114.740304, 120.744167, 129.178718 | 129.178718 |
| macro-edit | compile | borrowed40-direct | 238.051166, 257.292000, 257.753434, 260.417084, 290.921277, 300.423543, 332.315831, 352.558357, 506.635285 | 506.635285 |
| macro-edit | compile-plus-runtime-suite | borrowed40-direct | 238.959029, 258.090410, 258.660297, 261.350488, 291.780770, 301.461486, 333.381736, 353.692004, 507.742616 | 507.742616 |
| macro-edit | compile | borrowed40-retained | 379.676154, 396.595271, 403.401143, 414.208029, 431.599184, 443.273986, 499.739071, 555.912330, 647.592108 | 647.592108 |
| macro-edit | compile-plus-runtime-suite | borrowed40-retained | 380.682303, 397.813046, 404.271911, 415.264922, 432.567676, 444.290195, 500.687335, 556.996969, 648.680474 | 648.680474 |
| macro-edit | compile | stock | 92.737994, 93.026238, 94.181960, 101.275894, 103.657879, 105.571114, 113.589537, 118.153561, 161.309928 | 161.309928 |
| macro-edit | compile-plus-runtime-suite | stock | 93.435752, 93.725239, 94.884395, 102.034087, 104.431093, 106.337107, 114.490545, 119.098641, 162.283488 | 162.283488 |
| rejection | compile | borrowed40-direct | 163.286072, 168.021840, 169.223930, 185.061700, 198.054291, 214.479524, 219.887356, 257.468386, 264.499337 | 264.499337 |
| rejection | compile | borrowed40-retained | 162.347223, 176.180635, 179.187197, 180.397410, 188.478698, 221.566921, 242.469272, 261.423466, 274.637719 | 274.637719 |
| rejection | compile | stock | 32.870022, 33.861539, 33.943740, 34.294317, 35.127154, 38.559865, 38.883211, 39.783287, 41.888922 | 41.888922 |
| recovery | compile | borrowed40-direct | 232.798393, 233.536529, 237.199384, 246.813790, 258.097316, 266.296681, 298.947248, 319.090631, 321.691496 | 321.691496 |
| recovery | compile-plus-runtime-suite | borrowed40-direct | 233.564353, 234.548056, 238.058196, 247.737091, 258.961898, 267.142269, 299.871198, 320.134182, 322.846240 | 322.846240 |
| recovery | compile | borrowed40-retained | 373.547451, 379.696733, 386.242581, 386.464877, 400.317856, 431.919944, 457.280055, 459.323592, 472.312066 | 472.312066 |
| recovery | compile-plus-runtime-suite | borrowed40-retained | 374.468852, 380.765627, 387.179765, 387.623655, 401.220998, 432.846455, 458.374404, 460.328819, 473.298386 | 473.298386 |
| recovery | compile | stock | 87.595312, 95.950883, 97.115721, 98.646639, 101.558703, 101.580438, 102.734299, 106.891585, 111.750193 | 111.750193 |
| recovery | compile-plus-runtime-suite | stock | 88.318125, 96.567130, 97.797244, 99.446392, 102.285279, 102.330818, 103.593448, 107.658388, 112.544457 | 112.544457 |

Experimental component40 remains outside product34/published28. General native Rust/crates/metadata/rlib/Cargo, runtime-loader closure, stable-file-during-request assumptions, supervised lifecycle and full-project speed remain OPEN. All seven public-release gates remain held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC loader dependencies remain; no all-Rust or whole-toolchain memory-safety claim.
