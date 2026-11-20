# Native original-token parentheses — component45

The complete original parenthesized assertion source SHA256 `b33a77ba19587159f387a51dcad75d7280ebab84c1204eab538512cf4a76e930` now compiles, links and runs natively with exact stock runtime output. Its original copied44 L0809 rejection and stock compile/runtime witnesses remain retained. No successful native44 speed baseline exists for that source. The original full long-assert source `612287967c0d03d408b3704aa6451550e04b0555b74d559258f92dad417eef9b` and earlier full place/rvalue sources remain unchanged. Protected Lust240e and Rust5b7b stay unchanged.

The owned formatter pairs original delimiters and recovers every complete wrapper from widened scratch-AST spans. Each wrapper contributes an expression box matching the pinned Rust Paren printer; call delimiters and unit parentheses retain their structural roles. Parenthesized macro statements use expression boxes. Only the assertion-text marker changes; executable conditions, evaluation order, borrows, expression/pattern arenas, checks, panic, Drop and unwind remain native work. Full native rechecking and generation determine acceptance. No transformed AST survives requests. Attributes, expanded contexts, empty statements, binary/struct/generic/unmodeled printing remain explicit refusal frontiers. See the [source-bound design](../../proofs/native-assert-text.md).

Fresh frozen seven-file Sol review approved with no findings. Actual optimized direct21 and retained18 suites pass, followed by copied-component direct4, retained1 and host-input1 checks. Gates cover the complete source, double/triple wrappers, callees, closures, arguments, units, macro statements, blocks, receivers and trailing-dot floats; exact multiline panic payloads; native const and borrow/type rejection; once/Drop/unwind; retained nesting/token/value/position/signature edits, invalid edits and recovery against cold native and stock. Review is not execution or performance proof.

The first measurement attempt stopped before runtime because its generator also changed an earlier panic witness, leaving unbalanced delimiters. Its source, raw three events and native L0001 failure remain retained. The generator now isolates the appended function and checks single opening/closing matches. Fresh Sol generator-only recheck approved. Actual corrected full-workload native/stock compile/runtime parity passed before timing, and its bytes match the successful timed cold source. The compiler and prior optimized/copy correctness results are unchanged.

## Equivalent successful compiler costs

One campaign compares SAME successful optimized45 direct/retained and stock on the same generated mixed workload, including nested parenthesized long assertion failures and native recovery, platform inference, place assertions, Drop and unwind. Compiler opt3/CGU1/ThinLTO; application opt0/debug0/unwind/checks-on. Nine rotated samples per engine/phase; new retained process per sample; disk cache disabled before Check; shared OS pages untouched. All original sources, actual artifacts, runtime outputs and raw samples remain retained. Standalone repeat/no-op is not Cargo freshness; compile-plus-runtime is actual execution, not Cargo test. Shared-host activity limits causal attribution.

Rejecting44 and distinct earlier workloads supply no speed comparison. Cold/edit/rejection regressions versus stock are retained. A bounded no-op artifact-proof result does not establish full-project speed.

Median milliseconds, lower is better.

| Phase | Direct45 | Retained45 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 255.744 | 392.773 | 104.789 | 256.609 | 393.864 | 105.523 |
| repeat | 273.743 | 15.430 | 109.603 | 274.769 | 16.496 | 110.404 |
| noop | 270.689 | 13.455 | 112.177 | 271.578 | 14.477 | 113.015 |
| body-edit | 276.452 | 442.207 | 111.464 | 277.493 | 443.267 | 112.338 |
| signature-edit | 285.452 | 428.199 | 109.847 | 286.300 | 429.227 | 110.410 |
| type-edit | 272.114 | 411.018 | 108.721 | 272.927 | 412.243 | 109.516 |
| macro-edit | 251.640 | 390.448 | 97.831 | 252.456 | 391.348 | 98.485 |
| rejection | 178.887 | 174.655 | 36.617 | — | — | — |
| recovery | 261.163 | 388.612 | 105.520 | 262.030 | 389.524 | 106.242 |

Raw events: 757; SHA256 `22222e6753a4a32cf44a43ee5768d9419c2320590f74b11db373aa6f8063a017`. Summary51 SHA256 `e7ba8fd3f5d8b6069400e1d86d3045985d22e844e1901b14ad6d1967b7577123`. Core `998fb68cb56a4d8cc4d66f751f07efdb36e0c2de170c3f3604fd48fdcb8a64dc`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 409.502768616s` is separate from application performance. Raw verification independently checks all51 groups, unique samples0..8, sorted times, medians and maxima.

## All raw samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | paren45-direct | 195.064986, 206.743105, 228.628469, 249.363774, 255.744026, 260.143805, 286.192674, 293.915824, 320.531301 | 320.531301 |
| cold | compile-plus-runtime-suite | paren45-direct | 195.817257, 207.463983, 229.442157, 250.398893, 256.608977, 260.924626, 287.066785, 295.019415, 321.571230 | 321.571230 |
| cold | compile | paren45-retained | 314.400682, 349.693590, 374.678727, 375.812904, 392.772641, 420.458530, 424.372449, 476.495179, 498.416390 | 498.416390 |
| cold | compile-plus-runtime-suite | paren45-retained | 315.217357, 350.689493, 375.573787, 376.608499, 393.864406, 421.326979, 425.301623, 477.537689, 499.392412 | 499.392412 |
| cold | compile | stock | 87.377001, 91.016672, 95.082314, 98.723461, 104.789041, 105.608824, 112.632682, 118.481727, 124.172396 | 124.172396 |
| cold | compile-plus-runtime-suite | stock | 87.983303, 91.693858, 95.759222, 99.362990, 105.523110, 106.306655, 113.396850, 119.284736, 124.940977 | 124.940977 |
| repeat | compile | paren45-direct | 206.710751, 210.961156, 241.365025, 268.633450, 273.743095, 274.262799, 277.863412, 302.357904, 336.015259 | 336.015259 |
| repeat | compile-plus-runtime-suite | paren45-direct | 207.518961, 211.756318, 242.272071, 269.487869, 274.769435, 275.301012, 278.751134, 303.276691, 337.092636 | 337.092636 |
| repeat | compile | paren45-retained | 11.502941, 12.533700, 13.030356, 13.091202, 15.429771, 15.605870, 15.625307, 19.251522, 20.311116 | 20.311116 |
| repeat | compile-plus-runtime-suite | paren45-retained | 12.405899, 13.439304, 13.985876, 14.073490, 16.496011, 16.591224, 16.652490, 20.177667, 21.212575 | 21.212575 |
| repeat | compile | stock | 93.555440, 100.614191, 101.096102, 105.404388, 109.602515, 115.665653, 116.430271, 116.983183, 123.782473 | 123.782473 |
| repeat | compile-plus-runtime-suite | stock | 94.270168, 101.230269, 101.691554, 106.087418, 110.403552, 116.361872, 117.197087, 117.754469, 124.512492 | 124.512492 |
| noop | compile | paren45-direct | 198.781255, 217.587355, 231.252174, 269.499530, 270.689274, 284.629902, 287.197137, 294.997136, 303.163823 | 303.163823 |
| noop | compile-plus-runtime-suite | paren45-direct | 199.544337, 218.346601, 232.038353, 270.402523, 271.577649, 285.543138, 288.197069, 295.981653, 304.273275 | 304.273275 |
| noop | compile | paren45-retained | 11.201950, 11.699542, 12.752805, 13.065120, 13.454955, 14.652449, 17.498423, 17.542931, 20.384297 | 20.384297 |
| noop | compile-plus-runtime-suite | paren45-retained | 11.951089, 12.598477, 13.614695, 13.979719, 14.477043, 17.726818, 18.290575, 18.415777, 21.493274 | 21.493274 |
| noop | compile | stock | 87.674552, 88.249813, 94.535408, 104.485824, 112.176835, 115.920112, 116.300782, 123.086900, 123.809649 | 123.809649 |
| noop | compile-plus-runtime-suite | stock | 88.262855, 88.827136, 95.185913, 105.205205, 113.015333, 116.568712, 117.044666, 123.877285, 124.647652 | 124.647652 |
| body-edit | compile | paren45-direct | 199.663353, 215.050081, 244.101045, 274.483797, 276.452420, 282.653432, 288.419032, 293.255772, 308.199322 | 308.199322 |
| body-edit | compile-plus-runtime-suite | paren45-direct | 200.364707, 215.785906, 244.957471, 275.385093, 277.492946, 283.532474, 289.447871, 295.042102, 309.127830 | 309.127830 |
| body-edit | compile | paren45-retained | 317.813627, 351.430645, 367.799836, 426.033120, 442.207300, 447.475246, 451.292425, 454.842741, 460.375293 | 460.375293 |
| body-edit | compile-plus-runtime-suite | paren45-retained | 318.829589, 352.386563, 368.610064, 427.171190, 443.266640, 448.329821, 452.271234, 455.869583, 461.394633 | 461.394633 |
| body-edit | compile | stock | 90.183956, 92.491650, 97.125863, 109.647317, 111.464157, 111.821142, 113.691039, 118.131636, 125.310713 | 125.310713 |
| body-edit | compile-plus-runtime-suite | stock | 90.821669, 93.160613, 97.749850, 110.337129, 112.338011, 112.613027, 114.491226, 118.918095, 126.081987 | 126.081987 |
| signature-edit | compile | paren45-direct | 202.848772, 243.409419, 260.177150, 272.092614, 285.451772, 288.893076, 292.441728, 296.810961, 303.385571 | 303.385571 |
| signature-edit | compile-plus-runtime-suite | paren45-direct | 203.565010, 244.393411, 260.950495, 272.968977, 286.300350, 289.893202, 293.374361, 297.770251, 304.287688 | 304.287688 |
| signature-edit | compile | paren45-retained | 343.794598, 352.875484, 393.516551, 421.776813, 428.199208, 430.141875, 452.818028, 461.194755, 463.835310 | 463.835310 |
| signature-edit | compile-plus-runtime-suite | paren45-retained | 344.529309, 353.777391, 394.526149, 422.628130, 429.226509, 431.041053, 453.819248, 462.278553, 464.778047 | 464.778047 |
| signature-edit | compile | stock | 90.065211, 95.256612, 107.030058, 109.372105, 109.847281, 111.217128, 112.058384, 112.928406, 115.897762 | 115.897762 |
| signature-edit | compile-plus-runtime-suite | stock | 90.679922, 95.868859, 107.801195, 110.047893, 110.409795, 111.952405, 112.773232, 113.680926, 116.703293 | 116.703293 |
| type-edit | compile | paren45-direct | 214.335479, 248.433234, 251.700915, 257.111755, 272.114390, 276.084333, 285.720915, 290.882335, 309.955429 | 309.955429 |
| type-edit | compile-plus-runtime-suite | paren45-direct | 215.389770, 249.219866, 252.589713, 257.982158, 272.926595, 277.024610, 286.632856, 291.989476, 310.875896 | 310.875896 |
| type-edit | compile | paren45-retained | 323.971266, 372.184221, 391.719995, 397.382950, 411.018455, 425.351393, 432.113238, 437.055050, 437.676637 | 437.676637 |
| type-edit | compile-plus-runtime-suite | paren45-retained | 324.628676, 373.060163, 392.617765, 398.145839, 412.242767, 426.593019, 432.960867, 438.005479, 438.652918 | 438.652918 |
| type-edit | compile | stock | 90.862526, 101.611291, 102.300439, 106.537240, 108.721072, 108.998109, 109.775661, 110.804123, 111.281848 | 111.281848 |
| type-edit | compile-plus-runtime-suite | stock | 91.531692, 102.283257, 102.957796, 107.149080, 109.516020, 109.826525, 110.550725, 111.739253, 112.096319 | 112.096319 |
| macro-edit | compile | paren45-direct | 200.042572, 227.331954, 232.042250, 236.059776, 251.640113, 266.797821, 270.965377, 283.697631, 290.944635 | 290.944635 |
| macro-edit | compile-plus-runtime-suite | paren45-direct | 200.755713, 228.106917, 232.904115, 236.972806, 252.456261, 267.670438, 271.859146, 284.568112, 291.928441 | 291.928441 |
| macro-edit | compile | paren45-retained | 333.547557, 353.966555, 366.930698, 381.883704, 390.447684, 411.425474, 418.053073, 440.333765, 443.248619 | 443.248619 |
| macro-edit | compile-plus-runtime-suite | paren45-retained | 334.336682, 354.940201, 367.987671, 382.702227, 391.347885, 412.426822, 418.961859, 441.213219, 444.459632 | 444.459632 |
| macro-edit | compile | stock | 84.435663, 93.889848, 94.195734, 96.806719, 97.830821, 109.199336, 112.888135, 117.227945, 126.851576 | 126.851576 |
| macro-edit | compile-plus-runtime-suite | stock | 84.929846, 94.529066, 94.916344, 97.738155, 98.484833, 110.000972, 113.601876, 118.150997, 127.743710 | 127.743710 |
| rejection | compile | paren45-direct | 134.495665, 152.074845, 154.207863, 166.983937, 178.887098, 186.166065, 192.164461, 206.585467, 210.543324 | 210.543324 |
| rejection | compile | paren45-retained | 135.919268, 151.552068, 168.874257, 169.354403, 174.655054, 184.583346, 207.076400, 210.853882, 260.281142 | 260.281142 |
| rejection | compile | stock | 30.799438, 33.013602, 34.618013, 36.236689, 36.617472, 38.031702, 39.656523, 40.035225, 40.321415 | 40.321415 |
| recovery | compile | paren45-direct | 206.156560, 219.104004, 246.330652, 248.865248, 261.162884, 280.965691, 283.952416, 290.983144, 352.895669 | 352.895669 |
| recovery | compile-plus-runtime-suite | paren45-direct | 206.980811, 219.968308, 247.289019, 249.709005, 262.030294, 282.037860, 284.898283, 291.896286, 354.079451 | 354.079451 |
| recovery | compile | paren45-retained | 346.849370, 352.087051, 376.925030, 386.980513, 388.611884, 391.086691, 412.466169, 423.823928, 486.975338 | 486.975338 |
| recovery | compile-plus-runtime-suite | paren45-retained | 347.770064, 352.871832, 377.789272, 387.971168, 389.524499, 392.034646, 413.422469, 424.733425, 488.020708 | 488.020708 |
| recovery | compile | stock | 86.599827, 94.749258, 102.425815, 104.812250, 105.519683, 106.743780, 111.374804, 112.649696, 120.853675 | 120.853675 |
| recovery | compile-plus-runtime-suite | stock | 87.294384, 95.363434, 103.214288, 105.680040, 106.242025, 107.590521, 112.213284, 113.452702, 121.631335 | 121.631335 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc, Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
