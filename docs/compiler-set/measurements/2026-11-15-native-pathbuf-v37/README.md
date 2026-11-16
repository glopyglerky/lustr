# Native PathBuf constraint expansion — component37

The unchanged first witness with delayed `PathBuf` `.into()` and `Vec<PathBuf>::extend` failed in component36 with native L0610/L0640. Stock compiled and ran it. The owned checked-target rule now admits canonical platform `PathBuf` alongside `OsString`, only for the existing checked `&str`/String source forms. The platform-root/ADT/empty-generic guards, competing method/custom Extend refusal, original-source handling and full native recheck remain. No protected Lust/TinyCC/blueFe source changed. This is bounded language coverage, not general Rust inference.

The exact first source now compiles and runs through optimized native37, with stdout identical to the preserved stock runtime. The separate mixed fixture combines PathBuf alias conversions, old OsString support, arena ordering, Drop and unwind. All7 direct native/oracle suites pass in development and optimized libraries; five new invalid borrowing/move/mutability/trait/Extend cases remain rejected. The new retained test passes in both profiles across PathBuf-to-OsString target edits, invalid-source rejection and recovery, with real cold native/stock runtime agreement. The existing unrelated product and retained-host suites were not replayed as this rule does not change their mechanisms.

A fresh frozen gpt-6.1-sol HIGH/default read-only review approved all4 changed implementation/test files with no findings. Root authored code, tests and harness as gpt-6.1-sol XHIGH/default through T3 Code -> Codex harness. Reviewer ran no builds/tests and was task_cancel-retired after completion, with null-active/zero-pending terminal receipt. Root's actual runtime checks supply qualification. No release or product promotion occurred.

## Fresh successful native timing

Nine interleaved samples of a separate mixed PathBuf/Drop/unwind source generator, shared Intel Core Ultra5 235T Linux GNU x86_64 NAS, CPUs10–13/jobs2. One fixed successful component37 direct-native control, the SAME component37 retained ABI and stock rustc compile identical source with opt0/debug0/unwind/overflow checks/debug assertions. Both native paths use optimized opt3/CGU1/ThinLTO/line tables. Cold includes a fresh retained process; repeat/noop validate retained success. Source body/signature/type/macro edits, invalid edits and recovery are distinct. Every accepted artifact is real ELF; native/stock runtime stdout/stderr/status agree. No native probe invokes rustc or retries an executed application. Raw `prior-direct` is a legacy engine label; here it identifies successful component37, never the failing component36.

Median milliseconds, lower is better. Runtime is the same real Drop/unwind executable per accepted build; this is not Cargo test.

| Phase | Direct native37 | Retained37 | Stock | Direct + runtime | Retained + runtime | Stock + runtime |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 150.823 | 295.669 | 77.454 | 151.554 | 296.776 | 78.106 |
| repeat | 153.788 | 11.334 | 75.098 | 154.472 | 12.009 | 75.804 |
| noop | 152.956 | 10.103 | 75.495 | 153.634 | 10.974 | 75.974 |
| body-edit | 156.928 | 299.529 | 74.604 | 157.732 | 300.734 | 75.138 |
| signature-edit | 154.585 | 290.409 | 75.947 | 155.293 | 291.159 | 76.461 |
| type-edit | 148.227 | 300.200 | 76.331 | 148.876 | 301.379 | 76.894 |
| macro-edit | 156.610 | 291.420 | 74.878 | 157.343 | 292.326 | 75.513 |
| rejection | 89.618 | 98.276 | 28.070 | — | — | — |
| recovery | 151.247 | 292.083 | 78.146 | 152.010 | 293.079 | 78.682 |

Cold and edits remain slower than stock. Retained no-op10.103ms versus direct152.956ms and stock75.495ms is bounded unchanged-artifact evidence; it does not establish full-project speed. Do not compare these PathBuf rows to the distinct earlier OsString workload as an optimization gain. The [component36 proof-cost campaign](../2026-11-15-native-host-inputs-v36/README.md) separately retains its cold/edit regressions and coordinator Cargo no-op loss.

757 raw events; raw SHA256 `73b0635eed10182c4e82537d8ecbaa7b665999ee0c668436a64838f783781250`; summary `daafcf6a424ec6b7319688d653fde5e4c0f125022eb55a81527306e36a611538`. Summary retains all9 samples and maxima per row. Core `08a1f95aa4029c803311f0e1ddab0f4f86f524d4cdce6d91491768f2356089ec`; observer `0b77fb05f7f3ffbf244034aff4f639bac78931322b39f06db00fb9fe5c0d108f`; measured harness source `10f7b7a450302c67621469121eaf129aff7192c280acae0c16664b22ddf78262`. Exact commands, source/SDK/TinyCC identities, statuses, diagnostics and frozen helper sources remain in private evidence. The first execution passed,36.24s campaign wall time. The optimized compiler itself built in426.09s; that is compiler BUILD time, not application compilation performance.

This new component is experimental and outside product34/published28. No transformed AST survives a request. Compiler host-file input proof assumes stable files DURING each request; runtime-loader closure, broad configurations, general cache/Cargo, supervised product lifecycle, Rust metadata/rlib and full language/speed acceptance remain OPEN. All wider public-release gates remain held.
