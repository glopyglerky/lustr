# Native compiler entry candidate

This document describes transitional implementation and retained experiments.
The [governing plan](architecture-plan.md) requires an integrated native and
complementary compiler; the whole-program retry path described below is not
the destination. Read the current contract before making routing changes.

The owned original-AST panic-text extension also models list-array expressions
from their ordered native element IDs and exact original spans/commas/brackets.
It retains explicit wrappers and the pinned Rust array/list indentation boxes.
Array repeats bind both original native children, source semicolon and brackets,
with a separate indentation box and a break after the semicolon. The length is
printed as its anonymous-const value expression. Unrepresented attributes remain
outside this formatting rule.
Only derived panic text is seeded before full native rechecking; executable
array semantics and source remain unchanged. Component qualification and
measurements are separate from full native Rust, crates and product delivery.
The const-block extension uses the actual native block ID, original spans and
`const` token, then applies the pinned anonymous-const indentation box around
the existing block printer. Const evaluation and rejection stay in the native
compiler's complete recheck; the formatter supplies only derived panic text.

Status: source candidate, not a released or fully qualified replacement.

The current candidate combines a whole-target Lust path with an identified
Rust implementation and a bounded mixed bridge. Importing upstream Rust is
allowed; the full-program retry route below is transitional compatibility,
not completion of the requested compiler boundary. The retained experimental
`lustr_native_object_v1` phase experiment shares original native semantic/IR
APIs and emits real freestanding objects without fake rlib/rmeta. Its primitive
scope is under qualification. Crate resolution, metadata/imports, complete Rust
coverage, Cargo composition and measured project speed remain open.

The internal [typed artifact experiment](proofs/native-crates.md) adds actual
checked structural signatures and complete object closure, with independently
identified imports installed before native resolver rewriting. It materializes
foreign signature types in the consumer's existing type tables without source
or AST declarations, before initializer and body checking. The qualifier executes
independent scalar producer/consumer composition through native checking,
lowering and C generation. Scalar function values retain independent item
identity and reify to the recorded producer symbol; callee effects, borrow uses
and pointer evaluation before arguments remain in the native pipeline.
General library/Cargo composition remains open.
The producer artifact entry also uses the existing resolver. Its versioned
image retains inline modules, scoped visibility, namespace bindings and real
re-export targets; imported module lookup keys retain stable crate identity.
The qualifier executes path/re-export/glob consumers and rejects malformed
visibility, hidden/private/restricted paths and independent item substitution.
The integration is unadmitted and the production entry is unchanged.
Format sixteen retains bounded structural signature graphs for scalars, floats,
tuples, arrays and safe function pointers. It recomputes native
layout, Copy and drop facts and compares effective C passing rules before
installing signatures. Large and zero-sized returns use existing native backend
rules. Required memory calls close over Lust's existing C memory unit and the
exact licensed musl fill dependency, with localized symbols. Runtime-bearing
library images record remaining dependencies and actual function locations;
the consumer must match them to freshly compiled original runtime and location
providers before imported semantic definitions. Symbol kinds and qualified
relocation uses must agree. This does not qualify general no_std runtime support.
Original region signatures now accompany represented sized reference types.
The consumer validates their erased shapes and installs them before body work;
independent calls and local pointer coercions use the native borrow rules.
Borrowed callback exports preserve the original output-slot mask and width,
validate their bounded region shapes and enter the original pointer proof.
Unbounded named late-bound function declarations must match their original
native region names, count and binding classification. Lifetime-only function
declarations also retain early-bound parameters and declared outlives facts.
Independent names, canonical facts and binding flags validate before publication;
the original native engine handles calls and pointer generality. Free item
regions and type/const-generic region images still refuse.
Binders the original parser normalizes to elision retain that behavior; general
higher-ranked support is not established.

Slice and string reference nodes represent sized wide references with original
native layout, Copy/drop and effective C ABI checks. Slice element graphs keep
their own lifetime applications; no layout is fabricated for `[T]` or `str`.
Original native region/call/pointer/borrow/lowering machinery supplies semantics.
General unsized aggregates and real core/no_std runtime remain unqualified.

Closed nongeneric structs and enums retain original stable identities, variant
shapes and discriminants, field types and visibility. A proposed shape-only bridge lets every image's layout, Copy,
drop, effective ABI and region records validate before any imported semantic
payload or type name is published. The original `Packet` partial gate executes;
the unchanged array-backed nominal gate also executes large/nested and borrowed
consumers. The separate runtime gate checks independent producer and consumer
panic locations and malformed dependency refusal before imported semantic
publication. Tuple/unit constructors and enum variant imports/re-exports use
the original native resolver and privacy checks. Tagged and niche enum layouts
are recomputed, including every variant offset and tag field. Independent
producer/consumer runtime, empty-enum production, malformed-record rejection
and remapping have bounded evidence. General nominal graphs and no_std runtime
remain open.

Lifetime-only nominal records retain original declaration names and kept field
types alongside erased layouts. Separate applied nodes preserve lifetime
arguments, including nested nominal fields. Kept fields and function signatures
require complete applications before publication; omitted records cannot remove
a nested static relationship. Original native scopes and region tables supply
variance, outlives and borrowing. Applied enums retain their complete layouts.
Declared nominal lifetime outlives bounds retain canonical parameter relations,
including bounds to `'static`, in the original scope and region tables. A focused
original-checker interface validates staged nested static obligations before
imported definitions, names, constants or signatures are published, preserving
queued consumer obligations. Reference cycles name a complete definition in the
same image by stable identity, with separate kept lifetime arguments. Shapes
receive fresh consumer indexes before materialization; ordinary value edges
remain ordered. Recursive and mutually recursive runtime, borrow rejection,
malformed target/application, late ABI and remapping checks pass. Function
type/const generics and borrowed nominal pointer images remain unqualified.

A separate symbolic nominal table records type/const declaration kinds and
indexes, exact implicit Sized predicates and scalar, tuple and array fields.
Uninstantiated templates have no layout, Copy/drop or C ABI record. The original
native materializer builds concrete consumer instances, with names derived from
actual instance keys and inert table rows preserved. A focused resolver join
reads collected declarations when no local written header exists; the original
argument resolver and const typer enforce kinds, arity, types and ranges.
Bounded independent struct/enum and type/const runtime pairs pass, with move,
identity and typed-const rejection and checksum-valid malformed-template checks.
Every type parameter must occur in a field; unused const parameters remain legal.
General producer applications, lifetime-bearing symbolic fields, defaults,
derives, Drop, trait predicates and generic bodies still require records.

Closed exported constants carry stable identities, type roots, evaluated
scalar/aggregate/ADT values and promotion qualifiers. Original native initializer
type and borrow checks remain required. Values validate before imported semantic
publication and enter the consumer's existing constant tables with fresh indexes.
Actual transient initializer checks enter the original static const-function walk,
with explicit nested inline-const traversal. Earlier inaccessible private constants
with unrepresented values retain their bindings without an imported value grant.
Generic const bodies and reference/allocation/function identities, genuine core,
general type/generic const positions and Cargo integration remain open.

The primary Cargo project and command entry point are at the repository root.
`src/main.rs` accepts rustc requests and native whole-file `build`/`check`
commands. `src/lib.rs` recognizes the request and artifact contracts representable
by unchanged Lust. The compiler does not select languages by source keywords.

## Execution

1. Representable native requests use the unchanged Lust protocol and daemon.
   Lustr starts the original daemon synchronously for its first request, avoiding
   a separate cold compile followed by duplicate prewarming.
2. A single-file cold request can enter unchanged Lust modules through
   `crates/lustr-native-core`. This adds a C ABI outside the protected source.
   Arguments remain borrowed for the call; diagnostic bytes are copied through
   a C callback. Rust-owned allocations never cross that ABI. The entry permits
   one call per process. It is not a general in-process compiler service.
   Nondefault transparent-huge-page policies and front-image overrides retain
   the unchanged executable path. The library caller restores its original
   stack limit before any full-core fallback.
3. Other rustc requests use the unchanged full Rust executable, `rustc-full`,
   through `exec`. This preserves its startup TLS requirements. A late-loaded
   compiler-library experiment failed that requirement and is excluded.

The thin entry does not link Rust's compiler library or LLVM. The native path
does not load them. Full Rust compatibility comes from the separately identified
upstream Rust core; it is not newly implemented native Lust coverage.

The C ABI library uses PIC-built TinyCC from unchanged pinned TinyCC source.
Its cache namespace combines the installed library SHA256 with the pristine
native runtime-family manifest SHA256. Lustr adds that namespace to its native
request identity, without altering source bytes or the caller's environment.
Upstream cache keys also hash the actual TinyCC runtime. Packaging must generate
`lib/lustr-native.id` from those exact frozen inputs and verify it on extraction.

Cargo-backed Lust requests use the pinned Cargo's compatibility layout setting
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1`. It is carried in the request
because build-script execution restores the client's environment. An explicit
conflicting setting uses the full Rust path. No upstream Cargo or Lust patch is
applied. Build scripts and procedural macros remain real Cargo work.

Manifest requests first resolve their package and target through Cargo metadata.
Explicit package specifications use Cargo's own package-ID resolver, including
workspace-inherited versions. Name, kind and canonical source must agree on one
target. The validated member manifest and target are passed to native Lust;
fallback reuses the same selection and checks the artifact's complete identity.
This preflight can add manifest-request work. Its cost belongs in measurements;
it is not a speed improvement by itself. Single-file requests need no preflight.

Foreground Cargo and captured native subprocesses have owned process groups.
On SIGINT, SIGTERM, SIGHUP or SIGQUIT, Lustr kills its active group and terminates
with the original signal; explicitly ignored signals stay ignored. Errors drop
the child guard, which kills the group and waits for its leader. Normal completion
also closes remaining work in that group before reaping the leader.
An exit observer runs alongside output reading, so a descendant retaining a pipe
cannot prevent cleanup after its leader exits. These
guarantees exclude SIGKILL, deliberately detached descendants and the separately
managed upstream daemon. They do not imply an arbitrary process-tree sandbox.
Successful copied Cargo artifacts use a temporary sibling and an atomic rename;
an interrupted copy cannot publish a partial destination. Forced termination
can leave that temporary file. No crash-durability guarantee is made.

## Contracts and limits

### Experimental native constraint inference

The separate experimental `lustr_native_session_v1` entry retains an actual
native driver Session across supported edits and enters the requested native
phase directly. It shares that state with existing Lustr elaboration, then
retires transformed state after the request. Standalone linking currently adds
two private intermediate links because Lust couples semantic persistence and
base-library linking. The [shared-session contract](proofs/native-session.md)
records its bounded checks, input obligations and open performance/crate gates.
Product routing and the retained-artifact ABI remain separately qualified.

The opt-in ABI's owned frontend can also parse bounded boolean `unsafe` blocks
from original tokens, including initialized boolean locals with checked lexical
scope, shadowing and nested block capture. Direct safe Rust calls with boolean
inputs and output require original-context native target, safety and ABI checks
before collection returns. An explicit module composition references unchanged Lust
modules; separate parser and collection facades retain the original AST and
lower only safety-qualified clones before full native checks. The native
item parser and arena append API also integrate original local function items
including local `const fn` headers, into affected enclosing blocks without
changing their native qualifiers. Native const-function checking remains
required even for runtime-only calls. Native statement-item collection retains their
lexical scope, forward visibility, duplicate checks and separate function bodies.
Items within the bounded boolean safety derivation remain unsupported. The native
expander's exact injected assertion-message form is preserved through its payload
argument and call-span provenance. This supplies no general unsafe-operation
permission. The default ABI still includes pristine Lust's entry source.
See the [frontend contract and limits](proofs/native-unsafe-blocks.md).

The `native-inference` Cargo features are off by default, independently in the
root and `crates/lustr-native-core`. The owned ABI module reads actual checked
types and AST nodes after an ordinary native rejection. Its bounded rule handles
delayed `&str`/`String` and checked canonical owned `OsString`/`PathBuf` sources
to canonical platform `OsString`/`PathBuf` conversions. A checked shared reference
to the exact canonical platform `Path` or `OsStr` also supports delayed `PathBuf` conversion;
mutable references and user lookalikes refuse this extension. This covers
including non-Copy `Vec<OsString>`/`Vec<PathBuf>` `extend` arrays fixed by later constraints. Competing
source traits and custom `Extend` implementations refuse this rule. An owned
AST clone receives explicit qualified `Into` calls; original source bytes stay
unchanged. The arena reindexer preserves expression references and function and
closure ranges. Retained checks, passing marks and generic reuse are cleared,
then native collect/check/borrow checking/code generation run again. At most two
elaboration rounds can expose checked constraints hidden by an earlier rejected
conversion in the same function; unresolved programs remain failures.

The inference worker starts with one retained native `Check`, using its rejected
program directly for elaboration. It no longer runs a discarded full build
before that check. A passing check still requires the requested native `Build`,
including code generation and linking. Original build-only library/target
refusals run through the unchanged build entry before any source check. This
worker reuses its initial check only within one request. The product does not
retain inference sessions across requests.

A separate experimental `lustr_native_retained_v1` ABI can reuse one successful
inference executable in the same loaded compiler process, for at most 64 calls.
It accepts a complete Unicode client environment without mutating process
environment. Request/profile/features/target/shown-path, process environment,
working directory and actual Cargo graph must match. Bounded byte snapshots
cover the source tree and declared native library/TinyCC roots, including
directory entries and source symlinks. Native observed source and macro reads
must belong to those snapshots; module alternatives and observed path bindings
must still stand. Unknown reads, edits, output removal/tampering and changed
path bindings compile again from original source. Failures are never cached.
No elaborated AST survives a request. A hit verifies the existing ELF bytes
and permissions instead of checking or linking again.

The runtime source trees must remain fixed for this process: a changed or
unreadable native library/TinyCC input retires the instance with status 2,
requiring a new process. The declared extra source roots use
`LUSTR_INFERENCE_REUSE_ROOTS`; an undeclared observed input disables artifact
reuse. Tree collection bounds file/name/path bytes and entries before allocation.
The pinned Linux GNU x86_64 prototype also observes actual TinyCC header,
CRT, library and linker-script opens, including failed search candidates.
Session disk objects and manifests are disabled before checking. Every generated
C-unit object must match a replay through the same native compiler with a Rust
observer recording raw length-framed pathnames. In-process inputs use bounded
descriptor reads that preserve offsets and errno. Clock-macro expansion,
unknown observer/compiler identities and incomplete observations prevent reuse;
secondary compiler failures remain failures. Source edits still run the full
native pipeline. Files must stay stable during a request. Runtime-loader
closure and supervised product lifecycle remain open, and this is not admitted
as a general build cache, product service, daemon replacement, native Cargo
freshness or release. Its [fresh optimized measurements](measurements/2026-11-15-native-host-inputs-v36/README.md)
retain substantial miss-time regressions and a coordinator no-op slower than Cargo.
`tools/native-retained-probe.rs` and `tests/native_reuse.rs` exercise this ABI
directly. The original one-shot entry and product routes remain separate.

The Cargo validation bridge admits only a same-package bin/library closure
without build scripts. It intersects actual selected-unit Cargo environment
facts, requires every observed read to agree, and checks all recorded file,
environment and shown-path dependencies with the native validator. Position
reads, stale inputs, differing per-unit values and unsupported closures refuse
elaboration. The opt-in ABI conservatively refuses non-Unicode environments
before native setup. It does not mutate the process environment. A unique
scratch guard owns temporary output through normal return and unwinding;
process abort or forced termination cannot run its destructor.

The optional root product path invokes `lustr-native-worker` after native
rejection. It loads a separately installed `lib/liblustr_inference_core.so`;
the primary `liblustr_native_core.so` and existing native success path remain
separate. Both components need the matching runtime-family identity. Nondefault
caller huge-page policies retain the established executable/full-core route;
worker failures and signals stay failures. The published preview contains no
such worker or inference extension.

`tests/native_inference.rs` qualifies direct native ELF/runtime results against
the real Rust/Cargo oracle, seven invalid/ambiguous cases, controlled Cargo
environment/refusal contexts, the unchanged complete coordinator and a mixed
drop/unwind fixture. Product checks cover actual worker selection, nondefault
huge-page routing and terminal worker status/signal propagation. These bounded
Linux x86_64 checks do not establish full Rust, native rlib/metadata/dep-info,
general Cargo support, retained inference sessions, release admission or speed.

The owned PathBuf rule has an actual component36 native rejection and stock
runtime witness. Its mixed alias/OsString/PathBuf fixture checks arena ordering,
Drop and unwind; borrowing, moved String, mutability and ambiguous traits/Extend
remain rejected. Retained requests also compare PathBuf-to-OsString target
edits, rejection and recovery with cold native compilation and the stock oracle.
The product34 inference component remains separate and unchanged.

The shared `Path` rule preserves the original borrowed-path source and compares
native runtime output with stock Rust. Mixed aliases, arrays, closures, owner
mutation after copying and Drop/unwind are exercised together. Live-borrow owner
moves/mutation, moved results, immutable vectors and competing methods/Extend
remain rejected. Retained source/target edits, invalid edits and recovery are
checked against cold native compilation and the oracle. These are bounded
component tests; the product34 component and general native coverage remain
separate.

The shared `OsStr` rule preserves its original component40 rejection source.
The mixed fixture combines aliases, arrays, closures, prior Path and owned
conversions, Linux non-Unicode OS bytes, mutation after copying, Drop and unwind.
Live and expired borrows, moves, mutable references, lookalikes and ambiguous
methods/Extend remain subject to the complete native recheck. Direct runtime
parity, retained source/target edits, rejection and recovery qualify this rule;
they do not establish general language or crate coverage.

The checked assertion rule preserves the complete original mixed `OsStr`
source, including its shared slice/array `assert_eq!` comparisons. For a checked
shared array reference against a shared slice of the same concrete element type,
the owned AST adds native array-to-slice unsizing inside the original
`assert_eq!` or `assert_ne!` macro. Rvalues retain the direct cast. For a place,
an owned match first borrows both original operands in order. Fresh bindings
must be absent from the complete interner. Expression and pattern arenas and
function ranges are remapped together, including empty and nested arenas.
Each lazy message argument retains its original borrowed place and then uses
both operand bindings, keeping the original place loans live during message
evaluation. A failed first implementation omitted those uses and accepted an
invalid message assignment; its source and failure are retained. The corrected
development candidate rejects that case after elaboration and preserves packed
field rejection, non-Copy messages, formatting and temporary-message drops.
Equality and Debug must exist for the canonical element type. Full native
checking, borrowing and code generation run again; original source remains
unchanged. Unresolved, generic, mutable and heterogeneous forms remain outside
this rule. The existing two-round bound and retained original-source invalidation
still apply. An owned original-AST formatter now supplies a bounded native
extension for long message-free `assert!` conditions. It reads original source
tokens into a scratch AST, derives consistent/inconsistent line breaks, and
replaces only the unknown-text marker before full native rechecking. Original
condition evaluation, arenas, borrowing, Drop and unwind remain native work.
Expanded macro contexts, empty statements, items, attributes,
generics and unmodeled grammar retain L0809. The original mixed source and
long-false multiline payload have direct and retained development parity;
the [formatter contract and corpus](proofs/native-assert-text.md) retain its
proof limits and first payload indentation failure. This component expansion does not qualify the product route,
general Rust/Cargo coverage or a compilation speed gain.

The owned panic-text formatter also models binary operators from their original
AST and source-token boundaries, preserving explicit parentheses and pinned
Rust precedence/fixity. Unmodeled contextual fixups retain refusal. The original
condition and native executable evaluation, borrowing, const and unwind checks
stay unchanged. Direct/retained runtime, independent review and measurements
qualify this component separately; the full compiler contract remains open.

The original-AST panic-text formatter also handles bounded struct expressions
using exact path, field, shorthand and rest source identities. It models
consistent field boxes, conditional trailing commas and closing indentation.
The application and executable field AST are unchanged; full native checking
still determines acceptance. Attributes, generic paths and unmodeled contextual
forms retain refusal. The complete original struct assertion and scoped direct,
retained, review and measurement evidence remain separate from product and full
language qualification.

Tuple-field panic text uses the original scratch-AST index and exact source
tokens, including the native parser's two-node split of a float token such as
`value.0.1`. Both expression boxes and explicit parentheses are preserved;
noncanonical tokens and unmodeled contexts refuse. Source and executable field
access remain unchanged and subject to the complete native recheck. Component
runtime, retained edits/recovery, optimized measurements and independent review
qualify this expansion separately from the full compiler contract.

Tuple-literal panic text uses original ordered element spans and source commas.
Structural tuple parentheses stay distinct from extra source wrappers; singleton
commas and the pinned inconsistent-list layout remain explicit. Only the derived
panic literal changes before full native checking. Actual runtime, rejection,
retained edit/recovery, review and speed evidence qualify this bounded component
separately; full Rust and crate support remain open.

### Native source-library additions

After an ordinary native rejection, diagnostic bytes may suggest a library
retry. They never establish acceptance: unchanged Lust checks the entire
original program with `library/lustr`. The owned library shares original
platform modules by path and supplies Linux executable/shared-library naming
constants, `From<&OsStr>`, `From<&OsString>` and `From<&PathBuf>` for `PathBuf`,
`OsString::reserve`/`reserve_exact` and `PathBuf::reserve`/`reserve_exact`.
The capacity methods restore the
original owned buffer during unwinding. No application source is rewritten.

A stable installed component can retain an ordinary pristine Lust daemon.
Its core is a real hard link to the original core, preserving upstream's actual
device/inode/mtime wire identity. Its executable path selects a separate default
support prefix. Both support links must have their expected relative targets.
The manifest covers the complete owned and shared library source inventory and
the native runtime-family identity file. A successful content check and unchanged
manifest precede reuse; the full manifest digest joins the request's package
identity and a distinct socket key. Added, changed or unsealed sources and an
unsuitable component inode take the explicit cold source-library path. User
source/front overrides do not enter automatic retention.

This assumes a trusted, stable installation throughout a request. It is not a
source snapshot or protection against concurrent mutation. The identity file
does not authenticate the whole runtime by itself; extraction and package
verification remain separate obligations. Archive extraction must preserve the
core hard link. A copying tool that splits it deliberately loses retention.

Successful additions trace `lustr-native`; an expanded rejection and subsequent
compatibility result remain distinct. Runtime/rejection tests compare original
programs with the full Rust oracle, including non-Unicode values, capacity
overflow/unwind, source reseal/restore, added inputs, wrong core/support links
and caller macro environment. Canonical `type_name` involving the nested env
implementation still needs the full core. This is bounded library coverage,
not a full native language or crate interface.

The candidate's nine-sample single-file campaign retains both improvements in
unchanged daemon requests and regressions in cold compilation, edits and
rejection. Those measurements do not establish Cargo no-op or full-project
speed. The current native host is Linux x86_64 GNU on AerynOS 2026.08 with an
observed GLIBC 2.39 floor. The complete ELF audit also records libgcc, zlib and
both `/usr/lib/ld-linux-x86-64.so.2` and `/lib64/ld-linux-x86-64.so.2` interpreter
paths. Those host facilities remain dependencies. Wasm qualification belongs
to full-core linking, not native execution.

Native Lust has no Rust rlib/rmeta emitter. Requests requiring those artifacts,
Rust-compatible structured diagnostics, unrepresented flags or other targets
stay with the full core. Do not forge metadata or dep-info. Native whole-target
compilation and Cargo's per-crate artifact graph remain different interfaces.

The automatic native command interface currently covers `build FILE.rs` and
`check FILE.rs`. A native compiler rejection can fall back before any application
or test executes. Compiler crashes and other failures remain failures. No
executed application is retried. Other Cargo commands retain the separate
compatibility integration; this is not full native coverage of Cargo workspaces.

The full Rust component is an unmodified upstream distribution at source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, not a newly bootstrapped blueFe
compiler. [`upstream/bluefe-core.pin`](../../upstream/bluefe-core.pin) binds
compiler, Cargo, driver, LLVM and native binary inputs. Compiler-private metadata
used by withdrawn experiments is not a runtime requirement.

## Qualification

`tools/native-relink.rs` provides an offline recipient rebuild recipe for a
source kit, a matching reference runtime and a new work directory. Its separate
`prepare` and `build` steps preserve recipient TinyCC edits and regenerate all
native consumers, the native pin, runtime identity, source seal and hardlink.
The fixed host build clears caller target settings and rejects discovered
directory configuration and release-profile environment overrides. Its build
completion digest is invalidated before a new attempt; runtime qualification
is still separate. It does not install globally or provide atomic update,
rollback, arbitrary TinyCC correctness, relocation or privacy guarantees.

`tests/compiler_contract.rs` exercises the actual assembled runtime. It requires
explicit `LUSTR_TEST_PACKAGE` and a new `LUSTR_TEST_OUTPUT` directory, including
paths with spaces. Its opt-in tests cover native cold/daemon/edit requests,
full-language fallback, genuine metadata, structured rejection, build-script
reuse, proc macros, doctests, panic/Drop and Wasm linking. The packaged SDK tools
also exercise rustfmt reject/format/pass, Cargo fmt, direct Clippy and Cargo
Clippy rejection with an explicitly enabled boolean lint, and a valid Clippy
check. Ignored tests have not run merely because ordinary `cargo test` succeeds.

Selection regressions cover a nonexistent bin, multiple bins sharing a source,
the wrong package or source, inherited workspace versions and Cargo package
specifications. Cancellation fixtures stop the wrapper by PID and by process
group, and observe Cargo, a build script and its child. See the
[conditional selection and lifecycle proofs](proofs/native-entry-contracts.md).

Every artifact still needs frozen-byte independent review, relocation, notices,
privacy checks and equivalent-workload speed evidence. The preliminary process
dispatch experiment retained a regression; the revised entry's pilot is not
campaign acceptance. These checks do not prove whole-stack memory safety,
universal compiler correctness or a never-slower bound for all Rust programs.
The [full native compiler contract](native-contract.md) remains unfinished.

Work stays in the user's Lustr/blueFe forks. Both upstream compiler source trees
remain protected. See [Lust preservation](upstream-lust.md) and
[blueFe's Rust preservation policy](https://github.com/glopyglerky/blueFe/blob/main/docs/bluefe/upstream-preservation.md).
