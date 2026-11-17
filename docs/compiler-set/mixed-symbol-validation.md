# Mixed scalar object validation without a symbol-tool process

The mixed scalar emitter now reads its TinyCC object in process before the
existing rename and file/directory synchronization. It checks every symbol for
unrepresented imports and requires the exact expected function export. The
ordinary Lust pipeline, compiler selection, retries, retained publication,
failed-link recovery and full Rust behavior are unchanged.

The [last qualified profile](../../performance/2026-11-17-5a3f493-profile.md)
recorded a median 31.857 ms in the complete `nm` process call across 35 native
responses. Its body-edit request recorded 34.545 ms there within a 37.681 ms
native request. Those observations identify a costly operation; they do not
establish why that child call was slow. The replacement removes that process
while retaining an actual object check. Full compiler improvement requires a
fresh source-built correctness and paired benchmark campaign.

## Accepted object contract

The [owned reader](../../crates/lustr-native-core/src/mixed_symbols.rs) is safe
Rust using the standard library. It introduces no external dependency. Its
scope matches the mixed emitter's existing Linux x86_64 target restriction:
ELF64, little endian, relocatable, without program headers. It is not a generic
object reader or a replacement for semantic and ABI qualification.

Inputs are bounded to 64 MiB, 4,096 sections, 65,536 symbols, 4,096 bytes per
name and 4,096-byte section alignment. File reads remain bounded if an input
grows. Missing, nonregular and symlink object paths fail. Checked slices and
arithmetic guard all offsets before indexing.

The reader requires one complete symbol table, its valid string table, a zero
null symbol, consistent local-symbol ordering and valid section references.
Every other undefined symbol fails, including strong, weak, local, unknown-bind
and empty-name imports. Local symbols are examined before binding filters.
This closes a mismatch in the retained experimental reader, which skipped
local undefined symbols before checking their section. That separate research
reader is not reused or changed here.

The expected export must appear exactly once as a strong, default-visible
function occupying bytes in an executable section. Functions cannot point
outside their sections. Symbol and section names must terminate within the
bound. Tables and file-backed sections cannot overlap. Relocations must point
within their actual target section and valid defined symbols, excluding null
and file-metadata symbols.

The accepted section kinds are PROGBITS, SYMTAB, STRTAB, RELA and NOBITS,
alongside the required null section. Relocations are the scalar TinyCC forms
R_X86_64_64, PC32, PLT32, 32 and 32S. Extended section indexes, TLS, IFUNC,
dynamic symbol tables, unknown kinds, and absolute/common non-file definitions
fail closed. GNU nm can report no imports for some such objects; that alone
does not qualify them for this scalar boundary. Source requiring broader
semantics retains the existing full compiler path.

## Build identity and profiling

The native build helper hashes the new reader's complete bytes, with a domain
label, into `LUST_BUILD_HASH` and emits Cargo's source-change notification.
Missing reader source is a build failure. A production-helper regression
changes those bytes, checks that the identity changes, restores the original
identity and verifies the missing-source failure. The pristine Lust pin is
unchanged; the declared integration patch records the build-helper addition.

Emitter profile version 2 calls the operation `symbol-check`. Historical
version 1 records retain their `nm` name and original child-process meaning.
The strict reporter accepts each version's exact phases, rejects relabeling and
rejects mixing implementations in a campaign. The `5a3f493` original summary
and Markdown replay byte-for-byte. For the older `dec0ce2` archive, this change
matches the previous reporter's compatibility projection byte-for-byte; that
projection already added explicit unavailable-detail fields to its original
version 1 report. Failed responses still carry no timing decoration, preserving the
daemon's deterministic warm/fresh diagnostic comparison.

## Focused qualification and limits

[The executable tests](../../tools/test-mixed-symbols.py) compile actual TinyCC
objects with the production `-w -fno-asynchronous-unwind-tables` options and
compare them with GNU nm. Closed code/data objects pass both checks. Strong
and weak function/data imports fail both. Mutations cover local and empty-name
imports, bad bindings/kinds, missing/weak/nonfunction exports, truncation,
invalid or overlapping tables, symbol ranges, invalid relocation references,
unsupported absolute/common definitions and nonregular/oversized files.

The local run passed eleven focused tests, twenty-six profiling tests, two
component-replay tests and all seven shared source-guard tests. The exact
declared candidate integration check passed. Native `cargo check --locked
--offline` passed with the matching nightly and existing inherited warnings.
These are source/component checks, not a completed packaged-runtime or native
differential gate.

The [retained local component experiment](../../performance/components/2026-11-17-symbol-check/README.md)
contains 64 matched pairs on one identical TinyCC object. It measures only the
old `nm` invocation and the new in-process read/check. All 128 values remain
available. It does not establish a full compiler speedup, native Lust parity,
mixed Cargo speed acceptance or an explanation of the older hosted delay.

Run the focused checks after obtaining the pinned TinyCC build:

```sh
python3 tools/test-mixed-symbols.py --tcc /absolute/tcc/bin/tcc
python3 tools/test-mixed-profile.py
python3 tools/test-mixed-symbol-measurement.py
python3 tools/replay-mixed-symbol-measurement.py performance/components/2026-11-17-symbol-check
```

Before admission, execute the original full native gate and the complete
source-built package contract, including mixed libraries, invalid source,
failed-link recovery and all matched native/Cargo timing modes. Keep all new
results attributed to their exact source. The earlier passing campaigns do
not qualify this change.
