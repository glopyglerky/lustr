# Native compiler file-input proof cost — component36

Fresh nine-sample campaign on the shared Intel Core Ultra 5 235T NAS, Linux GNU x86_64/AerynOS, CPUs10–13/jobs2. This measures NEW optimized component36 against one fixed successful direct-native component35 control and actual stock Rust/Cargo. It imports no v35 measurements. All engines compile unchanged equivalent source, emit real ELF and match runtime stdout/stderr/status. Rejection stays status1; recovery is a fresh native compile. No rustc fallback exists in either native probe.

Both native libraries use opt3/CGU1/ThinLTO/line tables. Applications use opt0/debug0/unwind/overflow checks/debug assertions. Each sample starts a new bounded retained process; cold includes startup and first response. Only repeat/noop should hit. Every source edit runs original native checking/generation/linking and the new host-file/C-unit replay proof. Session disk caches are disabled before Check. Stock coordinator uses real Cargo, fresh target AND build roots per sample, then actual Cargo Fresh. Engine order alternates. OS pages are untouched. Shared-host noise and different process lifetimes limit generalization. Compile-plus-runtime is not Cargo test or Rust rlib equivalence.

## Mixed Drop/unwind and native inference

Median milliseconds, lower is better. Runtime is the same observed Drop/unwind program per accepted build.

| Phase | Direct native35 | Retained36 | Stock | Retained36 + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: |
| cold | 175.929 | 345.034 | 88.516 | 345.820 | 89.120 |
| repeat | 177.511 | 11.512 | 82.737 | 12.397 | 83.330 |
| noop | 176.013 | 11.046 | 84.426 | 11.826 | 85.107 |
| body-edit | 169.285 | 348.547 | 83.046 | 349.371 | 83.676 |
| signature-edit | 178.861 | 339.059 | 81.842 | 339.894 | 82.565 |
| type-edit | 180.228 | 348.464 | 83.207 | 349.319 | 83.905 |
| macro-edit | 177.269 | 329.166 | 81.936 | 330.050 | 82.546 |
| rejection | 100.290 | 115.397 | 32.071 | — | — |
| recovery | 176.887 | 353.487 | 90.469 | 354.340 | 91.138 |

## Unchanged complete coordinator

Runtime is the five real CLI cases, including actual plan behavior and invalid CLI status2.

| Phase | Direct native35 | Retained36 | Stock | Retained36 + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: |
| cold | 247.998 | 445.718 | 403.641 | 449.126 | 406.028 |
| repeat | 251.274 | 17.125 | 10.041 | 20.664 | 12.397 |
| noop | 243.364 | 11.931 | 10.024 | 15.277 | 12.339 |

## Result and exact evidence

The new proof adds substantial miss cost: mixed cold345.034ms versus direct175.929ms; coordinator cold445.718ms versus direct247.998ms and stock403.641ms. Mixed retained no-op11.046ms remains faster than one-shot direct176.013ms and stock rustc84.426ms. Coordinator no-op11.931ms is slower than actual Cargo10.024ms; compile-plus-five-CLI15.277ms is slower than stock12.339ms. Full native speed acceptance remains OPEN. This correctness proof was retained for its stale-result fixes, not claimed as a measured speed improvement. Do not transfer historical v35 gains to it.

1351 raw events. Raw SHA256 `d24eb25683f114c601622acd57c3c6077d41ce6aab64277c0a490357cb7fca36`; summary SHA256 `2351188317555377cd34475be8554711febca125cf110051a941ed026e0498f7`. Summary retains all nine samples and maxima separately for every row. Raw records bind exact commands, source/compiler/observer/TinyCC/SDK hashes, statuses and diagnostics. Component36 core `a928092f7fe244824807069531e5ebc7ee9ae6752b0847c473a4882a07180649`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`; direct35 core `e68fa363a7b9b6031d7a8cfee69c8decd24bae1bf08229f99025fbbc6c593f63`. The measured Rust harness is examples/native_host_inputs_measurement.rs, SHA`c7e9359e5b8b8a9f8239c59715f86439f98766dd8e67cf21480b2cc6b9fd548f`. Exact measured helper source snapshots remain in private evidence.

The first harness BUILD failed for a missing frozen-core environment setting; the second refused an incorrect Lust runtime pin. Both first build failures and the corrected successful build are preserved. No application measurement was started by those failed builds. The fresh measurement campaign passed in its first execution,56.48s; this duration is campaign wall time, not compilation performance.

The host-input scope is bounded to inspected TinyCC archive/child and Linux ELF file inputs, with stable files DURING a request. Runtime-loader closure, general cache/Cargo, product lifecycle, full native language and Rust metadata/rlib remain OPEN. Product34, published28 and all wider release gates are unchanged. Neither proof nor this timing campaign admits a service/default or whole-stack memory safety.
