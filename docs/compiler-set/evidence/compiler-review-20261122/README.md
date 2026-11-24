# Compiler review and profile correction, 22 November 2026

The audit keeps the retained Lust compiler and the scalar artifact work in
PR #33. It found a profile-semantics defect and a construction-time SDK loader
defect, corrected their request/probe boundaries, and added independent runtime
and DWARF regression gates.
This record qualifies only the source, packages and cases identified below.
It does not qualify a full Rust replacement or a compiler speedup.

The [architecture plan](../../architecture-plan.md) explains the next work.
The [completion checklist](../../completion-checklist.md) remains the sole
work register. In particular, a general mixed semantic-service proof starts
alongside ordinary Cargo/debug completion, before another broad scalar expansion.

## Baseline and preservation

| Identity | Value |
| --- | --- |
| Reviewed PR | [#33](https://github.com/glopyglerky/lustr/pull/33), open and unmerged |
| Published baseline | `514dfe1ef8b49160bf5c09458981768fbf4f2386` |
| Baseline source tree | `c2a11fa58ac5abef4e8da8247c8d6495ab7fe200` |
| Native integration tree | `296c3bb8d62d0b383e37f5e84416b9398af58c2e` |
| Pristine Lust commit | `240e08d91d4aed0299639f6f11fd522912b248f9` |
| Pristine Lust tree | `0ab1b09815802894e6078de0af59273cbdf46cb0` |
| Declared Rust source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Rust distribution | `nightly-2026-11-09`, Linux GNU x86-64 |

The pristine donor, declared native tree, complete integration patch and
qualification hashes agree. This repair does not change `compiler/native` or
remove donor functionality. The structural source/feature guards pass and still
refuse full-compiler release admission. Structural checks are distinct from
semantic coverage: they do not prove that every Rust program is accepted or
that every accepted program is correct.

The original hosted public campaign was independently downloaded and its ZIP
digest verified. [Run 45073103551, job 137364264163](https://github.com/glopyglerky/lustr/actions/runs/45073103551/job/137364264163)
used merge checkout `2e965671147d81f2df62a20895d5fba74fcf7631`, whose source tree
matches the baseline. Artifact [14774221657](https://github.com/glopyglerky/lustr/actions/runs/45073103551/artifacts/14774221657)
has SHA-256 `72153b89382c264d0f2b2ed13d3faa4f7d14fa2ae2b31698c6abda3e8ba2e1cf`.
Its actual report, archive outputs, independent consumers and histories were
inspected. The unwind-library history compiled `[3, 0, 1, 0, 0]` objects for
cold, no-op, edit, invalid and repair requests, with statuses
`[0, 0, 0, 101, 0]`. Metadata-only default executable checks also passed.

At the final baseline check, 19 of 21 checks had succeeded; the broad Linux
and macOS native gates were still running. Those pending checks are not counted
as successful and do not establish the repair's hosted result.

## Defect: omitted overflow checks ignored effective debug assertions

Pinned Rust's [`Session::overflow_checks`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_session/src/session.rs)
uses an explicit overflow setting when supplied, otherwise the effective
debug-assertion setting. Lustr's Cargo adapter independently defaulted both
settings to `on`; its direct adapter forwarded the debug override without
resolving the implied overflow setting. A successfully compiled valid request
could therefore abort where the pinned compiler wraps and exits successfully.
The Cargo defaulting code predates the unwind commit: Git attributes it to
`2749c2ff`, not `514dfe1e`.

The independent evaluator contains 24 paired cases: eight direct flag
combinations and four actual Cargo combinations, each exercising runtime
overflow and a debug assertion. All use O0, `panic=abort` and disabled debug
information. Cargo chooses the actual compiler request; explicit final flags
use `cargo rustc`. Compilation must succeed before runtime can pass. Expected
panics require the abort signal and the expected panic text; an arbitrary crash
or compiler error cannot satisfy them. The evaluator checks executed compiler
arguments, explicit overrides, direct last-option precedence and process traces.

| Baseline case | Pinned Rust result | Lustr result |
| --- | --- | --- |
| Direct debug assertions off; overflow omitted | `wrapped=0`, exit 0 | Overflow panic, abort |
| Direct repeated debug flags ending off; overflow omitted | `wrapped=0`, exit 0 | Overflow panic, abort |
| Actual Cargo request with debug assertions off; overflow omitted | `wrapped=0`, exit 0 | Overflow panic, abort |

[profile-before.json](profile-before.json) records all 24 cases, source and
tool identities, compile/runtime statuses and output hashes. The reference
passed 24/24; the original Lustr package passed 21/24. Execution traces found
no undeclared compiler fallback. The exact original report SHA-256 is
`1b6241196b29f32fe5a955cc006d395e4dfa463cbd9b18dad0bcf621cb850ef2`.

The correction resolves omitted overflow checks at the Rust request boundary.
Explicit settings remain authoritative, and direct duplicate flags keep their
last-option behavior. The Cargo artifact adapter retains its existing refusal
of duplicate codegen options. The native Cargo profile parser is unchanged.

Qualified repair source: local commit
`0cc0149f4b29e65fe40adcf9f59e42b7b11f23dd`, tree
`af1ffe2bcb1d67c6288ad2fddf87238271af5b7a`. Published commit
[`9185df0d`](https://github.com/glopyglerky/lustr/commit/9185df0d2d284d08db4fb015f40c67667a6e398c)
has exactly the same tree; the later audit documentation does not change the
compiled product or qualification evaluators.
The native build identity includes public adapter inputs, so qualification uses
a complete constructor rebuild rather than a replaced entry with stale native
receipts. [construction.json](construction.json) records that successful build,
the unchanged original package and native identity `df6faee19ddf9dd0`.

| Qualification | Original baseline | Repaired package |
| --- | --- | --- |
| Profile cases, unchanged evaluator | 21/24 | 24/24 |
| Independent pinned Rust profile cases | 24/24 | 24/24 |
| Existing public campaign, unchanged evaluators | Pass | Pass |
| Public campaign's total original consumers / runtime assertions | 16 / 4,194,432 | 16 / 4,194,432 |
| Unwind objects: cold / no-op / body edit | 3 / 0 / 1 | 3 / 0 / 1 |
| New frame gate on the repaired campaign | Separate baseline checks below | 6 positive walks; 2 rejected corruption controls |

All three previously failing profile requests now print `wrapped=0` and exit
successfully, matching the independent compiler. No undeclared compiler fallback
was observed. [profile-after.json](profile-after.json),
[public-campaigns.json](public-campaigns.json) and
[verification.json](verification.json) retain their source and evaluator bindings.
The raw corrected profile report SHA-256 is
`80106528f178cd95529f63e1d22ad20e8424c5654179a95ff2e61631bd109fb3`;
the corrected public report is
`bf5692f4e7f777418ed37cd95783591d614610c7d014b5c19f20c22823f8e193`.
Five focused Rust unit tests pass. The post-campaign preservation check reverified
13,122 source files and one gitlink, a clean checkout, and unchanged original
and corrected package inventories. These are executed source-bound results;
the new PR head's hosted workflow result is a separate observation.

This evaluator uses fresh source identities. It is cold profile correctness
evidence, not qualification of profile changes within a retained source
history, ordinary default Cargo builds or performance.

## Defect: Cargo's loader paths broke the pinned SDK identity query

A normal full construction exposed a second defect after native compilation.
The root build script's nested `rustc -vV` query inherited Cargo's
`LD_LIBRARY_PATH`. With the observed rustc-dev SDK layout, that selected an
identical driver copy under the target library directory. Its loader origin
could not find the pinned LLVM library in the SDK's main library directory.
The process exited 127 and construction stopped.

[sdk-identity.json](sdk-identity.json) retains the paired reproduction and two
SDK layout observations, with private absolute roots replaced by named tokens.
Compiler, Cargo, driver and LLVM hashes match the declared pins. With Cargo's
child loader path, the pinned compiler fails; with that variable removed from
the same child invocation, the unchanged SDK returns the exact pinned source
identity successfully. An independent reviewer reproduced the same pair.

The repair clears `LD_LIBRARY_PATH` only for the root build script's SDK
identity subprocess and reports its actual stderr if it fails. This matches
the existing constructor and native-core SDK queries. It does not modify the
SDK, the compiler hash checks, native policy receipts or product runtime
loading. The complete constructor passed against the same untouched SDK, including
the formerly failing entry build and the remaining native library/SDK assembly.
The earlier failed construction is retained as failed evidence, not relabeled
as a semantic test result. The first attempt's missing offline dependency cache
was a separate setup failure, resolved using the existing dependency cache.

## DWARF must drive the observed frame walk

The new frame gate consumes a successful public campaign. It verifies the
native archive members, relocated function bytes and linked frame records,
then walks `limit` to `project` to the original Rust `main`. Cold and edited
artifacts each stop at entry, after the prologue and at the return instruction.
GDB's selected-unwinder trace must bind `dwarf2` to both native frame levels and
their actual program counters. A correct-looking frame-pointer backtrace is
insufficient.

For each generation, a private executable copy receives one incorrect
`DW_CFA_def_cfa_offset` operand. Its ordinary runtime result, code bytes and
frame-record count remain unchanged; the same frame-sequence predicate must
reject its corrupted walk. Missing debugger support, inaccessible execution or
an unfamiliar trace format fails qualification rather than passing a negative
control.

[unwind-baseline.json](unwind-baseline.json) records these six positive walks
and two corruption controls per observer version. GDB 17.2, Debian GDB 16.3
and Ubuntu 24.04's GDB 15.0.50 pass the same frozen evaluator. Eleven evaluator
controls reject fallback unwinders, rejected DWARF attempts, incorrect caller
addresses, wrong frame levels and incomplete traces. These are scalar frame
walks. Rust source line/type/local debugging, panic propagation, cleanup and
catch behavior remain unqualified.

[unwind-after.json](unwind-after.json) records the same gate on the freshly
constructed repair, using GDB 17.2. All six walks and both controls pass.
Its raw report SHA-256 is
`40105374e89d304df99768e1ce0ccb8e4f0936e8e2ec0bd40d9fb0def8396aaf`.
All 25 remote outputs were rehashed; the 22 downloaded report/log/script files
were also checked locally. Compiler and original campaign binaries remain unchanged.

## Repeat and retain the evidence

The existing Linux compiler-set workflow now runs both gates against its
newly constructed package and retains diagnostics even when they fail. It
checks evaluator hashes before and after the campaign. Its tracked source
inventory now includes the public entry, build inputs and new evaluators as
well as the native/metadata sources.

With a normally constructed package and the pinned Linux SDK:

```sh
python3 tools/test-rustc-profile-defaults.py --self-test
python3 tools/test-native-unwind-frames-controls.py -v
python3 tools/test-public-native-library-controls.py -v
python3 tools/test-public-native-library.py \
  --package "$PACKAGE" --sdk "$SDK" --output "$RESULTS/public"
python3 tools/test-rustc-profile-defaults.py \
  --package "$PACKAGE" --sdk "$SDK" --output "$RESULTS/profiles"
python3 tools/test-native-unwind-frames.py \
  --artifact-dir "$RESULTS/public" --gdb "$(command -v gdb)" \
  --output "$RESULTS/unwind"
```

Each output directory must be new. The profile evaluator requires Linux and
`strace`; the frame evaluator requires GDB with Python and its documented named
frame-unwinder trace. Complete original reports and command output were retained
by the executing environment; committed JSON files are explicitly identified
summaries with original digests, not substitutes claiming identical raw bytes.
Public summaries omit private host paths.

## Remaining limits

Default-profile executable linking, Rust source debugging, general panic
cleanup, wider native artifact/ABI coverage, full semantic composition and the
full host/target/profile matrix remain open. No timing campaign in this review
establishes an overall speedup. Zero or one rebuilt object proves selective
work reuse for its history, not elapsed compiler or compile-plus-test speed.
The package remains partial and release admission remains unavailable.
