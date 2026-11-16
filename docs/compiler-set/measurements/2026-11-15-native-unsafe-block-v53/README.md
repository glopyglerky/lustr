# Owned native unsafe-block frontend — component53

The complete original assertion `df79606af726c65f282f7b63be1bac212f43189f83fef2558578169cb48fc908` compiles and executes natively with exact stock runtime output, panic text and Guard/drop behavior. The unchanged component52 refused live unsafe syntax with L0404; its rejection is not a successful timing baseline.

An explicit owned module composition references unchanged Lust modules. The complementary original-token parser creates actual UnsafeBlock nodes. A separate gate admits boolean literal bodies and nested blocks that perform no operation requiring unsafe permission. It lowers only a checked clone to native blocks with the same scopes and IDs; complete native checks and generation follow. Original source and the original AST remain available for panic formatting. Arbitrary unsafe operations and unimplemented enclosing grammar remain unsupported. See the [native interface and proof limits](../../proofs/native-unsafe-blocks.md).

First failures remain retained. Candidate1 incorrectly accepted a forbidden block-statement continuation; the corrected parser rejects it. An initial complete-work preflight refused unrelated expanded macro contexts; the guard now applies only to the affected function while native parsing preserves unrelated contexts. Sol R1 found the boundary and resource issues; R2 approved their frozen source correction. R3 reviewed the compiler hygiene delta without additional compiler findings and identified a test macro textual-scope error. The owner corrected that test ordering and qualified it with native and stock runtime controls. A fresh Sol R4 review approved all eleven final frozen source files, including the exact native expander bridge and corrected fixture.

The optimized29 direct and26 retained suites pass, then copied13 direct,1 retained and1 host-input test. Full generated native/stock preflight passed before timing. It preserves the complete mixed body with the intended unsafe repeat value, and its bytes match timed sample0 cold. Retained source/type/token/position edits, rejection and recovery remain separate from runtime parity.

## Equivalent successful work

SAME successful optimized53 direct/retained and stock, nine rotated samples per phase. Compiler opt3/CGU1/ThinLTO; application opt0/debug0/unwind/checks-on. Native disk caches disabled before Check, fresh bounded retained process per sample, OS pages untouched. Compiler build time is separate. Shared-host activity limits attribution. Standalone no-op is not Cargo freshness, and compile-plus-runtime is not Cargo test. No gain is inferred against rejecting52 or distinct older sources.

Median milliseconds, lower is better.

| Phase | Direct53 | Retained53 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 261.988 | 402.621 | 103.152 | 262.876 | 403.515 | 103.845 |
| repeat | 266.185 | 13.429 | 105.575 | 267.017 | 14.469 | 106.324 |
| noop | 264.756 | 13.528 | 108.221 | 265.749 | 14.501 | 108.950 |
| body-edit | 253.855 | 423.232 | 105.955 | 255.022 | 424.188 | 106.640 |
| signature-edit | 253.494 | 403.482 | 103.310 | 254.504 | 404.860 | 104.057 |
| type-edit | 272.396 | 386.750 | 108.530 | 273.341 | 387.606 | 109.291 |
| macro-edit | 263.119 | 425.535 | 108.394 | 263.992 | 426.620 | 109.192 |
| rejection | 183.790 | 186.418 | 38.047 | — | — | — |
| recovery | 253.737 | 369.729 | 104.695 | 254.609 | 370.599 | 105.340 |

Raw757 events SHA256 `b4b4be6a7111482600c6184bd33a30bea3cd6586e904ff39e868f5ee62b924e3`; summary SHA256 `7333df9d42c7b48b0d1f47f763218e58607e4d36540b052e353b6c511b84dd9f`; core `10104ffb2ae107537b138201ecddc8a5bf3936101fecc27aa32eae8a9717fb0b`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Independent verification checks all51 groups, unique samples0..8, sorted values, medians/maxima,81 actual direct commands,81 retained calls/18 hits and81 stock commands. Compiler build receipt: `exit status: 0; elapsed 375.922437167s`.

## Raw samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | unsafe53-direct | 212.713045, 222.108895, 230.250531, 257.475295, 261.987791, 270.098351, 283.810992, 322.124516, 341.142855 | 341.142855 |
| cold | compile-plus-runtime-suite | unsafe53-direct | 213.465110, 222.831361, 231.148319, 258.431010, 262.876229, 270.936067, 284.668790, 323.099573, 342.110367 | 342.110367 |
| cold | compile | unsafe53-retained | 352.462000, 354.105749, 397.065137, 397.244802, 402.620904, 405.796718, 436.793839, 474.682847, 482.612226 | 482.612226 |
| cold | compile-plus-runtime-suite | unsafe53-retained | 353.279006, 354.940341, 397.983160, 398.071306, 403.515383, 406.637830, 437.750748, 475.760521, 483.571741 | 483.571741 |
| cold | compile | stock | 92.592491, 94.254048, 96.708438, 98.295680, 103.152328, 106.450845, 110.041877, 114.189231, 122.108258 | 122.108258 |
| cold | compile-plus-runtime-suite | stock | 93.314539, 94.884905, 97.400278, 98.910584, 103.844676, 107.149202, 110.762775, 114.909087, 122.909150 | 122.909150 |
| repeat | compile | unsafe53-direct | 240.654344, 241.726514, 243.616114, 252.377614, 266.185072, 278.988640, 289.034195, 293.901880, 338.502622 | 338.502622 |
| repeat | compile-plus-runtime-suite | unsafe53-direct | 241.578408, 242.714608, 244.412883, 253.205217, 267.017194, 280.045226, 289.909346, 295.044766, 339.740999 | 339.740999 |
| repeat | compile | unsafe53-retained | 11.016539, 11.055712, 12.470061, 13.384033, 13.429459, 13.971677, 17.624612, 18.327732, 21.580132 | 21.580132 |
| repeat | compile-plus-runtime-suite | unsafe53-retained | 11.780113, 11.873963, 13.557558, 14.365141, 14.468941, 15.042174, 18.512923, 19.184658, 22.534178 | 22.534178 |
| repeat | compile | stock | 93.734444, 95.943767, 96.933025, 102.121728, 105.575350, 107.371302, 113.240132, 114.958036, 126.913904 | 126.913904 |
| repeat | compile-plus-runtime-suite | stock | 94.411831, 96.744772, 97.619694, 102.771902, 106.323903, 108.156190, 113.903206, 115.841712, 127.782914 | 127.782914 |
| noop | compile | unsafe53-direct | 224.216666, 230.151999, 235.495090, 243.780620, 264.756010, 271.219063, 288.276435, 305.070671, 327.661868 | 327.661868 |
| noop | compile-plus-runtime-suite | unsafe53-direct | 224.933887, 230.955932, 236.324664, 244.580513, 265.748771, 272.152777, 289.243315, 305.865650, 328.725320 | 328.725320 |
| noop | compile | unsafe53-retained | 10.605142, 10.897747, 11.439828, 13.066438, 13.528260, 15.104320, 18.962381, 20.581541, 21.212428 | 21.212428 |
| noop | compile-plus-runtime-suite | unsafe53-retained | 11.479467, 11.712702, 12.375597, 13.955216, 14.501283, 16.123311, 19.803338, 21.399430, 22.142053 | 22.142053 |
| noop | compile | stock | 90.283671, 95.232446, 96.708021, 100.500537, 108.220838, 111.689125, 112.247556, 112.479069, 125.744784 | 125.744784 |
| noop | compile-plus-runtime-suite | stock | 90.892619, 95.856169, 97.252534, 101.177326, 108.950182, 112.419782, 113.027683, 113.318887, 126.573990 | 126.573990 |
| body-edit | compile | unsafe53-direct | 213.973348, 235.488896, 246.938960, 250.467923, 253.855022, 269.162640, 285.114549, 311.917830, 314.242365 | 314.242365 |
| body-edit | compile-plus-runtime-suite | unsafe53-direct | 214.765021, 236.358731, 247.856152, 251.258985, 255.022456, 270.088492, 286.100282, 312.850338, 315.208159 | 315.208159 |
| body-edit | compile | unsafe53-retained | 358.561682, 360.981369, 374.756683, 420.632056, 423.232135, 428.355860, 442.057754, 447.996650, 508.992642 | 508.992642 |
| body-edit | compile-plus-runtime-suite | unsafe53-retained | 359.402221, 361.807138, 375.720716, 421.573369, 424.187829, 429.227631, 442.917087, 449.129038, 510.315774 | 510.315774 |
| body-edit | compile | stock | 88.000495, 96.781100, 101.004877, 104.449837, 105.955490, 111.060414, 111.762286, 123.853576, 126.038140 | 126.038140 |
| body-edit | compile-plus-runtime-suite | stock | 88.538842, 97.447330, 101.654455, 105.037849, 106.640412, 111.938099, 112.610325, 124.747174, 126.891483 | 126.891483 |
| signature-edit | compile | unsafe53-direct | 229.106585, 231.309476, 239.759868, 247.479149, 253.493557, 270.265635, 287.440109, 301.375682, 325.819593 | 325.819593 |
| signature-edit | compile-plus-runtime-suite | unsafe53-direct | 229.926039, 232.100827, 240.516718, 248.355406, 254.504399, 271.090043, 288.402535, 302.527608, 326.844236 | 326.844236 |
| signature-edit | compile | unsafe53-retained | 356.355263, 391.743762, 391.821432, 395.376988, 403.482479, 416.277126, 427.558222, 462.897837, 486.345023 | 486.345023 |
| signature-edit | compile-plus-runtime-suite | unsafe53-retained | 357.241927, 392.609244, 393.167718, 396.358378, 404.860117, 417.281839, 428.482138, 463.893629, 487.471601 | 487.471601 |
| signature-edit | compile | stock | 96.287180, 98.871892, 100.889640, 103.258483, 103.309778, 107.955586, 114.140471, 116.582405, 137.245796 | 137.245796 |
| signature-edit | compile-plus-runtime-suite | stock | 97.059982, 99.525870, 101.476944, 104.041695, 104.057318, 108.719760, 114.888571, 117.465638, 138.085593 | 138.085593 |
| type-edit | compile | unsafe53-direct | 221.878023, 225.470685, 251.003883, 258.913899, 272.396354, 289.230794, 290.643435, 294.188047, 300.785660 | 300.785660 |
| type-edit | compile-plus-runtime-suite | unsafe53-direct | 222.669598, 226.289649, 252.072919, 259.742509, 273.341472, 290.102769, 291.622162, 295.185925, 301.867989 | 301.867989 |
| type-edit | compile | unsafe53-retained | 358.391793, 380.986546, 384.667152, 384.770699, 386.749587, 412.234390, 423.399305, 433.377092, 463.163909 | 463.163909 |
| type-edit | compile-plus-runtime-suite | unsafe53-retained | 359.621819, 381.992057, 385.560565, 385.716894, 387.605743, 413.261653, 424.227016, 434.315831, 464.285427 | 464.285427 |
| type-edit | compile | stock | 97.601013, 98.704618, 99.895426, 102.528265, 108.529826, 109.555782, 112.263377, 118.739193, 127.233736 | 127.233736 |
| type-edit | compile-plus-runtime-suite | stock | 98.251755, 99.412979, 100.537400, 103.169894, 109.290624, 110.287083, 112.980465, 119.533377, 128.196666 | 128.196666 |
| macro-edit | compile | unsafe53-direct | 230.986625, 235.407767, 238.864637, 254.215536, 263.119287, 264.380314, 269.380796, 292.821572, 307.053891 | 307.053891 |
| macro-edit | compile-plus-runtime-suite | unsafe53-direct | 231.698228, 236.285310, 239.760675, 255.228855, 263.992233, 265.306782, 270.330043, 293.710234, 308.091522 | 308.091522 |
| macro-edit | compile | unsafe53-retained | 369.363263, 371.381131, 399.617209, 410.437367, 425.534558, 430.558076, 438.282059, 445.385591, 480.877615 | 480.877615 |
| macro-edit | compile-plus-runtime-suite | unsafe53-retained | 370.066901, 372.185157, 400.613499, 411.361948, 426.619553, 431.638872, 439.297766, 446.666429, 481.962622 | 481.962622 |
| macro-edit | compile | stock | 90.504191, 95.669382, 97.851580, 104.176642, 108.393629, 109.883466, 111.268314, 112.113729, 121.059027 | 121.059027 |
| macro-edit | compile-plus-runtime-suite | stock | 91.099805, 96.334257, 98.487938, 104.849963, 109.192348, 110.609741, 111.994805, 112.851807, 121.878669 | 121.878669 |
| rejection | compile | unsafe53-direct | 155.979803, 161.695938, 174.365089, 183.489606, 183.789866, 188.079493, 188.985526, 211.504000, 231.878835 | 231.878835 |
| rejection | compile | unsafe53-retained | 148.179690, 159.729628, 179.947477, 182.129592, 186.417571, 186.736878, 193.232197, 209.147414, 249.494202 | 249.494202 |
| rejection | compile | stock | 31.956286, 35.154730, 36.356989, 36.363005, 38.047164, 38.067161, 39.856261, 41.755830, 45.147447 | 45.147447 |
| recovery | compile | unsafe53-direct | 218.231308, 232.016848, 242.597399, 248.876956, 253.736561, 281.980106, 285.533398, 293.207958, 344.557816 | 344.557816 |
| recovery | compile-plus-runtime-suite | unsafe53-direct | 219.004964, 232.818074, 243.386048, 249.622392, 254.609213, 282.933028, 286.341405, 293.983953, 345.461193 | 345.461193 |
| recovery | compile | unsafe53-retained | 344.024405, 346.642939, 347.599804, 353.180997, 369.729419, 403.677169, 425.987371, 434.537989, 493.575918 | 493.575918 |
| recovery | compile-plus-runtime-suite | unsafe53-retained | 344.834618, 347.997004, 348.741704, 354.121356, 370.598619, 404.462468, 428.283495, 435.556922, 494.517181 | 494.517181 |
| recovery | compile | stock | 94.776299, 96.802706, 97.144436, 101.040508, 104.695035, 108.642906, 111.914310, 115.088176, 126.455769 | 126.455769 |
| recovery | compile-plus-runtime-suite | stock | 95.461500, 97.494703, 97.779500, 101.760323, 105.340290, 109.353127, 112.679659, 115.825224, 127.246076 | 127.246076 |

Full Rust and unsafe semantics, crates/metadata/rlib/Cargo, runtime-loader closure, supervised lifecycle and representative project speed remain OPEN. All seven public release gates are held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC/host linker remain dependencies. No all-Rust, whole-stack safety or universal speed claim.
