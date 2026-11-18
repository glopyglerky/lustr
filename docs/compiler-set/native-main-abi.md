# Native main function types

The native compiler called a Rust unit-returning main through a C
`void (*)(void)` pointer, although its emitted definition returns `lust_Unit`.
This candidate makes those types agree. It remains an unadmitted integration:
focused generated-C checks pass, but the local TinyCC runtime and retained-edit
qualification still fail.

## Defect and repair

The [preserved macOS gate](https://github.com/glopyglerky/lustr/blob/53c494dd40f48b192e9fac27c24566f0bc238d7c/docs/compiler-set/evidence/prior-macos-native-gate-20261118/README.md)
recorded 2,218 incompatible-function-pointer sanitizer failures. A new minimal
`fn main()` fixture reproduces that diagnostic with the original compiler and
the pinned Clang 23.1.5 runtime. The candidate passes the same fixture with the
same Rust oracle and sanitizer flags. Clang documents
[`-fsanitize=function`](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html)
as detecting indirect calls through an incorrect function-pointer type; it is
part of the `undefined` sanitizer group.

The production change is confined to `compiler/native/src/cgen.rs`:

- Ordinary unit main calls use `lust_Unit (*)(void)`.
- Release C declares a unit main as `lust_Unit name(void)`.
- Release C declares the panic hook runner with its actual signature,
  `lust_Unit name(uint8_t)`, matching its Boolean parameter and the runtime
  function-pointer type.
- Result and ExitCode mains continue through `lust_main_entry_p`, the existing
  typed Termination wrapper. Their actual return types must not be declared as
  either `void` or `lust_Unit` in the main unit. A fresh semantic flag uses the
  same return-type predicate as wrapper generation. If a required wrapper were
  absent, the generated fallback now aborts instead of calling an incompatible
  body signature. This is a fail-closed response to a broken internal invariant.

The main unit is regenerated for every request. Existing main-return
fingerprints and overlay initialization remain responsible for invalidation.
The flag is recomputed from the current program, not retained independently.
Independent source review checked all `main_unit` callers and the wrapper's
forced inclusion in the LLVM reference set. Execution after return-type edits
is still an open qualification requirement.

## Gates and limits

The [complete focused report](evidence/native-main-abi-20261118/README.md)
preserves all outcomes and exact source, fixture, binary and compiler hashes.
Pinned Clang 23.1.5 passes nine unchanged generated-C fixtures, six release-C
fixtures, and the minimal baseline-failure/candidate-pass regression. These
checks compare numeric exit status and stdout with rustc and reject sanitizer
diagnostics. They do not compare all program stderr bytes.

The same report records TinyCC timeouts, the initial incomplete attempt, an
LLVM 22.1.8 ASan failure, and a failed strict retained-edit run. Matching
baseline runs also fail the relevant TinyCC cases. One retained candidate case
crashes where the baseline times out; that difference remains unassigned.
No complete-platform, all-2,218-failures-resolved, panic/drop, unwinding-table,
or performance qualification is claimed.

The legacy daemon scenario runner treats two nonzero runtime results as an
unsupported case. That cannot qualify Result or ExitCode behavior. The new
[`test-native-main-transitions.py`](../../tools/test-native-main-transitions.py)
therefore runs alongside the historical suite. It requires accepted compilation,
evidence that both the repeated check and build use retained state, and exact
warm/cold/rustc exit status, stdout and stderr. Signals and timeouts fail. CI
retains its report and command streams even when the earlier native suite fails.
Local negative controls cover setup failure, partial timeout output, process
ownership and cleanup failure. The final runner has not yet passed a hosted
native campaign.

The pristine Lust revision and tree are unchanged. The declaration names native
tree `cfc7a49a820f7fbeed2e8b725bf779ccedf641dd` and normalized integration patch
`78b7306c0591110444adc16523c42b736bc2a19d`. The exact source guard passes in
candidate mode. Release admission still requires the remaining correctness,
independent exact-source review and appropriate performance evidence described
in [the upstream policy](upstream-lust.md).
