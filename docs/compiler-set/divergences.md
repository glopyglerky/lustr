# Fork ownership and divergences

Lustr and BlueFe are projects with their own entry points, source maps and design
decisions. Required upstream implementation remains where it is needed for a
working compiler and for reliable updates. Inherited branding and historical
research are identified rather than pasted into the primary project README.

## Lustr

Active work is full native Rust and native unit/artifact composition, with two
complementary native compilers under Cargo's graph. The explicit experimental
object ABI uses original native phase APIs once, then a freestanding Clang C
backend and actual ELF symbol-closure validation. It adds no host runtime unit
or rustc acceptance. Its bounded primitive unit is not full core/alloc, native
rlib/rmeta, Rust ABI interoperability or general Cargo support. The unchanged
Lust path and historical compatibility artifacts remain separate. TinyCC,
Clang/LLVM, the compiler's host runtime and existing upstream C remain explicit
dependencies; no whole-toolchain memory-safety claim follows.

The owned experimental native artifact stores actual checked structural signatures
and the complete object in a versioned format. A proposed early resolver hook
admits independent stable crate/definition identities before source rewriting;
consumer signature types enter the existing native tables before initializer
and body checking. A proposed typed call target connects calls to the
actual producer object symbols, with native consumer execution in the component
qualifier. Foreign function values keep complete definition identities, reify
through installed linkage and preserve callee effects and borrow uses. Pointer
calls capture their callee before arguments. Format fifteen stores bounded
scalar/float/tuple/array/safe-function-pointer type graphs with actual
native layout, Copy, drop and effective C ABI records. All imported nodes are
checked in the emission naming mode before any signatures are installed.
The graph also represents shared and mutable slice/string references as sized
wide values, preserving original element and lifetime relationships. Unsized
referents receive no invented layout or ABI. General unsized nominal types,
core identity and no_std runtime remain unqualified.
Large and zero-sized returns use the original backend rules. Compiler-generated
memory references close over the exact upstream Lust C memory unit and licensed
musl fill dependency, whose symbols become local to each object. Clang, the selected linker and object
localizer have explicit file identities. These remain C/tool dependencies;
foreign constant values, general region graphs, general nominal type/trait/drop
images, general library semantics, rustc metadata and Cargo remain open.
Represented sized references carry original native RegionSig records, checked
against erased types before any imported signatures or initializers. The
original loan/region engine handles independent calls and consumer-local
pointer coercions. Borrowed callback exports also retain original output-slot
masks and widths; bounds and shape checks precede the original pointer proof.
Unbounded named late-bound function declarations must match their original
native region names, count and classification. Lifetime-only function bounds and
early-bound parameters now carry independent declaration names, canonical native
facts and original binding flags. The original lexer supplies Unicode name
validation; original call and pointer checks enforce the constraints.
Type/const generics and free item regions still require further records.
Original binder normalization equivalent to elision remains distinct from
general higher-ranked support. This does not establish general lifetime support.
The producer now uses the shared native resolver through a proposed fresh-root
parser interface. Its owned image records actual inline modules, namespace
bindings, visibility scopes and re-exports. Import reconstruction uses stable
crate magic and native visibility checks; it invents no source declarations.
Additional source/expansion/dependency records and core/alloc identities remain
required. The bare-object route and production routing are unchanged.
See the [artifact contract](proofs/native-crates.md). Its digest helper explicitly
depends on the host's upstream SHA256 tool. The integration remains unadmitted.

Closed nongeneric struct and enum records are a further unadmitted image experiment.
All pending shapes, node properties and regions validate before imported semantic
payloads. The original `Packet` partial gate executes with stock-oracle parity;
the unchanged array-backed nominal gate also executes large/nested and borrowed
consumers. Explicit runtime/source-location records must match actual original
providers before imported semantic publication, including function/data use
checks and malformed-symbol refusal. Independent producer and consumer abort
locations have stock-oracle parity. This host-linked component does not qualify
general nominal semantics or a no_std runtime. The fill provider remains an
upstream C compatibility dependency; native Rust fill computation is open.
Tuple/unit constructors and enum variant imports, globs and re-exports now use
the original native resolver and privacy tables. Variant identities,
discriminants, fields, all layout offsets and tag/niche data are recorded and
recomputed before semantic publication. Native runtime and rejection gates
cover tagged/niche and nested payloads, matching, privacy, moves and distinct
identities. Empty-enum production, valid-checksum malformed records, remapping
and later-image ABI refusal have separate checks. Lifetime-only borrowed nominals
also retain original declared outlives facts and reference cycles through stable
definitions. Original variance, outlives and borrow checking govern the fresh
consumer tables; recursive runtime and malformed-image gates pass. Separate
symbolic type/const nominal declarations retain real parameter kinds and implicit
Sized obligations without assigning machine layouts to templates. Original
native materialization makes consumer instances, and collected declarations feed
the original argument resolver when no local source header exists. Independent
struct/enum and const-argument runtime and rejection cases pass. General applied
producer signatures, lifetime-bearing symbolic fields, defaults, derives, Drop,
trait predicates, generic bodies and full library/Cargo semantics remain open.

Closed constant records add stable identity, actual types and evaluated values
without producer AST, string/static indexes or function slots. The original native
initializer checks run before production; scalar ranges, aggregate/variant shapes
and promotion qualifiers validate before any imported semantic publication. Fresh
consumer constant tables retain namespace visibility and re-exports. Generic const
bodies, reference/allocation/function values and general type/generic const contexts
remain unqualified, alongside real core/alloc/no_std runtime and native Cargo.

The root Cargo workspace builds the compiler entry in `src/`. It uses
`serde_json`, a bounded native C ABI loader and the separately pinned full Rust
executable. The compatibility coordinator in `crates/lustr` remains independent,
with no registry or C build dependencies and Rust source that forbids unsafe
code. `crates/lustr-native-core` wraps unchanged native modules outside the
protected tree. Owned library additions live in `library/lustr`; explicit links
share original source instead of changing the protected baseline.

The opt-in native ABI now composes the exact upstream modules through owned
declarations, binding its parser and collection interfaces to owned facades.
Affected free functions are parsed from original tokens into actual unsafe-block
nodes. A separate structural safety gate checks boolean literals, initialized
boolean locals, lexical scope and shadowing in nested blocks,
then prepares a clone with unchanged scopes and arena IDs for native checking.
Calls carry pending obligations: collection returns only after original-context
native checking resolves every exact call to an admitted safe Rust function,
with checked boolean inputs/output and actual safety and ABI qualifiers.
Original local function items, including complete `const fn` headers, in affected enclosing blocks pass through the
pinned item parser and arena append API. Native statement-item collection owns
their scope, forward visibility, shadowing, duplicates and separate bodies.
The default ABI retains the original `main.rs` include. No protected source,
generated overlay or parser copy is used. The declared Lust integration diff is
currently empty; future focused shared interfaces require the source policy's
exact reference/patch/result binding, independent review and appropriate evidence.
The [interface record](proofs/native-unsafe-blocks.md) preserves the bounded
grammar, native-expander bridge, first failures and broader semantic frontier.

The optional `native-inference` modules add checked `OsString`/`PathBuf` constraint elaboration,
owned AST arena reindexing, conservative Cargo dependency validation and scoped
scratch cleanup outside `compiler/native`. The feature-gated product worker
loads a separate extension library after native rejection. Original native
success, upstream sources and published preview routing remain unchanged.
This experimental whole-target support is not native Rust rlib or general Cargo
interoperability; its native/runtime/refusal evidence is configuration-specific.
The separate retained ABI adds bounded process-local successful-artifact reuse,
with exact input/output comparisons and fresh compilation after source edits.
It does not retain an elaborated AST or alter the published/product path.
The pinned Linux host-file proof uses owned Rust libc wrappers and a Rust C-unit
observer, without changing protected Lust or TinyCC sources. Actual archive and
child-compiler identities gate eligibility; unknown reads and clock macros
prevent reuse. Fresh optimized cost measurements retain miss-time regressions
and a coordinator no-op slower than stock Cargo. Runtime-loader closure,
product lifecycle and general cache admission remain open obligations.
The owned assertion-text formatter recovers original parenthesis nesting from
source tokens and scratch-AST spans outside the protected trees. Only a bounded
panic-text marker changes before complete native rechecking. Source conditions,
evaluation, borrow/Drop/unwind semantics and original source stay native work.
Its component46 report also qualifies original binary operators, precedence,
lazy RHS/order/once and retained edits. Its [component47 report](measurements/2026-11-15-native-struct-v47/README.md)
adds source-bound struct fields, shorthand, updates and conditional commas,
preserving field order, borrowing, Drop, unwind, exact-payload checks and measured
regressions. The [component49 report](measurements/2026-11-15-native-tuple-literal-v49/README.md)
extends this bounded formatting to original tuple fields and literals, including
exact element/index tokens, structural parentheses and singleton commas. Other
unmodeled grammar, full native language and crate support remain open.

`crates/lustr-semantic` remains independent because its rustc-private interface
requires a different pinned development toolchain. It is not part of the normal
BlueFe application build. The stock GitHub workflows remain at `.github` because
GitHub discovers workflows there; native steps select their component directory.

The native package is named `lustr-native`. Its existing `lust`/`lust-core`
commands and protocol names remain compatibility interfaces. The optional pinned
ts-rust source is a native qualification fixture, not another product entry point.
Upstream licenses and source history remain intact.

Current owned changes include compiler request routing, Cargo target identity,
bounded foreground cleanup, source-library APIs and separately identified
retained library sessions. Compatibility backend selection and its qualification
remain separate. Earlier Lust kernel patches and experiments are historical;
they are not admitted to the active integration. Preserve the pristine pinned
reference plus a small explicit reviewed diff when necessary for shared phase or
retained-state interfaces. Correctness, independent review and appropriate
performance evidence govern admission; unrelated faster-results requirements no
longer block a necessary interface. Substantive missing support remains owned by
Lustr. The source guard binds the exact reference, patch and integrated tree and
rejects unexplained drift. This does not change blueFe's protected Rust policy.

## BlueFe

BlueFe retains exact pinned upstream Rust compiler, library, bootstrap and
stage0 source under its preservation guard. Its owned integration pins Lustr
and records dependency boundaries. The compiler candidate currently ships an
official unmodified SDK, not a newly bootstrapped fork-produced Rust compiler.
That dependency supplies full Rust compatibility while native replacement is
unfinished.

Old compiler/bootstrap overrides and heuristic stage0 selection were removed
from the active fork. Optional Cranelift research does not implement native
Lustr coverage and retains its separate unwind/drop and ABI obligations.

## Iteration policy

Make the working path usable before replacing compiler internals. Every inherited
component needs an identified role, preserved license, version boundary and a
reason to remain. Remove or replace it when equivalent functionality and focused
qualification exist. Keep source movement separate from semantic changes in
review and report any changed build or installation contracts.

Full native Rust support, persistent full-Rust analysis, native rlib publishing,
and migration toward memory-safe Rust remain future goals. TinyCC and LLVM are
current external code-generation dependencies. Safe Rust at a command boundary
does not prove their implementation safe. Track migrations and validate behavior
as they land; do not use the future goal to block a working toolchain.

The same ownership and evidence policy applies when working on the user's other
forks. Public repositories can supply frozen performance workloads without
changing their source, default branches, services or deployment state.

## Historical native retirement

An earlier fork revision removed `compiler/native/spikes`. Exact upstream
preservation subsequently restored the complete tree, including those probes.
They remain historical upstream research, not native language coverage or
current package acceptance. The earlier fork state remains in
[the prior source snapshot](https://github.com/glopyglerky/lustr/tree/39349b63b976e76702ca14fbb5f19718eaf0ed2d/compiler/native/spikes).

Required non-Rust code remains explicit: the current native compiler's C runtime
and generation path, the TinyCC interface, C ABI qualification fixtures, and
build/test/research helpers used by current development. Rust source, licenses
and the relevant compiler fixtures are retained. These exceptions preserve the
working toolchain while the native Rust migration remains future work.

## Matched compiler configuration study

The [CGU16/ThinLTO study](measurements/2026-11-09-cgu16-thinlto-v3/README.md)
compares stage2 compiler packages from the same source and Cargo. It changes
compiler CGU256/off-LTO to CGU16/ThinLTO, retaining std CGU256 and project settings.
Preparation and native build effects belong to the joint treatment. Lustr's
default routing and frontend settings remain unchanged. The numerical decision
passed for three metadata-only roots; stock Rust remained faster on those cases.
Full-build performance and individual knob effects require separate experiments.
The [study proofs](proofs/cgu16-thinlto.md) distinguish deterministic arithmetic
properties from statistical assumptions and compiler-correctness evidence.
