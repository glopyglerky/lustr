# Native shared slice/array assertions — component42

The full original mixed source SHA256 `328e723ef884df4cd62140a65ec97edf31ceffe862db87b755b2dc7f85150373` is unchanged and now compiles, links and runs natively. The actual copied41 L0110 failures and stock runtime witness remain preserved. This extends native checking through owned AST elaboration; protected Lust and Rust trees remain unchanged.

Checked shared array-reference rvalues against shared slices of the same concrete element type receive an array-to-slice coercion inside the original assertion macro. Canonical element equality and Debug must exist. The original macro retains operand order/evaluation, borrowing, failure-only messages, Debug formatting and panic behavior. Full native collect/check/borrowck/codegen/link follows; the existing two-round bound remains. Array-reference places, unresolved/generic/mutable/heterogeneous forms remain refused. Those places need their original macro borrow preserved before support can be admitted. No source-to-source rewriting, assert_eq-to-assert substitution, compatibility compiler or forged metadata supplies acceptance.

Qualification covers exact original mixed source; arrays/closures and prior conversions; both operand directions; zero and different lengths; bool/char/f64; custom element PartialEq/Debug/Drop; lazy message side effects; exact panic payload text, evaluation order and unwind drops. Invalid element types, ownership/lifetimes, aliasing, absent/ambiguous/lookalike traits, array-length typing and reference places/packed named and tuple fields remain rejected. Retained original-source edits (operand form, length, element type and values), rejection and recovery match cold native and stock. No transformed AST survives requests.

First development run failed overall on the newly written semantics test's three long message-free outer assert! conditions (L0809). The exact first source, native first diagnostics and stock compile/run success remain. Explicit messages were added only to those three new test-harness assertions; original full mixed source remains unchanged. L0809 stays OPEN. Fresh Sol review R1 found F42-01: TupleField was missing from the place exclusion. Its source, review and pre-fix build receipt are retained. Root added the refusal and two rejection guards, then rebuilt and requalified the corrected candidate. Fresh frozen R2 review and actual optimized/copy receipts are separate evidence. Root gpt-6.1-sol xhigh/default authored code/helpers/report; Sol high/default reviewed through T3 Code -> Codex. No Claude used.

## Successful equivalent workload

Nine samples/engine/phase on shared Linux GNU x86_64 NAS CPUs10-13/jobs2. SAME successful42 direct and retained plus pinned stock rotate every phase/sample. Native compiler opt3/CGU1/ThinLTO/line tables is unchanged; application opt0/debug0/unwind/overflow and debug checks on. Fresh retained process/sample, native disk cache disabled before Check, OS pages untouched. Exact source snapshots, command/identity/status/runtime/stdout/stderr and raw timings remain. ELF output and matching runtime are required; rejection must return status1. Stock recompiles standalone source each phase: its repeat/noop is not Cargo freshness. Compile-plus-runtime is actual execution, not Cargo test.

This language expansion has measured costs. Rejecting41 and distinct earlier workloads are not successful speed baselines. Cold/edit/rejection regressions against stock remain visible; unchanged artifact proof is a bounded metric. Full-project native speed remains OPEN.

Median milliseconds, lower is better.

| Phase | Direct42 | Retained42 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 257.272 | 423.946 | 103.693 | 258.255 | 425.090 | 104.375 |
| repeat | 247.750 | 12.892 | 99.990 | 248.589 | 13.794 | 100.759 |
| noop | 251.296 | 15.379 | 102.836 | 252.119 | 16.423 | 103.561 |
| body-edit | 255.199 | 425.959 | 103.161 | 256.183 | 426.832 | 103.871 |
| signature-edit | 250.132 | 384.281 | 94.155 | 250.897 | 385.054 | 94.860 |
| type-edit | 252.329 | 381.659 | 95.165 | 253.135 | 382.573 | 95.800 |
| macro-edit | 263.081 | 410.330 | 97.729 | 263.927 | 411.221 | 98.383 |
| rejection | 177.750 | 176.421 | 36.607 | — | — | — |
| recovery | 266.962 | 396.045 | 102.066 | 267.812 | 397.021 | 102.736 |

Raw757 SHA256 `68f10fb036576fdfa80bc60d0a915d8779edb42d3204db4b66c31486e731ac8f`; summary51 SHA256 `8d1bace1526e14f597169f5bf93e16005f5d8e389d8c4965edfa99e29649eb89`. Core `686c5a4c393f01acae99ef1c9ccd30ebbcb2fa7ac0092e09e9431d3666eacd68`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Corrected optimized compiler BUILD receipt `exit status: 0; elapsed 351.025130208s` is separate from application performance.

## Complete samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | assert42-direct | 203.907064, 237.165330, 243.359904, 256.710172, 257.272001, 265.485631, 284.896617, 285.776188, 299.442195 | 299.442195 |
| cold | compile-plus-runtime-suite | assert42-direct | 204.687716, 237.924183, 244.168606, 257.609372, 258.254690, 266.438340, 285.904240, 286.784444, 300.439197 | 300.439197 |
| cold | compile | assert42-retained | 343.631843, 368.942607, 417.346826, 418.946737, 423.946433, 430.431679, 445.803759, 453.933102, 457.775466 | 457.775466 |
| cold | compile-plus-runtime-suite | assert42-retained | 344.432836, 369.803825, 418.174490, 420.029603, 425.090296, 431.348683, 446.747605, 454.857719, 458.810085 | 458.810085 |
| cold | compile | stock | 95.090764, 96.229318, 102.215847, 103.317710, 103.693457, 107.718271, 110.358054, 111.051968, 112.588056 | 112.588056 |
| cold | compile-plus-runtime-suite | stock | 95.758809, 96.952791, 103.032818, 104.011834, 104.374957, 108.413098, 111.101400, 111.742655, 113.464724 | 113.464724 |
| repeat | compile | assert42-direct | 202.124694, 208.708218, 233.560631, 244.742232, 247.750428, 265.618921, 266.814689, 285.929435, 336.974938 | 336.974938 |
| repeat | compile-plus-runtime-suite | assert42-direct | 202.824640, 209.500585, 234.323677, 245.634725, 248.589070, 266.515872, 267.726552, 286.810556, 337.959971 | 337.959971 |
| repeat | compile | assert42-retained | 10.782763, 11.031943, 12.724404, 12.856331, 12.892245, 13.467231, 13.934342, 16.872311, 23.894632 | 23.894632 |
| repeat | compile-plus-runtime-suite | assert42-retained | 11.586220, 11.767249, 13.588253, 13.788655, 13.794295, 14.363793, 14.940550, 17.659591, 24.910100 | 24.910100 |
| repeat | compile | stock | 86.318616, 87.972851, 97.910762, 98.757844, 99.990108, 106.073733, 106.664902, 108.164868, 118.364842 | 118.364842 |
| repeat | compile-plus-runtime-suite | stock | 86.936546, 88.638944, 98.653152, 99.525538, 100.758791, 106.871447, 107.409214, 108.921970, 119.147689 | 119.147689 |
| noop | compile | assert42-direct | 210.324680, 213.391541, 238.705507, 239.510303, 251.296004, 254.484573, 281.640065, 308.876543, 330.383954 | 330.383954 |
| noop | compile-plus-runtime-suite | assert42-direct | 211.104497, 214.196511, 239.673751, 240.192541, 252.118544, 255.380856, 282.490292, 309.932291, 331.559119 | 331.559119 |
| noop | compile | assert42-retained | 10.472840, 11.273067, 11.401554, 12.164055, 15.379386, 17.146593, 18.509772, 19.405190, 23.734364 | 23.734364 |
| noop | compile-plus-runtime-suite | assert42-retained | 11.541961, 12.216463, 12.282475, 13.033578, 16.422966, 17.877186, 19.583498, 20.212246, 25.513867 | 25.513867 |
| noop | compile | stock | 85.507151, 89.084232, 92.843119, 98.988642, 102.835965, 103.852101, 107.474384, 111.269613, 121.641440 | 121.641440 |
| noop | compile-plus-runtime-suite | stock | 86.101116, 89.756762, 93.425772, 99.672938, 103.560696, 104.538526, 108.241628, 112.001378, 122.448499 | 122.448499 |
| body-edit | compile | assert42-direct | 210.902335, 211.717145, 214.588725, 244.056041, 255.199473, 255.975946, 289.035433, 331.901684, 367.395668 | 367.395668 |
| body-edit | compile-plus-runtime-suite | assert42-direct | 211.748849, 212.665895, 215.388559, 244.907142, 256.183174, 256.991447, 290.044510, 332.814013, 368.402564 | 368.402564 |
| body-edit | compile | assert42-retained | 318.812824, 349.953827, 357.467404, 413.803198, 425.958517, 450.137091, 499.829434, 513.579563, 529.155448 | 529.155448 |
| body-edit | compile-plus-runtime-suite | assert42-retained | 319.574103, 350.745848, 358.313858, 415.034905, 426.831728, 451.180877, 501.032285, 514.838962, 530.076511 | 530.076511 |
| body-edit | compile | stock | 86.291454, 88.063848, 91.102039, 100.168488, 103.160654, 106.209021, 107.587839, 115.208848, 126.265108 | 126.265108 |
| body-edit | compile-plus-runtime-suite | stock | 86.948797, 88.680450, 91.703463, 100.900884, 103.871428, 107.004785, 108.401011, 116.119246, 127.124763 | 127.124763 |
| signature-edit | compile | assert42-direct | 215.393167, 224.384043, 238.433366, 239.931648, 250.132256, 273.066331, 284.428725, 285.432820, 306.581775 | 306.581775 |
| signature-edit | compile-plus-runtime-suite | assert42-direct | 216.281524, 225.077893, 239.313280, 240.726844, 250.897211, 274.011649, 285.440392, 286.545991, 307.711383 | 307.711383 |
| signature-edit | compile | assert42-retained | 347.544865, 350.095925, 365.494848, 383.626899, 384.280785, 416.695195, 434.738566, 491.451651, 498.525343 | 498.525343 |
| signature-edit | compile-plus-runtime-suite | assert42-retained | 348.360119, 350.953041, 366.303212, 384.515690, 385.054446, 417.783337, 435.915424, 492.472982, 499.507281 | 499.507281 |
| signature-edit | compile | stock | 90.156800, 90.598358, 91.088653, 94.055508, 94.154515, 102.873630, 110.835381, 114.456091, 121.841632 | 121.841632 |
| signature-edit | compile-plus-runtime-suite | stock | 90.809851, 91.195389, 91.716130, 94.704240, 94.859701, 103.537232, 111.623866, 115.263166, 122.676300 | 122.676300 |
| type-edit | compile | assert42-direct | 197.137091, 210.381016, 231.576642, 250.817613, 252.328750, 252.361805, 258.927660, 291.291207, 317.120137 | 317.120137 |
| type-edit | compile-plus-runtime-suite | assert42-direct | 197.788342, 211.106374, 232.237689, 251.751450, 253.135380, 253.172956, 259.799276, 292.263657, 318.004439 | 318.004439 |
| type-edit | compile | assert42-retained | 322.730058, 330.652074, 339.265190, 371.707262, 381.658732, 384.666070, 426.122707, 441.676612, 474.567535 | 474.567535 |
| type-edit | compile-plus-runtime-suite | assert42-retained | 324.062752, 331.740787, 340.178024, 372.562964, 382.572708, 385.327459, 427.016137, 442.656982, 475.550264 | 475.550264 |
| type-edit | compile | stock | 87.836172, 90.715321, 94.033840, 94.264581, 95.164865, 97.188208, 101.896096, 114.751509, 118.677166 | 118.677166 |
| type-edit | compile-plus-runtime-suite | stock | 88.417574, 91.371731, 94.746225, 94.941542, 95.800407, 97.889948, 102.688601, 115.595976, 119.621653 | 119.621653 |
| macro-edit | compile | assert42-direct | 210.677073, 236.770026, 254.383609, 257.623779, 263.081298, 266.505702, 283.461343, 289.021026, 292.936598 | 292.936598 |
| macro-edit | compile-plus-runtime-suite | assert42-direct | 211.433918, 237.675698, 255.160461, 258.644730, 263.926901, 267.405179, 284.364960, 289.925377, 293.865840 | 293.865840 |
| macro-edit | compile | assert42-retained | 337.180175, 376.718137, 382.903450, 403.921502, 410.330191, 420.420079, 431.382465, 431.660932, 502.406740 | 502.406740 |
| macro-edit | compile-plus-runtime-suite | assert42-retained | 338.382777, 377.498684, 383.773914, 404.852009, 411.220796, 421.660065, 432.302751, 432.595909, 503.547645 | 503.547645 |
| macro-edit | compile | stock | 85.075345, 93.932647, 96.653553, 97.275067, 97.728613, 104.696196, 104.880943, 114.503333, 117.586369 | 117.586369 |
| macro-edit | compile-plus-runtime-suite | stock | 85.582675, 94.549940, 97.511755, 98.067934, 98.382921, 105.427968, 105.637684, 115.210702, 118.332551 | 118.332551 |
| rejection | compile | assert42-direct | 124.388707, 155.242363, 166.533436, 171.347485, 177.750238, 184.226279, 187.158018, 196.370308, 200.777285 | 200.777285 |
| rejection | compile | assert42-retained | 122.957265, 150.015064, 166.530682, 166.861413, 176.421072, 181.893809, 183.618269, 189.157631, 195.496150 | 195.496150 |
| rejection | compile | stock | 28.143601, 33.666501, 35.182765, 35.503413, 36.607299, 36.723641, 37.072468, 38.653311, 38.748691 | 38.748691 |
| recovery | compile | assert42-direct | 191.744840, 201.705719, 210.978695, 249.247532, 266.961589, 267.324775, 270.540701, 298.635930, 306.559816 | 306.559816 |
| recovery | compile-plus-runtime-suite | assert42-direct | 192.568346, 202.412711, 211.789390, 250.138994, 267.811765, 268.245958, 271.461965, 299.772529, 307.490211 | 307.490211 |
| recovery | compile | assert42-retained | 320.965292, 352.224258, 352.937391, 390.638271, 396.044513, 404.506304, 409.239284, 443.559815, 453.572333 | 453.572333 |
| recovery | compile-plus-runtime-suite | assert42-retained | 321.664178, 353.348524, 353.918216, 391.948966, 397.021070, 405.345373, 410.346528, 444.454017, 454.513694 | 454.513694 |
| recovery | compile | stock | 81.041849, 86.731030, 94.633920, 99.547320, 102.066336, 105.091540, 111.148099, 118.495117, 120.265785 | 120.265785 |
| recovery | compile-plus-runtime-suite | stock | 81.560334, 87.388410, 95.325115, 100.250177, 102.735933, 105.787349, 111.862320, 119.338142, 121.008203 | 121.008203 |

Component42 remains outside unchanged product34/published28. Full native language/crates/metadata/rlib/Cargo, loader closure, stable-files-during-request assumption, supervised lifecycle and full-project speed remain OPEN. All seven public-release gates remain held. Native Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC dependencies remain; no all-Rust or whole-toolchain safety claim.
