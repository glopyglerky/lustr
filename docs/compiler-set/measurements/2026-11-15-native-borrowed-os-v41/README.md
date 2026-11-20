# Native shared OsStr expansion — component41

The exact original delayed `&OsStr -> PathBuf` source, SHA256 `77579ca4eef72e39a3e1ace8374f776e29afc708d62935d38d3a8366915be09e`, compiles, links and runs natively with stdout `borrowed-native-os`. Copied40's first L0640 rejection and stock compile/runtime witnesses remain preserved. The original source bytes and protected upstream compiler trees remain unchanged.

The owned classifier admits a checked shared reference to the nongeneric canonical platform OsStr ADT. The canonical symbol becomes a shared reference in the qualified Into call, preserving the original receiver expression. Existing ambiguity/custom-Extend guards, arena reindexing, at most two checked elaboration rounds and complete native collect/check/borrowck/codegen/link remain. Mixed aliases, arrays, closures, prior Path and owned conversions, mutation after copying, Linux non-Unicode bytes and Drop/unwind match stock. Nine invalid live-borrow owner moves/mutation, expired borrow, moved result, immutable vector, competing method, custom Extend, shadow and mutable-reference guards reject. Retained source/target spelling/value edits, rejection and recovery match cold native and stock.

Fresh optimized direct13 and retained14 pass, excluding the three product tests because product34 remains unchanged. Copied41 direct2/retained1/host-linker1 checks are separate receipts. The first dev invocation failed overall: nine invalid guards and the exact original passed, while the new mixed fixture exposed the separate `assert_eq!` slice-to-array comparison L0110 gap. That failing mixed source and first diagnostics are preserved; its new non-Unicode comparison fixture was changed to slice-to-slice comparisons and the focused direct test passed. No compiler failure was reclassified. Root gpt-6.1-sol xhigh/default authored implementation and Rust helpers through T3 Code -> Codex. Fresh frozen seven-file gpt-6.1-sol high/default static review APPROVE/no blocking findings was task_cancel-retired with actual completed/null-active/zero-pending/no-background receipt. Review ran no tests.

## Successful equivalent workload

Nine samples per phase/engine on the shared Linux GNU x86_64 NAS, CPUs10-13/jobs2. Candidate41 direct, SAME candidate41 retained and pinned stock rustc rotate each phase/sample. Compiler opt3/CGU1/ThinLTO/line tables is unchanged; application opt0/debug0/unwind/overflow and debug checks on. Each sample uses a fresh retained process. Native disk caches are disabled before Check; OS pages are untouched. Commands, compiler/runtime/probe/harness identities, 81 exact source snapshots, statuses and executable output are retained. Accepted artifacts must be ELF with matching runtime/status/stdout/stderr; invalid edits reject status1.

This is a language expansion with measured costs, not a speed gain against rejecting40 or a comparison to a distinct earlier source. Cold/edit/rejection results are reported separately from unchanged-artifact proof. Stock recompiles a standalone file each phase; repeat/noop does not measure Cargo freshness. Compile-plus-runtime measures actual executed programs, not Cargo test. Full-project speed remains OPEN.

Median milliseconds, lower is better.

| Phase | Direct41 | Retained41 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 259.429 | 398.482 | 98.400 | 260.410 | 399.318 | 99.210 |
| repeat | 245.003 | 13.027 | 99.003 | 245.806 | 13.849 | 99.729 |
| noop | 248.422 | 13.778 | 97.701 | 249.340 | 14.749 | 98.521 |
| body-edit | 244.920 | 418.686 | 96.457 | 245.746 | 419.693 | 97.138 |
| signature-edit | 256.766 | 390.166 | 97.995 | 257.683 | 391.122 | 98.789 |
| type-edit | 247.939 | 403.211 | 96.965 | 248.766 | 404.108 | 97.723 |
| macro-edit | 251.329 | 397.176 | 98.909 | 252.277 | 398.078 | 99.640 |
| rejection | 174.004 | 180.915 | 33.922 | — | — | — |
| recovery | 251.360 | 395.535 | 99.307 | 252.183 | 396.711 | 100.121 |

Raw757 SHA256 `7383752c35e0f71c54e44bfc45b5537767fa75628311cf6a294dc96976dc4628`; summary51 SHA256 `20e12b44115ce6f1ed432db7baa16d43adc6ef30adaf6612859340f629890ab1`. Core `4bcd0c65ada9b95bf0cd045effffce9f122628ce142f59d163b83ca282a9620f`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Optimized compiler BUILD receipt `exit status: 0; elapsed 405.097196524s` is separate from application performance.

## Complete samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | borrowed41-direct | 184.486024, 197.567902, 222.511002, 258.237800, 259.429391, 261.906330, 263.874484, 268.286435, 278.757863 | 278.757863 |
| cold | compile-plus-runtime-suite | borrowed41-direct | 185.170061, 198.355821, 223.479339, 259.084184, 260.410404, 262.936564, 264.762090, 269.150019, 279.726628 | 279.726628 |
| cold | compile | borrowed41-retained | 316.698277, 318.423377, 335.873001, 392.645338, 398.482095, 398.923299, 399.714092, 423.539417, 430.031831 | 430.031831 |
| cold | compile-plus-runtime-suite | borrowed41-retained | 317.456124, 319.358108, 336.680683, 393.520675, 399.317504, 399.797713, 401.000019, 424.434142, 430.892222 | 430.892222 |
| cold | compile | stock | 82.002799, 83.985717, 93.189024, 96.666799, 98.399668, 99.571257, 101.039964, 102.856584, 104.172024 | 104.172024 |
| cold | compile-plus-runtime-suite | stock | 82.603905, 84.548800, 93.879170, 97.424255, 99.209887, 100.352285, 101.724896, 103.727403, 105.015400 | 105.015400 |
| repeat | compile | borrowed41-direct | 191.059146, 202.265590, 221.187953, 243.382916, 245.002957, 245.888506, 260.184924, 269.673010, 272.454561 | 272.454561 |
| repeat | compile-plus-runtime-suite | borrowed41-direct | 191.878190, 203.107497, 222.054073, 244.271122, 245.806044, 246.903615, 261.173652, 270.414487, 273.346270 | 273.346270 |
| repeat | compile | borrowed41-retained | 9.796736, 11.343022, 11.477906, 12.771188, 13.026618, 13.228403, 13.511112, 18.416141, 18.589119 | 18.589119 |
| repeat | compile-plus-runtime-suite | borrowed41-retained | 10.634810, 12.166747, 12.192930, 13.645129, 13.848662, 14.093136, 14.398141, 19.384108, 19.482548 | 19.482548 |
| repeat | compile | stock | 81.032604, 83.320677, 86.675073, 94.146463, 99.003278, 100.318435, 104.027563, 104.632121, 105.669299 | 105.669299 |
| repeat | compile-plus-runtime-suite | stock | 81.602494, 83.919665, 87.485688, 94.802385, 99.729444, 101.051093, 104.738656, 105.354931, 106.457121 | 106.457121 |
| noop | compile | borrowed41-direct | 186.346101, 205.900179, 211.655355, 244.595893, 248.422194, 252.362946, 267.788153, 274.324923, 297.175995 | 297.175995 |
| noop | compile-plus-runtime-suite | borrowed41-direct | 187.124038, 206.689446, 212.602262, 245.436260, 249.340302, 253.337245, 268.792481, 275.853815, 298.188539 | 298.188539 |
| noop | compile | borrowed41-retained | 9.851832, 11.823058, 12.372023, 13.064786, 13.777923, 15.560690, 16.234550, 16.587865, 19.922046 | 19.922046 |
| noop | compile-plus-runtime-suite | borrowed41-retained | 10.575916, 12.626866, 13.220057, 14.066497, 14.749273, 16.605480, 17.097543, 17.355497, 20.904975 | 20.904975 |
| noop | compile | stock | 80.209193, 84.506699, 87.393719, 95.027128, 97.700945, 98.997449, 102.058736, 107.009074, 107.114036 | 107.114036 |
| noop | compile-plus-runtime-suite | stock | 80.744758, 85.195364, 88.030048, 95.704020, 98.520515, 99.709049, 102.763015, 107.736388, 107.891122 | 107.891122 |
| body-edit | compile | borrowed41-direct | 191.171065, 204.247235, 211.583287, 238.207991, 244.920192, 247.091423, 249.103818, 272.122996, 275.235983 | 275.235983 |
| body-edit | compile-plus-runtime-suite | borrowed41-direct | 191.883957, 204.941612, 212.213025, 239.144945, 245.745908, 248.064811, 250.014824, 273.022602, 276.034943 | 276.034943 |
| body-edit | compile | borrowed41-retained | 315.695290, 319.972660, 337.382938, 374.164661, 418.685535, 424.136458, 435.659394, 441.276082, 442.707492 | 442.707492 |
| body-edit | compile-plus-runtime-suite | borrowed41-retained | 316.949255, 320.772921, 338.176366, 375.039067, 419.692941, 425.072221, 436.686820, 442.353002, 443.577843 | 443.577843 |
| body-edit | compile | stock | 79.673609, 81.608226, 85.302553, 95.662397, 96.457151, 97.432647, 100.821585, 101.466767, 103.849267 | 103.849267 |
| body-edit | compile-plus-runtime-suite | stock | 80.362339, 82.275497, 85.942715, 96.342219, 97.137646, 98.166595, 101.545700, 102.268242, 104.685982 | 104.685982 |
| signature-edit | compile | borrowed41-direct | 191.380393, 210.936268, 218.222834, 225.136298, 256.766062, 271.021388, 276.698203, 277.857799, 289.314263 | 289.314263 |
| signature-edit | compile-plus-runtime-suite | borrowed41-direct | 192.109614, 211.720530, 219.071076, 226.130647, 257.683141, 272.000091, 277.605206, 278.742842, 290.314532 | 290.314532 |
| signature-edit | compile | borrowed41-retained | 309.382269, 323.421080, 329.555037, 390.058811, 390.165708, 409.715055, 421.606671, 430.471792, 448.983722 | 448.983722 |
| signature-edit | compile-plus-runtime-suite | borrowed41-retained | 310.098098, 324.073256, 330.326394, 391.008025, 391.121945, 410.732896, 422.525625, 431.398501, 450.199537 | 450.199537 |
| signature-edit | compile | stock | 80.205289, 82.199586, 85.794608, 88.336631, 97.994949, 100.313507, 101.064190, 101.298753, 110.363897 | 110.363897 |
| signature-edit | compile-plus-runtime-suite | stock | 80.849692, 82.823071, 86.447248, 88.935105, 98.788696, 101.074938, 101.804703, 102.065389, 111.214020 | 111.214020 |
| type-edit | compile | borrowed41-direct | 185.626187, 202.114795, 216.675782, 232.945678, 247.939172, 256.702340, 263.810559, 263.841057, 272.048992 | 272.048992 |
| type-edit | compile-plus-runtime-suite | borrowed41-direct | 186.279910, 202.924959, 217.494395, 233.845565, 248.765910, 257.567411, 264.627830, 264.746138, 273.155776 | 273.155776 |
| type-edit | compile | borrowed41-retained | 300.145011, 308.588923, 338.168259, 372.622890, 403.211258, 405.489890, 411.537926, 413.075987, 422.860562 | 422.860562 |
| type-edit | compile-plus-runtime-suite | borrowed41-retained | 300.987896, 309.348436, 339.101885, 373.769821, 404.108097, 406.665272, 412.585046, 414.123305, 423.862612 | 423.862612 |
| type-edit | compile | stock | 80.713181, 82.963350, 88.667446, 96.624344, 96.965489, 98.964151, 100.677540, 102.724937, 103.717236 | 103.717236 |
| type-edit | compile-plus-runtime-suite | stock | 81.266123, 83.419441, 89.388484, 97.370086, 97.723478, 99.694864, 101.423075, 103.520280, 104.410174 | 104.410174 |
| macro-edit | compile | borrowed41-direct | 188.053335, 214.613744, 245.643355, 247.017739, 251.329113, 264.345031, 265.968601, 269.993430, 270.651677 | 270.651677 |
| macro-edit | compile-plus-runtime-suite | borrowed41-direct | 188.733726, 215.463003, 246.567963, 247.970714, 252.276971, 265.359938, 266.842121, 270.871069, 271.620016 | 271.620016 |
| macro-edit | compile | borrowed41-retained | 284.935594, 326.361667, 358.405905, 395.547683, 397.176153, 410.889287, 411.620531, 411.925523, 418.496980 | 418.496980 |
| macro-edit | compile-plus-runtime-suite | borrowed41-retained | 285.772542, 327.181143, 359.363397, 396.489265, 398.077814, 411.825389, 412.606979, 413.003000, 419.455730 | 419.455730 |
| macro-edit | compile | stock | 83.071251, 84.830407, 94.236494, 98.807136, 98.909172, 100.087522, 101.073647, 101.806732, 107.221805 | 107.221805 |
| macro-edit | compile-plus-runtime-suite | stock | 83.729115, 85.568756, 94.923148, 99.618704, 99.639598, 100.848890, 101.706543, 102.574700, 108.033925 | 108.033925 |
| rejection | compile | borrowed41-direct | 123.486536, 125.271409, 159.812019, 162.153426, 174.004293, 176.478173, 177.690416, 188.116387, 201.922827 | 201.922827 |
| rejection | compile | borrowed41-retained | 132.957105, 139.887718, 172.478182, 175.194995, 180.915271, 181.812571, 183.157336, 183.251870, 199.218196 | 199.218196 |
| rejection | compile | stock | 27.088772, 28.106526, 32.772126, 33.420037, 33.921800, 34.127889, 34.308194, 34.575304, 36.255235 | 36.255235 |
| recovery | compile | borrowed41-direct | 182.218310, 184.033366, 240.986373, 247.468962, 251.359812, 257.631290, 271.522310, 275.233305, 286.812639 | 286.812639 |
| recovery | compile-plus-runtime-suite | borrowed41-direct | 182.907185, 184.712940, 241.776710, 248.368567, 252.182835, 258.934302, 272.434724, 276.153730, 287.687970 | 287.687970 |
| recovery | compile | borrowed41-retained | 309.329076, 312.125007, 382.877837, 389.928613, 395.534667, 420.139421, 425.017760, 440.541204, 449.907349 | 449.907349 |
| recovery | compile-plus-runtime-suite | borrowed41-retained | 310.109881, 313.060472, 383.995610, 390.916404, 396.711370, 421.062278, 426.004742, 441.458326, 451.020940 | 451.020940 |
| recovery | compile | stock | 79.126619, 81.771418, 93.674616, 97.920051, 99.306779, 100.394924, 102.598341, 102.733260, 105.075873 | 105.075873 |
| recovery | compile-plus-runtime-suite | stock | 79.670865, 82.346523, 94.381273, 98.794136, 100.121284, 101.099846, 103.334121, 103.426032, 105.888053 | 105.888053 |

Observed cold direct41 / stock ratio: 2.636x (lower is better). This observation establishes no improvement over the prior rejecting component.

Experimental component41 remains outside product34/published28. General native Rust/crates/metadata/rlib/Cargo, runtime-loader closure, stable-file-during-request assumptions, supervised lifecycle and full-project speed remain OPEN. All seven public-release gates remain held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC loader dependencies remain; no all-Rust or whole-toolchain memory-safety claim.
