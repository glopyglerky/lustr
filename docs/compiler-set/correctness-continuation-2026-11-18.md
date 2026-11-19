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

## Release boundary

The Unicode repair, broader mixed admission, standalone frontend ownership,
complete native qualification and final package qualification remain separate
work. Publishing a correctness repair does not admit an installable replacement
or change the [performance acceptance criteria](practical-performance.md).
