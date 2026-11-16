# Native original-AST binary expressions — component46

The complete original binary assertion source SHA256 `73b5f6997aa0b6c3942c34fd84596385a887250b294f647f3f834bbaaddb0193` compiles, links and runs natively with exact stock runtime output. Its original copied45 L0809 rejection and separate stock compile/runtime witnesses remain retained. No successful native45 speed baseline exists for that source. Earlier full original sources remain unchanged, including parenthesized `b33a77ba19587159f387a51dcad75d7280ebab84c1204eab538512cf4a76e930` and long-assert `612287967c0d03d408b3704aa6451550e04b0555b74d559258f92dad417eef9b`. Protected Lust240e and Rust5b7b stay unchanged.

The owned formatter reads the original binary AST and verifies operand source boundaries and operator tokens. It emits the two operator breaks inside existing expression boxes and preserves every original parenthesis. Explicit pinned Rust precedence/fixity validates the operand tree; unmodeled direct closure/fixup forms refuse. Only the panic-text marker changes before full native checking, borrowing, generation and linking. Lazy RHS, evaluation order/once, const checks, hygiene, arenas, Drop and unwind remain executable native work. No transformed AST survives requests. See the [source-bound contract](../../proofs/native-assert-text.md).

Fresh actual gpt-6.1-sol HIGH/default frozen seven-file R1 approved without findings. Optimized direct22 and retained19 suites pass, followed by copied-component direct5, retained1 and host-input1 checks. Gates include all binary operators, arithmetic/bitwise/shift/comparison precedence, nested parentheses, lazy AND/OR, RHS order/once, exact multiline payloads, const/borrow/type/move rejection, prior mixed/coordinator and arena gates, retained operator/nesting/token/position/signature edits, invalid edits and recovery against cold native and stock. Review is not execution or speed proof.

## Equivalent successful work

The same successful optimized46 direct/retained and stock compile the same generated full mixed workload. Actual native/stock compile/runtime preflight passed before timing and its bytes match the timed cold source. The appended assertion generator preserves the complete earlier workload and isolates its final function. Nine rotated samples per engine/phase; new retained process per sample; disk caches disabled before Check; shared OS pages untouched. Compiler opt3/CGU1/ThinLTO, application opt0/debug0/unwind/checks-on. Compiler-build time is separate. Standalone no-op is not Cargo freshness, and compile-plus-runtime is actual execution rather than Cargo test. Shared-host activity limits causal attribution.

No comparison against rejecting45 or distinct earlier source is valid. Cold/edit/rejection regressions against stock remain visible. A bounded artifact-proof no-op does not establish full-project speed.

Median milliseconds, lower is better.

| Phase | Direct46 | Retained46 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 249.229 | 391.981 | 106.727 | 250.068 | 392.916 | 107.439 |
| repeat | 255.071 | 12.532 | 106.120 | 255.936 | 13.367 | 106.759 |
| noop | 274.066 | 12.758 | 101.206 | 275.040 | 13.883 | 101.946 |
| body-edit | 266.116 | 415.394 | 101.881 | 267.320 | 416.256 | 102.573 |
| signature-edit | 263.040 | 412.408 | 107.633 | 263.960 | 413.367 | 108.416 |
| type-edit | 258.344 | 404.792 | 104.151 | 259.267 | 405.697 | 104.922 |
| macro-edit | 257.011 | 393.904 | 102.152 | 257.953 | 394.747 | 102.819 |
| rejection | 179.794 | 186.597 | 36.482 | — | — | — |
| recovery | 251.915 | 392.097 | 103.971 | 252.865 | 393.070 | 104.646 |

Raw events: 757; SHA256 `5012e84b6c04b7f7f723b10c4acee1fa8c29f4c84b767d8dd3f45e005cc70822`. Summary51 SHA256 `df9969e50d83bc69821968e3a2512bc731d4c95b13c26dec137fce1a3a7322da`. Core `1f27bbf7bde4e9efb997cd094a5a41769d7e2cf50fde7948027d1a2c2b9fe67f`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 372.110063839s` is separate from application performance. Independent raw verification checks all51 groups, unique samples0..8, all sorted times, medians and maxima.

## All raw samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | binary46-direct | 217.340167, 227.454458, 241.394626, 244.107552, 249.229332, 249.404697, 278.721423, 284.473920, 381.994394 | 381.994394 |
| cold | compile-plus-runtime-suite | binary46-direct | 218.072018, 228.223880, 242.198588, 245.045947, 250.067933, 250.335044, 279.657354, 285.370314, 382.995347 | 382.995347 |
| cold | compile | binary46-retained | 346.108680, 366.492526, 373.815559, 382.027962, 391.980690, 397.951742, 423.070048, 423.784470, 557.294929 | 557.294929 |
| cold | compile-plus-runtime-suite | binary46-retained | 346.948133, 367.383050, 374.815875, 382.963349, 392.916072, 398.775505, 424.102237, 424.832831, 558.405098 | 558.405098 |
| cold | compile | stock | 93.510929, 95.783540, 100.056974, 101.995142, 106.727183, 108.669161, 112.380397, 112.635186, 132.000043 | 132.000043 |
| cold | compile-plus-runtime-suite | stock | 94.034555, 96.338432, 100.802867, 102.736804, 107.438739, 109.427551, 113.176313, 113.344407, 132.961966 | 132.961966 |
| repeat | compile | binary46-direct | 217.298528, 236.072004, 249.564423, 249.876487, 255.071428, 276.001799, 283.916545, 292.598885, 295.783626 | 295.783626 |
| repeat | compile-plus-runtime-suite | binary46-direct | 218.227160, 237.049832, 250.457430, 250.749579, 255.935687, 277.403988, 284.861195, 293.612565, 296.709776 | 296.709776 |
| repeat | compile | binary46-retained | 11.249250, 11.285018, 11.884248, 12.415679, 12.531881, 12.706376, 19.602517, 19.897555, 20.037241 | 20.037241 |
| repeat | compile-plus-runtime-suite | binary46-retained | 12.132954, 12.294762, 12.891337, 13.331321, 13.367434, 13.590348, 20.537054, 20.717365, 21.011916 | 21.011916 |
| repeat | compile | stock | 88.582511, 95.722249, 100.601562, 101.534042, 106.119947, 107.465113, 108.048867, 117.497456, 128.325849 | 128.325849 |
| repeat | compile-plus-runtime-suite | stock | 89.174150, 96.470949, 101.370156, 102.233399, 106.759281, 108.272758, 108.815167, 118.398250, 129.168088 | 129.168088 |
| noop | compile | binary46-direct | 221.953301, 224.085367, 242.917408, 259.768698, 274.066134, 284.334664, 285.492218, 313.740306, 325.394545 | 325.394545 |
| noop | compile-plus-runtime-suite | binary46-direct | 222.791235, 225.001446, 243.750377, 260.728414, 275.039543, 285.392015, 286.365998, 314.692618, 326.449819 | 326.449819 |
| noop | compile | binary46-retained | 10.840861, 11.186682, 11.827087, 12.471263, 12.757889, 13.202697, 18.912695, 19.573886, 21.202503 | 21.202503 |
| noop | compile-plus-runtime-suite | binary46-retained | 11.617592, 12.078830, 12.580008, 13.479545, 13.883153, 14.251743, 19.852991, 20.567507, 22.149235 | 22.149235 |
| noop | compile | stock | 94.267204, 96.676988, 97.264149, 101.056770, 101.205699, 111.480624, 112.061487, 114.744526, 120.534081 | 120.534081 |
| noop | compile-plus-runtime-suite | stock | 94.915353, 97.448842, 97.881022, 101.820640, 101.945614, 112.187105, 112.851341, 115.574836, 121.311997 | 121.311997 |
| body-edit | compile | binary46-direct | 228.410617, 242.484040, 248.084880, 248.202492, 266.116471, 270.842577, 280.156310, 293.735142, 321.688398 | 321.688398 |
| body-edit | compile-plus-runtime-suite | binary46-direct | 229.289419, 243.242508, 248.964187, 249.036105, 267.319892, 271.767228, 281.087395, 294.694625, 322.719675 | 322.719675 |
| body-edit | compile | binary46-retained | 354.840227, 387.049989, 391.111583, 414.375287, 415.394477, 432.622696, 444.143674, 471.678677, 508.099622 | 508.099622 |
| body-edit | compile-plus-runtime-suite | binary46-retained | 355.824124, 387.876375, 392.341477, 415.216707, 416.256339, 433.486201, 445.075866, 472.940663, 509.017892 | 509.017892 |
| body-edit | compile | stock | 92.882616, 98.047951, 100.059493, 100.997021, 101.880743, 104.448446, 109.073344, 112.419706, 131.690885 | 131.690885 |
| body-edit | compile-plus-runtime-suite | stock | 93.484856, 98.754853, 100.720015, 101.667317, 102.572971, 105.112473, 109.831191, 113.280075, 132.493566 | 132.493566 |
| signature-edit | compile | binary46-direct | 216.556305, 225.637397, 241.065629, 257.341175, 263.039819, 264.952823, 267.424018, 275.622314, 277.973126 | 277.973126 |
| signature-edit | compile-plus-runtime-suite | binary46-direct | 217.405328, 226.520571, 242.129554, 258.283308, 263.960075, 265.932811, 268.397841, 276.523990, 278.888928 | 278.888928 |
| signature-edit | compile | binary46-retained | 316.522330, 370.146359, 373.660019, 411.752396, 412.408276, 412.599645, 420.040340, 433.188538, 450.787764 | 450.787764 |
| signature-edit | compile-plus-runtime-suite | binary46-retained | 317.282802, 371.030073, 374.501625, 412.803634, 413.366830, 413.470999, 421.153973, 434.147641, 451.713360 | 451.713360 |
| signature-edit | compile | stock | 88.240817, 93.802677, 97.715364, 103.476474, 107.632783, 110.118815, 110.851785, 118.771787, 121.585900 | 121.585900 |
| signature-edit | compile-plus-runtime-suite | stock | 88.937921, 94.465290, 98.453885, 104.213183, 108.415830, 110.860390, 111.584835, 119.651337, 122.370232 | 122.370232 |
| type-edit | compile | binary46-direct | 220.634234, 239.572128, 245.844694, 251.797321, 258.344218, 269.187177, 274.175272, 284.074572, 341.175911 | 341.175911 |
| type-edit | compile-plus-runtime-suite | binary46-direct | 221.480214, 240.306602, 246.705911, 252.757620, 259.266908, 270.060286, 275.047133, 285.162276, 342.194906 | 342.194906 |
| type-edit | compile | binary46-retained | 336.733372, 366.463503, 385.189886, 388.074416, 404.791902, 418.438734, 422.786648, 438.609648, 583.572571 | 583.572571 |
| type-edit | compile-plus-runtime-suite | binary46-retained | 337.410419, 367.370838, 386.138451, 388.895540, 405.697083, 419.446194, 423.686303, 439.638546, 584.700361 | 584.700361 |
| type-edit | compile | stock | 90.103955, 96.026714, 102.235003, 102.635182, 104.150827, 106.814226, 109.620184, 110.869458, 114.033926 | 114.033926 |
| type-edit | compile-plus-runtime-suite | stock | 90.734766, 96.596438, 102.908745, 103.332236, 104.921923, 107.745907, 110.364606, 111.726300, 114.827471 | 114.827471 |
| macro-edit | compile | binary46-direct | 218.953184, 225.587965, 232.339717, 242.366021, 257.011289, 261.218609, 281.853050, 285.664600, 329.713381 | 329.713381 |
| macro-edit | compile-plus-runtime-suite | binary46-direct | 219.753830, 226.321702, 233.199002, 243.180890, 257.953223, 262.170992, 282.829019, 286.543336, 330.730492 | 330.730492 |
| macro-edit | compile | binary46-retained | 327.790854, 336.462355, 386.224602, 393.164935, 393.903611, 395.061778, 418.113983, 454.163528, 507.403851 | 507.403851 |
| macro-edit | compile-plus-runtime-suite | binary46-retained | 328.597815, 337.233470, 387.323599, 394.061565, 394.746810, 396.158274, 419.066675, 455.179279, 508.436670 | 508.436670 |
| macro-edit | compile | stock | 92.043249, 94.043332, 95.475562, 97.633087, 102.152115, 103.819801, 104.347500, 115.137661, 123.008768 | 123.008768 |
| macro-edit | compile-plus-runtime-suite | stock | 92.723729, 94.624783, 96.144144, 98.276280, 102.818846, 104.602497, 105.045901, 115.921298, 123.806573 | 123.806573 |
| rejection | compile | binary46-direct | 147.012416, 148.587147, 171.354200, 177.082091, 179.794258, 188.098719, 188.882521, 204.202940, 220.303438 | 220.303438 |
| rejection | compile | binary46-retained | 145.981930, 155.017578, 174.128772, 176.227091, 186.596928, 189.717345, 197.605309, 198.361714, 210.844711 | 210.844711 |
| rejection | compile | stock | 32.318338, 33.588875, 33.636491, 36.223121, 36.482320, 37.325508, 39.817126, 42.242949, 42.894908 | 42.894908 |
| recovery | compile | binary46-direct | 230.950859, 247.442850, 250.631813, 251.048853, 251.914888, 262.422402, 271.237014, 275.659018, 374.039223 | 374.039223 |
| recovery | compile-plus-runtime-suite | binary46-direct | 231.825476, 248.370202, 251.490850, 251.874031, 252.864865, 263.259139, 272.104389, 276.604155, 375.210142 | 375.210142 |
| recovery | compile | binary46-retained | 342.745251, 381.388409, 384.164217, 385.503206, 392.096884, 409.842520, 411.814265, 437.816388, 497.312447 | 497.312447 |
| recovery | compile-plus-runtime-suite | binary46-retained | 343.789978, 382.119895, 385.184478, 386.376726, 393.069568, 410.965587, 412.988371, 438.856652, 498.483959 | 498.483959 |
| recovery | compile | stock | 96.320975, 101.179559, 101.610823, 102.900141, 103.970666, 106.865612, 110.712631, 114.884087, 126.792805 | 126.792805 |
| recovery | compile-plus-runtime-suite | stock | 96.965421, 101.881698, 102.370426, 103.712060, 104.646038, 107.612621, 111.419063, 115.593413, 127.686049 | 127.686049 |

Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised product lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc and Linux/GLIBC remain dependencies. No all-Rust, whole-toolchain memory-safety or universal speed claim.
