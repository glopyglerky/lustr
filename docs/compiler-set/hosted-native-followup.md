# Hosted native qualification follow-up — 19 November 2026

The Linux job for PR #30 at `0170b9834a15f104f303d4497f93a00773564bd0`
passed the full ordinary differential corpus and both unwind suites, then
reported two separate qualification failures. This continuation fixes their
demonstrated causes. It does not turn that failed hosted job into a success or
establish package admission.

The actual Actions checkout was the synthetic merge
`91735c0778238f7ea3800c4ce6a4d91d2413e3ff`. Its Git tree
`0e132a28f854afa6544f63da633229061fd84ede` equals the published head's tree.
The [original job](https://github.com/glopyglerky/lustr/actions/runs/44465150002/job/135361018008)
and [source-bound verification record](evidence/hosted-native-followup-20261119/verification.json)
retain this distinction.

## Observed hosted results

| Check at the original source | Result |
|---|---|
| Native unit tests | 410 passed, 0 failed |
| Ordinary differential corpus | 5,540 passed, 0 failed |
| Unwind corpus through native execution | 65 passed, 0 failed, 0 unsupported |
| Unwind corpus through emitted C exception tables | 65 passed, 0 failed, 0 unsupported |
| Rust macro corpus | 44 passed, 14 stored expansion mismatches |
| Independent Unicode data verifier | Link failed before verifier execution |

The main gate stopped at the macro corpus. Its later sanitizer, fuzzer and
other stages therefore have no passing result from this run. The independent
unwind steps ran after the failure and passed. Neither newer NFC code nor the
new semantic-import slice was in this hosted source.

## Reconcile the macro oracle with the declared Rust version

The macro runner compares `rustc -Zunpretty=expanded` with stored canonical
tokens and separately compiles and runs each fixture. Fourteen stored token
files described Rust 1.99's builtin derive expansions, while the candidate
workflow now correctly uses the declared Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.

Before changing an expectation, the comparison ran both actual compilers:

- Rust 1.99 reproduced every one of the fourteen old `.expanded` files exactly.
- Pinned Rust produced the replacement expansions. A separate token audit
  established equality outside `#[automatically_derived]` implementations for
  every affected fixture. The number and placement of those implementations
  remained the same.
- All fourteen original programs compiled and ran under both compilers. All
  twenty-eight executions produced the original `.out` bytes with empty
  runtime stderr.

The changes agree with the pinned upstream implementation: derive signatures
and variant paths use `Self`; Debug derives use `debug_struct_fieldN_finish`
for both tuple and named forms and combine type and field names into one
string. See pinned
[Clone](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_builtin_macros/src/deriving/clone.rs),
[Default](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_builtin_macros/src/deriving/default.rs),
[shared derive generation](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_builtin_macros/src/deriving/generic/mod.rs)
and [Debug](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_builtin_macros/src/deriving/debug.rs).

Only the fourteen proven expansion files were replaced. The original fixture
sources, runtime outputs, macro normalizer, runner and rejection expectations
remain byte-identical. The separate derive-range audit was an investigation
step; the actual gate still compares the complete expansion, including every
derive implementation. The full unchanged macro runner then passed **58 cases,
0 failures**, with **70/70** required ts-rust macro names covered.

Per-fixture old/new hashes and both exact compiler identities are in the
verification record. Earlier Rust 1.99 expectations remain in Git history and
the preserved comparison evidence. This is a declared-version expectation
migration, with finite runtime controls; it is not a proof of equivalence for
all possible derives or a relaxation of the frozen historical oracle.

## Link the Unicode verifier against the acquired SDK

The hosted Unicode failure was `unable to find library
-lLLVM-23-rust-1.102.0-nightly`. It happened while building the independent
`rustc_private` data verifier. No Unicode comparison had executed.

The old command relied on `-C rpath`, whose compiler-library path selected
`lib/rustlib/<host>/lib`. The acquired SDK keeps LLVM under its top-level `lib`
directory. The earlier local SDK also contained duplicate LLVM files in the
target directory, so the old command passed locally even after inherited
loader-path variables were removed. That non-reproducing control is retained.

An isolated SDK view retained the same compiler and library bytes but exposed
LLVM only in the top-level SDK directory. The original command then reproduced
the hosted link error. The repaired command supplies
`-L native=<exact sysroot>/lib`; only the data verifier's execution receives
the corresponding loader search directory. Fixture programs retain their
previous environment.

With that layout, the targeted repaired verifier command built and executed
successfully with empty stderr. It checked all **1,112,064 Unicode scalar
values**, seven predicates, debug escaping, XID, combining classes and canonical
decomposition, plus **961 composition pairs** and the decomposition-pair
exclusions. The same candidate flags were used; this targeted check is distinct
from a new full hosted qualification of the composed compiler.

The original job logs and downloaded artifact remain preserved. The archive's
SHA-256 was checked against GitHub's artifact digest before its stderr was read.
No source pin, sanitizer probe, comparison normalizer or compiler assertion was
removed to obtain these local results.
