# Native safe Rust calls in unsafe bodies — component55

The complete original safe-call witness `99baf0de2a05ea298229c81007a12511c1d5e80d03d3c7b9d7fd596eafa48711` compiles and executes natively with stock output, original assertion text and Guard/drop behavior. Component54 rejected it at collection; that rejection is not a successful timing baseline.

The owned grammar records deferred calls and uses native checking in the real enclosing function to resolve each exact call. A direct, non-generic safe Rust target with a body and boolean inputs/output must qualify its actual safety, ABI, async/variadic and owner facts. Unknown targets refuse. Unrelated conversion-constraint diagnostics reach the complete production native Check, which retains the errors and failed program for existing bounded inference. Every recollection repeats call qualification. Complete checking, lowering, borrow checking and generation remain required before acceptance; source, original AST, scopes and arena IDs remain intact.

Fresh Sol R1 found that early whole-function query errors blocked existing delayed conversion inference in the same function. The native failure was reproduced and retained before the fix. Fresh R2 source review approved the correction; combined calls before/after conversion, move/shadow rejection and retained recovery now pass. A nested function item exposed a distinct parser gap and remains explicitly unsupported. First failures are retained. Actual unsafe/foreign/indirect/generic calls and broader grammar remain open.

Optimized32 direct and29 retained tests pass. Copied artifacts pass15 assertion tests, a separate combined direct test, old/local/call/combined retained tests and host-input invalidation. Full generated native/stock compile/runtime preflight precedes timing and equals timed sample0 cold.

## Equivalent successful work

SAME successful55 optimized direct/retained/stock; nine rotated samples per phase. Compiler opt3/CGU1/ThinLTO; applications opt0/debug0/unwind/checks-on. Native disk caches disabled before Check; fresh bounded retained process per sample; OS pages untouched. Shared-host noise limits attribution. Compiler-build cost is separate. Standalone no-op is not Cargo freshness, and compile-plus-runtime is not Cargo test. No gain is claimed against rejecting54 or distinct earlier sources. Cold/edit/rejection regressions are retained.

Median milliseconds, lower is better.

| Phase | Direct55 | Retained55 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 239.353 | 343.074 | 94.350 | 240.198 | 343.751 | 94.949 |
| repeat | 207.980 | 12.102 | 93.040 | 208.715 | 12.927 | 93.726 |
| noop | 205.335 | 12.653 | 86.415 | 206.088 | 13.499 | 87.041 |
| body-edit | 208.850 | 324.637 | 88.040 | 209.598 | 325.433 | 88.619 |
| signature-edit | 203.201 | 344.788 | 91.136 | 204.017 | 345.724 | 91.727 |
| type-edit | 229.886 | 364.292 | 98.346 | 230.705 | 365.259 | 99.001 |
| macro-edit | 236.443 | 398.065 | 103.299 | 237.332 | 399.003 | 104.026 |
| rejection | 182.725 | 182.720 | 36.256 | — | — | — |
| recovery | 267.988 | 416.242 | 104.720 | 268.956 | 417.235 | 105.499 |

Raw757 events SHA256 `dd6579c654933e36092ef94e40403ca66d057aba598d99b9b1529e6873f80163`; summary SHA256 `1c832c079ef4bbe2a94954a9f16ac82b904070fad09c886123ceed50a28c29a9`; core `d7f7efb0a807f7613c12e3c7d787472f7b68c83a893204612aba6d439943a703`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Independent verification checks all51 groups, unique samples0..8, sorted values, medians/maxima,81 actual direct commands,81 retained calls/18 hits and81 stock commands. Accepted compiler build: `exit status: 0; elapsed 370.750509554s`. R1 compiler build was an unaccepted qualification attempt, separate from application timing.

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | call55-direct | 186.206360, 199.245604, 201.815083, 214.496523, 239.353354, 242.699675, 258.869053, 282.479196, 288.484215 | 288.484215 |
| cold | compile-plus-runtime-suite | call55-direct | 186.793064, 199.916745, 202.472710, 215.315560, 240.198033, 243.614277, 259.768332, 283.407016, 289.420807 | 289.420807 |
| cold | compile | call55-retained | 300.343697, 303.970892, 317.458763, 322.709620, 343.074297, 418.601910, 419.432583, 457.167670, 457.991302 | 457.991302 |
| cold | compile-plus-runtime-suite | call55-retained | 301.025178, 304.796515, 318.220763, 324.105045, 343.750908, 419.918333, 420.392468, 458.237609, 459.097414 | 459.097414 |
| cold | compile | stock | 86.147363, 86.265116, 86.739910, 87.532789, 94.350068, 105.080628, 109.269142, 110.447420, 117.736987 | 117.736987 |
| cold | compile-plus-runtime-suite | stock | 86.703353, 86.830635, 87.372069, 88.118588, 94.948702, 105.785343, 110.081308, 111.158693, 118.488182 | 118.488182 |
| repeat | compile | call55-direct | 193.952685, 198.329422, 201.721315, 205.491080, 207.980208, 245.024461, 261.533278, 262.033872, 285.604008 | 285.604008 |
| repeat | compile-plus-runtime-suite | call55-direct | 194.642481, 199.018784, 202.500136, 206.406874, 208.715050, 245.780020, 262.404184, 262.976869, 286.533134 | 286.533134 |
| repeat | compile | call55-retained | 10.086511, 10.178985, 10.484313, 10.677708, 12.101514, 12.162770, 12.392376, 12.416907, 17.340260 | 17.340260 |
| repeat | compile-plus-runtime-suite | call55-retained | 10.791206, 10.874685, 11.309441, 11.426483, 12.927123, 13.073814, 13.173972, 13.321161, 18.261513 | 18.261513 |
| repeat | compile | stock | 83.889633, 85.985414, 87.001979, 87.224052, 93.039939, 95.802488, 104.373547, 107.672683, 109.844377 | 109.844377 |
| repeat | compile-plus-runtime-suite | stock | 84.502540, 86.591367, 87.571075, 87.843330, 93.725613, 96.430881, 105.121978, 108.480986, 110.526504 | 110.526504 |
| noop | compile | call55-direct | 192.519829, 193.700524, 196.012598, 201.957894, 205.334714, 255.899263, 259.723675, 267.905567, 274.986952 | 274.986952 |
| noop | compile-plus-runtime-suite | call55-direct | 193.290555, 194.468851, 196.797323, 202.659918, 206.087603, 256.683778, 260.605336, 268.789030, 275.720523 | 275.720523 |
| noop | compile | call55-retained | 9.622962, 9.643358, 11.705828, 11.821265, 12.653393, 15.060165, 16.574493, 16.781080, 18.542745 | 18.542745 |
| noop | compile-plus-runtime-suite | call55-retained | 10.403191, 10.420688, 12.584400, 12.600251, 13.499404, 15.715515, 17.450207, 17.512384, 19.494750 | 19.494750 |
| noop | compile | stock | 84.499919, 84.942308, 86.240084, 86.337689, 86.414657, 100.141178, 107.617002, 108.345530, 109.370209 | 109.370209 |
| noop | compile-plus-runtime-suite | stock | 85.129864, 85.746503, 86.861683, 86.905472, 87.041425, 100.882911, 108.345606, 109.158312, 110.118944 | 110.118944 |
| body-edit | compile | call55-direct | 192.924833, 200.149470, 205.390968, 205.995543, 208.849516, 261.136661, 263.898489, 273.447343, 277.398296 | 277.398296 |
| body-edit | compile-plus-runtime-suite | call55-direct | 193.595349, 200.930516, 206.091940, 206.840367, 209.597797, 262.054539, 264.905890, 274.320713, 278.295962 | 278.295962 |
| body-edit | compile | call55-retained | 307.949629, 318.449370, 319.227839, 320.320683, 324.636833, 399.396784, 418.779019, 426.703189, 429.137108 | 429.137108 |
| body-edit | compile-plus-runtime-suite | call55-retained | 308.670108, 319.203291, 320.092703, 321.160677, 325.432737, 400.675752, 419.912190, 427.637878, 430.078485 | 430.078485 |
| body-edit | compile | stock | 82.467327, 86.821495, 87.609336, 87.868726, 88.040433, 105.886479, 106.055713, 107.616496, 112.664676 | 112.664676 |
| body-edit | compile-plus-runtime-suite | stock | 83.063005, 87.341216, 88.196865, 88.456237, 88.618684, 106.600255, 106.809810, 108.359658, 113.439920 | 113.439920 |
| signature-edit | compile | call55-direct | 194.218715, 194.785315, 199.596690, 202.941743, 203.201121, 261.547387, 268.708813, 269.622767, 275.284367 | 275.284367 |
| signature-edit | compile-plus-runtime-suite | call55-direct | 194.846751, 195.430943, 200.372336, 203.651211, 204.017084, 262.432726, 269.658731, 270.492501, 276.138553 | 276.138553 |
| signature-edit | compile | call55-retained | 297.005857, 304.738928, 309.747467, 324.750805, 344.787915, 392.179109, 394.763377, 401.099236, 403.436911 | 403.436911 |
| signature-edit | compile-plus-runtime-suite | call55-retained | 298.039393, 305.463436, 310.478879, 325.478973, 345.723877, 392.976202, 395.736021, 401.935002, 404.406433 | 404.406433 |
| signature-edit | compile | stock | 85.325979, 85.966473, 86.109673, 86.383000, 91.135614, 101.577066, 101.959454, 102.197167, 105.649708 | 105.649708 |
| signature-edit | compile-plus-runtime-suite | stock | 85.852905, 86.513974, 86.795243, 86.973915, 91.727449, 102.341191, 102.727852, 103.462822, 106.384207 | 106.384207 |
| type-edit | compile | call55-direct | 197.050541, 198.526564, 199.351836, 200.274645, 229.885649, 246.124997, 264.546785, 277.324296, 278.004610 | 278.004610 |
| type-edit | compile-plus-runtime-suite | call55-direct | 197.747891, 199.245906, 200.032526, 201.020891, 230.705280, 246.885008, 265.352363, 278.283006, 278.838664 | 278.838664 |
| type-edit | compile | call55-retained | 305.539843, 312.979208, 319.626704, 319.652169, 364.291766, 393.715720, 404.643737, 416.174274, 436.625442 | 436.625442 |
| type-edit | compile-plus-runtime-suite | call55-retained | 306.212225, 313.724777, 320.431827, 320.460413, 365.259173, 394.516982, 405.713811, 417.079614, 437.712953 | 437.712953 |
| type-edit | compile | stock | 83.739901, 84.745978, 84.843457, 84.980231, 98.345619, 104.772698, 107.511773, 110.709539, 112.663471 | 112.663471 |
| type-edit | compile-plus-runtime-suite | stock | 84.406336, 85.308579, 85.416896, 85.641735, 99.001378, 105.617925, 108.184898, 111.383657, 113.438435 | 113.438435 |
| macro-edit | compile | call55-direct | 190.559091, 196.791740, 206.675951, 208.172734, 236.442571, 262.902165, 267.433041, 288.443388, 296.342841 | 296.342841 |
| macro-edit | compile-plus-runtime-suite | call55-direct | 191.160345, 197.461126, 207.408573, 208.973572, 237.332449, 263.965727, 268.309961, 289.339611, 297.285778 | 297.285778 |
| macro-edit | compile | call55-retained | 291.719758, 313.090266, 330.456071, 341.212253, 398.065189, 420.303752, 427.527646, 447.459241, 451.544780 | 451.544780 |
| macro-edit | compile-plus-runtime-suite | call55-retained | 292.411132, 314.139229, 331.753092, 342.118277, 399.002838, 421.264780, 428.697867, 448.394136, 452.458289 | 452.458289 |
| macro-edit | compile | stock | 80.939003, 86.428388, 87.003786, 89.894561, 103.298782, 104.404576, 105.889251, 108.657967, 111.468335 | 111.468335 |
| macro-edit | compile-plus-runtime-suite | stock | 81.543210, 87.026614, 87.691285, 90.574186, 104.025615, 105.061054, 106.574820, 109.404662, 112.310772 | 112.310772 |
| rejection | compile | call55-direct | 132.738145, 138.359340, 140.135580, 149.363221, 182.725474, 192.767778, 194.064551, 200.104492, 206.918039 | 206.918039 |
| rejection | compile | call55-retained | 134.600826, 140.683957, 141.962820, 152.383145, 182.720457, 202.510398, 207.129853, 214.181759, 221.864182 | 221.864182 |
| rejection | compile | stock | 30.409591, 31.212890, 32.509759, 34.764836, 36.255589, 37.312988, 38.345211, 41.001709, 41.222783 | 41.222783 |
| recovery | compile | call55-direct | 193.490630, 200.065804, 209.228817, 231.690866, 267.987959, 281.981849, 290.170997, 290.727488, 301.368850 | 301.368850 |
| recovery | compile-plus-runtime-suite | call55-direct | 194.158150, 200.755545, 210.062153, 232.556630, 268.956221, 282.920931, 291.180807, 291.621744, 302.290704 | 302.290704 |
| recovery | compile | call55-retained | 292.689866, 299.312373, 322.770647, 353.924876, 416.241643, 418.710040, 438.221077, 439.701144, 463.455279 | 463.455279 |
| recovery | compile-plus-runtime-suite | call55-retained | 293.692030, 300.136991, 323.669602, 354.624218, 417.235081, 419.908861, 439.412502, 440.543058, 464.348196 | 464.348196 |
| recovery | compile | stock | 81.258217, 87.532220, 92.246855, 95.606667, 104.720012, 106.299113, 106.698654, 110.421729, 128.419484 | 128.419484 |
| recovery | compile-plus-runtime-suite | stock | 81.892749, 88.134471, 92.801638, 96.268501, 105.498741, 107.103935, 107.395775, 111.261521, 129.196555 | 129.196555 |

Full Rust/unsafe semantics, crates/metadata/rlib/Cargo, runtime-loader closure, supervised lifecycle and representative unchanged-project speed remain OPEN. All seven public release gates are held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC/host linker remain dependencies. No all-Rust, whole-toolchain memory safety or universal speed claim.
