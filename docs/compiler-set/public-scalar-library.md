# Public retained scalar-library integration

Status: authored; the latest published public campaign failed. The corrected
source requires its own hosted construction and qualification.
This is a bounded public library path. It does not establish full Rust coverage,
release admission, or speed relative to stock Lust or Rust.

## Compiler and request ownership

The installed public rustc entry parses the actual Cargo invocation. It accepts
the represented edition-2024, Linux-x64 leaf-library request with explicit
panic=abort and embed-bitcode=no, opt-level=0 and debuginfo=0. Compile-time target
facts must identify the actual Linux GNU x86-64 little-endian 64-bit host before
argument normalization or library discovery; an omitted --target is never
converted from a different host into the supported target. Rust's default
overflow-checks/debug-assertions at O0 are retained; explicitly supplied values
are passed through. The actual crate name, metadata disambiguator, extra filename,
original absolute source path, working directory, check-cfg inputs, dependency
search paths and fixed installed SDK identity join the native session context.
Output paths remain per-request inputs. Every request checks its actual
RUSTC_FORCE_RUSTC_VERSION environment: absent or the pinned version is represented;
an empty or different override is refused before native preparation.

The native Cargo graph binds the real package, library target and source.
Dependencies, build scripts and selected features are refused by this first
library context. No manifest is erased. The existing native completion remains
restricted to one ASCII no_std source with flat, ordinary u64 functions, the
represented bitwise/direct-call expression graph, complete liveness and all
existing declaration/lint/IR ownership checks.

One native PreparedRequest supplies the current emitted objects and completed
declarations. A shared transport-independent service performs original metadata,
symbol and archive operations. The research caller uses the same service.
The installed compiler does not link the research qualifier or full Rust driver.

## Delivery contract

The admitted public output sets with `--out-dir` are
`dep-info,metadata,link` for `cargo build --lib` and
`dep-info,metadata` for `cargo check --lib`.
Names use the caller's crate name and extra filename, including the metadata's
unhashed extra-filename record. Typed Cargo diagnostics preserve actual native
codes, messages, spans, labels, fixes and rendered text. Artifact notifications
are issued only after successful publication.

All requested bytes are staged and synced before replacement. Metadata and
dep-info are replaced before the link archive. Multiple names cannot be
atomically renamed as a set; partial effects and unknown durability remain
terminal and identify the number of files replaced. There is no retry.
Unchanged exact output bytes preserve their existing inodes and modification
times when those outputs still exist. Cargo itself can remove intermediate
outputs before calling the compiler; a native no-op then republishes the missing
bytes without checking, TinyCC, metadata encoding or artifact tools. Failed
native checking emits no new artifacts. A durable replacement receipt is retained
before fallible cleanup; a later cleanup failure cannot be described as preserving
the old output set.

Metadata-only requests reuse Lust's completed checked declaration state.
The pinned Rust symbol export providers return no exports without code
generation. The check encoder therefore emits regular crate metadata with
definitions and signatures but no emitted symbols. It caches one checked
generation and publishes the dependency file before the metadata. The Rust
metadata reader and downstream check consumer exercise the output separately
from the retained native checker.

Both artifact modes retain the missing `eh_personality` record that pinned
Rust reports for the completed `no_std` leaf and its exact core/builtins
dependencies. Qualification compares their language item lists with the
original Rust artifacts. The record informs downstream checks and leaves the
request's panic strategy unchanged.

For checks and library builds, the same retained owner can import a single flat
scalar library through the `--extern` artifact supplied by Cargo. It reads
the pinned metadata's public definitions, signatures, crate identity and
dependency hashes, then installs them into Lust's resolver. A new provider
artifact invalidates the consumer's retained state before checking. The
consumer's metadata records the provider if a checked call uses it, in the
original dependency order. The authored scope covers the pinned SDK closure
and an edition 2024 leaf dependency with no build script or features. The
consumer archive retains calls to the actual provider's Rust symbol. Lustr
checks these relocations before publishing, and the evaluator links and runs a
third executable against both archives.

An ordinary `cargo run --bin` request can also build a hosted scalar
executable against the two native libraries. The retained owner associates
each actual `.rlib` with its matching full `.rmeta`, checks the dependency
identities and the objects named by the archive, then installs their scalar
declarations in Lust's resolver. The existing native build links the supplied
objects into the executable delivered to Cargo. Unchanged requests reuse the
published executable bytes. A body edit updates changed objects and relinks
with the retained objects. The executable gate compares cold, unchanged,
edited, invalid, repaired and dependency-edited requests with pinned Cargo
and Rust, including the runtime output.

The matching `cargo check --bin` uses the actual scalar `.rmeta` inputs
and publishes dep-info and the requested zero-byte binary `.rmeta`. The
pinned Rust compiler produces that empty output for executable crate types.
The two library dependencies still carry genuine encoded crate metadata.
The binary check does no object generation, archive construction or program
execution.

General Cargo dependency graphs, additional crate types and profiles, and
wider semantic completion remain open.

## Metadata options and retained native state

Pinned Cargo requests `-Z embed-metadata=no` for this library and supplies both
the resulting `.rlib` and `.rmeta` when another crate consumes it. The archive
still has a real `lib.rmeta` member: a small Rust metadata stub whose crate hash
matches the separately emitted full metadata. It also retains the original
`lib.rmeta-link` and actual native object members. Omitting metadata from the
archive entirely would not implement the upstream contract.

The represented unstable option has three states: absent/default,
`Some(true)`, and `Some(false)`. These remain distinct tracked metadata
identities. The stable `-C embed-metadata` channel is a separate upstream option
and remains unrepresented by this bounded adapter; it is not silently treated
as an alias. With the stable channel absent, default and explicit true embed
the full metadata, while explicit false embeds the matching stub.

Metadata options travel with each request, outside the retained native
`SessionKey`. The artifact identity includes the actual native snapshot and
embedding option. A changed embedding identity requires the corresponding
metadata and archive, while unchanged native definitions, checked facts,
symbols and TinyCC objects remain reusable. At most three embedding variants are retained for the current native generation,
with the last durable publication also protected. An exact unchanged request or
a return to a still-retained option must reproduce the earlier artifact bytes
with zero encoder, transformer and archive-builder work. Changing the embedding
option cannot duplicate native checking or TinyCC emission.

Cargo's `--force-warn=unused_crate_dependencies` is represented only after the
actual dependency-free context is admitted. With the requested JSON mode, the
response includes the original empty `unused_extern` result at `force-warn`
level, including the represented typed-error path. This is per-request
diagnostic behavior. The upstream lint option does not enter the crate hash,
and it must not invalidate metadata or native objects.

These distinctions follow pinned [Cargo invocation and dependency-path
construction](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/mod.rs),
Rust's [effective embedding
option](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_session/src/session.rs),
[tracked option definitions](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_session/src/options.rs)
and [metadata hash/stub
encoding](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_metadata/src/rmeta/encoder.rs).
They are authored implementation requirements until qualified on their exact
source.

## Retention and resource accounting

The owner binds LUSTR_LIBRARY_HISTORY_ENTRIES once at startup. The default is
128 inactive entries; an integer from zero through 128 can lower that real
resource budget. The same LRU implementation also enforces a 16 MiB known-payload
limit independently on each native-object, native-definition and artifact cache.
These are engineering limits, not a claim that a live program uses bounded memory.
Current definitions/objects, the last accepted published snapshot and in-flight
Arc owners remain protected. Allocator/map overhead, the live source and original
native caches are visible in the daemon's existing process RSS accounting.

Removed declaration facts, generation IDs and object joins retire together.
Their old native verdict, read, IR and C slots are invalidated at the same time,
so an evicted declaration reintroduced later joins the existing worklist once.
Pruning also runs after rejected native requests; repeated invalid declaration
renames cannot accumulate inactive fact or age-map entries indefinitely. Recent
object versions can still be reused until they actually age out. Retirement after
publication is infallible; it cannot turn a committed artifact into a reported
pre-publication failure.

Optional cache_history observations identify the immutable owner policy, actual
active/inactive entry and payload counts, evicted object serials, retired native
IDs and evicted symbol mappings. A lowered budget exercises the production LRU
cut; it is not a test-only eviction switch.

## Dependencies, portability and performance

The metadata encoder retains its no_std plus alloc boundary. Filesystem,
diagnostic transport, SDK verification and artifact tools are hosted services.
The existing retained daemon transport remains in this slice; the service itself
contains no Unix socket code. A socket-free persistent public host is unfinished.

The bounded package carries the exact core/compiler_builtins metadata/link pair,
the pinned llvm-ar and llvm-objcopy, their resolved required runtime libraries,
and notices. The installed closure records each ELF dependency edge and the
system library names/interpreter paths that remain host ABI inputs; those system
bytes are not copied from the compiler SDK. No compiler or compiler driver is
packaged for fallback. The official
pinned distribution advertises no separate static archive-tool component.
Its dynamic tool closure can require the 208,283,952-byte LLVM shared library;
this is an explicit size/portability limitation, not minimality or speed proof.

The installed SDK is verified once per owner and retained. File generations are
checked before requests without rehashing all SDK bytes. A changed generation
is terminal. Native function/object reuse and artifact reuse retain their actual
keys. No-op requests invoke no encoder, object transformer or archive builder.
Optional observations expose actual work counters, immutable object witnesses,
tool argv/results/durations and phase durations; independent process, archive and
consumer evaluation remains necessary. These observations are not benchmarks.

## Source and qualification records

The exact source inventory is upstream/public-library-integration.json.
The original source-slice inventories, native integration patch and dependency/tool
manifests remain explicit. tools/update-public-library-source-manifest.py only
regenerates source hashes; it performs no compilation or qualification.

The new independent public campaign must run real pinned Cargo, capture actual
requests, reject Cargo Fresh as compiler reuse evidence, verify native type-error
spans and recovery, inspect real archives and metadata, and execute an independent
original-Rust consumer after product tracing. Passing earlier research campaigns
does not qualify this new public source.

The existing campaign retains nine real Cargo histories, four direct eviction
replays, three direct no-op/invalid/repair requests, one warm version refusal and
two unintegrated-request refusals. Its twelve original-Rust consumers and
3,145,824 expected assertions remain required. An additional direct-request
group changes only the embedding option through explicit false, default,
explicit true and explicit false. Four additional original producers/readers
and consumers must verify format, identity, native-object reuse and 1,048,608
result assertions. A malformed embedding request between the last two states
must preserve outputs and leave the owner untouched. This group is not counted
as additional Cargo histories. The combined authored expectation is sixteen
consumers and 4,194,432 assertions; it is not a passing result.

The [source-bound attempt records](evidence/public-scalar-library-20261120/README.md)
preserve the failed real Cargo requests and separate research qualifications.
