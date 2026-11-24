# Native original-AST tuple fields — component48

The complete original tuple-field assertion SHA256 `d3cb87af961e7357cd2d2a34bf9882a93ee3a092c79b37f363e3a854484850f8` compiles, links and runs natively with exact stock runtime output. Copied47 refused it with L0809; its separate stock compile/runtime success is retained. No successful native47 speed baseline exists. Earlier complete struct/binary/parenthesis/long-assert/slice-array sources and protected Lust240e/Rust5b7b stay unchanged.

The owned formatter binds original TupleField indexes to exact canonical decimal tokens and validates the native parser's duplicate-span two-field float-token chains. Each field keeps its expression box and original parentheses. The first optimized suite had 23 passes and one four-index panic-layout mismatch. Its source, failed run, frozen first formatter and both payloads remain retained. The correction inherits the next outer break across End markers, including following punctuation in width resolution. Only derived panic literals change before full native checking, borrowing, generation and linking. No transformed AST survives requests, and no protected printer copy, overlay, application rewrite, rustc native fallback or metadata fiction is used. See the [design and proof limits](../../proofs/native-assert-text.md).

Corrected development seven formatter tests pass. Fresh actual gpt-6.1-sol HIGH/default frozen7 R2 approves the correction; R1 was interrupted and retired after the reproduced failure and supplies no approval. Corrected optimized direct24 and retained21 suites pass, followed by copied-component direct7, retained1 and host-input1 checks. Tests cover first/second indexes, comments, wrappers, calls, duplicate-span and longer chains, named/tuple combinations, order/once, borrowing, Drop, initializer unwind, const evaluation, index/type/move/borrow rejection, retained field/index/type/token/position/signature edits, rejection and recovery. All prior native gates remain active.

## Equivalent successful work

SAME successful optimized48 direct/retained and stock compile the same full generated mixed workload. Actual native/stock compile/runtime preflight passed BEFORE timing; its bytes match timed sample0 cold. Nine rotated samples per phase/engine, new retained process per sample, native disk caches disabled before Check, OS pages untouched. Compiler opt3/CGU1/ThinLTO and application opt0/debug0/unwind/checks-on stay fixed. Compiler-build time is separate. Standalone no-op is not Cargo freshness; compile-plus-runtime is not Cargo test. Shared-host activity limits attribution.

No comparison with rejecting47 or distinct earlier sources establishes a gain. Cold/edit/rejection regressions against stock remain visible. Bounded retained no-op improvement cannot establish full-project speed.

Median milliseconds, lower is better.

| Phase | Direct48 | Retained48 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 259.967 | 385.229 | 105.518 | 260.855 | 386.178 | 106.229 |
| repeat | 249.021 | 14.732 | 100.997 | 249.913 | 15.808 | 101.863 |
| noop | 257.343 | 13.763 | 101.370 | 258.312 | 14.739 | 102.016 |
| body-edit | 253.955 | 385.139 | 101.648 | 254.825 | 386.381 | 102.418 |
| signature-edit | 244.703 | 389.999 | 101.960 | 245.490 | 390.939 | 102.641 |
| type-edit | 250.517 | 385.706 | 104.314 | 251.378 | 386.630 | 104.938 |
| macro-edit | 258.470 | 410.941 | 106.809 | 259.291 | 411.846 | 107.712 |
| rejection | 172.467 | 182.378 | 36.685 | — | — | — |
| recovery | 256.678 | 394.476 | 104.950 | 257.689 | 395.489 | 105.669 |

Raw events: 757; SHA256 `a52ea5099ab5baeda621523c0f46c72790c977a7170e8f1c873ba715c0e166a2`. Summary51 SHA256 `34dff0e47870f0ce7cfab09390e57936388ec7b5afe6c38b53019891836d4f39`. Core `e9d16039cb0cef8818350cc0049edbae51e6351fb3fabcdf40f0ace5548fe605`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 376.829778736s` is separate from application performance. Independent raw verification checks every group, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Case | Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | --- | ---: |
| mixed | cold | compile | tuple48-direct | 211.784810, 212.751085, 214.503837, 241.332721, 259.967058, 266.542902, 286.270174, 322.852486, 337.753313 | 337.753313 |
| mixed | cold | compile-plus-runtime-suite | tuple48-direct | 212.682428, 213.539828, 215.424247, 242.182346, 260.854651, 267.418684, 287.237601, 323.863830, 338.709630 | 338.709630 |
| mixed | cold | compile | tuple48-retained | 339.633536, 345.585397, 366.159044, 383.708787, 385.229077, 439.801767, 469.927703, 512.212172, 538.610964 | 538.610964 |
| mixed | cold | compile-plus-runtime-suite | tuple48-retained | 340.313307, 346.463701, 366.995111, 384.634387, 386.178465, 440.995478, 470.901899, 513.295338, 539.736323 | 539.736323 |
| mixed | cold | compile | stock | 88.641628, 90.339024, 92.255562, 95.922317, 105.517543, 114.892625, 117.864080, 127.966896, 135.113955 | 135.113955 |
| mixed | cold | compile-plus-runtime-suite | stock | 89.246057, 91.099016, 92.837089, 96.641468, 106.228704, 115.816377, 118.610614, 128.796692, 136.067946 | 136.067946 |
| mixed | repeat | compile | tuple48-direct | 203.293602, 210.600850, 225.576496, 227.538313, 249.020955, 282.750797, 299.846610, 317.510160, 319.705510 | 319.705510 |
| mixed | repeat | compile-plus-runtime-suite | tuple48-direct | 204.029279, 211.314892, 226.357002, 228.415387, 249.912922, 283.757578, 300.864479, 318.472741, 320.747279 | 320.747279 |
| mixed | repeat | compile | tuple48-retained | 10.515628, 10.531287, 11.556608, 14.518207, 14.732362, 14.799681, 17.179515, 17.938838, 20.778802 | 20.778802 |
| mixed | repeat | compile-plus-runtime-suite | tuple48-retained | 11.248208, 11.329332, 12.321403, 15.490716, 15.808062, 15.946653, 17.995286, 18.834595, 21.829261 | 21.829261 |
| mixed | repeat | compile | stock | 91.504697, 93.383304, 94.124214, 94.155731, 100.996559, 123.086940, 125.501035, 125.556906, 130.862790 | 130.862790 |
| mixed | repeat | compile-plus-runtime-suite | stock | 92.300452, 94.121104, 94.774194, 94.837170, 101.863032, 123.911135, 126.180020, 126.353149, 131.790918 | 131.790918 |
| mixed | noop | compile | tuple48-direct | 197.590984, 211.574368, 237.008934, 255.124611, 257.343212, 277.185499, 279.079629, 286.065937, 382.131990 | 382.131990 |
| mixed | noop | compile-plus-runtime-suite | tuple48-direct | 198.384196, 212.297884, 237.820596, 256.066662, 258.311955, 278.118232, 280.029346, 286.917708, 383.220249 | 383.220249 |
| mixed | noop | compile | tuple48-retained | 10.354257, 11.244417, 11.646734, 13.671710, 13.763343, 14.716752, 15.619344, 19.147986, 19.435408 | 19.435408 |
| mixed | noop | compile-plus-runtime-suite | tuple48-retained | 11.164747, 12.024799, 12.428074, 14.688730, 14.739210, 15.618418, 16.749953, 20.009229, 20.496360 | 20.496360 |
| mixed | noop | compile | stock | 89.552390, 90.236679, 97.493678, 101.355268, 101.369548, 113.693123, 119.327156, 123.802467, 142.799647 | 142.799647 |
| mixed | noop | compile-plus-runtime-suite | stock | 90.230990, 90.966209, 98.140889, 102.008505, 102.016411, 114.435077, 120.029521, 124.559347, 143.775602 | 143.775602 |
| mixed | body-edit | compile | tuple48-direct | 196.524463, 212.425676, 216.455545, 237.205818, 253.954517, 262.980439, 277.556750, 291.038094, 343.182273 | 343.182273 |
| mixed | body-edit | compile-plus-runtime-suite | tuple48-direct | 197.310173, 213.095731, 217.269610, 237.919433, 254.824593, 263.829412, 278.351950, 291.752431, 344.354359 | 344.354359 |
| mixed | body-edit | compile | tuple48-retained | 314.817041, 340.790109, 353.983983, 369.505159, 385.138813, 395.196309, 445.036530, 459.766071, 500.672856 | 500.672856 |
| mixed | body-edit | compile-plus-runtime-suite | tuple48-retained | 315.552213, 341.488790, 354.971979, 370.214877, 386.380577, 396.193664, 446.200592, 460.932783, 501.911573 | 501.911573 |
| mixed | body-edit | compile | stock | 83.148276, 87.681840, 90.281662, 93.740404, 101.647589, 110.194497, 114.013366, 130.540285, 130.954564 | 130.954564 |
| mixed | body-edit | compile-plus-runtime-suite | stock | 83.706751, 88.263846, 90.965129, 94.441137, 102.417897, 111.030981, 114.841284, 131.158162, 131.852183 | 131.852183 |
| mixed | signature-edit | compile | tuple48-direct | 195.132670, 220.056359, 225.462519, 235.875879, 244.702806, 266.587653, 269.295122, 282.974024, 316.412250 | 316.412250 |
| mixed | signature-edit | compile-plus-runtime-suite | tuple48-direct | 195.766994, 220.749134, 226.257559, 236.662937, 245.489750, 267.480989, 270.100682, 283.890477, 317.280281 | 317.280281 |
| mixed | signature-edit | compile | tuple48-retained | 309.411699, 344.310437, 368.709102, 374.906266, 389.999161, 415.113837, 422.196648, 423.751693, 437.775262 | 437.775262 |
| mixed | signature-edit | compile-plus-runtime-suite | tuple48-retained | 310.113835, 345.110708, 369.632219, 375.816300, 390.939117, 415.899423, 423.280994, 424.635196, 438.826034 | 438.826034 |
| mixed | signature-edit | compile | stock | 89.059400, 91.319111, 98.705513, 101.211137, 101.960067, 103.327204, 103.682127, 111.757197, 117.467929 | 117.467929 |
| mixed | signature-edit | compile-plus-runtime-suite | stock | 89.605838, 92.042491, 99.387708, 101.876421, 102.640818, 104.077314, 104.390265, 112.354126, 118.295859 | 118.295859 |
| mixed | type-edit | compile | tuple48-direct | 194.124774, 221.868252, 239.647422, 250.483626, 250.516696, 259.082833, 266.062376, 274.096581, 293.692073 | 293.692073 |
| mixed | type-edit | compile-plus-runtime-suite | tuple48-direct | 194.857981, 222.781962, 240.429729, 251.353554, 251.378036, 260.026334, 267.067054, 274.962244, 294.643917 | 294.643917 |
| mixed | type-edit | compile | tuple48-retained | 326.055089, 370.644099, 374.955903, 379.616883, 385.706329, 397.065070, 423.291212, 434.088999, 452.668542 | 452.668542 |
| mixed | type-edit | compile-plus-runtime-suite | tuple48-retained | 326.854585, 371.499908, 376.130927, 380.494344, 386.629511, 398.022316, 424.209803, 435.016735, 453.658644 | 453.658644 |
| mixed | type-edit | compile | stock | 83.080574, 96.462071, 102.705362, 103.447040, 104.313651, 104.609163, 108.557488, 110.696188, 119.128177 | 119.128177 |
| mixed | type-edit | compile-plus-runtime-suite | stock | 83.635307, 97.107711, 103.486316, 104.228801, 104.938367, 105.328669, 109.394908, 111.361042, 119.924427 | 119.924427 |
| mixed | macro-edit | compile | tuple48-direct | 209.757833, 241.182696, 246.560924, 255.012991, 258.470015, 264.329495, 270.513054, 279.832293, 375.230535 | 375.230535 |
| mixed | macro-edit | compile-plus-runtime-suite | tuple48-direct | 210.523118, 241.987209, 247.343422, 255.986071, 259.291249, 265.263729, 271.355191, 280.740668, 376.250866 | 376.250866 |
| mixed | macro-edit | compile | tuple48-retained | 369.642347, 387.323510, 389.888773, 400.453414, 410.940680, 414.334957, 430.938554, 433.864968, 521.343540 | 521.343540 |
| mixed | macro-edit | compile-plus-runtime-suite | tuple48-retained | 370.295577, 388.355430, 390.681621, 401.497898, 411.845507, 415.228676, 431.951748, 434.744663, 522.693109 | 522.693109 |
| mixed | macro-edit | compile | stock | 90.216103, 100.604270, 105.156766, 106.207808, 106.809093, 111.092600, 114.597427, 115.802833, 136.644843 | 136.644843 |
| mixed | macro-edit | compile-plus-runtime-suite | stock | 90.748549, 101.320624, 105.822720, 106.852572, 107.712136, 111.909043, 115.239406, 116.501892, 137.524005 | 137.524005 |
| mixed | rejection | compile | tuple48-direct | 157.115011, 161.026776, 162.353427, 165.070767, 172.467247, 175.465637, 180.773517, 208.732726, 250.386294 | 250.386294 |
| mixed | rejection | compile | tuple48-retained | 164.266635, 168.216230, 175.503083, 179.146741, 182.377692, 184.394469, 188.560000, 250.233840, 269.850607 | 269.850607 |
| mixed | rejection | compile | stock | 32.733062, 35.041791, 35.448467, 35.535463, 36.685205, 38.125491, 39.351193, 42.460761, 47.273187 | 47.273187 |
| mixed | recovery | compile | tuple48-direct | 220.228505, 222.214657, 228.207058, 246.549122, 256.678218, 260.951486, 267.739783, 268.233934, 340.668265 | 340.668265 |
| mixed | recovery | compile-plus-runtime-suite | tuple48-direct | 221.105072, 222.935028, 228.996749, 247.404361, 257.688637, 261.842671, 268.774071, 270.688433, 341.688566 | 341.688566 |
| mixed | recovery | compile | tuple48-retained | 326.149237, 345.092799, 351.168459, 380.975717, 394.476131, 412.702091, 468.566080, 548.439195, 563.331867 | 563.331867 |
| mixed | recovery | compile-plus-runtime-suite | tuple48-retained | 326.998310, 346.181071, 352.105463, 381.894494, 395.488870, 414.016152, 469.653546, 549.648332, 564.449602 | 564.449602 |
| mixed | recovery | compile | stock | 91.628958, 91.950643, 98.868126, 100.520803, 104.950033, 106.773786, 113.404828, 119.395057, 129.934973 | 129.934973 |
| mixed | recovery | compile-plus-runtime-suite | stock | 92.245363, 92.594606, 99.528857, 101.216526, 105.668830, 107.494517, 114.175409, 120.253573, 130.784969 | 130.784969 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
