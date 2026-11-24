# Shared integer signatures and explicit ABI adapters

> Bounded research and source-bound evidence. The broad compiler-private SDK
> dependency used by this experiment is not an admitted selective complement.
> Native ownership and useful ABI/emission mechanisms remain subject to the
> [exclusive coverage contract](exclusive-coverage-contract.md), the
> [import inventory](../../upstream/complement-imports.json) and the current
> [native Cargo continuation](native-cargo-continuation.md). Successful fixtures
> do not establish full coverage, minimal imports or product feature reachability.

This extends the [shared definition service](shared-upstream-definitions.md)
through actual imported integer signatures. Lust still owns the unchanged
consumer's checked bodies. Imported Rust owns external definitions, nominal
identity, type and layout facts, ownership, instance symbols and function ABI.
The expansion adds no consumer Rust body analysis or whole-program retry.

## Represented facts

The provider carries signed and unsigned 8, 16, 32 and 64-bit integers and
pointer-width integers. A public nongeneric non-Copy transparent tuple struct
may wrap any of these integers, subject to the existing visibility, auto-trait,
drop, niche and layout checks. The actual field type is installed before native
checking; signedness is not inferred from size.

Safe nongeneric imported functions may mix these integers and nominal types in
their parameters and result. A function may have zero parameters. A dependency
may supply scalar functions without a nominal declaration. Every value must
have the declared upstream integer backend representation and the same native
layout. The whole signature also needs a represented native C calling mode.
References, unit values, 128-bit integers, general aggregates, generic
obligations, methods and drop/unwind remain outside this import boundary.

The target remains x86_64 GNU/Linux with an explicitly requested abort strategy.
The dependency's genuine metadata must itself require abort. The provider never
changes a caller's requested panic strategy.

## The two sides have distinct ABI records

The original equal-ABI guard rejected a narrow-integer campaign before native
checking. In the declared Rust implementation, the examined 8 and 16-bit Rust
arguments carry no extension attribute; C scalar arguments require zero or sign
extension. Deleting that difference from the comparison would lose an actual
calling-convention fact.

Native nominal values also have an important representation difference. The
native emitter uses actual one-field C structs, while Rust's `repr(transparent)`
ABI can classify the corresponding Rust nominal as a scalar. The provider
therefore creates a separate layout view for the native C struct. It preserves
the authoritative field offsets, size and alignment and marks the C view as an
aggregate. The pinned upstream `FnAbi::adjust_for_foreign_abi` classifies the
complete C signature, including register exhaustion. The original Rust layout,
type and `FnAbi` remain unchanged.

When these contracts differ, the provider emits an explicit LLVM adapter. Its
entry uses the actual native C mode and attributes; its inner call uses the
original Rust instance's mode and attributes. Integer widths come from checked
backend representations. A represented C aggregate cast must equal the full
typed single-integer `CastTarget`, including its attributes and absence of
padding registers. Additional modes or attributes remain an explicit integration
gap. No diagnostic text or Debug rendering chooses a mode or supplies a fact.

Admission also verifies the exact owned archive's symbol closure. A function
retained only as a metadata body, such as an `inline(always)` instance, needs
complementary body materialization before admission. Undefined object symbols
that are not supplied by the same owned archive need shared dependency or
runtime records. The service does not discover either requirement through a
failed native link, and it does not admit a runtime merely by matching names.

The declared upstream LLVM backend maps these Rust and C conventions to
`CCallConv`. The adapter preserves their different attributes and argument
representations. It neither generates replacement Rust source nor starts
another Rust frontend. Equal represented signatures keep direct linkage.

## Emission and ownership

The provider admits every required type, signature and adapter before native
checking. After the unchanged consumer passes native checking and the installed
records pass validation, preparation transfers an owned, single-use code plan
into the checked continuation. Semantic validation emits no code. A check or a
dropped prepared request destroys the unused plan without invoking `llc` or
materializing object files. Only consuming an admitted build runs the exact
SDK's `llc`. The ordinary native path and imports with equal represented ABIs
do not invoke it.

`upstream/bluefe-core.pin` declares the required `llc` executable hash and actual
LLVM DSO hash. Emission uses the SDK path, never ambient `llc`. The actual DSO is
`libLLVM.so.23.1-rust-1.102.0-nightly`; the similarly named `.so` linker script is
not a substitute. Tool identities are checked against the declaration before
and after emission. The manifest and generated LLVM input enter the owned
dependency records alongside the original archive.

The adapter uses stdin and stdout and creates no shared temporary pathname.
The bounded ELF validator checks emitted architecture, sections and symbols.
Its global function set must equal the planned adapter set; its undefined
symbols must equal the admitted original Rust instances. The checked
continuation owns the plan's immutable LLVM input, tool identities and original
dependency objects; the consumed plan returns validated adapter bytes for that
same build. It contains no borrowed Rust compiler context, cannot be cloned or
taken twice, and never reopens an archive to select replacement code.

A missing tool, failed invocation, malformed object or missing object is a
compiler failure. None authorizes a second implementation to compile the
consumer. Failed requests preserve the previous successful executable. The
internal qualifier's failure controls leave shared SDK files untouched.

The later [owned-emission repair](shared-import-emission.md) proves and fixes
adapter execution during preparation/checking. Its focused 29-control record
has its own exact source identity; the original semantic campaign below keeps
its predecessor scope.

## Qualification scope

The clean source-bound campaign at
`8bdb0d54db41330611a6f1b5bc2f2700183befbf`, complete tree
`6e86842a838b12ff023326b566de5c140a025f3e`, rebuilt the real qualifier and passed
all **120 compiler/runtime controls**. Its executable SHA-256 is
`3e91ac3442a62c6faec7b6332f473373385cb9f999d65e4f941c63505c6a65c0`.
The [verification record](evidence/shared-integer-abi-20261119/verification.json)
and [case inventory](evidence/shared-integer-abi-20261119/campaign.json) retain
commands, tool identities and original-output hashes. The
[actual LLVM adapters](evidence/shared-integer-abi-20261119/integer-adapters.ll)
show the 30 represented ABI bridges in the integer dependency.

An independent review reproduced the original missing-inline-body
[link failure](evidence/shared-integer-abi-20261119/original-inline-failure.json)
and reran the unchanged witness after repair. The
[repaired result](evidence/shared-integer-abi-20261119/independent-inline-repair.json)
refuses before native checking and preserves the prior executable byte for
byte, while matching Rust still executes successfully. That separate replay
records its own development-build executable identity rather than inheriting
the clean-build identity above.

`tests/shared-imports/integer-peer.rs` and `integer-consumer.rs` exercise every
declared integer width, mixed scalar/nominal parameters, primitive returns,
zero-argument functions and eight-argument calls. The last scalar arguments
cross the register/stack boundary. Calls execute directly and through native
function pointers.

For signed and unsigned 8-bit inputs, the consumer covers all 65,536 operand
pairs for each signedness. For the other eight integer types, it checks a
6 by 6 boundary grid. Every combination uses both call forms, for 262,720
checked results. The oracle evaluates the peer's wrapping arithmetic as a
separate `i128` formula followed by reduction to the result type. The largest
intermediate is far inside the `i128` range. The same unchanged consumer also
compiles and executes with the declared Rust compiler.

The campaign includes real type, arity, moved-value and function-pointer
rejections. It also tests a seventh nominal argument: the upstream C classifier
places that aggregate on the stack by value, which this adapter does not yet
encode. That valid Rust case returns `Unintegrated` before emission and preserves
the previous executable.

Two valid dependency cases explicitly require more integration before admission:
an `inline(always)` function whose body must be materialized from metadata, and
a checked-add function whose object references an unavailable panic runtime.
Both matching Rust controls execute successfully on the supplied valid inputs.
The import provider reports these different missing records before native
checking, with no compiler retry and no changed successful artifact.

Actual LLVM failure controls exercise a missing executable, a rejected
invocation, assembly in place of ELF and an empty object stream. A native
rejection is observed before adapter emission. A fresh successful request
recovers after failures, including removal of the original archive snapshot
after preparation. The Rust consumer `typeck_root` and complete `analysis`
queries remain panic traps throughout these executions.

The matching SDK's `llvm-tools` component is part of this expanded internal
service's required tool closure. Compiler-set CI already installs it. A future
public package must carry and qualify the same declared closure; this internal
qualification does not establish installation or relocation.

Imported requests still use a fresh session. General semantic continuation,
consumer Rust metadata, retained imported generations and public routing remain
open. These tests establish the stated finite boundary and failure behavior,
not correctness for every Rust program or a new latency result.
