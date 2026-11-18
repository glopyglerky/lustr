# Unicode semantics across the native and Rust paths

Status: an unresolved semantic boundary, not a performance regression. The
standalone native compiler built by the current complete-package constructor
combines Unicode 17 runtime tables with Unicode 18 constant evaluation. The
current scalar admission work excludes Unicode standard-library operations;
that boundary does not repair the standalone native route.

This audit describes published source
`429fe6054e6119221e598b95c2661665ecdd7be2` and the pinned imported Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. Its relevant native Unicode files are
unchanged from the frozen complete-package baseline
`3c378a11d60149a3ec99fa49ea1b336e9406fca8`. The
[sanitized source evidence](evidence/unicode-semantics-20261118/summary.json)
records exact Git blob and SHA256 identities, source-derived differences and
the limits of the initial source-only analysis. A later bounded compiled-program
witness, recorded separately below, confirms two of those predicted predicate
disagreements in the preserved native gate binary.

## Three identities that currently disagree

| Component | Declared or actual source | Unicode behavior |
| --- | --- | --- |
| Native upstream contract | The native README declares Rust 1.99, edition 2024; its CI selects Rust 1.99.0. | Unicode 17. |
| Checked-in native runtime | Its predicate table records generation by Rust 1.99.0, and its C runtime calls that table. | Unicode 17, independent of the compiler build host. |
| Native compiler build and imported full Rust | The complete constructor uses the pinned Rust 1.102 nightly SDK to compile native components; the full Rust source is `5b7b37…`. | The host's `char` implementation and the full Rust standard library use Unicode 18. |

The primary sources are the native
[README](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/README.md#L7),
[native CI](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/.github/workflows/ci.yml#L28),
[runtime table provenance](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/runtime/unicode_table.h#L1),
[constructor SDK selection](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/tools/build-mixed-toolchain.rs#L114),
[full Rust pin](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/upstream/bluefe-core.pin#L1),
and the pinned Rust
[Unicode version constant](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/unicode/unicode_data.rs#L150).

The constructor explicitly gives Cargo the selected SDK's `RUSTC` and uses
that helper for the native client/core build. This build-host choice reaches
program semantics because the native compiler reads host `char` behavior while
evaluating user source.
An independent native CI build using Rust 1.99 does not acquire Unicode 18
merely from the same repository revision; this finding is conditioned on the
complete package's pinned Rust 1.102 build host.

## Hosted version observation

The hosted baseline qualification in
[run 44363351381](https://github.com/glopyglerky/lustr/actions/runs/44363351381)
used the correct Rust 1.99 oracle. Its ordinary differential corpus reported
5,526 passes and one failure: `unicode_props` printed `(18, 0, 0)` for
`char::UNICODE_VERSION` in the native program, while Rust printed `(17, 0, 0)`.
That is an actual compiled-program observation. The
[complete failure disposition](complete-native-warm-comparison.md#third-hosted-failure-build-host-unicode-identity)
retains the exact source, package, artifact and command identities. It admitted
no performance samples.

## Bounded compiled-program witness

On 18 November 2026, one valid Rust program was built and run once with each of
the preserved native gate compiler and the matching Rust SDK. Both predicates
are valid in constant expressions: pinned Rust declares
[`is_lowercase`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/char/methods.rs#L904)
and
[`is_uppercase`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/char/methods.rs#L980)
as `pub const fn`, const-stable since Rust 1.84. The fixture defines:

```rust
const LOWER_CONST: bool = '\u{558}'.is_lowercase();
const UPPER_CONST: bool = '\u{a7dd}'.is_uppercase();
```

It also evaluates the same two predicates on characters supplied as command-line
arguments after compilation. This keeps the runtime inputs separate from the
source constants. The matching Rust compiler accepted the fixture without any
feature gate or diagnostic.

The exact standard output was:

| Line | Native gate compiler | Matching Rust SDK |
| --- | --- | --- |
| Reported version | `version (18, 0, 0)` | `version (18, 0, 0)` |
| Runtime input scalars | `inputs 1368 42973` | `inputs 1368 42973` |
| Constant predicates | `const true true` | `const true true` |
| Runtime predicates | `runtime false false` | `runtime true true` |

Both builds and both programs exited zero with empty standard error. The native
program therefore disagrees with its own constant evaluation and with the
matching Rust SDK for **U+0558 lowercase** and **U+A7DD uppercase**. This is now
an actual compiled-program observation for those two cases.

The native executable was the already-built `gate` profile `lust-core`, build
identity `b69824ccabcb4862`, SHA256
`505502d7c0fe91d388e46c22fab9e1b82453c34e2dc8b8c50bdf880ff6b1e9ad`.
Its source parent was `a1cd906cb90d2430611f39d7f3c3450d02d42299` (the same tree
as published `429fe605`), with the preserved admission-boundary patch SHA256
`4e2a256d68afcf361a52678af62973c9d7c300841cf9ece076d833d91398706f`.
That patch leaves the five relevant Unicode evaluator, code-generator and
runtime files unchanged. The reference was Rust 1.102.0-nightly, source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, LLVM 23.1.5, matching the explicitly
selected SDK in the native gate build command.

The run used direct `lust-core build`, `LUST_NO_CACHE=1` and
`LUST_NO_DAEMON=1`, with fresh isolated cache and temporary directories. It
started no daemon or socket and performed no retry. The fixed schedule contained
two compiler builds, two program executions and one SDK identity query. All
five commands completed; compiler, fixture and source-patch hashes matched
before and after. This was a local correctness witness using an existing gate
binary, not a release-package qualification or a new timing comparison.

The [sanitized witness record](evidence/unicode-semantics-20261118/runtime-witness.json)
contains the complete fixture, command forms, exact outputs, source and binary
hashes, standard-library identities and limitations. Raw command records,
generated programs and supporting runtime files are retained privately. The
frozen baseline and both compiler sources were left unchanged.

## Source-table comparison

The full-domain predicate differences below remain **source-table inference**;
only the two named predicate cases above have this new compiled witness.
A read-only Python decoder applied the
exact pinned Rust `bitset_search` rules to the fetched lowercase and uppercase
tables and compared the resulting sets with native C range tables. It included
Rust's explicit ASCII fast paths and checked that the decoded non-ASCII counts
match the generated Rust file's count comments. Whitespace was decoded from
Rust's lookup map; control characters were compared with Rust's explicit fixed
ranges. The decoder does not use Python's Unicode database.

| Predicate | Native Unicode 17 scalars | Pinned Rust Unicode 18 scalars | Added in Rust 18 | Removed |
| --- | ---: | ---: | ---: | ---: |
| `is_lowercase` | 2,595 | 2,740 | 145 | 0 |
| `is_uppercase` | 2,006 | 2,026 | 20 | 0 |
| `is_whitespace` | 25 | 25 | 0 | 0 |
| `is_control` | 65 | 65 | 0 | 0 |

The source-derived consequences include:

| Expression | Native runtime table | Native constant evaluation with the current host | Full Rust 1.102 source |
| --- | --- | --- | --- |
| `'\u{558}'.is_lowercase()` | `false` | `true` | `true` |
| `'\u{a7dd}'.is_uppercase()` | `false` | `true` | `true` |

These predictions from the bound sources agree with the two-case compiled
witness above. The full-domain counts and the additional witnesses in the source
evidence remain decoder results. The count comments are a decoding sanity check,
not a substitute for full compiler qualification.

## The source paths responsible

Both native
[`char_assoc_const`](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/ty/consteval.rs#L2973)
and the checker's
[associated-constant handling](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/check/mod.rs#L12297)
read `char::UNICODE_VERSION` from the Rust host. The
[constant evaluator](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/ty/consteval.rs#L5201)
also calls host `char::is_lowercase`, `is_uppercase`, `is_whitespace` and
`is_control`. The current host's lower/uppercase methods resolve through the
[pinned Rust character implementation](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/char/methods.rs#L904)
and its
[Unicode lookup aliases](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/unicode/mod.rs#L10)
to the Unicode 18 tables.

Ordinary native method calls instead lower through
[native code generation](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/cgen.rs#L8623)
to the C runtime's
[Unicode range lookups](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/runtime/core.c#L1667).
Those tables remain Unicode 17. Agreement of whitespace and control for these
two pins does not remove the build-host dependency in their constant evaluator.

The versioned surface is wider than these predicates. Native
[case conversion](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/runtime/core.c#L2375)
uses generated case maps and final-sigma properties. Its
[debug escaping table](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/runtime/escape_table.h#L1)
explicitly names Rust 1.99, and its
[identifier/NFC tables](https://github.com/glopyglerky/lustr/blob/429fe6054e6119221e598b95c2661665ecdd7be2/compiler/native/src/xid_table.rs#L1)
name Unicode 17. Those families were source-audited for provenance, not
exhaustively compared or newly executed here. They must be considered when
changing the declared Unicode version.

## Required repair contract

1. **Choose the Rust semantics being promised.** Record the exact Rust source
   commit and Unicode version. With the current imported full SDK unchanged,
   the combined target is Rust 1.102 / Unicode 18. Preserving a separate native
   Rust 1.99 experiment does not establish equivalence with that target.
2. **Import the matching upstream Unicode data once.** Bind its source and
   generation inputs. Derive native constant evaluation, generated runtime
   lookup and the reported version from that same data, independently of the
   Rust compiler used to build lustr. Existing upstream generators or directly
   imported upstream lookup implementation are appropriate starting points;
   a second handwritten semantic table is not required.
3. **Cover the affected surfaces consistently.** Predicate membership, case
   conversion and contextual rules, debug escaping, identifiers and NFC need
   explicit version alignment. A capability not yet aligned stays outside
   native admission. Changing only `UNICODE_VERSION` would conceal the proven
   const/runtime disagreement. Regenerating only the C table would leave
   constant evaluation dependent on future build hosts.
4. **Qualify native and complementary behavior together.** Check the version,
   constant/runtime agreement, boundary scalars such as U+0558 and U+A7DD,
   and the affected full-domain/table tests against the same declared Rust
   reference. Exercise cold and retained native state as well as the imported
   Rust path. Preserve invalid-input handling and observable output; require
   actual compiler/runtime evidence before admitting those capabilities.
5. **Keep the historical baseline frozen.** Do not edit it, change its oracle,
   normalize the mismatch away or exclude its failed fixture. A future source
   repair must update the explicit integration identity and receive its own
   qualification. The old failed comparison remains failed.

Until that work is qualified, the standalone native Unicode gap remains an
explicit blocker. The current scalar admission boundary prevents it from
becoming an assumed capability of that narrower path; it does not establish
general native/full-Rust semantic equivalence.
