# Checked integer and module boundaries

This increment expands the existing mixed scalar boundary. It keeps Lust's
prepared checked state, real Rust symbols and metadata, and fatal native errors.
It does not complete the [integrated compiler architecture](architecture-plan.md).
Cold mixed compilation still uses Rust analysis, and the standalone public entry
still contains whole-program retry.

## Fixed complementary-function crash

A valid function with an elided reference parameter could crash the mixed
compiler before native admission:

```rust
fn borrowed(x: &u32) -> u32 { *x }
fn main() { println!("{}", borrowed(&42)); }
```

The old path opened the function signature's lifetime binder, then put that
signature inside `Binder::dummy` while constructing its C ABI query. A reference
parameter can retain a bound region, so Rust correctly rejected the malformed
internal query with an escaping-bound-variable panic. The scalar type check
ran too late to keep this complementary function out of that query.

The represented-signature check now runs before C signature construction.
References, aggregates, generic types and other unrepresented signatures stay
with imported Rust. No native compilation is attempted for the reference example.
The preserved compiler exits 101 and emits no executable with explicit mixed
binary/link options. The repaired compiler and matching Rust both compile and
print `42`; both reject a boolean returned from that `u32` function with E0308
and no output artifact.

## Integer ABI coverage

The request remains restricted to the existing Linux x86_64 target and qualified
profile. The boundary queries the pinned Rust implementation's actual Rust and C
`FnAbi` and requires matching direct argument and return modes. It still rejects
unrepresented runtime operations before native emission.

| Signature component | Current direct boundary |
| --- | --- |
| `i32`, `u32`, `i64`, `u64`, `isize`, `usize` parameters | Eligible when the actual ABI and body checks pass. |
| Integer returns from 8 through 64 bits, including pointer widths | Eligible when the actual ABI and body checks pass. |
| `i8`, `u8`, `i16`, `u16` parameters | Remain complementary. Rust uses no argument-extension attribute, while C requires sign or zero extension for these parameters on the pinned target. |
| 128-bit integers, references, aggregates, floating point, booleans and unit | Remain outside this scalar object boundary. |

Lust already represents narrow integer bodies. That does not establish foreign
calling compatibility. Extending narrow parameters needs an adapter derived
from the actual upstream ABI, followed by runtime qualification. The equality
check is unchanged; this increment does not treat equal storage size as an ABI
proof.

## Module identity and retained work

The parsed-candidate inventory now includes plain functions in already-loaded
inline modules. It skips attributes that can transform a module or function,
macros, generic functions, associated methods and nested function declarations.
External source modules not yet loaded at the parsing callback remain outside
this inventory.

Rust normalizes identifier spellings, while Lust currently requires identifiers
already in NFC. A candidate must therefore have source identifier spellings
that match Rust's canonical parsed identifiers. This is checked per identifier,
with raw/lifetime token prefixes accounted for. Decomposed parameter and local
names stay with imported Rust before a native request. Canonical Unicode names
and Unicode comments remain eligible. Required raw-keyword quoting is preserved
in the internal function projection. This check does not normalize or alter the
user's source.

Candidate lookup uses the actual body span and hygiene context. Checked HIR
ownership must identify a plain function owned by the crate or a module. Cold
object identity still comes from Rust's real symbol and definition hash. The
retained path now resolves each stored definition hash to the current HIR owner
before looking up its candidate. A short name is never the lookup key. Therefore
`kernel`, `first::kernel` and `second::nested::kernel` cannot borrow each other's
body or native object merely because their short names match.

Certificate version 6 rejects earlier coverage certificates. Existing header,
context, dependency, diagnostics and literal-edit restrictions remain. An edit
still uses native preparation and emission only after the existing checks pass;
other changes continue through the imported analysis already in the same request.

## Verification and limits

The [verification record](evidence/mixed-integer-modules-20261119/verification.json)
binds the local results to the tested source files and compiler identities.

- The real Rust driver tests use parsing, HIR, type checking and actual ABI
  queries. The old candidate inventory fails the added width/module cases.
  A negative reference case independently exposed the malformed-binder panic.
- Direct retained native sessions emit signed/unsigned 32-bit operations and
  narrow-return conversions. Real matching Rust callers link and execute cold,
  unchanged and edited native objects. The two new tests make **1,179,738** value
  comparisons against wider-integer modular arithmetic, across 18 objects.
- Each object runs 65,541 inputs: every 16-bit pattern and five additional wide
  boundary values. The conversion cases exhaust every 8/16-bit output pattern.
  They do not exhaust the full 32-bit input domain or prove all Rust arithmetic.
- The standalone mixed-driver reference regression succeeds with zero native
  definitions, and invalid Rust remains rejected. This proves the complementary
  crash repair independently of daemon availability.
- `tools/mixed-scalar-qualification.rs` adds the hosted Cargo gate for seven
  native definitions, duplicate module names, retained edits and checks, the
  borrowed complement, invalid input and recovery. It compares mixed execution
  with the matching Rust executable. Source-built performance stages depend on
  this gate.

The local environment does not permit Unix daemon sockets. The direct-session
tests and real ABI tests are executable evidence, but they do not substitute
for this new full Cargo/daemon gate. Its hosted result must be checked against
the published source revision. No new latency or package-release claim follows
from this increment.
