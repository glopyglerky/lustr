# Experimental native semantic artifacts

The owned `native_crate` component produces and reads a versioned native
semantic artifact from the existing checked native object pipeline. It stores
actual function signatures, visibility, stable crate/definition pairs and the
emitted object with explicit runtime dependencies. Format ten retains the producer's actual module
tree, definition scopes, namespace bindings and re-export targets and adds a
structural type graph. It does not emit rustc metadata or put an object in
Cargo's graph as a Rust library. The bare-object entry and production routes
remain unchanged.

The producer accepts the existing primitive unit request with an explicit
`--emit=lustr-native-crate` internal qualifier entry. Actual native collection,
body checking, lowering, borrowing and C generation precede publication. No
synthetic source declarations supply signatures. This schema refuses
unrepresented nominal forms, generic bodies, qualifiers and region forms.
Closed constants retain stable definition identities, actual type graph roots,
evaluated scalar/tuple/array/ADT values and original promotion qualifiers.
All original initializer type and borrow checks precede production. The decoder
checks typed values, ranges, variant/field shape and cumulative depth/work bounds.
All images' values, native ABI and region records validate before imported
definitions, constant values, type names or signatures become visible. Consumer
tables assign constant indexes afresh; no AST/string/static/function index crosses
this value schema. Both public and private constants retain real namespace
visibility and re-export targets. Reference/allocation/function-valued constants,
generic const bodies and general const use in type/generic positions need further
records and native interfaces. This is no general cross-crate const-body interpreter.

The artifact entry also applies the original native static const-function walk
to its actual transient initializer checks, including nested inline const blocks.
Two small proposed interfaces expose those checks and explicit inline traversal;
the default native routes retain their prior policy. A retained short-circuit
local-call invalid acceptance motivated this join. Closure and nested-function
bodies keep the original walk's scope boundaries. Unrepresented private constants
may keep their earlier inaccessible resolver binding without an imported value;
public constant definitions require a represented typed record.

Artifact production enters the existing native resolver and resolved collector.
A proposed root-parser interface receives the original expanded root stream in
a fresh one-shot resolver. The owned entry consumes the original admitted
`no_std` attribute into its AST, retaining its tokens' spans, then delegates the
remaining stream to the existing parser. Ordinary and retained resolver entries
use their original parser. The bare-object entry retains its earlier path
restrictions. This is a crate-policy boundary, not general core/alloc support.
The initial policy check and resolver currently lex the root separately; no
single-lex or negligible-cost claim follows.

The artifact records the exact original source digest, native runtime family,
compiler and C toolchain artifact digests, format/pipeline generation and fixed
host target, edition and profile policies. The caller must establish the actual
compiler/backend identities; this internal API is not production admission.
The artifact backend is selected by an absolute executable path, canonicalized
once and hashed. That same path is executed, with identity checks before and
after successful code generation. The toolchain digest also binds the explicit
linker and object localizer. Their original invocation names are preserved
because a symlink's name can select a driver mode; hashes follow the same paths
before and after use. Selected files must remain stable during the
request; these checks do not establish a filesystem lease.
The closed primitive producer has no dependency crates, features, custom cfg or
build-script reads. These zero sets cannot describe arbitrary Cargo units.
Unknown format/generation/type tags, inconsistent identities, duplicate defs,
missing function symbols and incomplete ELF object closure refuse loading.
An SHA256 checksum covers the complete semantic and object payload. The
experimental digest implementation invokes the upstream `sha256sum` host tool.
This is corruption detection, not authentication of an untrusted producer.
Additional attributes, macros, external source files, expansion inputs and
extern declarations still need their own input/semantic records and refuse.
Unresolved external paths cannot substitute the host std compatibility layer
for genuine core/alloc identities.

Each decoded artifact owns its data. The store assigns consumer-local CrateId,
Sym and ModuleId values afresh. It retains the producer's stable crate and
definition identity independently of the consumer bin and its interner order.
Same-named crates can have distinct disambiguators and be imported under
different aliases. Conflicting builds under one stable identity refuse; exact
aliases share one definition. Native ABI/binary interoperability remains open.
Imported module keys use the resolver's stable crate magic, so two same-named
crates remain distinct while aliases share a root. Definition identities and
object symbols remain independent of their exported names. The decoder checks
module/definition identities and namespace targets, then reuses native resolver
restriction, access and visibility-subset rules. A recomputed checksum cannot
make a malformed visibility restriction or wider re-export internally valid.
Public re-exports through hidden modules retain their actual targets.

A small proposed Lust interface installs the store in fresh resolver tables
after ordinary extern preludes and before any source job registers, imports or
rewrites items. The existing import/visibility/namespace machinery resolves
those actual bindings. This entry always uses a new one-shot resolve cache;
there is no unkeyed imported state authorizing incremental reuse. The ordinary
resolver entry and its cache behavior remain separate. The integration remains
unadmitted under the pristine-reference policy.

The store materializes signatures into the consumer's existing GlobalTys and
Defs as foreign definitions. It creates no function AST or local FnInfo row.
The proposed native interface now carries a complete foreign DefId through
argument checking and lowering to a distinct native call target. Its installed
linkage carries the store's stable DiskDefId and actual object symbol; no local
FnInfo is fabricated. The existing C generator emits the checked native call.
Fresh collection installs these real definitions before checking any initializer
or body. Static checks reject non-const foreign calls in dead branches,
short-circuit expressions, named/inline/anonymous constants and constant bodies.
The executable component reuses the existing original-AST assertion text
derivation before complete native checks; source and executable conditions stay
unchanged.
Foreign-call and function-value IR refuse retained-cache reconstruction until
their complete import input contract is qualified. A foreign function item
retains its full independent definition identity and zero-sized item type.
Reification uses the actual producer symbol and installed signature, with no
local function slot. The native checker consumes reconstructed safe function
pointers, including bounded borrowed callback parameters and returns. It
rejects unrepresented pointer qualifiers, binders and regions.

Callee expressions execute before arguments, including zero-sized item
expressions and references. Borrow checking sees their place uses. Indirect
calls copy the evaluated pointer before arguments can change its original
place. Shared borrows of imported function items use the existing function-item
promotion rule. Direct and value calls retain static constant-context checks;
foreign item values inside inline constants still require const-value identity
records and explicitly refuse in the bounded evaluator. Generic bodies,
general borrowed signatures and Rust ABI interoperability remain open.

The type graph uses bounded topological wire indices, independently of native
Ty, SigId and ConstId allocation. Its grammar currently covers scalars, floats,
tuples, concrete arrays, sized references and safe function pointers, including
bounded borrowed callbacks. Each node carries
its native layout, Copy/drop facts and emitted C size/alignment/passing rules.
Decoding reconstructs native types and checks layout and structural Copy/drop
facts. Before inserting any imported signatures, the consumer rechecks every
node's actual Copy/drop and C ABI under the same naming mode as emission.
Node, edge, expanded-work, depth, array-length and layout-size bounds constrain
even a small graph that repeatedly references large subgraphs. Unsupported
region forms and nominal/drop/trait/generic semantics are refused.

Format fourteen adds sized shared/mutable references to slices and `str`.
The wire carries the original region, mutability and prior slice element node;
the consumer interns the original unsized referent and its sized reference.
Only that reference has layout/Copy/drop/C ABI facts. Nested element regions
enter the existing region walker and pointer slot checks. No referent layout,
UTF-8 data, producer dense type IDs or source syntax is manufactured. General
unsized aggregates, core/prelude identity and no_std runtime remain open.

Sized shared and mutable reference nodes retain their region and mutability.
Each function also records its actual native RegionSig: regioned parameter and
return type roots, universals, outlives facts, diagnostic names and early-binding
markers. Region roots must erase to the independently recorded check types;
unrepresented trait/opaque bounds refuse. Native layout,
Copy/drop and effective C ABI checks still cover every node. Mutable references
are not Copy. All region and type records from every image validate before any
foreign definition is inserted or any initializer or body is checked.

The consumer installs region signatures under complete foreign definition IDs
in the native region tables. Calls enter the original universal-instantiation,
variance and loan machinery. Consumer-local safe borrowed pointer types retain
their original native binder information and use the same item-to-pointer proof
as local items. Exported borrowed callbacks retain their original output-slot
mask and width; unsupported free regions and binders refuse. Unbounded named
late-bound function lifetimes reuse the recorded native RegionSig. Every written
declaration must match the original name, declaration count and binding
classification. Format thirteen additionally retains lifetime-only function
bounds and early-bound parameters. Independent declaration names use the original
lexer's Unicode XID/NFC rules; canonical declared outlives facts and early flags
must agree with the original signature. Named and elided universals remain
distinct, and facts cannot claim anonymous input lifetimes. All records validate
before semantic publication. Native calls and pointer coercions enter the original
constraint and generality checks. Type/const generics, unsized types and general
higher-ranked exported signatures remain open.

Closed nongeneric structs and enums now carry original stable definition identities,
ordered variants, their shapes and resolved discriminants, field names/types
and exact field visibility, with native default layout,
non-Copy and no-drop facts. Graph edges remain bounded and topological; producer
ADT/type indices never cross the image. `Defs::register_adt_shape` is a proposed
small bridge interface, with no `DefData` insertion. Lustr stages all image shapes,
checks every node's actual native layout/Copy/drop/effective C ABI and every region
signature, then publishes imported type names, semantic payloads and signatures.
No AST, function row, local alias or producer source is fabricated.

The separately labeled original-`Packet` gate executes three native/stock-oracle
pairs and rejects private-field access/construction, moved values and distinct
identical-layout nominal identities. Checksum-valid identity, field, layout,
Copy/drop, ABI, function-visibility and missing nominal-record mutations are
rejected. The full nominal gate now produces the unchanged array-backed image
and executes large/nested and borrowed consumers with stock-oracle parity.
Its earlier runtime-closure failures remain retained. The ADT qualifier produces
tuple/unit structs and enums, including an empty enum. Six independently
identified producer images and six native/stock-oracle runtime pairs exercise
constructors, variant imports/globs/re-exports, nested and large payloads,
tagged and niche layouts, independent identities and cross-crate matching.
Private tuple fields/constructors, moves, argument types, distinct nominal
identities and non-exhaustive matches retain actual native and oracle rejection.
Checksum-valid malformed variant identities, discriminants, shapes, field
edges/privacy, offsets, tag/niche data and namespace targets refuse decoding.
A malformed later enum ABI refuses collection before any earlier image's
semantic definitions, type names, region signatures or initializers are published.
Fresh interner and existing local ADT/type/region rows preserve imported enum
records without inventing function rows; that remapping check is installation
only. The checksum does not authenticate a producer or prove its code matches
its semantic records. Recursive graphs, type/const generic, trait/impl
and drop semantics remain unqualified. SDK compiler bootstrap is not native
compiler bootstrap; host-linked consumers do not qualify no_std runtime support.

Lifetime-only nominal definitions now retain their original declaration names
and kept field types separately from erased checker fields. Applied types carry
their actual lifetime arguments and reference the stable definition. Every kept
field and function-signature context requires complete nominal applications,
including nested tuples and arrays; omitted kept records cannot erase a nested
static application. Applied enums retain full tag, niche and variant-offset
layout data. Fresh native lifetime scopes and field-region tables feed the
existing variance, outlives and borrow checks. Canonical declared lifetime
outlives facts enter the original scope and region tables. The original native
outlives checker validates staged definitions before semantic publication.
Format twelve represents reference cycles with stable nominal identities and
complete lifetime applications; value edges retain the bounded topological
graph. Self-recursive and mutually recursive consumers execute, including
mutable recursion. Borrow escapes, moves, distinct identities, mutable aliasing
and invariance reject. Checksum-valid unknown/foreign targets, omitted or invalid
applications and invalid mutability refuse. A later malformed reference ABI
refuses before imported definitions, names, constants, signatures or initializers.
Fresh-table remapping preserves the exact recursive target and queued consumer
obligations. These are bounded checks, not a soundness proof. Type/const generic
parameters and borrowed nominal function-pointer images still require further
records. The integration remains unadmitted.

Tuple/unit constructor value bindings, tuple field privacy and enum variant
bindings are reconstructed in the existing resolver tables. An enum variant
target retains its actual parent identity and variant name, including genuine
re-exports. Enum field visibility must equal the defining enum's visibility.
Producer two-segment paths are admitted only for an actual resolver-known enum
variant; this grants no external core/alloc or associated-item records. The
native layout engine recomputes all variant offsets and explicit or niche tag
data before semantic publication. The integration patch is unchanged by this
owned metadata extension and remains unadmitted.

The C generator reuses existing parameter-copy/drop and hidden-return-pointer
rules for independent symbols, including large and zero-sized arrays. If the C
backend introduces memory calls, the producer compiles Lust's exact upstream
memory unit, checks actual symbol closure/overlap, links it relocatably and
localizes its definitions. Independent images retain distinct memory copies
without exporting or interposing host symbols. The strict memory closure preserves
the original export sets and requires no undefined symbols. The separate exact
musl v1.2.5 fill provider supplies `memset`, exports only that function, has no
imports and is also localized. Its complete license and provenance are retained
under `upstream/runtime/musl-1.2.5`. These are upstream C
compatibility dependencies, not newly Rust-implemented memory routines or general
freestanding runtime qualification.

Runtime-bearing library images record the exact original runtime/fill source
digest, remaining ELF import edges and each function's stable definition,
source path and first line. They are explicit dependencies of a library,
not self-contained freestanding objects. Before imported semantic definitions,
the consuming operation compiles the original runtime and original location
providers and checks actual definitions, collisions and dependency uses.
Undefined `STT_NOTYPE` is not a kind grant: the bounded x86_64 interface requires
qualified call/jump or qword-load relocation/instruction evidence and matching
provider kinds. TLS stays distinct and unrepresented TLS imports refuse.
Unnamed, contradictory, function/data-confused and unknown host imports refuse.
Checksums detect corruption; they do not authenticate executable code.

The consumer uses that runtime object only within the same request and rebuilds
the original canonical location unit across independent producer and consumer
definitions. No previous request's object or transformed semantic state supplies
an answer. The runtime gate exercises normal calls and function pointers,
producer and consumer abort locations, checksum-valid malformed dependencies
before initializer checking, missing runtime admission and memory collisions.
It also checks 9,600 actual fill calls for return values, byte conversion and
range canaries. The final GNU host link still uses libc, libm and the host loader;
core/alloc, custom panic handlers and complete no_std runtime acceptance remain
required. Generated inputs and first failures are retained for the operation
owner's checkpoint and cleanup.

Producer and consumer reject unrepresented release/LLVM environment overrides
and check the effective native ABI under the actual generation context. The
process environment must remain fixed. This includes effective process-cached
settings; observing the environment alone cannot establish the selected ABI.

The SDK-bootstrapped `native-crate-qualification` example exercises real native
producers and fresh native resolution, two same-named crates with different
signatures, aliases, privacy, namespace misuse, ambiguity, corruption and
compiler/backend changes. It checks producer stable identity after consumer
identity/interner changes. The expanded qualifier includes independently
compiled producer/consumer execution, argument/return/arity/generic/const/value
rejections, function-item identity and pointer coercions, callee/argument
ordering, moved/borrowed callee rejection, and separate-process signed/unsigned
128-bit direct and pointer ABI cases. These passed
in the bounded debug component alongside the original native phase regression.
An earlier candidate incorrectly accepted a short-circuit non-const call in a
constant; its emitted ELF was retained without execution. The corrected gate
requires native rejection and an independent stock-oracle rejection.
Later function-value candidates omitted callee effects and accepted a moved
mutable-reference callee. The invalid emitted artifact was retained without
execution; the valid effect witnesses exposed runtime failures. A separate
valid pointer witness exposed reading the callee after an argument changed it.
The corrected gates retain those sources and require the expected runtime or
borrow rejection. Existing constant-pointer diagnostics remain distinct from
unsupported constant-image scope.
The namespace gate also executes qualified paths, function/module re-exports
and globs, checking output parity with an explicit stock oracle. It rejects
hidden modules, private/restricted functions, wrong argument types and distinct
equal-signature function items. Same-named module keys and reverse lookup,
alias roots and recomputed-checksum invalid visibility records are checked.
Its SDK compiler bootstrap is not a native compiler bootstrap. This component
has a fixed profile and host runtime; Cargo, general no_std runtime, full Rust
and equivalent-work speed remain unqualified. Production routing is unchanged.
The type qualifier additionally exercises compound producer/consumer execution,
callback signatures, array/tuple returns, pointer calls, Copy behavior, fresh
type-table remapping and independent stock-oracle output parity. Valid-checksum
mutations reach the layout, Copy/drop, graph-reference, bound and effective-ABI
checks; an ABI mismatch must refuse before initializer or body work. The first
tuple refusal, missing memory closure, linker-driver failure and helper build
failure remain retained as separate attempts. Source review first identified an
ABI record taken outside emission mode; the corrected comparison covers every
node before signature installation. These are bounded component results, not
full semantic images, Cargo/native compiler bootstrap or a speed result.
The region qualifier executes the unchanged originally refused borrowed source
and independent shared/mutable/array/tuple reference consumers, plus local
borrowed pointer calls, with stock-oracle output parity. It requires native
loan/type rejection for escapes, mutable moves/aliases, nested static-reference
invariance, independent mutable-input aliasing and pointer generality. Valid
checksums do not bypass region-count, universal, root-index or erased-shape
checks. A separate installation check perturbs the interner/type tables while
preserving existing local region records; it does not execute that Program.

Full Rust remains required: complete module and source/input semantics, types/layout/drop,
traits/impls/coherence, generic bodies/obligations, macros, core/alloc/preludes,
panic/unwind, target/runtime/ABI closure and complete semantic inputs. Cargo's
existing unit graph, jobserver and parallel units remain the composition path.
No new scheduler, frontend, type system, backend or daemon is introduced here.

The design follows the distinction between semantic metadata and code closure
in the [Rust compiler development guide](https://rustc-dev-guide.rust-lang.org/backend/libs-and-metadata.html).
That guide describes rustc's format; this experiment does not claim to implement
or interoperate with its private binary format. Existing native objects,
shared-session proofs, pristine Lust source and protected Rust pins remain
separate qualification scopes.
