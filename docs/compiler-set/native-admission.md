# Native admission before scalar emission

This candidate implements the first admission boundary required by the
[governing architecture](architecture-plan.md). It separates a represented
native unit from work requiring complementary support **before C generation**,
then preserves actual compiler failures through the mixed driver. It does not
complete the full Lust/Rust boundary or qualify a package release.

## The defect

Previously, the mixed driver treated native status 1 as a reason to leave a
definition with LLVM. The same status covered both an unrepresented runtime
operation and a genuine TinyCC, filesystem, symbol or object failure. Retained
edits also converted several object and relink failures into cache misses.
Successful ordinary compilation could therefore conceal a native implementation
failure.

The old native `check` path returned success before the mixed runtime/ABI
eligibility checks. A passing check did not prove that the selected function
could cross the native object boundary. Calling check and then compiling the
same unit again would not fix that contract.

## One checked invocation

The mixed native request still runs the existing Lust frontend, body checking,
borrow checking and lowering. After those phases succeed,
[`mixed_unit::prepare`](../../crates/lustr-native-core/src/mixed_unit.rs) examines
that invocation's actual `Program`, selected definition, stable slot, signature
and lowered IR. It checks the represented target/profile, scalar ABI and runtime
operations before C generation.

An admitted unit receives a private, non-cloneable `PreparedMixed` value. It
borrows the request and exact checked `Program` and is consumed by emission in
the same driver invocation. No second check/build request is used. A metadata
check returns the explicit checked outcome after admission and generates no C.

The total-operation proof excludes unrepresented calls, panic operations,
aggregate/runtime dependencies and scalar representations absent from the small
C unit. This includes 128-bit integer helpers and unit-valued locals. These are
limitations of this bridge, not claims that Lust itself cannot compile them.
The Rust-facing boundary remains narrower still. Its HIR and actual FnAbi
checks admit represented nongeneric functions with matching integer signatures.
The [integer and module increment](mixed-integer-modules.md) expands this to
32-bit arguments, matching narrow returns and plain functions in loaded inline
modules. Narrow arguments still require adapters, and wider Lust support remains
outside this bridge.

Generated helper closure, actual ELF symbol/relocation checks and object I/O
remain necessary after admission. A failure there is an implementation or
artifact error. It cannot be reported as complementary capability.

## Explicit outcomes

[`protocol.rs`](../../compiler/native/src/protocol.rs) gives mixed responses a
versioned fifth header field. Ordinary native replies keep their four-field
format. The existing compiler build identity continues to separate incompatible
clients and daemons.

| Outcome | Meaning | Mixed driver action |
| --- | --- | --- |
| `mixed-v1:checked` | The metadata request passed native checking and admission | Retain the checked boundary; imported Rust writes real metadata |
| `mixed-v1:emitted` | The admitted definition produced its validated native object | Rename its exact symbol and install it in the mixed output |
| `mixed-v1:complementary` | A required capability is absent from this bridge, established before emission | Keep that definition with the imported Rust implementation in this invocation |
| `mixed-v1:failed` | Frontend, invariant, emitter, object or other compilation failure | Fail the compiler request |

Checked, emitted and complementary outcomes use status zero. Failed outcomes
preserve a nonzero status. Both the protocol reader and mixed caller reject
inconsistent or unclassified replies. An emitted reply must identify the exact
expected unit symbol; a checked reply cannot satisfy an object request. Diagnostic
text does not determine any capability decision.

The strict mixed transport may start a daemon when no connection accepted the
request, or when a different build explicitly rejected it before compilation.
Once request writing begins, a write/read/protocol failure is an error, not a
reason to replay an ambiguously completed compile. The mixed daemon also returns
panics and invariant failures without recompiling the request in a fresh session.
Existing ordinary native daemon policy is separate and unchanged.

Explicit configuration disabling the native daemon remains a separate policy
decision before a native request. A startup, transport or compiler failure is
never interpreted as that configuration choice.

## Retained state and errors

Native metadata checks retain a boundary proof bound to the selected definition,
body and signature identities, behind the existing native read-set validation.
Unchanged checks may reuse a previously classified result. An edited or
renumbered body must establish the current proof; a generic successful check
response cannot substitute for admission.

The mixed certificate schema is now **6**. Version 5 introduced the required
admission proof; version 6 also invalidates earlier coverage certificates before
the integer/module expansion and definition-based candidate lookup. Earlier
certificates are not accepted. Native and imported Rust build identities remain
part of the cache identity.

[`mixed_cache.rs`](../../crates/lustr-semantic/src/mixed_cache.rs) validates the
complete retained certificate and required native object paths before starting an edit.
A missing or stale cache entry can still request normal analysis in the same
invocation. Once native work starts, source-write, emission, rename, sync and
relink errors are fatal. Cold mixed compilation applies the same rule. The last
successful certificate and its object remain immutable until a new complete
compilation succeeds.

Unchanged supported checks do not gain an additional frontend pass or per-unit
filesystem validation. The ordinary Lust NoChange shortcut remains in place.
This change does not claim a measured latency reduction.

## Observable qualification

The opt-in mixed profile uses schema **3** to record the actual native outcome.
A complementary result has no emitter record because no object was generated.
An emitted result still requires exactly one complete successful emitter record;
a failed response cannot invent one. The validator retains historical schemas
1 and 2 with their original meanings.

The [local verification record](evidence/native-admission-20261118/verification.json)
binds the results to native build **`baea78ae8589c84b`**, the exact changed source
and the matching imported Rust SDK. All compiler and test children completed.

| Validation | Observed result | Scope |
| --- | --- | --- |
| Native focused filter | 3 client and 11 core tests passed | Direct retained `Session` checking/emission, plus separately identified wire and daemon-guard controls; protocol tests run in both binaries |
| Native-to-Rust handoff | 12 scalar value checks passed across cold, unchanged and edited native objects | Actual Rust callers linked and executed each of the three native objects; no daemon socket |
| Imported Rust driver | Full mixed-driver binary built; typed-response validation test passed | Accepts only classified, consistent outcomes and exact expected symbols |
| Complementary Rust control | Both compilers ran the valid program with output `6 1`; both rejected invalid code with E0308 and no metadata artifact | Actual generic/trait, Vec, Drop and async source; trace confirms zero native definitions, so this is not a mixed-participation claim |
| Profile and qualifier | 36 synthetic profile controls passed; updated real qualifier compiled | New retained failure/recovery operations await hosted execution |
| Source and release guards | Exact candidate identity passed; release admission refused as expected | Pristine Lust pin unchanged; integration status remains candidate |

The existing hosted results at
[`429fe605`](hosted-qualification-2026-11-18.md) predate this boundary and do not
qualify it. Real retained emitter-I/O and link-recipe failure/recovery cases are
part of the mixed qualification harness; they require the hosted daemon
environment. A test merely accepting an unclassified native failure would not
qualify this change. No new compiler performance measurements were taken.

## Remaining architecture work

Cold mixed compilation still performs whole-crate Rust analysis, and the native
bridge still represents only scalar units. The standalone public route still
has the previously documented whole-program retry. This change does not turn
either limitation into the target architecture.

The next boundary work must carry the rest of Lust's supported semantics,
runtime/ABI requirements and dependency ownership into the integrated compiler,
using imported upstream Rust services for the complement. Preserve Lust's
frontend and retained path as that coverage expands. The new structured outcome
must not become a renamed failed compile or a language selector.
