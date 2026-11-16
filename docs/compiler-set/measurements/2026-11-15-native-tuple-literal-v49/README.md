# Native original-AST tuple literals — component49

The complete original tuple-literal assertion SHA256 `969972eb87c7a664b6e7c54cd51f9cb92e871715ce04957a145042a055b3f61b` compiles, links and runs natively with exact stock runtime output. Copied48 refused it with L0809; the separate stock compile/runtime success is retained. No successful native48 speed baseline exists. Earlier complete original fixtures and protected Lust240e/Rust5b7b stay unchanged.

The owned formatter binds ordered original tuple elements and commas to their exact source spans, preserves structural and extra parentheses separately, and emits the pinned inconsistent-list box and singleton comma. Only derived panic literals change before full native rechecking, borrowing, generation and linking. No transformed AST survives requests; no protected printer copy, overlay, application rewrite, rustc native fallback or metadata fiction. See the [design and proof limits](../../proofs/native-assert-text.md).

Optimized direct25 and retained22 suites pass, followed by copied-component direct9, retained1 and host-input1 checks. A fresh actual gpt-6.1-sol HIGH/default R2 review plus MEDIUM/default R3 changed-test review supply the independent decision recorded with this campaign. Cases cover singleton/multiple elements, trailing commas, comments, wrappers, units, nested chains, closure/call contexts, broken layouts, order/once, borrows, Drop, unwind, const evaluation, invalid type/index/move/borrow, retained element/type/token/position/signature edits, rejection and recovery.

The first source review caught F49-001: new measurement arm names were missing from dispatch and summary aggregation. This was corrected before any V49 campaign ran. A second source review approved that correction. The first direct invocation then retained 24 passes and four harness failures: three separately configured product tests lacked their required package setting, and an invalid-borrow case reused an earlier successful case's output name. Both native and stock actually rejected that borrow. The corrected negative case has a unique output name and keeps the exit-1/no-artifact assertions. The component suite uses the same selection as V48; product34 remains separately retained. The third source review approved the isolated test correction. No compiler change or failed campaign was relabeled as success.

## Equivalent successful work

SAME successful optimized49 direct/retained and stock compile the same full generated mixed workload. Actual full native/stock compile/runtime preflight passed BEFORE timing; its bytes match timed sample0 cold. Nine rotated samples per phase/engine, new retained process per sample, native disk caches disabled before Check, OS pages untouched. Compiler opt3/CGU1/ThinLTO and application opt0/debug0/unwind/checks-on stay fixed. Compiler-build time is separate. Standalone no-op is not Cargo freshness; compile-plus-runtime is not Cargo test. Shared-host activity limits attribution.

No comparison with rejecting48 or earlier distinct sources establishes a gain. Cold/edit/rejection regressions against stock remain visible; bounded no-op improvement cannot establish full-project speed.

Median milliseconds, lower is better.

| Phase | Direct49 | Retained49 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 242.516 | 395.385 | 102.422 | 243.286 | 396.309 | 103.185 |
| repeat | 251.397 | 13.791 | 104.025 | 252.192 | 14.637 | 104.735 |
| noop | 247.613 | 12.216 | 102.032 | 248.483 | 13.216 | 102.848 |
| body-edit | 235.847 | 399.654 | 103.838 | 236.977 | 400.741 | 104.391 |
| signature-edit | 247.881 | 401.640 | 102.285 | 248.713 | 402.600 | 102.946 |
| type-edit | 254.373 | 405.141 | 102.570 | 255.378 | 406.201 | 103.376 |
| macro-edit | 264.675 | 411.628 | 103.869 | 265.661 | 412.671 | 104.654 |
| rejection | 173.548 | 177.941 | 37.596 | — | — | — |
| recovery | 255.893 | 402.641 | 106.604 | 256.713 | 403.796 | 107.320 |

Raw events: 757; SHA256 `7958939ac1c3e13583469428a409fdda84566fec9d67b0ce163367849c24f045`. Summary51 SHA256 `27e7444dde3a06de42e38b86c5cfa0570b46e0e8cb49bca9b5cc31f61ac0c3b2`. Core `c9abf1cf0ece502511f48eaf4848ea752c65304bea2e1b84565030ff0d6d3118`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 371.100069154s` is separate from application performance. Independent raw verification checks every group, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Case | Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | --- | ---: |
| mixed | cold | compile | tuple49-direct | 214.002744, 217.470334, 225.019712, 234.379487, 242.515906, 247.250107, 255.237374, 269.516900, 272.372946 | 272.372946 |
| mixed | cold | compile-plus-runtime-suite | tuple49-direct | 214.708828, 218.222545, 225.832531, 235.238878, 243.285880, 248.056539, 256.149067, 270.430098, 273.293848 | 273.293848 |
| mixed | cold | compile | tuple49-retained | 354.833916, 356.476171, 381.884384, 390.660617, 395.385289, 401.078673, 403.699938, 407.544783, 434.099316 | 434.099316 |
| mixed | cold | compile-plus-runtime-suite | tuple49-retained | 355.728058, 357.250995, 382.825606, 391.503564, 396.309267, 401.905293, 404.552784, 408.488300, 435.102059 | 435.102059 |
| mixed | cold | compile | stock | 92.671495, 95.742803, 95.766104, 100.960584, 102.421842, 104.451448, 105.357620, 107.529988, 115.543084 | 115.543084 |
| mixed | cold | compile-plus-runtime-suite | stock | 93.304146, 96.366556, 96.408387, 101.571208, 103.184688, 105.189523, 106.006727, 108.197299, 116.260194 | 116.260194 |
| mixed | repeat | compile | tuple49-direct | 215.979679, 216.617999, 241.078492, 247.100782, 251.396902, 259.145013, 260.410553, 261.211286, 271.086308 | 271.086308 |
| mixed | repeat | compile-plus-runtime-suite | tuple49-direct | 216.834014, 217.406002, 241.934882, 247.946416, 252.191619, 260.038409, 261.404396, 262.017934, 272.001741 | 272.001741 |
| mixed | repeat | compile | tuple49-retained | 11.046654, 11.387398, 12.779494, 12.920130, 13.790594, 14.073543, 16.843972, 18.027596, 18.818853 | 18.818853 |
| mixed | repeat | compile-plus-runtime-suite | tuple49-retained | 11.791597, 12.144536, 13.639835, 13.737597, 14.636760, 15.060565, 17.764285, 18.954009, 19.784265 | 19.784265 |
| mixed | repeat | compile | stock | 88.750314, 91.252707, 97.265139, 102.124411, 104.025271, 106.845579, 111.035854, 111.326111, 112.165450 | 112.165450 |
| mixed | repeat | compile-plus-runtime-suite | stock | 89.501667, 91.838191, 97.989750, 102.735961, 104.734725, 107.602342, 111.857343, 112.066600, 112.961523 | 112.961523 |
| mixed | noop | compile | tuple49-direct | 209.147275, 230.861657, 239.812162, 244.280512, 247.613015, 247.982854, 253.086957, 277.291147, 294.999320 | 294.999320 |
| mixed | noop | compile-plus-runtime-suite | tuple49-direct | 209.866308, 231.640333, 240.687048, 245.191137, 248.482646, 248.808129, 253.952858, 278.229406, 295.876682 | 295.876682 |
| mixed | noop | compile | tuple49-retained | 10.467500, 11.585571, 11.587950, 11.838033, 12.215987, 12.687258, 16.687647, 19.308280, 19.476567 | 19.476567 |
| mixed | noop | compile-plus-runtime-suite | tuple49-retained | 11.182711, 12.388714, 12.412192, 12.741734, 13.215642, 13.522051, 17.635609, 20.240203, 20.401881 | 20.401881 |
| mixed | noop | compile | stock | 89.780325, 96.020488, 96.593042, 101.587343, 102.031820, 102.446298, 102.855221, 105.465008, 109.705309 | 109.705309 |
| mixed | noop | compile-plus-runtime-suite | stock | 90.423308, 96.658805, 97.159279, 102.212693, 102.848306, 103.094086, 103.505858, 106.197891, 110.697370 | 110.697370 |
| mixed | body-edit | compile | tuple49-direct | 203.948202, 211.479815, 232.829255, 234.630273, 235.846558, 239.909339, 270.668441, 276.364858, 301.829680 | 301.829680 |
| mixed | body-edit | compile-plus-runtime-suite | tuple49-direct | 204.760034, 212.219206, 233.680712, 235.526589, 236.976548, 240.704702, 271.583735, 277.262424, 302.739100 | 302.739100 |
| mixed | body-edit | compile | tuple49-retained | 346.185681, 347.445468, 364.405294, 367.902455, 399.654147, 401.546994, 403.877406, 426.243068, 455.042487 | 455.042487 |
| mixed | body-edit | compile-plus-runtime-suite | tuple49-retained | 347.060505, 348.417360, 365.176286, 368.732168, 400.741438, 402.767218, 404.703334, 427.187945, 456.090291 | 456.090291 |
| mixed | body-edit | compile | stock | 90.119160, 91.681797, 98.745580, 101.013420, 103.837627, 104.786988, 106.389049, 109.329955, 116.407461 | 116.407461 |
| mixed | body-edit | compile-plus-runtime-suite | stock | 90.710291, 92.329075, 99.401813, 101.835568, 104.390584, 105.553332, 107.098949, 110.052770, 117.292757 | 117.292757 |
| mixed | signature-edit | compile | tuple49-direct | 195.841294, 219.840987, 221.779768, 237.511748, 247.880537, 249.892477, 253.315855, 270.894002, 292.714207 | 292.714207 |
| mixed | signature-edit | compile-plus-runtime-suite | tuple49-direct | 196.634262, 220.686008, 222.524548, 238.516202, 248.712869, 250.826528, 254.137482, 271.763921, 293.624766 | 293.624766 |
| mixed | signature-edit | compile | tuple49-retained | 332.752613, 333.548543, 353.934557, 388.931463, 401.640264, 409.306273, 422.462945, 424.215008, 456.266975 | 456.266975 |
| mixed | signature-edit | compile-plus-runtime-suite | tuple49-retained | 333.649000, 334.366742, 354.725053, 389.720202, 402.599741, 410.307221, 423.399722, 425.087465, 457.283412 | 457.283412 |
| mixed | signature-edit | compile | stock | 86.362576, 92.341395, 98.841743, 100.344970, 102.285068, 103.951993, 111.393237, 116.561523, 118.167396 | 118.167396 |
| mixed | signature-edit | compile-plus-runtime-suite | stock | 87.017028, 92.995903, 99.495938, 101.142381, 102.945523, 104.656982, 112.160397, 117.334072, 118.899555 | 118.899555 |
| mixed | type-edit | compile | tuple49-direct | 203.674496, 203.777178, 218.776030, 225.159171, 254.373015, 257.361636, 270.841941, 313.588961, 332.521137 | 332.521137 |
| mixed | type-edit | compile-plus-runtime-suite | tuple49-direct | 204.323025, 204.584777, 219.555408, 225.914204, 255.378434, 258.194166, 271.725539, 314.497253, 333.451414 | 333.451414 |
| mixed | type-edit | compile | tuple49-retained | 303.985286, 312.940528, 338.711696, 386.287641, 405.140891, 441.674855, 446.165953, 482.417141, 510.751524 | 510.751524 |
| mixed | type-edit | compile-plus-runtime-suite | tuple49-retained | 304.718514, 314.061607, 339.509758, 387.292894, 406.200613, 442.644105, 447.147785, 483.510076, 511.736461 | 511.736461 |
| mixed | type-edit | compile | stock | 85.222709, 87.091456, 94.493716, 94.950383, 102.569695, 107.443671, 110.039798, 111.826388, 123.105268 | 123.105268 |
| mixed | type-edit | compile-plus-runtime-suite | stock | 85.749807, 87.841829, 95.174268, 95.527062, 103.376490, 108.166480, 110.820735, 112.403969, 124.021100 | 124.021100 |
| mixed | macro-edit | compile | tuple49-direct | 207.179941, 216.920462, 221.698926, 240.721340, 264.675181, 272.344342, 282.034319, 310.994970, 335.893603 | 335.893603 |
| mixed | macro-edit | compile-plus-runtime-suite | tuple49-direct | 207.916931, 217.628792, 222.493555, 241.579033, 265.661018, 273.357344, 283.024505, 312.099915, 336.913603 | 336.913603 |
| mixed | macro-edit | compile | tuple49-retained | 331.413746, 347.668414, 380.190733, 386.124163, 411.628273, 431.983602, 445.189923, 453.825954, 514.669867 | 514.669867 |
| mixed | macro-edit | compile-plus-runtime-suite | tuple49-retained | 332.163820, 348.557523, 381.046556, 387.308022, 412.670900, 432.905368, 446.005649, 454.716089, 515.708485 | 515.708485 |
| mixed | macro-edit | compile | stock | 89.278160, 91.002419, 97.849265, 98.483628, 103.869044, 110.532767, 112.882462, 114.112684, 136.097551 | 136.097551 |
| mixed | macro-edit | compile-plus-runtime-suite | stock | 89.894930, 91.701910, 99.191040, 99.602916, 104.654246, 111.232898, 113.614139, 114.815165, 136.927351 | 136.927351 |
| mixed | rejection | compile | tuple49-direct | 147.112190, 147.641575, 165.769753, 172.070535, 173.547908, 183.658006, 189.023771, 204.163024, 205.405842 | 205.405842 |
| mixed | rejection | compile | tuple49-retained | 149.568287, 151.810891, 165.401210, 171.412502, 177.941293, 182.228214, 197.687786, 204.492461, 238.943442 | 238.943442 |
| mixed | rejection | compile | stock | 33.211401, 33.350789, 34.638826, 35.724276, 37.595975, 38.365416, 38.915624, 39.412635, 41.773957 | 41.773957 |
| mixed | recovery | compile | tuple49-direct | 223.573074, 230.007961, 241.434777, 241.489776, 255.892624, 258.252597, 264.651068, 265.631388, 278.281568 | 278.281568 |
| mixed | recovery | compile-plus-runtime-suite | tuple49-direct | 224.266599, 230.858774, 242.338192, 242.560188, 256.712838, 259.049259, 265.555454, 266.483335, 279.219636 | 279.219636 |
| mixed | recovery | compile | tuple49-retained | 357.886094, 359.261850, 373.413562, 375.541453, 402.641470, 406.932420, 409.776512, 421.863343, 424.503372 | 424.503372 |
| mixed | recovery | compile-plus-runtime-suite | tuple49-retained | 358.758526, 360.191050, 374.221074, 376.695948, 403.795709, 407.983156, 410.629654, 422.823356, 425.515502 | 425.515502 |
| mixed | recovery | compile | stock | 92.828427, 93.629823, 97.116897, 98.139874, 106.604038, 107.392210, 107.959618, 111.069677, 116.734934 | 116.734934 |
| mixed | recovery | compile-plus-runtime-suite | stock | 93.539652, 94.256340, 97.771256, 98.877838, 107.320014, 108.110818, 108.686718, 111.760569, 117.592412 | 117.592412 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
