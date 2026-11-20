# Retained native check measurements — 2026-11-15

The private opt-in candidate removes one redundant failed native build. It starts with a retained unchanged native Check and uses that checked failure for owned constraint elaboration, followed by complete native collect/check/borrow checking/generation/linking. Passing ordinary checks still require a real Build. The original library/foreign-target build guards remain authoritative. The primary native core, entry and worker binaries are unchanged from v33. Cross-request inference sessions and dependency-validated no-op reuse are not implemented.

Nine samples per engine/phase were interleaved with alternating order on the shared Intel Core Ultra 5 235T Linux x86_64 host, CPUs 10–13, Cargo jobs 2. Both inference compilers use the pinned unmodified Rust 1.102 SDK as producer, opt3/CGU1/ThinLTO/line tables. Applications use opt0/debug0/unwind with overflow checks and debug assertions. Cold disables native caches and starts fresh Cargo target AND build directories; OS pages remain warm/uncontrolled. Repeat/noop reuse directories; there is no native daemon. Single-file noop is an identical compiler request; coordinator stock noop is real Cargo Fresh. The measured root and worker are identical between products; only the inference DSO and dependent seals differ. Shared-host noise remains.

All successful product samples assert native traces without full-core fallback. All successful engines independently emit ELF artifacts and match runtime stdout/stderr/status. The coordinator uses unchanged full owned source without unrelated bench WIP, and five real CLI cases including invalid-CLI status 2. Stock compiles its actual library/bin Cargo closure; native whole-target results do not establish Rust rlib/metadata interoperability. Actual stock -vv flags and Fresh output were inspected. Rejection product time includes explicit compatibility rejection; the direct ABI supplies native-only rejection. Compile-plus-runtime is not Cargo test.

## Unchanged full coordinator

Medians in milliseconds; lower is better.

| Phase | Prior native product | New native product | Stock Cargo/rustc | Prior compile + CLI | New compile + CLI | Stock compile + CLI |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 594.157 | 537.983 | 415.693 | 598.733 | 542.591 | 418.848 |
| repeat | 587.214 | 538.393 | 10.800 | 591.328 | 542.916 | 14.328 |
| noop | 627.012 | 489.039 | 10.430 | 632.597 | 493.764 | 13.760 |

## Mixed inference workload

The unchanged workload combines OsString constraints, macros, body/signature/type edits, overflow checks, closures and drop during unwind. Recovery follows an invalid edit.

| Phase | Prior product | New product | Prior direct ABI | New direct ABI | Stock rustc |
| --- | ---: | ---: | ---: | ---: | ---: |
| cold | 306.570 | 229.580 | 242.907 | 162.053 | 80.948 |
| repeat | 312.319 | 227.225 | 244.570 | 166.994 | 81.979 |
| noop | 295.268 | 227.945 | 239.099 | 172.206 | 83.581 |
| body-edit | 291.227 | 220.932 | 243.700 | 175.921 | 84.722 |
| signature-edit | 305.798 | 233.378 | 227.642 | 157.798 | 80.413 |
| type-edit | 298.450 | 236.202 | 244.149 | 174.611 | 82.719 |
| macro-edit | 331.862 | 259.820 | 262.970 | 183.493 | 84.528 |
| rejection | 327.758 | 223.281 | 173.913 | 105.530 | 31.583 |
| recovery | 297.510 | 224.228 | 262.504 | 184.747 | 87.718 |

Relative to the paired prior candidate, cold inference compilation decreases 25.1% in the product and 33.3% in the direct ABI; cold coordinator product time decreases 9.5%. Every new native phase remains slower than stock. This is a bounded improvement over the prior native candidate, not acceptance of the full speed requirement. No speed ratio against failed pristine inference compilation is claimed.

The supported-source control cold medians were 108.909 ms matched pristine, 109.253 ms product, 71.793 ms stock. The primary product's earlier 1.99 producer differs from the matched 1.102 control; no subset change is attributed to this optimization. All control samples and regressions remain in the raw record.

## Checks and receipts

All nine optimized integration suites passed, including invalid/ambiguous programs, unchanged coordinator, Cargo environment/refusal contexts, mixed drop/unwind, real final Build failure, THP routing and worker status 3/SIGTERM propagation. The primary ABI also type-checks with inference disabled. A frozen independent review found no P0/P1 and one P2: Check could replace the original foreign-target Build refusal. The unfixed development artifact reproduces status 1 instead of 2; the corrected optimized artifact matches pristine status 2 and exact diagnostic. First failure and review are preserved. An initial development build failed on a cfg expression placement; the corrected rebuild passed.

Actual native visit logs show the unchanged coordinator executes two driver calls now versus three before. Final native Build still reports collect=2125 and checked=1027, with 1,350 functions, 1,027 generated functions, 14 compiled units and a linked ELF. This is retained work within one request, not no-op or general incremental acceptance.

The complete campaign retained 3,834 raw events, source/compiler/runtime identities, commands, durations, statuses, traces and stdout/stderr. No failed sample was silently retried.

- Raw SHA256: `49ef85e0ac5b16bed8d71493bfc3001658aa03e787cc3abb5f59c567652cc8be`.
- Summary SHA256: `f05900667fe5f9dc51549566f8ac9935735f87106e891ba14fa69f2e09b444ce`.

The published preview remains unchanged. Full native Rust, metadata/rlibs, general Cargo and required native speed remain open. The upstream L0809 assert-without-message gap remains separate and open.
