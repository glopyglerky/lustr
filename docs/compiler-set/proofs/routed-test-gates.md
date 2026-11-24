# Historical coordinator test-route binding

The coordinator test route described in the
[original record](https://github.com/glopyglerky/lustr/blob/ea26fe16636d390763257a19c19df3db992cdbcb/docs/compiler-set/proofs/routed-test-gates.md)
belongs to a retired full-compiler package. It is not a current Lustr test or
compilation entry. Follow the [exclusive coverage contract](../exclusive-coverage-contract.md)
and [current measurement guide](../benchmark-method.md).

The original proof checked exact OS-string identity for the selected Cargo,
rustc and rustdoc paths, direct argument-vector forwarding, actual backend
selection, nonempty test execution, panic recovery and destructor behavior.
Those are useful evidence requirements. They did not establish exclusive Lust
ownership or a selective Rust complement: both backend lanes used a complete
Rust frontend.

The original source, command recipe, bounded native tests and limitations remain
available at the immutable link above. The
[historical frontend measurements](../measurements/2026-11-09-shared-v2/README.md)
retain their actual direct-gate limitation. No result is transferred to a later
compiler, and no new acquisition command is recommended here.

## Preserved source-bound proof and checks

The following proof and checks retain the original coordinator implementation
scope. They do not describe an available current product route.

## Conditional invocation statement

Let `R` be the absolute root, `P` the absolute coordinator executable, `e` one
of `auto`, `rustc` or `cranelift`, and `A` the complete Cargo argument vector.
The helper requires that `A` starts with `test`, contains no `--no-run`, and
that the inherited selectors equal:

```text
CARGO   = R/bin/cargo
RUSTC   = R/bin/rustc
RUSTDOC = R/bin/rustdoc
```

The equalities use exact OS strings, not basenames, substring tests or a guessed
compiler family. Missing or different selectors fail with validation exit 42
before starting the coordinator. Paths are absolute. No shell performs expansion.

For accepted inputs, the child invocation is exactly `P` with arguments
`[--bluefe-root, R, --engine, e] ++ A`, where `++` means sequence concatenation.
`std::process::Command` receives each OS string directly. The helper inserts
nothing into `A`. Under the qualified coordinator's
[root selection contract](../bluefe-performance.md), root
selection sets the three variables to the same values already checked here.
Therefore this route cannot silently select a different root by overriding a
declared selector. This is a statement about command construction, conditional
on the supplied coordinator's behavior and trusted tool identities. It is not
a sandbox or an attestation of an arbitrary executable named `lustr`.

The coordinator's documented backend rules still apply. Caller profiles,
target choices and later configuration can preserve or override a backend
selection. A requested engine alone does not prove which backend executed.
Inspect actual root compiler commands and backend-load evidence for the
configuration being qualified. No project flags or settings should be removed
to make a gate pass.

## Executed checks and remaining limits

The opt-in native fixture uses the produced preview compiler and its matching
unwind-enabled Cranelift component. It runs the same nonempty tests through
LLVM and Cranelift, checks the actual selected-root `rustc --test` command,
and requires the backend path on the Cranelift invocation. The test body catches
a panic and asserts exactly one Drop. Three mismatched or missing core selectors
and a `--no-run` request must fail validation before child-output capture. These are
bounded tests of the real route, not a compiler-equivalence proof.

The helper retains the child's exit status and length-delimited raw stdout and
stderr in its stderr evidence stream. For comparison, it emits the same sorted
test names/counts as the direct-Cargo gate. An unsuccessful child, absent
nonempty test harness or invalid route returns validation 42; that is not
application acceptance. Raw evidence is separate from normalized comparison
bytes. The [measurement method](../benchmark-method.md) defines those limits.

This route qualifies neither every dependency/backend ABI nor all project
tests or doctests on a target. It does not apply `--frontend-jobs` to `test`:
that coordinator option is restricted to Cargo `rustc`. The published
[historical frontend measurements](../measurements/2026-11-09-shared-v2/README.md)
retain their original direct-gate limitation. No historical observation or
speed ratio is changed by this helper addition.
