# Shared upstream definitions in a native checked request

The [integer-signature continuation](shared-integer-abi.md) extends this original
u64-only slice with actual field types, mixed signatures and explicit LLVM ABI
adapters. The original qualification results below retain their source scope.

The [consumer-metadata dependency record](shared-consumer-metadata.md) identifies
the next required joined completion service before the original Rust encoder
can publish metadata for native-owned consumer bodies.

The first connected semantic service imports actual Rust definitions before
Lust checks an unchanged consumer. A native body constructs a nominal aggregate,
moves it into a Rust function, receives the same nominal type, and continues
native work. The consumer is checked and lowered once by Lust. The ordinary
owning `PreparedRequest` carries that exact program and those bodies into native
emission.

This is a bounded first slice of completion-checklist step 2. It is available to
the internal qualification executable, not normal public routing. The public
standalone retry and the broader supported-semantics integration remain
unfinished. Worker and process ownership do not need to change for later
services; this experiment does not prescribe an in-process rewrite.

## Facts arrive before native checking

The pinned imported Rust compiler opens the original consumer and its genuine
dependency archive. Its shared global context performs parsing, expansion and
resolution. The callback stops before consumer body analysis. It queries the
external dependency's actual definitions, instantiated types, visibility, Copy
and drop facts, auto-trait obligations, layout, function and region signatures, instance
symbols and `FnAbi` records.

The provider mounts these definitions in Lust's existing resolver and installs
their semantic records during collection, before initializers or bodies are
checked. It does not generate source declarations or a separate kernel file.
Each imported nominal and function preserves the full upstream `DefPathHash`:
the stable crate identity and local definition hash. The session-local crate
handle is checked for prior ownership. A crate table in `Program` supplies the
actual stable identity to all imported definition fingerprints, including
nominal layout and structural reads.

Lust then resolves, checks, borrow checks and lowers its original consumer
bodies against those records. Its existing ownership rules reject use after
moving a non-Copy imported value. Inherent implementation ownership now also
uses the definition's real crate, so an absent external source span cannot make
a foreign type local. The existing orphan check uses that same crate identity.

After successful checking, the provider verifies the installed layout, ownership
and call representation again. The immutable native program, lowered bodies,
dependency reads and imported input snapshots enter the ordinary checked
continuation. Emission consumes that state without another native check. No
Rust arena reference enters the native retained session.

The [owned-emission continuation](shared-import-emission.md) separates adapter
validation from execution: checks and abandoned preparations drop a single-use
plan, while an admitted build consumes it.

## Admission is a semantic and ABI proof

The first profile admits public, nongeneric, one-field tuple structs declared
`repr(transparent)` over `u64`, with public fields, no niche, no custom alignment
or packing, no non-exhaustive restriction, no Copy implementation and no drop
requirement. The actual upstream trait solver must establish Send, Sync and
Unpin. Negative or unresolved answers produce an explicit unintegrated result.
Additional external inherent methods or trait implementations also require
records this slice does not yet transport and are refused before native checking.

Imported functions must be safe, non-const, nongeneric Rust functions taking
and returning one admitted nominal type. Admission obtains both the actual
instance's Rust `FnAbi` and the equivalent C signature's `FnAbi`. Both argument
and result must use direct passing with equal typed `ArgAttributes`; this
compares every field through the pinned compiler's `PartialEq` implementation,
not a Debug rendering. Fixed argument counts and unwind behavior must agree.
The actual signature binder must be empty. Its admitted nominal inputs and
output contain no regions, and that explicit region signature is installed for
the existing borrow checker's function-item-to-pointer and call proofs.
For the qualified target, pinned LLVM lowering maps both canonical Rust and C
conventions to `CCallConv`. Lust's actual C representation must match the
upstream layout and must not introduce a pointer parameter, hidden result
pointer or dropped parameter.

The profile is limited to x86_64 GNU/Linux with an explicitly requested abort
strategy. The dependency's genuine metadata must itself require abort. An
unwind request is refused; its flags are never rewritten. A matching byte size
alone cannot admit a type or call.

## Code belongs to the same dependency generation

An internal `Archive` constructor owns one complete rlib snapshot and extracts
object members from that snapshot. Its fields are private: the provider cannot
be constructed from unrelated metadata and object vectors. Capture verifies
the snapshot bytes and checks that Rust's `used_crate_source` names this exact
archive. Native checking retains the archive bytes, imported compiler/crate
identity and object bytes through emission.

Emission materializes those owned objects beside the native objects and uses
the ordinary native linker. It never reopens the original dependency to choose
replacement code. A control removes the snapshot pathname after preparation;
the already prepared request still executes correctly from its owned inputs.
Object materialization and link failures return a compiler failure and preserve
the last successful executable. A later fresh request can recover.

Imported sessions are deliberately single-use and fresh. Their input generation
is not yet part of the existing retained or disk-cache contract. A second
imported request is unintegrated; an ordinary request against that imported
session fails. The ordinary native worker and its retained path continue using
their existing preparation and allocation modes.

## Embedding allocation and failure outcomes

The pinned Rust compiler DSO and native image allocator cannot share arbitrary
Rust-owned allocations. An isolated counterfactual reproduced the failure:
the same driver-linked program and archive input segfaulted with a private
mimalloc global allocator and succeeded with System. The explicit
`upstream-semantic` embedding feature therefore uses the compiler DSO's System
allocation contract. It does not change the ordinary native build's allocator.
Every exported native entry refuses front-image settings under this feature,
including settings supplied through the sequential client environment.

Native body and coherence rejections retain status 1. Unintegrated semantic or
profile capabilities return status 2. Provider, object and linker failures
return status 3. All preserve a previous successful executable. The internal
qualifier also fails closed with status 3 when the imported compiler stops
before its after-expansion callback: the upstream driver collapses early
diagnostic errors into one outcome. The original diagnostic is retained. A
typed distinction between early invalid consumer input and infrastructure or
metadata failure remains necessary before public routing; diagnostic text is
not a capability classifier.

## Qualification

The [source-bound record](evidence/shared-upstream-definitions-20261119/verification.json)
contains exact compiler and executable identities, source digests, commands,
passing controls, initial failures and limits. The fixtures and the campaign
are in `tests/shared-imports` and `tools/test-native-upstream.py`.

The campaign executes 65,541 direct nominal value calls and 65,541 calls through
function pointers, comparing both their arithmetic oracles and matching Rust
executables. It exercises invalid move/type inputs,
external inherent and orphan implementations, valid local-trait implementation,
positive and negative auto traits, unsupported layouts and profiles, dependency
body edits and reverts, immutable emission inputs, image-mode refusal and real
object/link failure followed by recovery. The imported compiler's consumer
`typeck_root` and complete `analysis` query providers are panic traps. Passing
calls therefore require zero queries to either provider; no fabricated type
checking result stands in for a Rust body.

The declaration profile, external source metadata, call ABI and runtime outcome
are real. Wider aggregates, generic and const obligations, imported trait and
method records, drop glue, unwinding, macro/hygiene composition, mixed consumer
Rust metadata output and retained imported generations remain unfinished. The
ordinary native path is not routed through this experiment. No latency, complete
architecture or installable-package claim follows from this first slice.

## Combined source and required CI

The [combined-source record](evidence/shared-upstream-definitions-20261119/combined-ci-verification.json)
rebuilds this service after the native NFC and Cargo reader repairs. All 76 real
compiler/runtime controls pass again. The qualifier executable has SHA-256
`7b52b4be6d18e990a852bfb5e889e6b5ef00b2a5b71043fe5bae8133773f7ed9`;
the record binds its source trees, compiler/TinyCC identities, build commands
and original output hashes. An actual Rust 1.99 control is refused before the
service is built or loaded.

`tools/qualify-shared-upstream.py` builds the real qualification executable with
the declared SDK and discovers it from Cargo's JSON artifact report. It keeps
build failures, runtime failures and the campaign summary. Compiler-set CI now
requires this independent campaign before any performance campaign. Workflow
controls verify that a failed, skipped or cancelled shared-service gate blocks
timing admission while allowing other correctness suites to run. These controls
are distinct from the 76 real compiler/runtime cases.
