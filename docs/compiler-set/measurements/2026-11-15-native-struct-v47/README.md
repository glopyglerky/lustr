# Native original-AST struct expressions — component47

The complete original struct assertion SHA256 `7469eba3a0b4a45a15d36150ed2ff69d0d4bd3cbb9c1eaa144da7271c2d4341a` compiles, links and runs natively with exact stock runtime output. Its copied46 L0809 rejection and stock compile/runtime witnesses are retained. No successful native46 speed baseline exists for this source. Earlier original binary, parentheses, long-assert, slice/array place and rvalue sources remain unchanged. Protected Lust240e and Rust5b7b stay unchanged.

The owned formatter binds each struct path, ordered field name/value, shorthand, colon, separator and rest expression to original tokens and native scratch-AST spans. Consistent boxes add a trailing comma only at a broken final field and retain the closing offset. Only unknown panic literals change before complete native checking, borrowing, generation and linking; source and executable AST stay unchanged. There is no retained transformed AST, protected printer copy, overlay or rustc native fallback. See the [source-bound design and limits](../../proofs/native-assert-text.md).

Fresh actual gpt-6.1-sol HIGH/default frozen seven-file R1 approved without findings. Optimized direct23 and retained20 suites pass, followed by copied-component direct6, retained1 and host-input1 checks. The struct corpus checks explicit/shorthand/raw/numeric fields, empty/nested/qualified structs, updates, comments, long broken payloads, field order/once, borrowing, Drop, initialization unwind, const/type/move rejection and attribute refusal. Retained field/name/type/rest/token/position/signature edits, invalid edits and recovery agree with cold native and stock. Prior binary/parenthesis/coordinator/mixed/arena gates pass. A separate direct tuple-field-access formatting gap is retained with actual failure and stock witness; numeric construction coverage does not claim that access syntax.

## Equivalent successful work

SAME successful optimized47 direct/retained and stock compile the same full generated mixed workload. Actual native/stock compile/runtime preflight passed before timing; its bytes match timed sample0 cold source. Nine rotated samples per engine/phase; a new retained process per sample; native disk caches disabled before Check; shared OS pages untouched. Compiler opt3/CGU1/ThinLTO and application opt0/debug0/unwind/checks-on remain fixed. Compiler-build cost is separate. Standalone no-op is not Cargo freshness; compile-plus-runtime is executed work rather than Cargo test. Shared-host activity limits attribution.

No comparison with rejecting46 or distinct earlier sources is valid. Cold/edit/rejection regressions against stock remain visible. A bounded artifact-proof no-op does not establish full-project speed.

Median milliseconds, lower is better.

| Phase | Direct47 | Retained47 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 269.037 | 403.926 | 113.469 | 269.920 | 404.837 | 114.224 |
| repeat | 267.928 | 14.075 | 110.989 | 268.949 | 15.038 | 111.822 |
| noop | 276.747 | 13.590 | 113.394 | 277.610 | 14.543 | 114.118 |
| body-edit | 279.595 | 431.067 | 109.849 | 280.590 | 432.148 | 110.626 |
| signature-edit | 260.334 | 405.814 | 107.435 | 261.345 | 406.979 | 108.135 |
| type-edit | 262.195 | 417.333 | 105.071 | 263.351 | 418.209 | 105.874 |
| macro-edit | 263.417 | 400.026 | 110.622 | 264.287 | 400.956 | 111.326 |
| rejection | 183.081 | 182.254 | 38.733 | — | — | — |
| recovery | 267.781 | 423.089 | 110.108 | 268.691 | 423.952 | 110.919 |

Raw events: 757; SHA256 `5b7d54e29acf3b219a0812ad784e60fb83df7aa21d9f9c214c4a3ebb67c66ccb`. Summary51 SHA256 `96872579b563cededa03499113cb8e513bafb6d2de794f639dd94128a303174f`. Core `7c2244c71c6228bc0d48bf6088638017d2ca7182d47f21db6fcc0db43a6edb37`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 376.812331448s` is separate from application performance. Independent raw verification checks every group, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Case | Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | --- | ---: |
| mixed | cold | compile | struct47-direct | 198.704278, 200.732917, 255.879132, 265.838031, 269.037361, 280.173408, 286.788110, 289.286352, 299.507350 | 299.507350 |
| mixed | cold | compile-plus-runtime-suite | struct47-direct | 199.437301, 201.439252, 256.729111, 266.669622, 269.919979, 280.931738, 287.776476, 290.367278, 300.372932 | 300.372932 |
| mixed | cold | compile | struct47-retained | 312.440733, 340.891269, 386.792175, 402.427821, 403.926083, 416.056946, 442.987349, 447.176160, 457.255131 | 457.255131 |
| mixed | cold | compile-plus-runtime-suite | struct47-retained | 313.266028, 341.680458, 387.639260, 403.424334, 404.836832, 417.033447, 443.816772, 448.242534, 458.155584 | 458.155584 |
| mixed | cold | compile | stock | 89.895767, 92.596493, 104.581804, 111.387789, 113.469061, 116.610969, 117.069751, 117.913955, 119.013797 | 119.013797 |
| mixed | cold | compile-plus-runtime-suite | stock | 90.503397, 93.191719, 105.242756, 112.090164, 114.224233, 117.467869, 117.825690, 118.641024, 119.832961 | 119.832961 |
| mixed | repeat | compile | struct47-direct | 220.508860, 261.526789, 264.092692, 267.673798, 267.927759, 279.222192, 301.949094, 308.326038, 316.575000 | 316.575000 |
| mixed | repeat | compile-plus-runtime-suite | struct47-direct | 221.385506, 262.260957, 265.036435, 268.603461, 268.949377, 280.205632, 302.813269, 309.225274, 317.503096 | 317.503096 |
| mixed | repeat | compile | struct47-retained | 11.026789, 11.903915, 12.949037, 13.438147, 14.074543, 14.670710, 18.944631, 19.975998, 21.461022 | 21.461022 |
| mixed | repeat | compile-plus-runtime-suite | struct47-retained | 11.899656, 12.743711, 13.805589, 14.501225, 15.038325, 15.667454, 19.844848, 20.874891, 22.505570 | 22.505570 |
| mixed | repeat | compile | stock | 91.625898, 92.674474, 106.210097, 107.659809, 110.988656, 111.955415, 117.019308, 119.424502, 125.230640 | 125.230640 |
| mixed | repeat | compile-plus-runtime-suite | stock | 92.230275, 93.305109, 106.885157, 108.634610, 111.821893, 112.697363, 117.845253, 120.287676, 125.975851 | 125.975851 |
| mixed | noop | compile | struct47-direct | 240.725093, 256.379876, 265.155823, 274.458586, 276.747218, 278.675035, 288.142390, 296.784578, 349.928453 | 349.928453 |
| mixed | noop | compile-plus-runtime-suite | struct47-direct | 241.558298, 257.235932, 266.085924, 275.456065, 277.610013, 279.667197, 289.160965, 297.639276, 350.933003 | 350.933003 |
| mixed | noop | compile | struct47-retained | 10.850839, 11.072319, 12.973600, 13.421948, 13.589789, 14.170667, 19.595417, 20.100701, 20.259666 | 20.259666 |
| mixed | noop | compile-plus-runtime-suite | struct47-retained | 11.652158, 11.907043, 14.085383, 14.324795, 14.542922, 15.222371, 20.498096, 21.035759, 21.252473 | 21.252473 |
| mixed | noop | compile | stock | 93.323634, 100.443535, 109.605823, 112.727744, 113.394083, 114.894426, 116.473016, 118.696872, 120.163338 | 120.163338 |
| mixed | noop | compile-plus-runtime-suite | stock | 94.073069, 101.125052, 110.476442, 113.592658, 114.118283, 115.658861, 117.317719, 119.503649, 120.976724 | 120.976724 |
| mixed | body-edit | compile | struct47-direct | 240.094448, 246.310383, 260.651912, 263.200044, 279.594740, 285.546913, 287.003920, 289.104522, 357.372972 | 357.372972 |
| mixed | body-edit | compile-plus-runtime-suite | struct47-direct | 240.860504, 247.098774, 261.575869, 264.054679, 280.589894, 286.460138, 287.976570, 290.114831, 358.591772 | 358.591772 |
| mixed | body-edit | compile | struct47-retained | 361.394197, 365.712989, 401.290407, 412.585782, 431.066734, 442.624243, 444.904153, 463.344840, 479.031174 | 479.031174 |
| mixed | body-edit | compile-plus-runtime-suite | struct47-retained | 362.212414, 366.462829, 402.522041, 413.490483, 432.147889, 443.621908, 445.664282, 464.342881, 480.130191 | 480.130191 |
| mixed | body-edit | compile | stock | 93.934729, 100.779269, 104.119130, 108.697029, 109.849409, 112.659084, 112.671954, 114.617210, 127.144612 | 127.144612 |
| mixed | body-edit | compile-plus-runtime-suite | stock | 94.549810, 101.440483, 104.767390, 109.397730, 110.626285, 113.429541, 113.496209, 115.420438, 128.231620 | 128.231620 |
| mixed | signature-edit | compile | struct47-direct | 218.467512, 241.617746, 244.919946, 245.890580, 260.333905, 267.037738, 300.225806, 304.727200, 341.423667 | 341.423667 |
| mixed | signature-edit | compile-plus-runtime-suite | struct47-direct | 219.317356, 242.456801, 245.800232, 246.758805, 261.345226, 267.909570, 301.178724, 305.618804, 342.502838 | 342.502838 |
| mixed | signature-edit | compile | struct47-retained | 359.273913, 377.509272, 394.764833, 402.760258, 405.813513, 429.509330, 431.751422, 453.600301, 489.058673 | 489.058673 |
| mixed | signature-edit | compile-plus-runtime-suite | struct47-retained | 360.161862, 378.513797, 395.713793, 403.606103, 406.979365, 430.413712, 432.623371, 454.632997, 490.126792 | 490.126792 |
| mixed | signature-edit | compile | stock | 92.396204, 102.367528, 103.035899, 107.184413, 107.435457, 112.811031, 119.917839, 124.526424, 130.430099 | 130.430099 |
| mixed | signature-edit | compile-plus-runtime-suite | stock | 93.009614, 103.036954, 103.707922, 107.946463, 108.134680, 113.538610, 120.683805, 125.452688, 131.356336 | 131.356336 |
| mixed | type-edit | compile | struct47-direct | 215.831549, 218.227281, 231.616525, 256.241960, 262.194808, 268.511878, 284.239813, 285.975122, 290.654529 | 290.654529 |
| mixed | type-edit | compile-plus-runtime-suite | struct47-direct | 216.541894, 218.999841, 232.269906, 257.226825, 263.350827, 269.463648, 285.226727, 286.964729, 291.586640 | 291.586640 |
| mixed | type-edit | compile | struct47-retained | 319.963238, 368.840975, 394.590241, 414.373530, 417.333206, 428.886951, 430.362239, 437.897819, 482.690905 | 482.690905 |
| mixed | type-edit | compile-plus-runtime-suite | struct47-retained | 320.854360, 369.814107, 395.483855, 415.337104, 418.208651, 429.994454, 431.376336, 438.812665, 483.751005 | 483.751005 |
| mixed | type-edit | compile | stock | 88.235377, 93.959535, 99.533555, 104.651484, 105.071281, 112.609760, 113.692168, 118.138555, 122.675874 | 122.675874 |
| mixed | type-edit | compile-plus-runtime-suite | stock | 88.826080, 94.581891, 100.181563, 105.390788, 105.874276, 113.357565, 114.426944, 118.955994, 123.530820 | 123.530820 |
| mixed | macro-edit | compile | struct47-direct | 202.128272, 218.237983, 229.967445, 247.077184, 263.416947, 263.927932, 272.563819, 277.838356, 278.563002 | 278.563002 |
| mixed | macro-edit | compile-plus-runtime-suite | struct47-direct | 202.821234, 219.159636, 230.726382, 247.957811, 264.286537, 264.764378, 273.621135, 278.699560, 279.451418 | 279.451418 |
| mixed | macro-edit | compile | struct47-retained | 308.154934, 315.126513, 370.519978, 390.886900, 400.025679, 401.561488, 402.650123, 432.692829, 463.851090 | 463.851090 |
| mixed | macro-edit | compile-plus-runtime-suite | struct47-retained | 308.948672, 315.910859, 371.711520, 391.730622, 400.956149, 402.435788, 403.543538, 433.625406, 464.786552 | 464.786552 |
| mixed | macro-edit | compile | stock | 84.738939, 94.067474, 103.141726, 106.011969, 110.622027, 110.968192, 112.462648, 115.388438, 118.999552 | 118.999552 |
| mixed | macro-edit | compile-plus-runtime-suite | stock | 85.305789, 94.691355, 103.788126, 106.692271, 111.326252, 111.714149, 113.068313, 116.129287, 119.823615 | 119.823615 |
| mixed | rejection | compile | struct47-direct | 136.134755, 152.357526, 179.654754, 181.777986, 183.080776, 185.868686, 193.786065, 194.853431, 195.877282 | 195.877282 |
| mixed | rejection | compile | struct47-retained | 137.061013, 150.127280, 166.111194, 181.076193, 182.254306, 194.485857, 203.449225, 206.743916, 220.084129 | 220.084129 |
| mixed | rejection | compile | stock | 30.954323, 34.195578, 36.293391, 36.719911, 38.733180, 40.637521, 40.674226, 40.900483, 41.922350 | 41.922350 |
| mixed | recovery | compile | struct47-direct | 198.451281, 227.705651, 230.609757, 233.171206, 267.780922, 277.772113, 287.139646, 288.317959, 303.200639 | 303.200639 |
| mixed | recovery | compile-plus-runtime-suite | struct47-direct | 199.163710, 228.521158, 231.522621, 234.039914, 268.691384, 278.802423, 288.045338, 289.234539, 304.305457 | 304.305457 |
| mixed | recovery | compile | struct47-retained | 299.314137, 345.636295, 361.456971, 370.964207, 423.088972, 444.334663, 448.109143, 474.982302, 480.728636 | 480.728636 |
| mixed | recovery | compile-plus-runtime-suite | struct47-retained | 300.083641, 346.561753, 362.464461, 371.865368, 423.951704, 445.296220, 449.332802, 476.014347, 481.709387 | 481.709387 |
| mixed | recovery | compile | stock | 91.329751, 92.333135, 96.156708, 99.953957, 110.108443, 110.591709, 113.944666, 114.224315, 117.816279 | 117.816279 |
| mixed | recovery | compile-plus-runtime-suite | stock | 91.905954, 92.916908, 96.789273, 100.803232, 110.919196, 111.315388, 114.721630, 114.967025, 118.616557 | 118.616557 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
