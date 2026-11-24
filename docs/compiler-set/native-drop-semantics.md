# Native partial drops and pinned `ManuallyDrop`

This repair binds two native behaviors to the declared Rust source,
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`: the order and panic continuation of
partially moved values, and the representation used by the supported
`ManuallyDrop` overlay. The source-bound results are recorded in
[the verification record](evidence/native-drop-repair-20261119/verification.json).

## Reproduced failures

The preserved candidate built from `3e5cd0ba6` compiled valid programs and
returned success while producing different destructor order from the matching
Rust compiler. The existing `drop_partial_move_array` fixture dropped the
remaining tuple field after later array elements. In
`drop_partial_move_scope`, the branch that moved no elements nevertheless
dropped a tracked array in reverse order.

The native emitter explicitly reversed the sequence of untouched array ranges
and individually tracked elements. The pinned Rust implementation builds that
sequence in index order. Its control-flow ladder is constructed backwards and
entered at the last constructed block, which preserves the sequence's forward
execution order. See
[`open_drop_for_array` and `drop_halfladder`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_mir_transform/src/elaborate_drop.rs).

A further regression showed that correcting order was insufficient. With
array element zero moved away, a panic from the destructor of element two
skipped the destructor of element three. This occurred with both the native
explicit-unwind implementation and emitted C/C++ using unwind tables. A
second destructor-panic control incorrectly reached user code after
`catch_unwind`, whereas Rust aborted.

## Continuation through the remaining value

The emitter now traverses array parts in index order. A partial drop that can
panic receives one continuation boundary around its complete remaining
projection tree. Its existing move flags determine which fields still own
values. The boundary holds the first panic while later owned fields and
elements are dropped, then propagates that original panic. It uses the same
runtime continuation and double-panic helpers as ordinary whole-value glue.

Nested fields share the outer exception record. An inner field cannot
propagate its panic before the enclosing value's later fields have been
dropped. Array loops keep their continuation inside each iteration, and
cleanup after enum `case` and `default` labels receives the same checks.
Box deallocation remains after the drop of its remaining contents.

The change preserves the previously repaired erased callback ABI: emitted
drop functions accept `void *`, then recover their typed pointer locally.
The new calls use those actual function types.

## `ManuallyDrop`

The old native overlay kept an obsolete private `MaybeDangling<T>` wrapper.
Its derived Debug output included that wrapper, for example
`ManuallyDrop { value: MaybeDangling((0, None)) }`. The pinned
[`ManuallyDrop` definition](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/mem/manually_drop.rs)
stores `T` directly and derives Debug over it.

The native overlay now stores `T` directly too. Construction, extraction,
dereferencing and the observable layout use that representation. Existing
tests cover Debug, Default, comparison, hashing, const patterns, deref
coercions, layout and suppression of automatic drops. This does not add
support for `T: ?Sized` or the unsafe `drop` and `take` methods.

## Finite-domain verification

The final source candidate was built with the exact pinned SDK. The following
332 scenarios ran through native TCC and separately compiled GCC 14 C/C++ with
unwind tables. In both modes, stdout, stderr and exit status matched pinned
Rust. The destructor logs also matched independently calculated expectations.

| Scenarios | Domain |
| --- | --- |
| 48 | Every subset of four conditional moves, with ordinary elements, nested tuple fields, and reassignment |
| 16 | Every subset of four conditional moves followed by a panic before cleanup |
| 80 | Every subset of four moved elements and each of four destructor-panic positions, plus no panic |
| 144 | Every subset of four moved tuple fields and each of eight destructor-panic positions, plus no panic |
| 36 | Every subset of two tracked positions within an eight-element array, with each destructor-panic position plus no panic |
| 8 | Both enum variants and each destructor-panic position plus no panic, followed by an enclosing struct field |

For each scenario, the independent oracle lists every still-owned element in
index and field order. A single-panic case is caught exactly when its selected
panic position remains owned. Exhausting these finite input domains checks
those ownership and ordering claims; it is not a proof of every Rust program
or of the compiler as a whole.

The two original failing fixtures and the existing `ManuallyDrop` checks were
also rerun. The initial focused run covered 23 fixtures: 22 matched byte for
byte, and the remaining overflow-panic case differed only in its process ID.
No differential mask or executable expectation was weakened.

### Double-panic qualification boundary

At source commit `e5a336e95`, both native modes abort with `SIGABRT` after the
second destructor panic, and neither reaches the unexpected-recovery marker.
The printing-hook control still exposes a separate runtime defect: the
cleanup-abort helper bypasses the installed hook for Rust's final nounwind
panic. Its output therefore differs. Both complete reports deliberately keep
their overall result failing until the runtime follow-up is included and
qualified with this fixture.

The record does not claim a macOS run, a Clang sanitizer run, full native-suite
qualification or a performance result. Those remain separate checks of the
composed source candidate.

### Combined runtime repair

The subsequent combined source at `af0a281f2d1fae04e8ebfd5aa2c4872b20245c04`
includes the cleanup hook repair. Its
[combined verification](evidence/native-drop-repair-20261119/combined-verification.json)
passes all six fixtures in both native setjmp and GCC exception-table modes.
All 332 finite scenarios match both pinned Rust and their independent ordering
oracle. The double-panic fixture now matches exact stdout, stderr and `SIGABRT`
in both modes. This closes the specific failing hook comparison above; its
original failed reports remain unchanged. The full platform and sanitizer gates
still qualify their own exact source and toolchains.

## Reproduction

Use the matching `nightly-2026-11-09` SDK and the native README's TCC setup.
Build the native compiler into the unchanged harness's expected target
directory, then run it against the new fixtures with the matching Rust oracle:

```sh
cargo +nightly-2026-11-09 build --locked --release \
  --manifest-path compiler/native/Cargo.toml \
  --target-dir compiler/native/target --bin lust --bin lust-core
PANIC=unwind LUST_STRICT=1 rustup run nightly-2026-11-09 \
  bash compiler/native/scripts/difftest.sh drop_partial_move
```

The harness's existing EH lane additionally exercises the emitted C/C++ with
unwind tables. The source record includes binary and source digests. The
private qualification archive retains original failing executables, exact
commands, emitted C/C++, all runtime outputs and the independent oracle used
for the finite domains.
