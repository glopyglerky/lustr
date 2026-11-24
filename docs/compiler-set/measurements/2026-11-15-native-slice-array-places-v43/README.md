# Native slice/array reference-place assertions — component43

The preserved full mixed source SHA256 `9b677d4ceb867fee6852384802f0557ccbaf569aaec39f30c41304c19c5333e2` now compiles, links and runs through the native component. The copied42 L0110 rejection and stock witness remain. The earlier full rvalue mixed source `328e723ef884df4cd62140a65ec97edf31ceffe862db87b755b2dc7f85150373` also remains unchanged and accepted. Protected Lust240e and Rust5b7b source trees are unchanged.

Owned AST elaboration borrows both original operand places in order, then performs same-element array-to-slice unsizing inside the original equality/inequality macro. Synthetic bindings must be absent from the complete interner. Expressions, patterns and function ranges are remapped together, including empty, adjacent, nested and closure arenas. Each lazy message retains its original borrowed place and uses both operand bindings afterward, so the cast cannot release the original operand loans before user message code. Existing rvalue casts retain their path. Canonical equality/Debug and full native collect/check/borrowck/codegen/link remain required; no transformed AST survives a request. Unresolved/generic/mutable/heterogeneous forms remain outside this rule. No source rewriting, assert_eq-to-assert shortcut, protected patch, compatibility compiler or forged metadata supplies this acceptance.

Qualification covers unchanged full mixed source; both directions and named/tuple/index/deref/self-field places; zero/different lengths, bool/char/f64; custom equality/Debug/Drop; evaluation order/once, lazy messages, named/repeated/width formatting, borrowed non-Copy messages, temporary-message Drop, exact panic payloads and unwind; mutation after an assertion returns; expression/pattern arenas and retained operand/length/type/value edits, rejection and recovery against cold native and stock. Eleven new invalid guards require an actual native recheck and retain lifetime, aliasing, packed named/tuple, const-length and heterogeneous rejection. Existing invalid/trait identity/ambiguity guards remain.

The first V43 match implementation falsely accepted assignment to an operand place from a failure message; its source and failed test are preserved. The corrected implementation adds the late binding uses and rejects the original counterexample with L0312 after elaboration. Initial invalid harness cases that rejected before expansion are retained as insufficient proof, separately from the strengthened guards. Initial missing-raw-field compiler error and unstaged-identity probe failure are retained. Fresh frozen11 gpt-6.1-sol HIGH/default review APPROVE found no F43 issues. Root gpt-6.1-sol xhigh/default authored implementation/helpers/report, through T3 Code -> Codex; no Claude used. Source review is separate from actual optimized/copy tests. L0809 remains OPEN.

## Same successful candidate costs

Nine rotated samples per engine/phase on shared Linux GNU x86_64 NAS CPUs10-13/jobs2. SAME optimized43 direct and retained ABI plus pinned stock compile exactly the same generated full mixed place workload. Native compiler opt3/CGU1/ThinLTO/line tables is unchanged; application opt0/debug0/unwind/overflow+debug checks on. Fresh retained process/sample; native disk cache disabled before Check; OS pages untouched. Every source snapshot, identity, command, output, status, raw timing and rejection is retained. Stock standalone recompilation repeat/noop is not Cargo freshness. Compile-plus-runtime means actual application execution, not Cargo test.

Rejecting42 and distinct earlier workloads are not successful performance baselines. Cold/edit/rejection comparisons with stock retain the actual regressions; a bounded no-op artifact-proof win cannot close full-project speed. Shared-host noise limits causal interpretation.

Median milliseconds, lower is better.

| Phase | Direct43 | Retained43 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 215.313 | 338.039 | 90.792 | 216.014 | 338.810 | 91.400 |
| repeat | 212.691 | 11.056 | 88.427 | 213.493 | 11.906 | 89.008 |
| noop | 208.129 | 12.431 | 87.021 | 208.966 | 13.372 | 87.659 |
| body-edit | 226.462 | 350.997 | 88.416 | 227.452 | 352.334 | 89.163 |
| signature-edit | 220.386 | 358.232 | 92.903 | 221.183 | 358.967 | 93.524 |
| type-edit | 204.401 | 321.725 | 86.206 | 205.194 | 322.682 | 86.923 |
| macro-edit | 196.002 | 350.384 | 84.893 | 196.816 | 351.217 | 85.392 |
| rejection | 151.760 | 148.734 | 32.670 | — | — | — |
| recovery | 219.201 | 328.632 | 89.167 | 220.036 | 329.378 | 89.766 |

Raw757 SHA256 `f7dd6bdde1e8aed6619fc09ddc05be061ce5298d1cb0c7e79d464788de325237`; summary51 `a03e9141c0249769553e8be3c8d053113a8de1a9ef2cd4a603dc96363a5df265`. Core `4111f2f97d762d9942be743ac9f01000320b1578ad1be57441c490b2e660e9ce`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`. Compiler BUILD receipt `exit status: 0; elapsed 398.570017597s` is separate from application timing.

## All raw samples

| Phase | Kind | Engine | Sorted nine ms | Maximum ms |
| --- | --- | --- | --- | ---: |
| cold | compile | place43-direct | 187.006202, 188.251289, 195.517665, 198.226222, 215.312700, 223.816037, 239.812632, 251.471839, 276.592992 | 276.592992 |
| cold | compile-plus-runtime-suite | place43-direct | 187.633623, 188.941340, 196.233621, 198.945177, 216.013694, 224.604650, 240.637631, 252.342558, 277.436476 | 277.436476 |
| cold | compile | place43-retained | 299.569479, 303.002677, 303.282701, 318.764787, 338.038889, 353.296454, 386.539486, 391.870314, 401.717956 | 401.717956 |
| cold | compile-plus-runtime-suite | place43-retained | 300.383987, 303.803016, 304.157408, 319.566742, 338.810172, 354.373887, 387.384959, 392.786054, 402.573975 | 402.573975 |
| cold | compile | stock | 80.976969, 82.967284, 85.416772, 86.695631, 90.792280, 91.431322, 98.389254, 99.295324, 109.094127 | 109.094127 |
| cold | compile-plus-runtime-suite | stock | 81.507491, 83.572912, 86.071432, 87.411902, 91.399761, 92.076311, 99.069608, 99.940760, 109.860683 | 109.860683 |
| repeat | compile | place43-direct | 189.456716, 192.116997, 192.856706, 209.419687, 212.690731, 215.143682, 237.341300, 250.604826, 255.474756 | 255.474756 |
| repeat | compile-plus-runtime-suite | place43-direct | 190.216359, 192.771327, 193.538934, 210.237697, 213.493498, 215.808849, 238.311361, 251.477888, 256.296165 | 256.296165 |
| repeat | compile | place43-retained | 9.873301, 10.528898, 10.712593, 10.793819, 11.055887, 11.460895, 12.304425, 14.364368, 18.856579 | 18.856579 |
| repeat | compile-plus-runtime-suite | place43-retained | 10.519757, 11.255603, 11.602297, 11.641594, 11.905619, 12.194135, 13.048643, 15.341243, 19.826238 | 19.826238 |
| repeat | compile | stock | 78.615801, 79.332913, 81.929568, 84.430378, 88.426947, 89.803753, 98.308317, 99.626172, 103.386025 | 103.386025 |
| repeat | compile-plus-runtime-suite | stock | 79.182580, 79.862805, 82.558218, 85.022089, 89.008051, 90.409352, 99.050331, 100.257147, 104.206257 | 104.206257 |
| noop | compile | place43-direct | 190.967412, 196.281043, 199.263642, 205.801970, 208.129446, 215.212922, 241.314745, 251.966200, 294.074229 | 294.074229 |
| noop | compile-plus-runtime-suite | place43-direct | 191.473459, 197.027114, 200.020556, 206.521304, 208.966306, 216.102044, 242.293157, 252.748978, 295.046643 | 295.046643 |
| noop | compile | place43-retained | 9.470666, 9.601508, 9.899542, 11.046922, 12.431387, 15.724403, 16.116368, 17.585548, 19.067280 | 19.067280 |
| noop | compile-plus-runtime-suite | place43-retained | 10.113224, 10.269166, 10.630152, 11.955217, 13.372059, 16.385954, 17.112016, 18.583038, 19.811549 | 19.811549 |
| noop | compile | stock | 78.026842, 79.598559, 81.928613, 85.062661, 87.020628, 90.382625, 95.960873, 99.146435, 117.922141 | 117.922141 |
| noop | compile-plus-runtime-suite | stock | 78.592177, 80.240811, 82.499434, 85.608750, 87.659069, 91.033865, 96.714139, 99.770733, 118.698852 | 118.698852 |
| body-edit | compile | place43-direct | 192.942254, 195.967952, 201.756229, 211.197733, 226.462217, 233.117406, 243.292163, 249.819487, 304.211326 | 304.211326 |
| body-edit | compile-plus-runtime-suite | place43-direct | 193.704765, 196.730492, 202.532450, 211.949645, 227.451968, 233.965382, 244.179656, 250.706413, 305.242493 | 305.242493 |
| body-edit | compile | place43-retained | 314.155295, 315.231556, 320.301153, 342.359392, 350.996725, 353.569170, 362.334292, 388.843796, 406.311365 | 406.311365 |
| body-edit | compile-plus-runtime-suite | place43-retained | 314.966567, 316.215811, 320.951718, 343.280889, 352.333856, 354.545047, 363.397656, 389.668247, 407.146128 | 407.146128 |
| body-edit | compile | stock | 79.365046, 80.712325, 83.139022, 86.380121, 88.415919, 93.104250, 95.412982, 100.670294, 129.235582 | 129.235582 |
| body-edit | compile-plus-runtime-suite | stock | 79.910337, 81.248306, 83.803909, 86.979649, 89.162852, 93.757561, 96.128506, 101.483787, 130.020452 | 130.020452 |
| signature-edit | compile | place43-direct | 199.214619, 199.412057, 203.266562, 209.477216, 220.385688, 233.337620, 234.252323, 237.676285, 255.820337 | 255.820337 |
| signature-edit | compile-plus-runtime-suite | place43-direct | 199.882971, 200.131379, 203.907157, 210.158425, 221.183027, 234.147823, 234.989848, 238.478970, 256.651851 | 256.651851 |
| signature-edit | compile | place43-retained | 295.798693, 301.034007, 320.286231, 336.508833, 358.232054, 359.166138, 369.354140, 369.975421, 403.813145 | 403.813145 |
| signature-edit | compile-plus-runtime-suite | place43-retained | 296.535307, 301.770669, 321.412072, 337.406541, 358.966530, 360.018587, 370.398938, 371.044478, 404.945916 | 404.945916 |
| signature-edit | compile | stock | 80.814013, 82.230417, 82.830826, 86.633586, 92.902849, 95.593148, 98.113107, 100.080842, 105.348226 | 105.348226 |
| signature-edit | compile-plus-runtime-suite | stock | 81.490834, 82.759130, 83.372462, 87.213557, 93.524451, 96.277146, 98.774583, 100.936131, 106.077417 | 106.077417 |
| type-edit | compile | place43-direct | 192.263412, 196.699865, 198.221343, 200.591168, 204.401224, 207.722500, 208.814808, 214.129105, 273.215646 | 273.215646 |
| type-edit | compile-plus-runtime-suite | place43-direct | 192.935519, 197.453225, 198.979025, 201.325280, 205.194000, 208.433914, 209.555245, 214.964873, 274.062853 | 274.062853 |
| type-edit | compile | place43-retained | 295.391332, 303.447849, 310.244998, 311.751384, 321.725191, 329.849443, 340.036176, 349.416840, 379.123798 | 379.123798 |
| type-edit | compile-plus-runtime-suite | place43-retained | 296.117637, 304.228844, 311.011913, 312.391396, 322.682056, 330.615042, 340.784428, 350.192897, 379.970916 | 379.970916 |
| type-edit | compile | stock | 78.659237, 81.118436, 82.596691, 83.300049, 86.206172, 86.737074, 87.163329, 90.145010, 98.372627 | 98.372627 |
| type-edit | compile-plus-runtime-suite | stock | 79.216964, 81.643560, 83.219017, 83.860189, 86.922602, 87.303601, 87.860432, 90.813816, 99.119771 | 99.119771 |
| macro-edit | compile | place43-direct | 189.950082, 191.419277, 193.388465, 194.989104, 196.001815, 204.677641, 241.860703, 244.495316, 268.497967 | 268.497967 |
| macro-edit | compile-plus-runtime-suite | place43-direct | 190.732551, 192.100297, 194.220093, 195.680562, 196.816472, 205.385662, 242.696411, 245.248665, 269.290440 | 269.290440 |
| macro-edit | compile | place43-retained | 296.764357, 302.904875, 318.952596, 330.002663, 350.384434, 374.298147, 378.748976, 394.963367, 406.313622 | 406.313622 |
| macro-edit | compile-plus-runtime-suite | place43-retained | 297.509690, 303.572807, 319.995967, 330.786100, 351.217000, 375.105454, 379.699623, 396.000131, 407.312275 | 407.312275 |
| macro-edit | compile | stock | 81.074147, 81.426167, 82.423781, 83.732285, 84.892855, 87.890303, 88.051504, 96.126695, 105.413937 | 105.413937 |
| macro-edit | compile-plus-runtime-suite | stock | 81.778117, 81.932513, 82.969477, 84.304685, 85.391670, 88.555560, 88.706043, 96.753443, 106.115506 | 106.115506 |
| rejection | compile | place43-direct | 127.409011, 129.226566, 130.796200, 150.211306, 151.760311, 164.199611, 173.169412, 174.801970, 176.423887 | 176.423887 |
| rejection | compile | place43-retained | 125.203667, 134.707285, 134.984597, 136.048072, 148.734497, 153.548295, 173.669690, 174.793518, 176.281983 | 176.281983 |
| rejection | compile | stock | 29.286651, 29.734767, 29.788260, 31.050473, 32.669807, 32.818444, 34.694548, 35.729313, 36.187560 | 36.187560 |
| recovery | compile | place43-direct | 189.798540, 196.365365, 201.039586, 218.458088, 219.201423, 228.069332, 238.444855, 243.411106, 244.851137 | 244.851137 |
| recovery | compile-plus-runtime-suite | place43-direct | 190.526289, 197.029441, 201.745354, 219.330669, 220.035827, 228.841823, 239.279050, 244.239257, 245.728967 | 245.728967 |
| recovery | compile | place43-retained | 288.401688, 292.122593, 324.681810, 327.643018, 328.631560, 346.889571, 379.051178, 379.822007, 405.832623 | 405.832623 |
| recovery | compile-plus-runtime-suite | place43-retained | 289.040169, 292.930753, 325.564985, 328.426540, 329.377902, 347.779792, 379.891344, 380.659072, 406.649414 | 406.649414 |
| recovery | compile | stock | 80.786472, 82.298401, 86.599742, 88.174285, 89.166772, 90.429932, 94.852067, 96.995901, 97.772849 | 97.772849 |
| recovery | compile-plus-runtime-suite | stock | 81.337342, 82.859517, 87.080845, 88.777066, 89.766207, 91.025780, 95.486397, 97.658674, 98.520720 | 98.520720 |

Component43 remains outside unchanged product34/published28. Full native Rust, crates/metadata/rlib/Cargo, runtime-loader closure, stable-files-during-request assumptions, supervised lifecycle and full-project speed remain OPEN. All seven public-release gates held. Lust/TinyCC C/runtime, mimalloc, Linux/GLIBC are dependencies; no all-Rust or whole-toolchain memory-safety claim.
