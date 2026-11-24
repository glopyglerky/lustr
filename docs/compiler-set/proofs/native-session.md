# Shared native driver session

This is historical component evidence. The diagnostic-driven session and
retained inference ABIs, along with their general executable qualifiers, are
retired by the sole-goal audit after `d1328bc2`. Their retries cannot be restored
as product behavior. The ordinary Lust retained daemon is preserved. Follow the
[current exclusive contract](../exclusive-coverage-contract.md); the source,
fixtures and receipts below keep their original scope.

This record describes the retained V59 component. The subsequent unadmitted
[standalone-output candidate](native-standalone-output.md) separates semantic
persistence from linking; its qualification does not alter the V59 evidence.
The later [borrowed-phase candidate](native-phases.md) consumes actual current
Checked results while preserving the fresh-collection proof and warm refusal.

Status: experimental source. Bounded runtime, standalone closure, calling-thread,
call-bound, cross-ABI serialization and runtime-retirement gates passed. Fresh
independent source review approves this bounded boundary. The installed compiler and
existing compatibility routes retain their separate contracts. Full Rust,
native crate artifacts, Cargo integration and equivalent complete-work speed
remain open.

`lustr_native_session_v1` uses the existing native `driver::Session`, including
its interner, resolver cache, original AST, definition identities, kept programs
and native read/invalidation machinery. It enters the requested native Build or
Check directly. An accepted supported request does not first run a separate
Check to select an engine. A rejected request can supply that same Session to
the existing Lustr elaboration. Acceptance still requires the complete native
driver to pass after elaboration. The current elaborator can inspect function
bodies again and perform up to two final driver rechecks; this interface does
not yet expose every checked body result as a reusable semantic query.

The entry serves at most 64 sequential calls on one calling thread. All native
pipeline ABIs share a process entry lock; the session entry also binds its
calling thread. The Session stays on the heap in thread-local owned state.
The compiler catches a panic at the ABI boundary and retires partly mutated
session state. Normal retirement settles native background phase work before
dropping state and removing its private generated directory.

The collection facade records weak identities only while this native phase is
active: its actual input AST, collected AST, resolution tables and crate
identity. Semantic elaboration requires those identities to match the current
original AST and retained program. Evidence is cleared before each driver phase
and at request exit. An incremental rejection with only an older put-back
program keeps its rejection. This conservative boundary can refuse a valid
extension after an edit until a qualified phase interface exposes the current
incremental semantic generation; cold acceptance is not claimed as warm coverage.

The full encoded request, process environment, client environment and shown
paths, current directory, actual Cargo graph and canonical root bindings select
the retained context. A context change constructs a fresh Session. Each call
checks the existing native runtime source/TinyCC snapshot contract. A changed
or unobservable runtime retires the instance. Source and macro edits still go
through the original native driver's resolver and semantic invalidation checks.
Inputs must remain stable during a compilation, as in the existing ABI.

This entry caches no executable answer. It disables disk objects/manifests and
forces all C-unit objects to compile again. The existing driver's persistence
flag couples semantic read-set retention to its base-library link. The entry
therefore produces that intermediate in owned scratch and links the complete
fresh object set as the requested standalone executable with the existing
TinyCC linker. There are two extra intermediate links today. A focused native
phase interface could remove them; their cost must be measured before any
speed admission. Release-C, emitted-C and test-harness requests remain outside
this experimental entry. This preserves host compiler/link input acquisition
while retaining native semantic state and generated C under the driver's own
dependency checks. It supplies no general host-input artifact reuse proof.
The separately qualified retained-artifact API and its observation/replay rules
remain separate.

An elaborated AST has no cross-request equivalence proof. The shared entry
retires transformed state after that request, including a successful request.
It may retain original native semantic state across supported source edits and
ordinary language rejections. It does not keep a failed native executable as a
successful answer or execute applications inside compiler selection.

`tools/native-session-qualification.rs` compares actual shared native builds,
cold native builds, pristine Lust where supported, and an explicit independent
stock oracle. Its cases cover native semantic reuse, module/body and included
input edits, signature and move rejection, recovery, generics, drops, unwinding
and the unchanged complete existing complement fixture. Compiler statistics
must show one driver result on supported builds and real native semantic reuse;
they are structural evidence, not application-speed measurements. The harness
and probes are bootstrapped independently. Every application runtime result is
attributed to the compiler that produced its executable.

The first complete gate observes zero native parse, collect, check and generic
discovery work on its no-op, alongside six freshly compiled native C units.
Module/body and included-input changes, signature and move rejections, recovery,
generics, drops, unwinding and the unchanged complete complement fixture match
the independent controls. This is one bounded Linux x86-64 debug component
result. It is not speed, general Cargo, native crate-artifact or full-language
qualification. Separate harness modes check the actual final ELF dependencies
after compiler exit, the calling-thread boundary, the 64-call bound and runtime
generation retirement without replaying the applications.

This implementation uses the existing Lustr-owned composition of pristine Lust
modules. The declared Lust integration diff remains empty. No new frontend,
type system, backend or daemon is introduced. The actual dependencies remain
Rust bootstrap, Lust's native pipeline, TinyCC/C, mimalloc, the Linux/GLIBC
runtime and host linking; this is not a whole-toolchain Rust memory-safety claim.
