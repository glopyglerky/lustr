# Selective native metadata encoder

This selective crate writes the pinned Rust metadata format from an already
completed native library and its actual emitted function inventory. Its public
retained caller is authored under the exclusive-native product feature and awaits
exact-source hosted qualification. It does not parse a Rust consumer, run Rust
analysis, synthesize HIR or typed bodies, or retry compilation.

The first caller is the separate native metadata qualification executable. Its private
`CompletedUnit` establishes the admitted declaration, body, privacy and lint
facts using the unchanged source and the existing native checked program. It
joins checked calls to lowered/generated calls and actual emitted ELF symbols.
The research caller converts those actual facts to `Input`, invokes this encoder,
and constructs metadata and object members for an original Rust consumer. It is
separate from the installed compiler and has its own required CI campaign. The
new public retained owner uses the same artifact service with actual request,
source and output facts; it does not link the research qualifier. The public record
types in this serialization crate do not themselves certify source correctness.
The encoder checks that the definition, call and emitted-instance inventories
are complete, unique and mutually consistent before writing any metadata.

## Deliberate first boundary

The completed native producer admits a flat `#![no_std]` library containing
ordinary safe nongeneric functions with primitive `u64` parameters/results and
the producer's represented scalar bodies. Public functions may call actual
private helpers. The original request is edition 2024, Linux x86-64, optimization
level zero, no debug information, and abort panic strategy. Its actual overflow
and debug-assertion settings are retained. No option is changed to avoid MIR,
inlining, attribute or panic obligations.

This writer currently requires ASCII/LF source, original absolute Unicode source
and working-directory paths, and ordinary identifier spelling. Raw identifiers,
other declarations/types, macros, modules, generics, attributes, wider profiles,
other targets and general libraries remain unfinished integration. Refusing
those records does not reassign Lust-owned operations to a complement.

Both the original public and private definitions have real Rust identity
mappings and real native instances. The mapping supplies original v0 symbols
for an identity-preserving object-symbol rename; it does not invent source
`export_name` attributes. Public symbol exports follow the original applicable
export rules. The original generic-export filter visits this nonempty ordinary
instance inventory and finds no generic arguments. The separate
`exportable_items` field is empty because the original provider is restricted to
`sdylib`. Neither result stands in for an empty monomorphization graph.

No MIR is encoded for the admitted original nongeneric, nonconst, noninline
functions at the actual requested optimization level zero. A wider request must
establish its real MIR obligations before it can use this boundary.

## Original-source cut and ownership

The source root is
[`rust-lang/rust@5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`](https://github.com/rust-lang/rust/tree/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494).
`source-slices.json` records each original file hash, required operation,
semantic owner and adaptation. The original MIT and Apache licenses accompany
the extraction.

| Slice | Required operation | Native adaptation |
| --- | --- | --- |
| `src/upstream` | Original scalar/string/LEB128 memory encoding and metadata v12 schema | Excludes decoders, generic compiler serialization and unrelated dependencies |
| `src/table.rs` | Original dense/sparse metadata tables | Bounded mapped indices replace compiler `IndexVec`; original widths, thresholds and layout remain |
| `src/identity.rs` | Original stable crate/definition IDs, collision checks and v0 symbols | Actual native identities are associated with Rust IDs rather than cast to them |
| `src/upstream/stable_hash` and `odht_writer.rs` | Original stable hashing and definition-map writer | `core` names and required writer operations replace `std` dependencies; original crate bytes/decoder remain independent test oracles |
| `src/encode.rs` | Original lazy references, symbols, spans, definition/type/source tables and crate records | Completed native facts replace semantic query lookups for this explicit scope |
| `src/sdk.rs` | Immutable SDK facts and represented-context validation | No file, JSON, hash-tool or process access |
| `../lustr-metadata-sdk` | Host loading of the construction-qualified SDK receipt | Reads and verifies the declared metadata/code artifacts once; no user-source compiler invocation |

The semantic adapters are source-owned specializations of the cited original
operations. This does not claim that the full `TyCtxt` query machinery executes.
The imported closure excludes the Rust parser, resolver, type checker, MIR
builder/optimizer, borrow checker, interface/driver and LLVM backend. The encoder
does not import a second implementation of native-owned body compilation.

The metadata hash uses the original XXH3 algorithm over the original encoded
bytes and a native semantic supplement. That supplement binds checked source,
request, object, SDK and profile identities. It is a conservative native change
detector, not byte-for-byte reproduction of Rust's HIR fingerprint.

The public request now represents the pinned Cargo `-Z embed-metadata` choice.
Absent, explicit true and explicit false remain distinct tracked metadata hash
inputs. Only false uses a small original-format `CrateHeader` stub inside the
archive's `lib.rmeta` member; the standalone `.rmeta` remains full. The stub
has no unhashed root and carries exactly the full metadata SVH. The separate
`-C embed-metadata` option is not admitted by this request boundary.

These artifact variants share native checked definitions, generated bodies and
TinyCC objects. The artifact service retains at most three choices for the
current native snapshot, plus the last durable publication. A new choice encodes
metadata and builds its archive without repeating native body work; a retained
choice reuses its immutable bytes. Actual timings remain unqualified.

The pinned forced unused-dependency lint is a per-request host response, outside
metadata identity. This admitted leaf has no explicit extern dependencies, so
the public owner emits the original empty `unused_extern` force-warn JSON event
after resolved-program admission, including on a subsequent type error. Early
context refusals do not fabricate that analysis event. Lint and JSON-only options
do not change the metadata hash or artifact variant.

The original `RUSTC_FORCE_RUSTC_VERSION` override is outside the represented
request policy. Both host callers must reject an incompatible override; the public owner checks every actual client request. The
encoder neither reads mutable process environment nor silently changes the
pinned Rust version used in crate identity.

## Real SDK context

The construction tool uses the original pinned metadata reader and a fixed
original `no_std` reference to establish the exact dependency context. It has
two ordered unconditional dependencies: public `core`, then private
`compiler_builtins`. This is the program's actual core environment; the request
is never rewritten to `no_core`.

The caller must bind the receipt to that reviewed construction output.
`lustr_metadata_sdk::open` checks the exact closed dependency order/kind/privacy, both `.rmeta`
and `.rlib` paths/lengths/digests, original metadata magic/version/root bounds and
the semantic hash in each `.rmeta` header. Supplied JSON alone is not evidence
that an arbitrary SDK is compatible. Opening is a host construction cost, and
the resulting context is intended to be retained; encoding has no SDK file or
subprocess access.

## Checkpoint status and required qualification

The standalone research caller now joins the native generation to the metadata
and rlib-link writers. The [hosted campaign](../../docs/compiler-set/evidence/native-metadata-join-20261120/README.md)
passed on published source `974eb097ea9fc84a18c716fe0c2d469fdfcfab6c`, tree
`948074da4b3b23ed25fa789b17a924555f24e4ca`: the original Rust metadata reader and
ordinary `--extern` caller accepted the fixed native library, with 262,152
runtime assertions, 31 named controls and eight complete candidate process
traces. Private-access/signature rejection, mapping/member mutations and
failure/recovery controls are retained alongside the earlier failed attempt.
This finite fixture is not general metadata coverage. Source completion and
serialization costs still require separate measurement; no compiler speed
claim follows from this result.

The serialization crate uses `core` plus `alloc`, with an actual library check
on `x86_64-unknown-none`. Its production dependencies are `md-5` with default
features disabled and `twox-hash` with only allocation/XXH3-128 enabled. Required
original stable hashing and ODHT writer operations are source-owned slices;
their unchanged published crates are development-only differential oracles.
The map writer retains upstream's SSE2 group match/empty operations on applicable
build targets and its original word fallback elsewhere. Moving to `core` does
not replace those operations with a scalar lane scan or runtime detection.
The four focused controls check original crate identity, stable-hash byte
semantics, Unix path/OS-string identity, and complete ODHT bytes plus independent
decoding through collisions, replacements and growth.

The separate SDK adapter retains host filesystem/JSON/SHA256 dependencies.
`Input` carries the original Unicode POSIX path strings; its core path identity
adapter is checked against the original host `Path`/`OsStr` behavior. Building
the serializer for a target without `std` proves this library's dependency cut,
not new output-target coverage, a `no_std` compiler or socket-independent retained
compilation.

The shared/public integration is authored but unqualified. Full Rust consumer
metadata, wider declaration/lint/privacy obligations, the justified complementary
implementation and overall speedup remain incomplete. The central import inventory
and structural guard declare the selective product source separately from the
research qualifier. The CI checks source records, the core dependency cut,
original byte/hash controls and SDK receipt refusals. Existing campaigns qualify
only their declared scalar-library scope on their exact source. They do not
establish this new public caller, a full Rust compiler or a release.


## Retained caller source

A new retained candidate in the separate qualifier consumes the actual native
Session/PreparedRequest path and original per-function TinyCC objects. The
encoder now binds a canonical producer-object inventory instead of assuming
one object. Each entry records its planned archive member, original producer
object digest and exact source definitions. The host verifies symbol renaming
and separately records final member bytes, avoiding a metadata/symbol/hash
cycle. No upstream source, dependency or pin was broadened by this interface
change. The [retained caller description](../lustr-native-metadata-qualification/README.md#retained-native-candidate)
records its implementation and retained historical evidence. The
[public library contract](../../docs/compiler-set/public-scalar-library.md) records
the newly authored shared service, actual Cargo request boundary and its pending
exact-source qualification.
