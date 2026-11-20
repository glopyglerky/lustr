# Native unsafe boolean-local bodies — component54

The complete original unsafe-let witness `4109a92f5c3c0a4e4dacce92d3b9754b52677d76a5311919c95ddd09055e9be4` compiles and executes natively with stock runtime output, original assertion text and Guard/drop behavior. Component53 rejected that source at its collection safety gate; its rejection is not a successful speed baseline.

The owned checker derives safety for initialized boolean locals from the original AST, resolving only already-bound local names with their syntax context. It checks initializers before introducing bindings and restores lexical scope on every nested-block return. Nested unsafe capture requires an actual successful traversal from the outer block. It lowers the checked clone with identical block/arena identities before complete native collection, type/borrow/const checking and generation. Original AST/source remain intact. Calls, pointer operations, arbitrary captures/types/patterns, broader grammar and actual unsafe semantics remain open.

Optimized30 direct/27 retained tests, copied14 assertion tests and copied old/new retained plus host-input checks pass. Full generated native/stock compile/runtime preflight precedes measurement and equals timed sample0 cold. Frozen15 actual fresh Sol HIGH/default review is retained separately.

## Equivalent successful work

SAME successful54 optimized direct/retained/stock, nine rotated samples per phase. Compiler opt3/CGU1/ThinLTO; applications opt0/debug0/unwind/checks-on. Native disk caches disabled before Check, new bounded retained process per sample, OS pages untouched. Shared-host noise limits attribution. Compiler-build time is separate. Standalone no-op is not Cargo freshness and compile-plus-runtime is not Cargo test. No gain versus rejecting53 or distinct earlier sources. Cold/edit/rejection regressions remain visible.

Median milliseconds, lower is better.

| Phase | Direct54 | Retained54 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 253.554 | 411.325 | 106.743 | 254.454 | 412.203 | 107.433 |
| repeat | 260.796 | 11.862 | 101.731 | 261.715 | 12.734 | 102.444 |
| noop | 260.958 | 13.496 | 101.570 | 261.815 | 14.552 | 102.276 |
| body-edit | 258.627 | 422.560 | 99.056 | 259.558 | 423.602 | 99.820 |
| signature-edit | 269.158 | 394.776 | 99.644 | 270.102 | 395.821 | 100.330 |
| type-edit | 259.747 | 404.580 | 100.477 | 260.621 | 405.485 | 101.139 |
| macro-edit | 268.420 | 419.330 | 108.098 | 269.203 | 420.260 | 108.737 |
| rejection | 188.290 | 190.788 | 39.638 | — | — | — |
| recovery | 272.150 | 430.874 | 107.662 | 273.069 | 431.864 | 108.354 |

Raw757 events SHA256 `8edbccd9c7834e8452c3fd9214741e59fda03358275f5760859c2dee9be63b94`; summary SHA256 `43384a67c6ef3cbe21ce35d88b6598cdbf1933482159487e547114ddaa302456`; core `ab35b54fbfc6835a132a5169c9b0b1e3ed254c91c16dc0b58aa9a6b13b13f83c`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Independent verification checks all51 groups, unique samples0..8, sorted values, medians/maxima,81 actual direct commands,81 retained calls/18 hits and81 stock commands. Compiler build receipt: `exit status: 0; elapsed 410.589208426s`.

## Raw samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | unsafe54-direct | 198.320222, 200.736825, 211.027172, 221.813015, 253.553627, 286.450723, 307.929263, 308.558452, 361.925973 | 361.925973 |
| cold | compile-plus-runtime-suite | unsafe54-direct | 199.190891, 201.424397, 211.747535, 222.693837, 254.453874, 287.438998, 309.032448, 309.508860, 362.902975 | 362.902975 |
| cold | compile | unsafe54-retained | 316.729963, 318.827438, 347.154785, 370.553347, 411.324827, 421.255862, 423.193243, 449.026709, 454.995654 | 454.995654 |
| cold | compile-plus-runtime-suite | unsafe54-retained | 317.500793, 319.701783, 348.255328, 371.769349, 412.202505, 422.221179, 424.235343, 450.108748, 455.948197 | 455.948197 |
| cold | compile | stock | 84.282949, 87.951459, 88.970602, 96.657859, 106.743374, 109.955932, 117.067509, 125.458992, 127.214572 | 127.214572 |
| cold | compile-plus-runtime-suite | stock | 84.972210, 88.487701, 89.557588, 97.308091, 107.433204, 110.636092, 117.878366, 126.215885, 128.025507 | 128.025507 |
| repeat | compile | unsafe54-direct | 198.011315, 211.076118, 217.229885, 235.849733, 260.795939, 260.850356, 262.697632, 263.771440, 286.179544 | 286.179544 |
| repeat | compile-plus-runtime-suite | unsafe54-direct | 198.669165, 211.893205, 217.995430, 236.655648, 261.715159, 261.729455, 263.623034, 265.076335, 287.143297 | 287.143297 |
| repeat | compile | unsafe54-retained | 9.786057, 10.926385, 11.093969, 11.278262, 11.861750, 12.188803, 14.162312, 19.210462, 19.679889 | 19.679889 |
| repeat | compile-plus-runtime-suite | unsafe54-retained | 10.510067, 11.663068, 11.944842, 12.069055, 12.733725, 13.008772, 15.099031, 20.465037, 20.701618 | 20.701618 |
| repeat | compile | stock | 84.604300, 86.232890, 87.939594, 90.721369, 101.731260, 104.044468, 105.701404, 106.454608, 120.065062 | 120.065062 |
| repeat | compile-plus-runtime-suite | stock | 85.231748, 86.834285, 88.522422, 91.432331, 102.444209, 104.896144, 106.521607, 107.146602, 120.793242 | 120.793242 |
| noop | compile | unsafe54-direct | 201.238255, 212.430154, 213.007577, 246.552334, 260.957583, 262.491211, 264.810588, 290.832848, 312.742626 | 312.742626 |
| noop | compile-plus-runtime-suite | unsafe54-direct | 201.916146, 213.152299, 213.825337, 247.402146, 261.815387, 263.301335, 265.774127, 291.798956, 313.708393 | 313.708393 |
| noop | compile | unsafe54-retained | 10.463680, 11.811178, 12.141110, 12.521369, 13.496418, 13.816240, 17.199632, 18.027063, 18.822605 | 18.822605 |
| noop | compile-plus-runtime-suite | unsafe54-retained | 11.176445, 12.800217, 13.018152, 13.437638, 14.551719, 14.842974, 18.011584, 18.691660, 19.696056 | 19.696056 |
| noop | compile | stock | 85.773932, 88.746726, 89.672973, 99.773115, 101.569766, 104.739990, 105.055121, 108.515724, 118.431623 | 118.431623 |
| noop | compile-plus-runtime-suite | stock | 86.377401, 89.350282, 90.259966, 100.447448, 102.276082, 105.434825, 105.662323, 109.224074, 119.208985 | 119.208985 |
| body-edit | compile | unsafe54-direct | 203.018846, 216.681334, 220.617138, 249.268600, 258.626926, 266.077789, 266.701014, 284.335333, 328.628416 | 328.628416 |
| body-edit | compile-plus-runtime-suite | unsafe54-direct | 203.720814, 217.484425, 221.313698, 250.189312, 259.557612, 266.963508, 267.841398, 285.151997, 329.650013 | 329.650013 |
| body-edit | compile | unsafe54-retained | 328.081216, 330.986758, 364.235692, 419.570838, 422.560126, 428.442509, 436.578037, 446.276935, 474.521162 | 474.521162 |
| body-edit | compile-plus-runtime-suite | unsafe54-retained | 328.995971, 331.801730, 365.180741, 420.701714, 423.602314, 429.398653, 437.466044, 447.135756, 475.579880 | 475.579880 |
| body-edit | compile | stock | 89.490275, 91.982532, 97.893930, 98.962707, 99.056285, 103.193136, 106.639583, 107.495480, 126.540296 | 126.540296 |
| body-edit | compile-plus-runtime-suite | stock | 90.105035, 92.599633, 98.548856, 99.612226, 99.819520, 103.899896, 107.339490, 108.259308, 127.497534 | 127.497534 |
| signature-edit | compile | unsafe54-direct | 204.241222, 209.131159, 235.026002, 260.626468, 269.158348, 269.949432, 282.096546, 294.214344, 303.571904 | 303.571904 |
| signature-edit | compile-plus-runtime-suite | unsafe54-direct | 204.936103, 209.812176, 235.779982, 261.518551, 270.101580, 270.937424, 283.048117, 295.180012, 304.559979 | 304.559979 |
| signature-edit | compile | unsafe54-retained | 317.284656, 326.160486, 370.585455, 375.512173, 394.776153, 426.642378, 446.888732, 465.106093, 490.692123 | 490.692123 |
| signature-edit | compile-plus-runtime-suite | unsafe54-retained | 318.192639, 327.138905, 371.488929, 376.661273, 395.821428, 427.508756, 447.771077, 466.112754, 491.641870 | 491.641870 |
| signature-edit | compile | stock | 87.984861, 90.399355, 95.541285, 96.765851, 99.643943, 103.441551, 105.339444, 108.785504, 122.823767 | 122.823767 |
| signature-edit | compile-plus-runtime-suite | stock | 88.627626, 91.068114, 96.156758, 97.351850, 100.330004, 104.188554, 106.149196, 109.556111, 123.688131 | 123.688131 |
| type-edit | compile | unsafe54-direct | 201.168646, 211.411100, 222.366891, 226.622240, 259.747369, 270.861064, 277.523681, 293.074933, 378.580375 | 378.580375 |
| type-edit | compile-plus-runtime-suite | unsafe54-direct | 201.897158, 212.225137, 223.179872, 227.399330, 260.621002, 271.780439, 278.451270, 294.016519, 379.625425 | 379.625425 |
| type-edit | compile | unsafe54-retained | 323.736504, 324.442091, 343.419495, 353.934617, 404.579694, 429.949439, 445.687442, 447.785409, 616.898372 | 616.898372 |
| type-edit | compile-plus-runtime-suite | unsafe54-retained | 324.683897, 325.321672, 344.398157, 354.870348, 405.485190, 430.999977, 446.666332, 448.708517, 617.894976 | 617.894976 |
| type-edit | compile | stock | 86.445438, 91.292793, 92.495472, 93.588596, 100.476864, 102.915406, 106.941771, 112.332054, 131.643863 | 131.643863 |
| type-edit | compile-plus-runtime-suite | stock | 87.023459, 91.874110, 93.169001, 94.240675, 101.139158, 103.729445, 107.575343, 113.103587, 132.463708 | 132.463708 |
| macro-edit | compile | unsafe54-direct | 201.078769, 201.829451, 223.301143, 239.140607, 268.420336, 277.980157, 285.252667, 285.615741, 333.856539 | 333.856539 |
| macro-edit | compile-plus-runtime-suite | unsafe54-direct | 201.817025, 202.516330, 224.053241, 240.049917, 269.203099, 278.907802, 286.154999, 286.549016, 334.961039 | 334.961039 |
| macro-edit | compile | unsafe54-retained | 307.725454, 356.596218, 357.777339, 358.137691, 419.330361, 431.298717, 433.011293, 448.562930, 554.299302 | 554.299302 |
| macro-edit | compile-plus-runtime-suite | unsafe54-retained | 308.919375, 357.750598, 358.766394, 359.549620, 420.259702, 432.228868, 434.201460, 449.696186, 555.460389 | 555.460389 |
| macro-edit | compile | stock | 85.512142, 86.386838, 92.074451, 101.592715, 108.098286, 110.150509, 111.214315, 115.270089, 130.102705 | 130.102705 |
| macro-edit | compile-plus-runtime-suite | stock | 86.087453, 87.082103, 92.652440, 102.271766, 108.736708, 110.918977, 111.985220, 115.965930, 130.911214 | 130.911214 |
| rejection | compile | unsafe54-direct | 140.225879, 152.648807, 156.082471, 166.670828, 188.289535, 193.158528, 195.858559, 214.307896, 229.248228 | 229.248228 |
| rejection | compile | unsafe54-retained | 135.511863, 144.257123, 153.879504, 171.381473, 190.788442, 198.321734, 212.320805, 214.404623, 248.382886 | 248.382886 |
| rejection | compile | stock | 31.249015, 31.709981, 32.562253, 33.394202, 39.638020, 40.657225, 40.958253, 41.007577, 42.183607 | 42.183607 |
| recovery | compile | unsafe54-direct | 200.474354, 202.362576, 221.552303, 241.407045, 272.149616, 288.652502, 289.528261, 298.567753, 351.255287 | 351.255287 |
| recovery | compile-plus-runtime-suite | unsafe54-direct | 201.167321, 203.169515, 222.354064, 242.167791, 273.069172, 289.807684, 290.651704, 299.453896, 352.317493 | 352.317493 |
| recovery | compile | unsafe54-retained | 310.752923, 322.550907, 324.714114, 378.898820, 430.874067, 449.797247, 465.010631, 475.021556, 479.635291 | 479.635291 |
| recovery | compile-plus-runtime-suite | unsafe54-retained | 311.935586, 323.438867, 325.754121, 379.784530, 431.864208, 450.858681, 466.140056, 475.996723, 480.580138 | 480.580138 |
| recovery | compile | stock | 84.407057, 86.045814, 92.517501, 94.277098, 107.662401, 114.733774, 119.463252, 124.946716, 126.173932 | 126.173932 |
| recovery | compile-plus-runtime-suite | stock | 84.944317, 86.727573, 93.173873, 94.877176, 108.353591, 115.498654, 120.297317, 125.703118, 126.946952 | 126.946952 |

Full Rust and unsafe semantics, crates/metadata/rlib/Cargo, runtime-loader closure, supervised lifecycle and representative project speed remain OPEN. All seven public release gates are held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC/host linker remain dependencies. No all-Rust, whole-stack safety or universal speed claim.
