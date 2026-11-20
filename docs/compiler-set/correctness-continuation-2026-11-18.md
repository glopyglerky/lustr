# Compiler correctness continuation

This continuation starts from PR #29, published commit
`a9fd4c6336b1e39c4f584f7da50a437c5c248a0d`. The governing
[integrated compiler plan](architecture-plan.md) remains required. The changes
and finite-domain checks below do not establish full compiler correctness or
completion of that architecture.

## Completed qualification of the preceding source

The [PR #29 Compiler set run](https://github.com/glopyglerky/lustr/actions/runs/44420439480)
finished successfully with all ten jobs passing. This includes the actual
source-built mixed compiler, retained metadata, mixed-library runtime cases,
the full entry contract and the recorded performance campaign. These results
belong to that source, not to a later repair.

The [native Linux job](https://github.com/glopyglerky/lustr/actions/runs/44420439581/job/135211657133)
also passed its full gate. The [native macOS job](https://github.com/glopyglerky/lustr/actions/runs/44420439581/job/135211656703)
failed during sanitization. Its 47 reported failing cases exposed indirect
destructor calls through incompatible C function-pointer types. This is a
generated-code defect. A Linux GCC sanitizer pass does not establish that the
indirect function types are valid.

## Exact erased callback types

Generated drop functions previously accepted a pointer to their particular
concrete type, while vtables, channels and unwind cleanup called them through
`void (*)(void *)`. A cast of a function pointer does not change the function's
actual type.

All generated drop functions now accept `void *` and convert that object
pointer to the concrete type inside the body. This keeps their signatures
consistent for direct and indirect calls. The runtime-owned String panic
payload and thread exception catcher use correctly typed adapter functions.
The repair preserves destructor execution and the existing sanitizer checks.

The [regression gate](../../tools/test-native-erased-callbacks.py) asks the C
compiler to verify the types of the actual emitted declarations. It also
compiles and executes the emitted C and compares its status and stdout with
native execution and the selected Rust reference. Its first ten fixtures cover
dynamic trait objects, weak references, channels, panic payloads, thread
results, unwind drops and synchronization. The preceding compiler fails this
gate on the runtime String callback. The repair passes 126 declaration checks
across 60 emitted C units and all ten runtime comparisons. A subsequent run of
all 47 fixtures that failed on macOS passes 2,647 declaration checks across 444
emitted C units and all 47 runtime comparisons. This local result
does not replace the full macOS sanitizer gate. Panic stderr remains covered
by the existing full differential suite.

## Exhaustive 8-bit arithmetic checks

The [arithmetic fixture](../../compiler/native/tests/pass/integer_exhaustive_8bit.rs)
checks all 65,536 signed pairs and all 65,536 unsigned pairs. Addition,
subtraction and multiplication are evaluated with 32-bit intermediates, which
can represent every intermediate in these 8-bit domains. The expected wrapping
result is the residue modulo 256. Signed interpretation maps residues 128
through 255 to -128 through -1. Checked operations use range membership;
saturating operations clamp to the range; overflowing operations must return
both the residue and the correct overflow flag.

Division and remainder use wider signed arithmetic. The fixture separately
checks zero divisors and the signed minimum divided by -1. Euclidean remainder
uses the nonnegative residue below the divisor's absolute value. Shift checks
cover every 8-bit value with twelve counts, including zero, width boundaries
and `u32::MAX`. Expected left shifts use widened multiplication. Expected signed
right shifts use floor division. Rotations use independently widened bit
expressions. Bitwise operations, comparisons, absolute differences, negation,
bit reversal and population counts are also checked.

Native compilation and the pinned Rust compiler both execute all checks with
the same successful output. A separate GCC undefined-behavior sanitizer build
also passes. The local combined address/undefined/leak run executes the checks
but exits unsuccessfully because LeakSanitizer cannot operate under the
execution environment's tracing. That failure is retained and does not count
as sanitizer qualification. The hosted full sanitizer gates remain required.

These checks establish the enumerated finite domains and operations. They are
not a proof for all integer widths, floating point, all Rust semantics, or the
mixed Rust/C ABI. The fixture remains in the normal differential corpus so
future emitter changes exercise the same independent arithmetic formulas.

## Shared Unicode semantics

The [Unicode repair](unicode-semantics.md) now derives compiler and runtime data
together from the declared Rust source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.
Both the component build and the combined callback/Unicode build pass all
1,112,064 scalar values. The actual runtime comparison reads 114,666,345 output
bytes from each compiler and finds them equal. The two original predicate
witnesses also agree during constant evaluation and runtime. A retained native
Session test covers cold, unchanged, body-edited, constant-edited and reverted
builds. These are checks of the named finite domains and operations.

## Wider checked mixed functions

The [integer and module extension](mixed-integer-modules.md) admits compatible
32-bit signatures and narrow integer returns, retains exact Rust/C ABI equality,
and binds equal-named module functions to their real definition identities.
Narrow integer arguments stay with imported Rust because their ABI extension
attributes differ. Borrowed parameters are excluded before constructing a
dummy function ABI: the previous ordering panicked on a valid Rust function.
Identifier spelling is also a checked capability precondition. Rust accepts and
normalizes decomposed identifiers; native Lust currently requires NFC. Such
definitions remain in the imported compilation instead of failing a later
native projection. Canonical Unicode identifiers, raw identifiers and Unicode
comments retain their eligible paths.

The source-bound record contains 34 semantic-driver tests, 13 focused native
tests and 21 workflow guards. Actual Rust callers execute 18 cold/unchanged/edited
native object handoffs, including 1,179,738 independent arithmetic assertions
for the added integer signatures. The source-built Cargo gate is a separate
hosted requirement; local object linkage does not establish that gate.

## One declared Rust source and preserved corpus changes

The active native jobs now install and verify the same SDK source as the
integrated complement. An unchanged ordinary corpus run of the combined
callback/Unicode source produced 5,432 passes and 98 failures. Every failing
case was preserved before the harness cleanup. Earlier Rust 1.99 passes do not
qualify the native runtime against Rust 1.102.

The native String and iterator repairs follow the pinned upstream
[String implementation](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/alloc/src/string.rs),
[iterator methods](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/iter/traits/iterator.rs)
and [ordering defaults](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/cmp.rs).
Character collection allocates from its initial lower size hint, then asks
again during extension. This preserves both capacity and observable hint calls;
collection from references retains its distinct allocation behavior. UTF-16
lossy conversion follows the same initial allocation rule. Iterator minimum and
maximum, including keyed forms, compare the candidate with the current best
through `PartialOrd`; comparison calls and equal-value selection are tested.
Ten focused programs match the pinned compiler after these repairs. The
[version-alignment record](evidence/native-version-alignment-20261118/verification.json)
binds them and 23 acceptance/rejection checks to the exact candidate native tree
and binary. Both removed box-pattern regressions are accepted by the preceding
native compiler and rejected by the repaired compiler and pinned Rust.

Twenty existing negative fixtures are now valid Rust at the declared pin.
Each was independently checked with both Rust 1.99 and the declared compiler:
the old compiler rejects it, the declared compiler accepts it, and native
check/build still reject it with the expected native diagnostic. Their intact
program bodies now live in `tests/reject`, the existing category for valid Rust
outside the native subset. The [migration record](evidence/corpus-migration-20261118/verification.json)
retains each original hash, migrated hash and diagnostic. This makes the missing
native coverage explicit and keeps both acceptance and rejection checks active.
The removed box-pattern grammar is separately tested as invalid Rust; raw
identifiers remain in the positive fragment test.

The initial PR #30 native jobs stopped on two old generated-C unit expectations
after the callback signature repair. Their exact declaration expectations are
updated, and all 34 local C-generation tests pass. The initial Cranelift project
benchmark also stopped at native unit qualification before timing. These failed
runs are preserved; they are not native platform or performance passes.

## Direct host startup

The [host audit and CI guard](host-portability.md) found no existing systemd
dependency in active source, build rules, workflows, lockfiles or compiler loader
graphs. The daemon starts directly. The guard rejects service-manager calls,
packages and unit files, with injected negative controls proving rejection.
This removes no nonexistent package and requires no background service manager.

## Release boundary

Standalone frontend ownership, full Lust/complement composition, complete native
qualification and final package qualification remain required. The wider integer
bridge is still narrower than Lust's supported language; cold mixed compilation
still performs whole-crate Rust analysis. Publishing these correctness repairs
does not admit an installable replacement or change the
[performance acceptance criteria](practical-performance.md).
