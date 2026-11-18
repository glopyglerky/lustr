# Native main C type repair

This candidate ports the narrow, already investigated type correction from
[PR #24](https://github.com/glopyglerky/lustr/pull/24) onto the current PR #26
native tree. It does not import that branch's older integration identities or
its separate macOS sanitizer-installation work.

The original `cgen.rs` is byte-identical to the PR #24 failing baseline:
SHA-256 `8ab4c3e87610597e5d9c339afce898973b8e7913a532aef97c7fec59638dccdd`.
It emits a unit main definition returning `lust_Unit`, then calls it through a
`void (*)(void)` pointer. Release C also declares the panic hook runner as
`void name(void)`, although the actual function and runtime require
`lust_Unit name(uint8_t)`.

## Scoped repair

Only `compiler/native/src/cgen.rs` changes in production native source:

- Call ordinary unit main through `lust_Unit (*)(void)` and declare its named
  release-C body with the same return type.
- Declare the named panic hook runner with its actual `uint8_t` parameter and
  `lust_Unit` return.
- Keep Result and ExitCode main calls in the existing typed Termination
  wrapper. Recompute the wrapper requirement from the current program using
  the same predicate as wrapper generation; do not declare their bodies as
  unit or void functions. A missing required wrapper aborts instead of calling
  an incompatible function type.

The ported codegen file has SHA-256
`1cbce6afffc8f3df96032b5588736cbd861033feba8111bec5047889472c12b1`.
The main unit continues to regenerate on every request. Retained frontend and
no-change materialization logic are unchanged.

The focused fixtures are `main_unit_abi.rs` and the four-version
`mainret04_typed_entry_transition` scenario: Result error, unit, Result of
ExitCode, unit. The strict runner requires successful check/build, retained
daemon evidence, and exact warm/cold/Rust exit status and stdout/stderr. A
signal, timeout, lost retained state or cleanup failure fails the gate. CI runs
this gate and retains its report/streams on both native platforms.

Independent source review confirmed the exact types and existing wrapper
selection, all four production `main_unit` callers, unchanged NoChange source,
and strict CI failure/evidence handling.

## Evidence boundaries

The [original focused evidence](https://github.com/glopyglerky/lustr/blob/fa25833bfab60b8f3689f71fad603be417b8de2c/docs/compiler-set/evidence/native-main-abi-20261118/README.md)
contains a real baseline failure and corrected pass under pinned Clang 23.1.5:
UBSan rejects the incompatible unit-main indirect call in the baseline.
Nine corrected generated-C fixtures and six release-C fixtures pass their
documented checks. Those runs used the older PR #16 native tree on macOS arm64;
they are supporting evidence for this exact codegen change, not a qualification
of the current complete native tree.

That same older investigation retains plain TinyCC timeouts and a candidate
warm SIGSEGV where its baseline timed out. The cause remains unassigned.
Current PR #26's separate five cold Linux return cases passed through the
native library. Neither observation supersedes the other: they differ in
source, platform and retained state. Cold runtime success also does not make
the incompatible C function type correct.

Current-port local qualification is being recorded separately. The local host
has no available pinned Clang executable, and a real Unix-domain daemon is
blocked by the observed socket `EPERM` restriction. No local sanitizer pass or
warm-transition pass is claimed. Hosted exact-source qualification remains
required. The integration declaration retains `candidate` status; source-guard
success does not admit an installable package or establish a new speed result.
