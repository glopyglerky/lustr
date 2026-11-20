# Native paired C-unit replay attempt v38

Primary body-edit median: fixed successful37 retained 387.849 ms; candidate38 retained 329.191 ms, 15.12% reduction. Threshold was >=10%; decision: **KEEP**. Six development native host-proof tests, seven optimized direct suites and ten optimized retained suites pass. A fresh frozen gpt-6.1-sol HIGH/default static review approved the change with no findings; the reviewer ran no tests or measurements.

The first development retained run had eight passes and two private linker-overlay permission failures. Both affected fixtures pass with the owned copy writable in the final optimized ten-suite run. First failures remain preserved. The optimized compiler build took 422.74 seconds, separately from application timings. Inherited nonfeature/product/older broader gates were not replayed or claimed freshly on38.

One isolated owned host-input proof change runs at most two native TinyCC replay children with unique per-unit paths, reaps started siblings before return and merges complete observations in sorted order into the original globally bounded proof. Protected Lust/blueFe/Rust sources, application bytes, checks, panic/unwind behavior and release compiler recipe are unchanged.

Nine samples, same successful PathBuf/Drop/unwind source generator as37, four arms rotated per phase and sample. Direct37 is successful native compilation; baseline37 retained uses that same fixed component, and candidate38 retained adds only the replay change. Stock Rust is a differential oracle and comparison, never the native implementation. No36->37 feature-speed comparison. CPUs10-13/jobs2; shared NAS, OS pages untouched. New bounded native process per sample, disk cache disabled before Check, real runtime/status/stdout/stderr agreement and invalid-program rejection required. No fallback. Cold and edit compilation still trail stock. No-op differences are observed samples; replay work is skipped on hits, so those differences do not establish replay acceleration.

Raw: `raw.jsonl` SHA256 `ae19a1e8e13f99d8a2be27da9a75b3f2505945572a9d92e24c9c0253a09e7081`. Summary: `summary.json` SHA256 `abaf473964fca02dd19f676e2cad7e1bb502c32dfc052e15f0f2e2d0dff5c13b`. Exact commands, request receipts, source hashes and compiled identities are retained alongside them; source generators/support/harness are frozen privately. All nine sorted values and maxima are below. Application opt0/debug0/unwind with overflow/debug checks on; compiler opt3/CGU1/ThinLTO/line tables unchanged. Compiler bootstrap build duration is separate.

## compile, milliseconds

| Phase | Direct37 | Retained37 | Candidate38 | Stock | Retained median reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| cold | 197.705 | 409.508 | 354.376 | 92.400 | 13.46% |
| repeat | 186.500 | 13.132 | 12.778 | 91.163 | 2.70% |
| noop | 189.432 | 17.386 | 11.892 | 91.640 | 31.60% |
| body-edit | 191.209 | 387.849 | 329.191 | 89.483 | 15.12% |
| signature-edit | 188.379 | 369.805 | 341.953 | 91.282 | 7.53% |
| type-edit | 194.792 | 371.977 | 335.633 | 94.443 | 9.77% |
| macro-edit | 190.182 | 352.992 | 326.243 | 89.495 | 7.58% |
| rejection | 110.006 | 110.309 | 108.354 | 31.907 | 1.77% |
| recovery | 186.182 | 374.230 | 321.267 | 88.813 | 14.15% |

## compile-plus-runtime-suite, milliseconds

| Phase | Direct37 | Retained37 | Candidate38 | Stock | Retained median reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| cold | 198.513 | 410.438 | 355.429 | 93.166 | 13.40% |
| repeat | 187.312 | 14.076 | 13.698 | 91.823 | 2.69% |
| noop | 190.372 | 18.553 | 12.698 | 92.309 | 31.56% |
| body-edit | 192.031 | 388.696 | 330.066 | 90.217 | 15.08% |
| signature-edit | 189.189 | 370.520 | 342.851 | 92.002 | 7.47% |
| type-edit | 195.584 | 372.927 | 336.903 | 95.134 | 9.66% |
| macro-edit | 190.947 | 353.841 | 326.944 | 90.343 | 7.60% |
| recovery | 187.131 | 375.167 | 322.022 | 89.550 | 14.17% |

## Complete samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | direct37 | 180.608612, 188.981551, 190.416507, 193.186846, 197.705158, 198.494074, 198.697577, 216.870555, 224.684286 | 224.684286 |
| cold | compile | baseline37-retained | 373.409466, 387.251674, 391.831556, 399.472807, 409.507622, 410.286537, 412.865622, 423.490884, 436.457414 | 436.457414 |
| cold | compile | candidate38-retained | 329.702472, 329.909070, 345.433079, 346.554973, 354.376303, 357.025619, 363.678574, 367.358200, 419.071378 | 419.071378 |
| cold | compile | stock | 86.120237, 86.931327, 91.937359, 92.279475, 92.399846, 97.658841, 100.013894, 101.307135, 114.094229 | 114.094229 |
| cold | compile-plus-runtime-suite | direct37 | 181.364392, 189.895260, 191.299663, 194.062151, 198.513045, 199.328696, 199.638960, 217.770505, 225.610977 | 225.610977 |
| cold | compile-plus-runtime-suite | baseline37-retained | 374.314197, 388.070695, 392.970845, 400.496747, 410.437672, 411.308923, 413.835529, 424.336257, 437.468983 | 437.468983 |
| cold | compile-plus-runtime-suite | candidate38-retained | 330.526465, 330.723541, 346.442694, 347.686870, 355.428647, 357.891227, 364.674189, 368.238373, 420.242108 | 420.242108 |
| cold | compile-plus-runtime-suite | stock | 86.731552, 87.684423, 92.674205, 92.991815, 93.166487, 98.368094, 100.758995, 102.192822, 114.967990 | 114.967990 |
| repeat | compile | direct37 | 182.133022, 183.180323, 185.868672, 186.310012, 186.499656, 188.797635, 199.521953, 221.212040, 232.689355 | 232.689355 |
| repeat | compile | baseline37-retained | 11.220696, 12.659361, 12.837182, 12.872561, 13.131888, 14.270824, 14.887524, 19.547525, 23.579978 | 23.579978 |
| repeat | compile | candidate38-retained | 11.184630, 11.972172, 12.378025, 12.615998, 12.777883, 14.812784, 14.860614, 17.778413, 18.696885 | 18.696885 |
| repeat | compile | stock | 82.071647, 88.222784, 89.368655, 90.293902, 91.163200, 91.541689, 94.222866, 102.725331, 111.324317 | 111.324317 |
| repeat | compile-plus-runtime-suite | direct37 | 182.984076, 184.016546, 186.760495, 187.124228, 187.311866, 189.773870, 200.421472, 222.249109, 233.678390 | 233.678390 |
| repeat | compile-plus-runtime-suite | baseline37-retained | 12.020094, 13.572164, 13.662898, 13.846689, 14.076119, 15.132411, 15.958710, 20.692106, 24.566809 | 24.566809 |
| repeat | compile-plus-runtime-suite | candidate38-retained | 12.255007, 13.063163, 13.457555, 13.530298, 13.697736, 15.920939, 16.751406, 18.913762, 19.543593 | 19.543593 |
| repeat | compile-plus-runtime-suite | stock | 82.653281, 89.007844, 90.114293, 91.013118, 91.823490, 92.301967, 94.942737, 103.568846, 112.147714 | 112.147714 |
| noop | compile | direct37 | 160.113870, 182.674022, 187.298344, 187.671314, 189.431931, 192.016767, 198.129365, 212.532085, 237.842537 | 237.842537 |
| noop | compile | baseline37-retained | 11.468968, 11.758352, 11.824049, 14.306048, 17.385966, 18.450357, 18.505526, 19.580701, 22.675222 | 22.675222 |
| noop | compile | candidate38-retained | 10.939288, 11.340976, 11.623566, 11.662000, 11.892130, 12.693788, 13.997071, 18.314319, 21.639404 | 21.639404 |
| noop | compile | stock | 80.596252, 87.697728, 88.722289, 90.955755, 91.640455, 92.250453, 94.168916, 103.195555, 108.651105 | 108.651105 |
| noop | compile-plus-runtime-suite | direct37 | 160.980423, 183.618706, 188.100488, 188.489032, 190.372071, 192.817621, 199.057342, 213.515738, 239.023987 | 239.023987 |
| noop | compile-plus-runtime-suite | baseline37-retained | 12.266811, 12.623674, 12.781727, 15.182074, 18.553279, 19.327579, 19.340408, 20.412531, 23.655423 | 23.655423 |
| noop | compile-plus-runtime-suite | candidate38-retained | 11.787184, 12.288619, 12.579202, 12.692757, 12.698345, 13.646581, 14.908639, 19.176269, 22.550467 | 22.550467 |
| noop | compile-plus-runtime-suite | stock | 81.175730, 88.375464, 89.438659, 91.649197, 92.308619, 92.954287, 94.862303, 103.909923, 109.501633 | 109.501633 |
| body-edit | compile | direct37 | 167.025180, 174.530095, 180.666657, 184.502825, 191.209366, 200.896287, 201.674867, 223.850454, 265.875873 | 265.875873 |
| body-edit | compile | baseline37-retained | 344.083721, 359.284422, 363.136730, 373.504243, 387.848910, 388.947424, 402.094903, 481.307440, 510.441454 | 510.441454 |
| body-edit | compile | candidate38-retained | 297.535652, 316.251731, 321.320730, 322.874644, 329.191376, 338.619893, 343.028706, 345.667534, 458.456266 | 458.456266 |
| body-edit | compile | stock | 85.226508, 87.713188, 88.881787, 89.472450, 89.483110, 90.749425, 93.427802, 94.490679, 117.960913 | 117.960913 |
| body-edit | compile-plus-runtime-suite | direct37 | 167.828783, 175.225932, 181.542496, 185.373766, 192.030926, 201.907498, 202.477293, 224.788777, 266.983225 | 266.983225 |
| body-edit | compile-plus-runtime-suite | baseline37-retained | 345.317636, 360.391024, 364.280861, 374.243033, 388.695967, 390.084628, 403.165715, 482.435281, 511.537105 | 511.537105 |
| body-edit | compile-plus-runtime-suite | candidate38-retained | 298.489745, 317.058248, 322.446139, 323.885026, 330.066254, 339.852018, 343.874989, 346.535974, 459.505344 | 459.505344 |
| body-edit | compile-plus-runtime-suite | stock | 85.856718, 88.367244, 89.492940, 90.201066, 90.216581, 91.448350, 94.130281, 95.226105, 118.796810 | 118.796810 |
| signature-edit | compile | direct37 | 165.092337, 167.722497, 170.895611, 185.588668, 188.378953, 191.943242, 205.320295, 213.220063, 269.628502 | 269.628502 |
| signature-edit | compile | baseline37-retained | 327.039917, 350.237065, 362.168897, 366.304356, 369.805042, 399.317110, 401.487134, 408.723566, 515.034359 | 515.034359 |
| signature-edit | compile | candidate38-retained | 292.193673, 300.958126, 329.413378, 336.317044, 341.952810, 346.991102, 356.625511, 357.046465, 401.088776 | 401.088776 |
| signature-edit | compile | stock | 85.013341, 86.686766, 86.871382, 91.158526, 91.281774, 91.502289, 94.104952, 95.493733, 107.431138 | 107.431138 |
| signature-edit | compile-plus-runtime-suite | direct37 | 165.889695, 168.619842, 171.605513, 186.379054, 189.189232, 192.647213, 206.319422, 214.110309, 270.750180 | 270.750180 |
| signature-edit | compile-plus-runtime-suite | baseline37-retained | 327.894250, 351.186499, 362.926211, 367.155683, 370.520451, 400.163285, 402.330615, 409.589773, 516.088646 | 516.088646 |
| signature-edit | compile-plus-runtime-suite | candidate38-retained | 293.320970, 301.781943, 330.269278, 337.230855, 342.850977, 347.850738, 357.545950, 357.971635, 402.056537 | 402.056537 |
| signature-edit | compile-plus-runtime-suite | stock | 85.645246, 87.412933, 87.643575, 91.914487, 92.001871, 92.239644, 94.834362, 96.235663, 108.238633 | 108.238633 |
| type-edit | compile | direct37 | 159.609569, 171.891009, 179.328442, 181.512613, 194.791683, 201.514928, 205.617660, 215.646795, 245.556338 | 245.556338 |
| type-edit | compile | baseline37-retained | 330.466266, 337.213875, 350.297682, 360.004659, 371.976559, 392.933297, 405.603153, 408.709818, 486.690960 | 486.690960 |
| type-edit | compile | candidate38-retained | 286.638622, 310.763521, 322.999740, 326.465935, 335.633398, 360.256173, 374.552338, 386.679865, 391.924959 | 391.924959 |
| type-edit | compile | stock | 76.674493, 80.222381, 86.094926, 87.571147, 94.443027, 96.282940, 97.559855, 100.468304, 105.381914 | 105.381914 |
| type-edit | compile-plus-runtime-suite | direct37 | 160.463997, 172.740453, 180.089230, 182.466166, 195.584122, 202.518700, 206.459090, 216.585127, 246.698826 | 246.698826 |
| type-edit | compile-plus-runtime-suite | baseline37-retained | 331.259619, 338.324754, 351.226223, 360.901496, 372.926780, 394.151697, 406.624722, 409.625588, 487.645023 | 487.645023 |
| type-edit | compile-plus-runtime-suite | candidate38-retained | 287.442127, 311.516874, 324.342657, 327.331758, 336.902860, 361.265980, 375.610266, 387.746467, 392.894406 | 392.894406 |
| type-edit | compile-plus-runtime-suite | stock | 77.148404, 80.832153, 86.755951, 88.292512, 95.134221, 97.022347, 98.427518, 101.255998, 106.236298 | 106.236298 |
| macro-edit | compile | direct37 | 155.915775, 161.409610, 171.847269, 178.277662, 190.182144, 191.070335, 201.119777, 204.758789, 220.536882 | 220.536882 |
| macro-edit | compile | baseline37-retained | 330.933892, 335.266042, 336.030675, 352.981167, 352.992033, 399.416458, 407.396824, 409.742758, 410.074271 | 410.074271 |
| macro-edit | compile | candidate38-retained | 283.988565, 285.102562, 302.949660, 310.351020, 326.242637, 353.873577, 356.761732, 384.319681, 426.706604 | 426.706604 |
| macro-edit | compile | stock | 78.739981, 81.460092, 83.190707, 83.983639, 89.494705, 92.958730, 98.038583, 101.829152, 105.143997 | 105.143997 |
| macro-edit | compile-plus-runtime-suite | direct37 | 156.656706, 162.134074, 172.625735, 179.243300, 190.946850, 191.882322, 201.970910, 205.549988, 221.397718 | 221.397718 |
| macro-edit | compile-plus-runtime-suite | baseline37-retained | 331.769838, 336.215121, 336.698880, 353.785219, 353.840719, 400.311307, 408.301506, 410.680945, 410.965860 | 410.965860 |
| macro-edit | compile-plus-runtime-suite | candidate38-retained | 284.804039, 285.895941, 303.713785, 311.342433, 326.944223, 354.737359, 357.640481, 385.256572, 427.694832 | 427.694832 |
| macro-edit | compile-plus-runtime-suite | stock | 79.419626, 82.126135, 83.769211, 84.520956, 90.342667, 93.725215, 98.883370, 104.497910, 105.942996 | 105.942996 |
| rejection | compile | direct37 | 92.496023, 97.580875, 106.348274, 107.950624, 110.005650, 114.710354, 119.839996, 122.103966, 130.907014 | 130.907014 |
| rejection | compile | baseline37-retained | 98.175648, 101.010844, 105.602653, 107.733014, 110.309393, 112.992402, 117.096318, 125.244956, 134.365026 | 134.365026 |
| rejection | compile | candidate38-retained | 87.571507, 102.549943, 104.344208, 105.286419, 108.354271, 110.993238, 123.566874, 128.137928, 148.592652 | 148.592652 |
| rejection | compile | stock | 27.957603, 28.237858, 29.309895, 31.471343, 31.906792, 32.287429, 32.488270, 33.666555, 37.684591 | 37.684591 |
| recovery | compile | direct37 | 166.315110, 172.316156, 173.985177, 180.716585, 186.182229, 192.747696, 194.331853, 200.216048, 221.091454 | 221.091454 |
| recovery | compile | baseline37-retained | 293.390705, 318.942647, 341.596492, 351.817122, 374.229504, 375.632392, 395.747537, 421.722582, 427.443131 | 427.443131 |
| recovery | compile | candidate38-retained | 270.566531, 283.081038, 302.161753, 307.221055, 321.267022, 329.076787, 329.184818, 348.186385, 355.416514 | 355.416514 |
| recovery | compile | stock | 81.974457, 83.336706, 83.732904, 87.858008, 88.813279, 89.081275, 93.477167, 95.291947, 95.548637 | 95.548637 |
| recovery | compile-plus-runtime-suite | direct37 | 167.210206, 173.061475, 174.771813, 181.555035, 187.131063, 193.573984, 195.112397, 201.119664, 222.069398 | 222.069398 |
| recovery | compile-plus-runtime-suite | baseline37-retained | 294.172440, 320.185181, 342.468317, 352.552821, 375.166833, 376.445556, 396.851355, 422.620809, 428.370758 | 428.370758 |
| recovery | compile-plus-runtime-suite | candidate38-retained | 271.403184, 283.926357, 302.995201, 308.065895, 322.021971, 329.955732, 330.099328, 349.110793, 356.311779 | 356.311779 |
| recovery | compile-plus-runtime-suite | stock | 82.625026, 84.036678, 84.341892, 88.484738, 89.549686, 89.837339, 94.168178, 96.007491, 96.286468 | 96.286468 |

This bounded workload does not establish full-project speed, native Rust rlib/metadata interoperability, runtime-loader dependency closure, general Cargo freshness, supervised lifecycle or full Rust support. No default, installed product or public-release admission follows. Cold/edit results versus stock and no-op results remain distinct. Stable files DURING each request remain assumed.
