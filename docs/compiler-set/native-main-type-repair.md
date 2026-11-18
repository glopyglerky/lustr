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
The main unit continues to regenerate on every build. Retained frontend and
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

## Current-tree local qualification

An isolated build of commit `6ccd976bfa80a41a94810992b349cc39f0b1310f`
produced a new standalone native client/core pair using the repository's
`gate` correctness profile. SDK and TinyCC inputs were reused read-only;
build outputs and intermediates were separate from the existing package.
The original package's recorded bytes and inodes remained unchanged.

Five focused fixtures ran twice: ordinary C generation and named release-C
generation. Each of the ten native executables matched the pinned Rust oracle's
numeric status, stdout and stderr exactly. All 40 compile/run commands completed.
The daemon and native cache were disabled; no compiler retry occurred.

| Fixture | Expected exit | Ordinary | Release C |
| --- | ---: | --- | --- |
| `main_unit_abi` | 0 | Pass | Pass |
| `main_result_err` | 1 | Pass | Pass |
| `exit_code_main` | 42 | Pass | Pass |
| `exit_code_result_ok` | 4 | Pass | Pass |
| `panic_hook_abort` | 101 | Pass | Pass |

[Sanitized verification data](evidence/native-main-type-2026-11-18/verification.json)
records exact source, compiler, fixture and stream hashes. The current native
integration tree is `d9ca06dc18dde19bf5bd2af688adf75a041b12a5`, normalized
patch `d026010dab63b9078f4f36eefcbe2f1b7868b64c`. Candidate source verification
passed; ordinary release-admission mode correctly refused candidate status.

No pinned Clang executable was available for this local run, and a real
Unix-domain daemon is blocked by the observed socket `EPERM` restriction. No
current-tree function-sanitizer or warm-transition pass is claimed. The local
run did not reconstruct the combined package or measure compiler performance.
Hosted exact-source sanitizer, strict retained-return transitions and platform
qualification remain required. The integration declaration retains `candidate`
status; these scoped checks do not admit an installable package.
