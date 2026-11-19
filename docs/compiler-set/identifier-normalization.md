# Native identifier normalization

Status: the native lexer now accepts valid non-NFC identifier spellings and
interns their canonical NFC names using the declared Rust compiler's upstream
implementation. Source-bound local qualification passed on x86_64 Linux. The
[verification record](evidence/identifier-normalization-20261119/verification.json)
identifies the source, binaries, dependency archives and observed limits. This
closes the native NFC gap recorded by the corrected architecture checklist;
it does not complete the mixed scalar bridge or the general compiler boundary.

## Exact upstream implementation

The declared Rust source is
[`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`](https://github.com/rust-lang/rust/tree/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494).
Its [lexer](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_parse/src/lexer/mod.rs)
uses `is_nfc_quick` and `UnicodeNormalization::nfc` from
`unicode-normalization`. Native identifier interning now calls those same
upstream APIs. The former independent normalization algorithm and the `L0002`
rejection of valid decomposed names have been removed.

Both native Cargo lockfiles retain the exact dependency closure from the
declared Rust source's lockfile:

| Dependency | Version | Registry archive SHA256 |
| --- | --- | --- |
| `unicode-normalization` | 0.1.25 | `5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8` |
| `tinyvec` | 1.10.0 | `bfa5fdc3bce6191a1dbc8c02d5c8bffcf557bafa17c124c5264a458f1b0613fa` |
| `tinyvec_macros` | 0.1.1 | `1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20` |

The locked registry packages supply the actual upstream source implementation.
Original license and copyright notices are retained under
[`compiler/native/licenses/identifier-normalization`](../../compiler/native/licenses/identifier-normalization).
The [provenance manifest](../../compiler/native/unicode-semantics.json) records
the archive and upstream commit identities. Both lockfiles contribute to the
native build identity, so changing the resolved implementation invalidates old
artifacts. The Unicode qualifier checks their versions, registry sources and
checksums against the exact pinned Rust lockfile before compiling fixtures.

This normalization implementation uses **Unicode 17**, as do the pinned Rust
compiler's identifier tables. The pinned standard library uses **Unicode 18**.
These are separate upstream data families; changing the lexical tables to match
the standard-library version would introduce a different incompatibility. The
[Unicode data repair](unicode-semantics.md) remains in force.

## Names, source text and raw tokens

Only identifier and lifetime names normalize before interning. Original byte
spans remain unchanged, and string, raw-string, character and other literal
contents retain their written bytes. Canonically equivalent identifiers resolve
to the same symbol in constants, functions, modules, paths and macros. Duplicate
canonical definitions therefore reach ordinary duplicate-definition rejection.

Identifier and lifetime tokens now carry an explicit raw marker. A decomposed
ordinary name can have the same byte length as a raw precomposed name; source
length cannot distinguish them. Packed token streams, macro matching and
token-based cache fingerprints preserve the marker independently of canonical
name and span. Existing syntax checks that compare names continue to compare
their canonical symbols.

The raw marker has two observable printing rules at the declared Rust pin:

| Context | Written input | Observed text |
| --- | --- | --- |
| `stringify!` token | `r#café` | `r#café` |
| Parsed name in an assertion condition | `r#café == 2` | `café == 2` |
| Raw keyword in an assertion condition | `r#type == 2` | `r#type == 2` |
| Raw name inside a macro argument | `has!(r#café)` | `has!(r#café)` |
| String literal inside a macro argument | `has!("é")` | `has!("é")` |

The native and native-core assertion printers follow that distinction. Parsed
AST names omit unnecessary raw prefixes, matching the pinned Rust
[AST printer](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_ast_pretty/src/pprust/state.rs)
and [identifier print-mode rules](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_span/src/symbol.rs).
Nested macro tokens keep their explicit raw marker. Literal spelling remains
unchanged in both contexts.

## Failure first and final qualification

The preserved preceding compiler rejects the valid constant/runtime and macro
fixtures with `L0002`; matching Rust accepts and executes them. After the first
normalization repair, a further runtime witness found an unnecessary `r#` in an
assertion panic payload. That mismatch was preserved and repaired before the
final qualification.

| Check | Result |
| --- | --- |
| Cold native versus exact Rust | Six NFC programs match byte for byte, including constants, raw names, lifetime labels, combining-mark reordering, Hangul, canonical singleton equivalence, macros, included tokens and module paths |
| Assertion panic payloads | All 11 observed payloads match, including raw Unicode names, required raw keywords, field/method/constructor names, macro tokens, literal bytes and labels |
| Canonically duplicate functions | Native rejects with `L0106`, Rust with `E0428`; neither writes an artifact |
| Original spans and packed tokens | Fifteen spellings retain their original byte spans and explicit rawness through packing and unpacking |
| One retained native session | Cold, unchanged, decomposed body, equal-byte-length rawness edit, decomposed declarations and reverted source execute the expected results; an invalid duplicate edit preserves the exact preceding executable |
| Native-core assertion printer | Seven parsed-condition controls pass; the same included native retained and packed-token tests pass |
| Existing lexer/macro corpus before the final assertion-text repair | 184 comparisons match Rust; one known reference-limit outcome is retained separately below |
| Existing assertion/stringify/raw-identifier fixtures after the final repair | Nine comparisons match exact Rust |
| Full Unicode qualifier after the repair | All 1,112,064 scalar data checks and 961 composition-pair checks pass; actual compiled runtime streams contain 114,666,345 equal bytes |
| Native build compatibility | Rust 1.99 checks both native binaries with the locked dependency closure; this is a compilation check, not a full Rust 1.99 runtime qualification |

The retained test changes `caféé` into `r#caféé`: their source byte lengths are
equal, but macro arm selection and `stringify!` output differ. It then changes
declaration spellings, rejects a canonical duplicate, and reverts to the original
program. This exercises observable reuse behavior instead of checking only a
hash implementation.

For the existing `macro_doubling_fuzz.rs` rejection fixture, both the preceding
native compiler and the NFC candidate reject with `L0801`. The exact Rust
reference emits syntax diagnostics and is then killed by signal 9. That fixture
already documents its reference expansion limit and requests a five-second
timeout; the private bounded comparison used a sixty-second subprocess limit.
Its preserved outcome is **184 matches plus one reference-limit result**, not
185 completed Rust comparisons. The nine final assertion-focused comparisons
use the final binary and are reported separately.

## Remaining scope

The [mixed scalar projection](mixed-integer-modules.md) still retains its
separately qualified canonical-source-spelling admission rule. The native lexer
can now normalize decomposed names, but the projection's retained definition and
source identity behavior for those spellings has not been qualified. Such
definitions remain with imported Rust in that bridge; this restriction is not
attributed to the repaired lexer.

A combined source in which an `include!` introduces a macro before an
out-of-line module also exposed an existing `L0809` macro-scope restriction.
The preserved failed fixture is retained; the independent included-token and
out-of-line-module controls pass. This change does not claim that combined case.

Qualification here used direct cold compilation and retained in-process
sessions on x86_64 Linux, without a daemon, retry or fallback. The finite-domain
Unicode checks cover the named scalar operations. They do not prove arbitrary
normalization strings or whole Rust programs correct, establish macOS
qualification, complete imported semantic services, or admit a release package.

The checked-in entry point is:

```sh
python3 tools/qualify-native-unicode.py \
  --native-core /path/to/lust-core \
  --rustc /path/to/pinned-sdk/bin/rustc \
  --output /path/to/new-evidence-directory
```

Run the retained test `nfc_identifiers_survive_retained_edits_and_rejected_duplicates`
and the token test `normalized_identifier_tokens_preserve_spans_rawness_and_packing`
in the same source-built native test executable. The native-core build also
contains `nfc_ast_names_and_macro_tokens_keep_distinct_printing_rules`.
