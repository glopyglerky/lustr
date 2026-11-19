# Target Unicode semantics

The native compiler's Unicode behavior follows imported Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. It does not follow the Rust compiler
used to build the native implementation.

| Surface | Declared data |
| --- | --- |
| Character predicates, case conversion and Debug escaping | The pinned Rust standard library, Unicode 18.0.0 |
| Identifier membership | The pinned compiler's `unicode-ident` 1.0.24, Unicode 17.0.0 |
| Identifier normalization data | The pinned compiler's `unicode-normalization` 0.1.25, Unicode 17.0.0 |

These versions deliberately differ, exactly as they do in the declared Rust
source. U+0558 and U+A7DD have the new standard-library properties while the
compiler still rejects them as identifiers. The native lexer retains its
existing policy requiring NFC input; these data checks do not claim that policy
matches Rust's automatic normalization of all identifiers.

## Generation

`scripts/gen-unicode.sh` checks the selected SDK's complete source commit against
`upstream/bluefe-core.pin`. `tools/gen_unicode_semantics.rs` collects every valid
Unicode scalar from that SDK once and emits both `src/unicode_data.rs` and the
three C runtime tables. Predicate membership, case maps, Debug escaping and the
effective final-sigma properties all come from that same SDK. The compiler's
version constants, constant evaluator and synthesized literal escaping read the
checked-in Rust data. Ordinary builds do not regenerate it.

```sh
RUSTC=/path/to/pinned-sdk/bin/rustc bash compiler/native/scripts/gen-unicode.sh
RUSTC=/path/to/pinned-sdk/bin/rustc bash compiler/native/scripts/gen-unicode.sh --check
```

The check mode generates into a temporary directory and compares every output
byte. A different SDK fails before any data file is written. The earlier
upstream generators remain preserved source; they are not the Lustr generation
entry. `unicode-semantics.json` records file digests and the pinned compiler's
dependency identities. Unicode data is covered by `LICENSE-UNICODE`.

`scripts/gen-xid.py` retains Unicode 17 UCD inputs for the compiler dependencies.
The independent verifier checks its resulting data against the actual pinned
dependencies, including every scalar's canonical decomposition and every
composition pair.

## Qualification

```sh
python3 tools/qualify-native-unicode.py \
  --native-core /path/to/source-built/lust-core \
  --rustc /path/to/pinned-sdk/bin/rustc \
  --output /path/to/new-evidence-directory
```

The SDK needs `rustc-dev` for the compiler-dependency checks. The qualification
checks all 1,112,064 Unicode scalars for compiler predicate/escape behavior and
identifier/normalization data. It then compiles the same runtime program with
native and matching Rust, compares every output byte for seven predicates,
case mapping, char/string Debug and four final-sigma contexts, and runs the
constant/runtime regression. A hash identifies the preserved output; equality
is established by direct byte comparison.

The native test `unicode_semantics_survive_retained_body_and_constant_edits`
also checks cold, unchanged, body-edited, constant-edited and reverted builds in
one real retained compiler session. This test requires no daemon socket.

These checks cover the named Unicode operations. They do not prove the whole
compiler correct, qualify every host, or admit an installable release. Frozen
baselines retain their original Unicode disagreement and Rust 1.99 oracle.
