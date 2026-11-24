# Native inference candidate measurements — 2026-11-15

The opt-in native candidate compiles the unchanged complete `crates/lustr`
coordinator and matches five real Cargo/rustc CLI cases. Its compilation is
slower than stock in this campaign. Native language/crate completion and the
required speed remain open. The published preview is unchanged.

## Configuration and meaning

Nine interleaved samples per engine/phase ran on the shared Intel Core Ultra 5
235T Linux x86_64 host, CPUs 10–13, Cargo jobs 2. Application settings were opt0,
debug0, unwind, overflow checks and debug assertions on. The compiler component
itself was built at opt3/CGU1/ThinLTO with line tables. Cold means native caches
disabled and fresh Cargo target AND build directories, with OS pages untouched.
Repeat/noop reuse those directories. No native daemon was used. Single-file
`noop` means another identical compiler request; stock rustc still recompiles.
The coordinator stock noop is a genuine Cargo Fresh result with no rustc call.

Every successful product timing has a native engine trace and excludes full-core
fallback. Direct ABI timings invoke no rustc fallback. Rejection product timing
includes the explicitly recorded compatibility rejection path; the direct ABI
column supplies native-only rejection latency. Both are retained distinctly.
Source bytes and runtime stdout/stderr/status are compared between independently
built engines. Recovery follows an invalid edit. Compile-plus-runtime-suite
includes self-asserting drop/unwind execution, or all five coordinator CLI cases;
it does not claim that a native Cargo unit-test harness is implemented.

The unchanged coordinator is compiled as one native target, versus Cargo's real
library/bin closure. This compares the requested executable and observed behavior,
not Rust library metadata interoperability. Actual stock compiler invocations,
Fresh results, source/dependency hashes and compiler/runtime hashes are in raw
records and captured logs. Selected Cargo dev flags were inspected in actual
`-vv` output. The primary product Lust binary retains its earlier 1.99 producer;
the matched native control, new inference component and stock oracle use the
pinned 1.102 SDK. The producer difference remains a control confounder; a subset
ratio cannot be attributed solely to this frontend change.

## Unchanged full coordinator

Medians in milliseconds; lower is better.

| Phase | Native product compile | Cargo/rustc compile | Native compile + CLI suite | Cargo/rustc compile + CLI suite |
| --- | ---: | ---: | ---: | ---: |
| cold | 639.770 | 471.450 | 643.695 | 475.190 |
| repeat | 676.783 | 11.240 | 681.949 | 15.201 |
| noop | 680.190 | 11.677 | 684.890 | 15.185 |

The candidate has no retained inference session or manifest no-op implementation.
Its repeated native compilation and Cargo preflight work are included. A cold
native gain or fast no-op is not established.

## Mixed inference workload

This owned workload combines checked OsString conversions, macros, struct and
signature edits, closures, overflow checks and drop during unwind. All successful
native/runtime comparisons passed. The unchanged coordinator above is separate
whole-project evidence.

| Phase | Native product | Direct native ABI | Stock rustc |
| --- | ---: | ---: | ---: |
| cold | 307.529 | 245.795 | 81.991 |
| repeat | 334.658 | 246.456 | 84.717 |
| noop | 297.249 | 250.879 | 85.297 |
| body-edit | 293.485 | 247.809 | 83.639 |
| signature-edit | 285.763 | 217.651 | 77.571 |
| type-edit | 278.057 | 219.359 | 78.170 |
| macro-edit | 279.907 | 217.434 | 76.294 |
| rejection | 245.652 | 158.695 | 27.982 |
| recovery | 294.366 | 222.964 | 80.894 |

All phases are slower than stock. A separate supported-source control also ran
against matched pristine Lust: cold medians were 147.420 ms matched, 141.475 ms
product and 88.984 ms stock. No fast-path improvement is inferred from that small
difference under the producer/shared-host confounders. No gain against a failed
pristine inference compilation is claimed.

## Receipts and preserved failure

Private campaign receipts contain 2,818 raw events: 2,323 single-file and 495
coordinator. Each retains source identities, argv, cache state, routing, status,
raw duration and stdout/stderr. Median and nine-sample p90/max values remain in
summaries. The initial harness stopped after completing the single-file samples
because its copier encountered src/bin. That first failure remains preserved.
The copier was fixed and only the coordinator portion ran separately; successful
single-file samples were not replayed.

- Single-file raw SHA256: `441d22047ab76139ef83311f605f2906f7c183bb79fd072031fcb9ad276b2aa4`.
- Single-file summary SHA256: `dda41d66703637a85363841fcf98fa6d0a77271e71c122f7be8acd5c161c3538`.
- Coordinator raw SHA256: `b43da34e3747bda99dd90fa1ba0b7f0a55dca848ec1748e4c0603fffde15facd`.
- Coordinator summary SHA256: `aa1a2cf977c374d24b40c89acb9438d1a4bf77f015aab12e524a02a7d4059bd2`.

The next optimization must remove redundant native frontend passes and establish
sound retained inference/dependency reuse, while preserving original success,
full rechecking, stale/refusal gates and runtime parity. Those changes are not
implemented by this report. No release admission, full Rust, metadata/rlib,
general Cargo, never-slower or whole-toolchain memory-safety claim follows.
