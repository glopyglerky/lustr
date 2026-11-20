# Selective native metadata encoder

This standalone research crate writes the pinned Rust metadata format from an
already completed native library and its actual emitted function inventory.
It is not reachable through a product Cargo feature. It does not parse a Rust
consumer, run Rust analysis, synthesize HIR or typed bodies, or retry compilation.

The intended first caller is the separate native consumer qualifier. Its private
`CompletedUnit` establishes the admitted declaration, body, privacy and lint
facts using the unchanged source and the existing native checked program. It
joins checked calls to lowered/generated calls and actual emitted ELF symbols.
The committed qualifier stops at that completed-record and C ABI object boundary;
it does not yet convert those facts to `Input` or call this encoder. The public record
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

The original `RUSTC_FORCE_RUSTC_VERSION` override is outside the represented
request policy. The host qualifier must reject an incompatible override; the
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

This checkpoint builds as a standalone library with the exact pinned SDK. No
committed native caller invokes its metadata or rlib-link writer. It
does not yet establish that an ordinary Rust caller accepts the produced
metadata or executes the renamed native object correctly. That joined gate,
including private-access rejection and malformed/missing/duplicate mapping
controls, is required before a compatibility claim. Source completion and
serialization costs must be reported separately from native body compilation;
no compiler speed claim follows from this build.

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

The seam is currently separate research components, not integration into the public
retained compiler. Full Rust consumer metadata, wider declaration/lint/privacy
obligations, the justified complementary implementation and overall speedup
remain incomplete. The central import inventory and structural guard track
these research components without admitting them to the product. The CI checks
their source records, core dependency cut, original byte/hash controls and SDK
receipt refusals. Those checks do not qualify joined Rust metadata or a release.
