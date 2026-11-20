# Native metadata join qualification

This standalone research executable consumes one actual native checked unit,
completes its declaration and emitted-instance records, invokes the selective
metadata encoder and constructs a Rust library archive. The candidate invokes
Clang for the existing native C emitter and the pinned LLVM object/archive
utilities. It does not invoke or link a full Rust compiler.
The captured Clang command disables implicit configuration and explicitly uses
no stack protector for the represented request. Windows control-flow guard is
disabled in this Linux target context. This Clang research emitter is separate
from stock Lust's TinyCC path and establishes no equivalent-speed claim.

The fixed scope is a flat ASCII/LF `no_std` library of ordinary safe nongeneric
`u64` functions, local calls and represented scalar bodies. It uses edition 2024,
Linux x86-64, optimization zero, no debug information and abort panic behavior.
No caller option is rewritten to bypass an obligation. The public compiler,
retained sessions, general library metadata and the wider complement remain
unintegrated. This is not a compiler performance benchmark.

`tools/test-native-consumer-metadata.py` supplies a fixed independent source and
requires actual process tracing. After the candidate publishes its artifact,
the pinned original Rust compiler reads its metadata and compiles an ordinary
Rust caller using `--extern consumer=...`. The caller uses both direct calls and
Rust function values. No C declarations or direct object linker arguments can
stand in for crate metadata interoperability.

The campaign checks 262,152 explicit runtime assertions, public/private access,
signature/arity failures, a rejected private body, changed receipt/version
contexts and artifact preservation after publication failure. It also corrupts
actual completed/emitted mappings and creates actual missing/duplicate-member
archives. Each corruption must be rejected by the consuming implementation.
Successful oracle commands require empty stderr. Default mitigation records
must allow the ordinary caller while a stronger stack-protector request remains
incompatible. No warning is suppressed.

The former uncommitted campaign failed when the encoder omitted upstream's
disabled mitigation records. The current source restores the pinned original
`StackProtector(None)` and `ControlFlowGuard(Enabled(false))` records. A disabled
stack protector and an absent record have different meanings to the original
consumer. The new caller also binds the SDK receipt on the single read used to
construct its immutable facts.

`source-slices.json` identifies the original metadata object-wrapper branches,
their target assumptions, licenses and adapted files. Encoder source provenance
remains in its own inventory. Runtime generation facts stay native-owned; no
second parser, checker, MIR builder or complete driver is imported.

The Compiler set workflow builds and executes this campaign separately. A
source change, authored test, old failed campaign or encoder-only test does not
qualify this joined source. Read the exact workflow result and retained reports
before reporting a pass. `--functional-only` is an explicit local diagnostic
mode, leaves trace qualification false, and is not used by required CI.
