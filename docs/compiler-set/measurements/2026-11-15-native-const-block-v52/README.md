# Native original-AST const blocks — component52

The complete original const-block assertion SHA256 `cfceef37c9056029b90b4482434269ecbe477b2352aa832eb76f07b09e2af688` compiles, links and runs natively with exact stock runtime output. Copied51 refused it with L0809; separate stock compilation/runtime succeeded. The rejected source supplies no successful native51 speed baseline. Earlier complete originals and protected Lust240e/Rust5b7b remain unchanged.

The owned formatter binds actual original native block IDs, spans and the sole const prefix token. A separate indentation box encloses const, a nonbreaking space and the existing block printer without another expression box around that block. Only derived panic literals change before complete native rechecking, borrowing, generation and linking. No retained transformed AST, protected printer copy, overlay, source rewrite, rustc native fallback or metadata fiction. See the [design and proof limits](../../proofs/native-assert-text.md).

Optimized direct28 and retained25 suites pass, followed by copied direct12, retained1 and host-input1 checks. A fresh actual gpt-6.1-sol HIGH/default R1 approved the frozen8 source scope; its completed and task_cancel-retired receipt is retained. The inherited F51-R1-001 first source and correction remain authoritative. Cases cover the complete original, nested blocks, long bodies, let/statement/tail expressions, tuples/arrays, comments, parentheses, valid const values and rejected const type/runtime-capture/nonconst-call/mutability/panic cases. Prior order/once/Copy/borrow/move/Drop/unwind gates remain, with retained body/type/token/position/signature edits, rejection and recovery.

## Equivalent successful work

SAME successful optimized52 direct/retained and stock compile the identical complete generated mixed workload. Full native/stock compile/runtime preflight passed BEFORE timing; bytes match timed sample0 cold. Nine rotated samples per phase/engine, fresh retained process per sample, native disk caches disabled before Check, OS pages untouched. Compiler opt3/CGU1/ThinLTO and application opt0/debug0/unwind/checks-on stay fixed. Compiler-build time is separate. Standalone no-op is not Cargo freshness; compile-plus-runtime is not Cargo test. Shared-host activity limits attribution.

No gain follows from comparison with rejecting51 or earlier distinct sources. Cold/edit/rejection stock regressions remain visible; a bounded no-op win does not close full-project speed.

Median milliseconds, lower is better.

| Phase | Direct52 | Retained52 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 229.570 | 341.987 | 96.670 | 230.450 | 342.787 | 97.386 |
| repeat | 229.122 | 11.915 | 97.339 | 229.939 | 12.768 | 98.094 |
| noop | 246.711 | 12.545 | 97.441 | 247.652 | 13.452 | 98.180 |
| body-edit | 246.694 | 403.398 | 104.732 | 247.625 | 404.270 | 105.529 |
| signature-edit | 248.494 | 401.413 | 102.455 | 249.260 | 402.338 | 103.290 |
| type-edit | 240.271 | 400.660 | 104.579 | 241.166 | 401.712 | 105.173 |
| macro-edit | 256.817 | 400.617 | 98.428 | 257.765 | 401.509 | 99.055 |
| rejection | 178.026 | 170.043 | 37.137 | — | — | — |
| recovery | 235.009 | 363.654 | 94.993 | 235.770 | 364.489 | 95.734 |

Raw events: 757; SHA256 `3194b1990b9d972232d1d9204ff130883277d2211057287cf0db63cd9e495cb8`. Summary51 SHA256 `79cf8f085f5d55d9860db06c4ee4999ced3336ad6c92ecb644fb34b346ca17a4`. Core `ad2cf9bd758811b59a2c1c39dabf017d64e41b8086d80b2077f7c91bb75eaac7`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 393.707502134s` is separate from application performance. Independent raw verification checks every group, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Case | Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | --- | ---: |
| mixed | cold | compile | const52-direct | 189.526795, 193.048462, 196.286505, 215.547406, 229.569822, 231.230020, 256.410402, 260.927554, 287.727836 | 287.727836 |
| mixed | cold | compile-plus-runtime-suite | const52-direct | 190.316507, 193.815669, 197.055233, 216.265015, 230.449834, 232.020989, 257.240141, 261.733295, 288.733235 | 288.733235 |
| mixed | cold | compile | const52-retained | 296.732227, 313.441680, 315.636381, 323.280579, 341.987157, 360.823160, 380.569040, 415.972003, 459.713640 | 459.713640 |
| mixed | cold | compile-plus-runtime-suite | const52-retained | 297.415029, 314.420534, 316.701840, 323.966750, 342.786712, 361.707428, 381.691487, 416.945627, 460.797425 | 460.797425 |
| mixed | cold | compile | stock | 83.826778, 88.199606, 89.111938, 89.638065, 96.670067, 101.333733, 104.104854, 105.666746, 116.009928 | 116.009928 |
| mixed | cold | compile-plus-runtime-suite | stock | 84.381818, 88.783941, 89.756982, 90.199764, 97.386005, 102.040970, 104.772843, 106.339761, 116.797238 | 116.797238 |
| mixed | repeat | compile | const52-direct | 185.558961, 193.828590, 202.712358, 209.917500, 229.121530, 238.371953, 252.458898, 291.534895, 296.784039 | 296.784039 |
| mixed | repeat | compile-plus-runtime-suite | const52-direct | 186.211614, 194.493057, 203.539023, 210.644405, 229.938597, 239.234807, 253.284077, 292.422713, 297.714240 | 297.714240 |
| mixed | repeat | compile | const52-retained | 10.447054, 10.463044, 10.689267, 11.508574, 11.915182, 12.293686, 12.299632, 18.453352, 20.135612 | 20.135612 |
| mixed | repeat | compile-plus-runtime-suite | const52-retained | 11.114230, 11.202408, 11.319858, 12.265059, 12.767798, 13.123253, 13.172797, 19.270191, 21.214889 | 21.214889 |
| mixed | repeat | compile | stock | 83.807153, 85.032610, 85.466251, 86.184643, 97.338751, 101.228378, 101.950865, 104.688327, 115.649097 | 115.649097 |
| mixed | repeat | compile-plus-runtime-suite | stock | 84.442848, 85.618773, 86.087523, 86.776243, 98.093510, 101.888415, 102.676416, 105.333510, 116.583234 | 116.583234 |
| mixed | noop | compile | const52-direct | 189.200299, 191.692455, 194.418434, 235.347202, 246.710844, 250.525796, 261.048296, 262.397429, 276.759628 | 276.759628 |
| mixed | noop | compile-plus-runtime-suite | const52-direct | 189.852441, 192.307873, 195.143412, 236.226744, 247.651997, 251.249850, 261.973723, 263.208422, 277.849013 | 277.849013 |
| mixed | noop | compile | const52-retained | 9.981552, 10.538302, 10.626141, 11.527522, 12.544733, 12.933852, 15.515579, 18.468756, 18.661942 | 18.661942 |
| mixed | noop | compile-plus-runtime-suite | const52-retained | 10.665500, 11.361964, 11.477200, 12.333092, 13.452106, 13.851490, 16.210649, 19.379948, 19.659441 | 19.659441 |
| mixed | noop | compile | stock | 81.187838, 82.777023, 87.128654, 92.358921, 97.440676, 99.707108, 102.640718, 111.170411, 111.550576 | 111.550576 |
| mixed | noop | compile-plus-runtime-suite | stock | 81.906208, 83.313641, 87.833308, 93.022003, 98.180339, 100.360192, 103.309940, 111.904770, 112.331326 | 112.331326 |
| mixed | body-edit | compile | const52-direct | 190.815295, 195.111843, 199.463576, 246.228212, 246.694041, 251.727453, 282.446321, 284.647661, 285.208442 | 285.208442 |
| mixed | body-edit | compile-plus-runtime-suite | const52-direct | 191.432187, 195.883743, 200.229524, 247.140349, 247.625343, 252.560474, 283.314731, 285.544323, 286.242269 | 286.242269 |
| mixed | body-edit | compile | const52-retained | 309.490675, 309.944648, 321.780046, 393.606034, 403.397982, 409.535524, 437.262593, 467.301557, 467.505012 | 467.505012 |
| mixed | body-edit | compile-plus-runtime-suite | const52-retained | 310.315590, 310.755478, 322.522044, 394.549641, 404.270351, 410.429066, 438.259842, 468.272956, 468.524572 | 468.524572 |
| mixed | body-edit | compile | stock | 85.847044, 86.402328, 88.095856, 103.562411, 104.731623, 112.990318, 114.252104, 114.479306, 115.951650 | 115.951650 |
| mixed | body-edit | compile-plus-runtime-suite | stock | 86.520355, 87.027489, 88.644535, 104.277236, 105.529485, 113.778352, 115.073817, 115.337911, 116.676695 | 116.676695 |
| mixed | signature-edit | compile | const52-direct | 191.809710, 205.509376, 210.893058, 243.733882, 248.494271, 258.404209, 282.962390, 300.551181, 316.478842 | 316.478842 |
| mixed | signature-edit | compile-plus-runtime-suite | const52-direct | 192.465342, 206.298179, 211.605751, 244.611729, 249.259965, 259.300048, 283.985213, 301.519138, 317.588079 | 317.588079 |
| mixed | signature-edit | compile | const52-retained | 298.500087, 319.401507, 327.790545, 388.936662, 401.413152, 422.689152, 426.424745, 446.531866, 484.652853 | 484.652853 |
| mixed | signature-edit | compile-plus-runtime-suite | const52-retained | 299.263477, 320.174337, 328.857081, 390.097860, 402.338346, 424.057544, 427.607829, 447.513793, 485.771298 | 485.771298 |
| mixed | signature-edit | compile | stock | 86.388949, 89.097422, 89.271390, 100.146778, 102.454615, 111.161246, 115.680048, 118.451060, 122.254385 | 122.254385 |
| mixed | signature-edit | compile-plus-runtime-suite | stock | 86.848195, 89.697722, 89.840269, 100.813243, 103.289701, 111.984988, 116.492759, 119.220283, 123.050503 | 123.050503 |
| mixed | type-edit | compile | const52-direct | 189.554317, 198.625038, 223.178808, 236.415041, 240.271464, 251.409775, 259.756930, 271.820468, 290.169103 | 290.169103 |
| mixed | type-edit | compile-plus-runtime-suite | const52-direct | 190.179029, 199.373095, 223.960571, 237.303838, 241.165990, 252.332048, 260.628900, 272.905387, 291.120085 | 291.120085 |
| mixed | type-edit | compile | const52-retained | 304.407333, 314.354092, 326.973458, 385.334753, 400.659598, 409.103146, 412.309370, 421.579684, 459.451123 | 459.451123 |
| mixed | type-edit | compile-plus-runtime-suite | const52-retained | 305.090840, 315.140587, 327.720870, 386.412424, 401.711891, 410.130055, 413.317429, 422.493631, 460.345942 | 460.345942 |
| mixed | type-edit | compile | stock | 81.987001, 86.754037, 92.080194, 103.884946, 104.578859, 106.151315, 108.866122, 109.758350, 112.821232 | 112.821232 |
| mixed | type-edit | compile-plus-runtime-suite | stock | 82.526695, 87.352428, 92.651241, 104.861682, 105.172503, 106.849273, 109.674730, 110.592068, 113.626610 | 113.626610 |
| mixed | macro-edit | compile | const52-direct | 192.409204, 209.259469, 229.664046, 232.567766, 256.817340, 260.484935, 264.885008, 284.728172, 308.321102 | 308.321102 |
| mixed | macro-edit | compile-plus-runtime-suite | const52-direct | 193.062687, 210.062112, 230.468083, 233.668989, 257.765479, 261.355237, 265.758200, 285.685147, 309.476165 | 309.476165 |
| mixed | macro-edit | compile | const52-retained | 302.904931, 318.286907, 366.108964, 386.512326, 400.617057, 416.067718, 418.402355, 418.767631, 429.724501 | 429.724501 |
| mixed | macro-edit | compile-plus-runtime-suite | const52-retained | 303.749232, 318.977680, 366.992886, 387.288289, 401.508752, 417.094519, 419.440828, 419.987917, 431.010833 | 431.010833 |
| mixed | macro-edit | compile | stock | 85.681477, 87.756680, 88.384360, 97.567605, 98.428359, 108.886583, 110.429503, 113.823684, 120.096278 | 120.096278 |
| mixed | macro-edit | compile-plus-runtime-suite | stock | 86.262723, 88.364048, 88.984696, 98.308396, 99.055242, 109.604155, 111.158904, 114.823451, 120.872401 | 120.872401 |
| mixed | rejection | compile | const52-direct | 133.771130, 135.525738, 150.813865, 162.869419, 178.026063, 181.261807, 188.939110, 189.084166, 189.968096 | 189.968096 |
| mixed | rejection | compile | const52-retained | 135.471583, 139.287420, 158.016864, 168.982526, 170.042804, 185.403730, 186.454983, 191.581875, 199.652642 | 199.652642 |
| mixed | rejection | compile | stock | 31.012943, 31.688562, 32.750104, 34.166423, 37.136834, 37.961954, 39.817826, 40.271167, 40.535790 | 40.535790 |
| mixed | recovery | compile | const52-direct | 195.134024, 196.158538, 219.791097, 228.813306, 235.009088, 240.147315, 247.378739, 272.621515, 273.890757 | 273.890757 |
| mixed | recovery | compile-plus-runtime-suite | const52-direct | 195.859071, 196.828588, 220.596020, 229.671096, 235.769671, 241.103844, 248.311069, 273.567825, 274.843419 | 274.843419 |
| mixed | recovery | compile | const52-retained | 303.140282, 304.012748, 335.989471, 350.256174, 363.653921, 364.694242, 396.186655, 405.557437, 421.676430 | 421.676430 |
| mixed | recovery | compile-plus-runtime-suite | const52-retained | 304.144025, 304.762063, 337.138218, 351.104913, 364.488994, 365.603497, 397.591980, 407.837606, 422.676927 | 422.676927 |
| mixed | recovery | compile | stock | 83.244746, 87.448982, 88.862485, 89.498231, 94.992577, 97.270033, 108.633564, 109.188476, 109.209168 | 109.209168 |
| mixed | recovery | compile-plus-runtime-suite | stock | 83.739383, 88.114509, 89.518244, 90.087454, 95.733555, 97.875632, 109.462654, 109.870108, 109.949869 | 109.949869 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
