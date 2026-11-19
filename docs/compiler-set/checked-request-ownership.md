# Owned native checked requests

The native driver now separates preparation from emission through a real owning
continuation. Ordinary `driver::compile` uses this boundary; it is not a separate
compiler, a second check request, or a source classifier.

`driver::prepare(&request, &mut session)` runs the existing native frontend,
including retained-state validation, checking, lowering and borrow checking. A
successful uncached preparation returns `Preparation::Ready(PreparedRequest)`.
That value owns the actual `Program`, lowered bodies, local type arenas,
dependency reads, source snapshot and emission bookkeeping. It borrows the
request and session until it is consumed. No later request can mutate the same
session while the continuation exists.

`PreparedRequest::emit(self)` consumes that state once and enters the original
native generator and linker. A check request completes without generation. A
validated cached check can return `Preparation::Complete` directly. Dropping a
prepared request emits nothing and keeps its passing semantic state in the
session; it does not replace the last output artifact.

## Ownership of types and bodies

A lowered native body previously borrowed `Program::tys` and its function's
predicate environment. Returning it alongside its owning program would create
a self-reference. The continuation does not use a lifetime cast or a cloned
semantic image to work around that relationship.

Preparation moves each body's existing type arenas and IR allocations into a
private owning representation. It verifies that the removed global-type borrow
belongs to this exact program, and records the function identity of a nonempty
predicate environment only after checking that borrow's identity. The detached
type store cannot answer type queries. Emission reattaches those allocations to
the same privately owned program and predicate slice. No type ids are translated,
no read sets are reconstructed, and no body is parsed, checked or lowered again.

The ordinary retained worklist, slot identities, invalidation, no-op checks,
body-edit behavior, code generation, object validation and linker error behavior
remain in their existing implementations.

## Outcomes are not a language selector

`Preparation::Rejected` preserves a native frontend rejection. It does not claim
that the source is invalid Rust: existing native diagnostics still include both
invalid input and unavailable frontend support. `Preparation::Failed` preserves
I/O, routing and implementation failures. Neither authorizes a second compiler
attempt. The separate `Complementary` variant is produced only from the existing
explicit mixed capability outcome, never by matching diagnostic text or treating
status 1 as unsupported language.

This ownership boundary does **not** remove the transitional standalone retry in
`src/main.rs`. It supplies the missing continuation for successful native work;
it does not yet turn partially checked native work into deferred imported Rust
semantic requests. No full architecture or package completion follows from it.

## First connected imported service

The [shared upstream definitions service](shared-upstream-definitions.md) now
connects genuine imported Rust metadata, layout, obligation and `FnAbi` queries
to this continuation. It installs actual external nominal and function records
before native checking, then emits the same checked consumer state and links
owned code from that dependency generation. The first qualification slice is
limited to transparent non-Copy `u64` aggregates on x86_64 Linux with an explicit
abort profile. It does not change normal public routing.

The connection carries actual Rust definition/instance identity and ABI facts
into the emission plan. Wider aggregate layouts, narrow-argument extension,
drop glue and unwinding cannot be inferred from native storage size. Further
integration must make unsupported native semantic queries explicit and
suspend them before emission so imported services can resolve them inside the
same compilation. Redirecting a rejected whole-source request to another
compiler would leave that work unfinished.

## Verification

The [source-bound verification record](evidence/checked-request-ownership-20261119/verification.json)
records the compiler identity, tests and nonpassing controls.

The focused native tests exercise the owning API with real generated executables
and retained sessions. They cover structs, references, a generic instance and a
closure; captured-source emission after the source file is replaced by invalid
Rust; unchanged and edited bodies; invalid input preserving an existing artifact;
abandoned preparation; link failure followed by native recovery; and checks that
emit no code. The tests compare the original lowered IR allocation addresses
with the owned representation and verify that the predicate environment remains
the exact original slice.

The existing mixed admission/object tests and native driver tests remain separate
regression gates. This refactor makes no latency claim. Performance qualification
must use the complete source revision and matching native build identity.
