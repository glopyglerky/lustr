# Native local const function items - component57

The complete original local const-fn witness `dd89186ffc83445506c0601a5f3e7d15ab9e67253855b8bc5502b735779aec98` compiles and executes natively with stock output, original assertion text and Guard/drop behavior. Component56 rejected its Const header in the owned parser; that rejection is not a successful timing baseline.

Owned header discrimination identifies the actual Fn keyword after Const, while the complete unchanged qualifier/header/body tokens pass to pinned native parse_items. Native scan supplies only a boundary. Ast::append_chunk shifts all native expression, pattern, type, block, string and statement-item references, preserving FnQual and FnDef ranges in the original lexical Stmt::Item. Native collection and full checking retain forward visibility, shadowing, duplicates, capture rejection, type/borrow/move/Drop/unwind checks and const evaluation. Runtime-only const fn bodies must also pass native const restrictions, including non-const calls and destructors. Exact safe Rust call target, ABI and boolean-signature obligations remain unchanged. No parser/checker copy, source rewrite, qualifier stripping, arbitrary permission or fallback.

Fresh Sol HIGH/default R1 approved the frozen integration. A fresh R2 reviewed final source and explicit local const-evaluation tests. Both completed and were task_cancel-retired. Optimized34 direct and31 retained tests pass, plus copied const-fn/nested/conversion direct/retained gates,15 assertion tests, old/local/call retained tests and host-input invalidation. The complete generated native/stock compile/runtime preflight precedes timing and equals timed sample0 cold.

## Equivalent successful work

SAME successful57 optimized direct/retained/stock; nine rotated samples per phase. Compiler opt3/CGU1/ThinLTO; applications opt0/debug0/unwind/checks-on. Native disk caches disabled before Check; fresh bounded retained process per sample; OS pages untouched. Shared-host noise limits attribution. Compiler-build cost is separate. Standalone no-op is not Cargo freshness, and compile-plus-runtime is not Cargo test. No gain is claimed against rejecting56 or distinct earlier sources. Cold/edit/rejection regressions are retained.

Median milliseconds, lower is better.

| Phase | Direct57 | Retained57 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 252.314 | 388.445 | 102.668 | 253.116 | 389.460 | 103.385 |
| repeat | 252.393 | 12.506 | 100.177 | 253.292 | 13.400 | 100.870 |
| noop | 241.690 | 11.836 | 102.881 | 242.581 | 12.718 | 103.528 |
| body-edit | 240.917 | 386.151 | 96.582 | 241.849 | 387.256 | 97.231 |
| signature-edit | 241.211 | 365.031 | 97.070 | 242.061 | 365.982 | 97.650 |
| type-edit | 221.970 | 369.743 | 93.136 | 222.769 | 370.474 | 93.776 |
| macro-edit | 241.930 | 366.383 | 101.763 | 242.728 | 367.159 | 102.529 |
| rejection | 162.564 | 167.535 | 36.113 | — | — | — |
| recovery | 254.663 | 369.804 | 97.370 | 255.540 | 370.723 | 98.031 |

Raw757 events SHA256 `31e712b3ad6e669e03c95298960e851c35d8edc75c874fd750f965edd6fdf783`; summary SHA256 `f5df12ed7802013359626284cecdcf9ee6a0e20ae8f7402b22cc9f483d3914f6`; core `2dc79921bc62f0ed91afdfa06d8c65ffd203896f225d4fc1d949a09b488cb663`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Independent verification checks all51 groups, unique samples0..8, sorted values, medians/maxima,81 actual direct commands,81 retained calls/18 hits and81 stock commands. Accepted compiler build: `exit status: 0; elapsed 432.130982467s`. Compiler-build cost is separate from application timing.

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | constfn57-direct | 214.575842, 223.693171, 232.335503, 235.234517, 252.313970, 257.349672, 262.308875, 270.162224, 278.973842 | 278.973842 |
| cold | compile-plus-runtime-suite | constfn57-direct | 215.373000, 224.566327, 233.063170, 236.018037, 253.115949, 258.212989, 263.238305, 270.992819, 279.927950 | 279.927950 |
| cold | compile | constfn57-retained | 316.860503, 353.260047, 377.015193, 386.945998, 388.444777, 395.399468, 407.235270, 409.619508, 481.904082 | 481.904082 |
| cold | compile-plus-runtime-suite | constfn57-retained | 317.712397, 354.080178, 378.259918, 387.979761, 389.459610, 396.196108, 408.602116, 410.522582, 482.899360 | 482.899360 |
| cold | compile | stock | 86.791409, 94.937099, 97.303936, 99.751256, 102.668089, 106.229038, 108.071938, 108.903231, 111.277940 | 111.277940 |
| cold | compile-plus-runtime-suite | stock | 87.354764, 95.478092, 97.932085, 100.451748, 103.384552, 107.022956, 108.882738, 109.621704, 111.988222 | 111.988222 |
| repeat | compile | constfn57-direct | 215.880025, 240.268808, 243.143806, 248.361956, 252.393359, 258.219540, 259.258427, 279.057698, 281.098148 | 281.098148 |
| repeat | compile-plus-runtime-suite | constfn57-direct | 216.612244, 241.016203, 243.961319, 249.218798, 253.291958, 259.080264, 260.351827, 280.007618, 282.002237 | 282.002237 |
| repeat | compile | constfn57-retained | 10.815019, 11.091743, 11.880688, 11.993613, 12.506382, 12.692428, 13.156670, 17.618446, 20.280440 | 20.280440 |
| repeat | compile-plus-runtime-suite | constfn57-retained | 11.534764, 11.844779, 12.672709, 12.814366, 13.399642, 13.604472, 14.090303, 18.499535, 21.153504 | 21.153504 |
| repeat | compile | stock | 88.710605, 91.885090, 98.989786, 99.620667, 100.176884, 104.679100, 107.784853, 108.427809, 112.821765 | 112.821765 |
| repeat | compile-plus-runtime-suite | stock | 89.297700, 92.536952, 99.742509, 100.367292, 100.870046, 105.443563, 108.442757, 109.215382, 113.526608 | 113.526608 |
| noop | compile | constfn57-direct | 229.381978, 237.134912, 240.537724, 240.584713, 241.689768, 249.585857, 255.842532, 256.288282, 324.754755 | 324.754755 |
| noop | compile-plus-runtime-suite | constfn57-direct | 230.156279, 237.984378, 241.403813, 241.494340, 242.581110, 250.519048, 256.741080, 257.105822, 325.729547 | 325.729547 |
| noop | compile | constfn57-retained | 10.315911, 10.425646, 11.299537, 11.356199, 11.835748, 16.477315, 17.608334, 18.517362, 21.509994 | 21.509994 |
| noop | compile-plus-runtime-suite | constfn57-retained | 11.096989, 11.316346, 12.123194, 12.171152, 12.717828, 17.138058, 18.598213, 19.488388, 22.454916 | 22.454916 |
| noop | compile | stock | 90.186301, 92.935329, 93.640412, 96.204250, 102.881040, 103.560623, 104.633075, 105.660635, 123.761269 | 123.761269 |
| noop | compile-plus-runtime-suite | stock | 90.844624, 93.594467, 94.259746, 96.825761, 103.528125, 104.156614, 105.279537, 106.392662, 124.565100 | 124.565100 |
| body-edit | compile | constfn57-direct | 209.709074, 223.415675, 228.568557, 234.376379, 240.916556, 244.601433, 246.998966, 262.170361, 316.293256 | 316.293256 |
| body-edit | compile-plus-runtime-suite | constfn57-direct | 210.398466, 224.199105, 229.417948, 235.391091, 241.848921, 245.442605, 247.839383, 263.143171, 317.434807 | 317.434807 |
| body-edit | compile | constfn57-retained | 329.312665, 359.437447, 371.751243, 385.611078, 386.150801, 392.493411, 404.165305, 426.146005, 449.496328 | 449.496328 |
| body-edit | compile-plus-runtime-suite | constfn57-retained | 330.047600, 360.173855, 372.560159, 386.688361, 387.255987, 393.835053, 404.999096, 427.147245, 450.453134 | 450.453134 |
| body-edit | compile | stock | 88.080575, 92.244383, 93.456649, 95.215234, 96.582249, 99.071068, 99.809841, 104.599849, 109.604812 | 109.604812 |
| body-edit | compile-plus-runtime-suite | stock | 88.761174, 92.974037, 94.103360, 95.830884, 97.230848, 99.718303, 100.416659, 105.298131, 110.350941 | 110.350941 |
| signature-edit | compile | constfn57-direct | 207.066642, 207.442136, 213.668999, 215.424891, 241.210607, 253.258002, 263.857130, 274.728582, 283.747577 | 283.747577 |
| signature-edit | compile-plus-runtime-suite | constfn57-direct | 207.866032, 208.189581, 214.352717, 216.238190, 242.061427, 254.110950, 264.792239, 275.665378, 284.644182 | 284.644182 |
| signature-edit | compile | constfn57-retained | 322.998214, 343.208031, 343.888968, 354.595330, 365.031165, 399.196958, 427.564710, 428.145129, 431.235852 | 431.235852 |
| signature-edit | compile-plus-runtime-suite | constfn57-retained | 323.722800, 344.012231, 345.036188, 355.632869, 365.982303, 400.378164, 428.575596, 429.149506, 432.605239 | 432.605239 |
| signature-edit | compile | stock | 85.980907, 87.636405, 87.995372, 96.944965, 97.070034, 104.359492, 105.431773, 108.109513, 111.932821 | 111.932821 |
| signature-edit | compile-plus-runtime-suite | stock | 86.512463, 88.235365, 88.669872, 97.617295, 97.649685, 105.160932, 106.061091, 108.865902, 112.772871 | 112.772871 |
| type-edit | compile | constfn57-direct | 202.640993, 209.011467, 212.489337, 215.389373, 221.970086, 250.587348, 252.341886, 267.190720, 267.941820 | 267.941820 |
| type-edit | compile-plus-runtime-suite | constfn57-direct | 203.280889, 209.814173, 213.170013, 216.065540, 222.768964, 251.468220, 253.278858, 268.133954, 268.807257 | 268.807257 |
| type-edit | compile | constfn57-retained | 298.246382, 312.769564, 317.817611, 365.287957, 369.743131, 386.760304, 415.043604, 426.331689, 459.393937 | 459.393937 |
| type-edit | compile-plus-runtime-suite | constfn57-retained | 298.922268, 313.883102, 318.552297, 366.095391, 370.473600, 387.869889, 416.128628, 427.748203, 460.634271 | 460.634271 |
| type-edit | compile | stock | 88.733669, 88.956955, 90.106477, 91.639205, 93.136210, 103.855158, 107.449213, 107.554533, 126.221600 | 126.221600 |
| type-edit | compile-plus-runtime-suite | stock | 89.244034, 89.723252, 90.634685, 92.205911, 93.775736, 104.575575, 108.143009, 108.237195, 128.416189 | 128.416189 |
| macro-edit | compile | constfn57-direct | 206.752052, 217.870228, 228.322028, 238.817246, 241.930319, 265.442092, 266.680433, 279.742068, 283.478838 | 283.478838 |
| macro-edit | compile-plus-runtime-suite | constfn57-direct | 207.490661, 218.660322, 229.165174, 239.541533, 242.728250, 266.234779, 267.594542, 280.650389, 284.447340 | 284.447340 |
| macro-edit | compile | constfn57-retained | 323.908913, 335.349592, 363.209927, 363.950323, 366.382943, 405.151958, 414.506677, 445.890745, 452.521189 | 452.521189 |
| macro-edit | compile-plus-runtime-suite | constfn57-retained | 325.219458, 336.161503, 364.013653, 364.866968, 367.158718, 405.966007, 415.723416, 446.907407, 453.550028 | 453.550028 |
| macro-edit | compile | stock | 85.629362, 88.168964, 95.681276, 96.611235, 101.763093, 102.481387, 102.657447, 107.936719, 110.753298 | 110.753298 |
| macro-edit | compile-plus-runtime-suite | stock | 86.344941, 88.791481, 96.983121, 97.245159, 102.528717, 103.158351, 104.354872, 108.793970, 111.579025 | 111.579025 |
| rejection | compile | constfn57-direct | 153.460006, 153.594252, 154.373787, 159.929634, 162.564023, 178.497742, 180.188374, 192.135421, 206.821500 | 206.821500 |
| rejection | compile | constfn57-retained | 140.736380, 149.547578, 152.731197, 164.896622, 167.535399, 187.284052, 187.387934, 189.236758, 208.353968 | 208.353968 |
| rejection | compile | stock | 32.115430, 32.432764, 34.628521, 35.463997, 36.112812, 37.139708, 38.734487, 39.829947, 41.299891 | 41.299891 |
| recovery | compile | constfn57-direct | 216.340740, 228.775022, 237.716560, 242.434972, 254.662926, 255.156627, 256.822317, 261.219178, 278.598731 | 278.598731 |
| recovery | compile-plus-runtime-suite | constfn57-direct | 217.126872, 229.651454, 238.540719, 243.304918, 255.540230, 256.013736, 257.746274, 262.256643, 279.589837 | 279.589837 |
| recovery | compile | constfn57-retained | 317.922106, 321.631790, 330.948324, 365.618302, 369.804159, 387.751714, 396.654095, 418.492535, 425.655868 | 425.655868 |
| recovery | compile-plus-runtime-suite | constfn57-retained | 318.683120, 322.400637, 331.723432, 366.435930, 370.723317, 388.768356, 397.538711, 419.389128, 426.656068 | 426.656068 |
| recovery | compile | stock | 88.872368, 91.437665, 92.435521, 92.859329, 97.369538, 98.805214, 102.022309, 107.500830, 109.681101 | 109.681101 |
| recovery | compile-plus-runtime-suite | stock | 89.558563, 92.068884, 93.096801, 93.455554, 98.030636, 99.490838, 102.758082, 108.361499, 110.413259 | 110.413259 |

Full Rust/unsafe semantics, crates/metadata/rlib/Cargo, runtime-loader closure, supervised lifecycle and representative unchanged-project speed remain OPEN. All seven public release gates are held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC/host linker remain dependencies. No all-Rust, whole-toolchain memory safety or universal speed claim.
