# Public retained scalar-library integration

Status: authored, awaiting exact-source hosted construction and qualification.
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

The admitted public output set is exactly dep-info,metadata,link with --out-dir.
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

Metadata-only cargo check, public --extern ingestion, other crate types,
unrepresented options/profiles and wider completion remain integration work.

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
