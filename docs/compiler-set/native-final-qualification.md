# Final native continuation qualification

This record separates the full combined native run from the later argument
pointer correction. The [combined verification](evidence/native-final-20261119/combined-verification.json)
and [argument verification](evidence/native-final-20261119/argv-verification.json)
retain the exact source, binary and output identities. The imported Rust
reference remains `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.

## Recovered full combined run

The preserved native source at `b728b0e48dcb084756312b8d7f89cbd0c829f0c8`,
native tree `a4fabdaf1b9ccbd29c8303075abb8a2e8fb7515d`, completed these
existing gates:

| Gate | Passed | Failed |
| --- | ---: | ---: |
| Native client unit tests | 18 | 0 |
| Native core unit tests | 413 | 0 |
| Full ordinary differential corpus | 5,546 | 0 |
| Full unwind corpus, including ordinary pass cases | 2,309 | 0 |
| Unwind-specific corpus through GCC exception tables | 65 | 0 |

The same source also completed the declared Unicode qualifier, including all
1,112,064 scalar values and 114,666,345 equal bytes of compiled runtime output,
and the build-script edit test under both Cargo output layouts. The existing
macro expansion-limit fixture retains its declared Rust timeout behavior.
Neither it nor any other comparison was reclassified during this recovery.

The original logs, compiler hashes, frozen library bytes, test-output hashes and
native tree identity were checked again before recording these results. This
is recovered evidence, not a rerun. The compiler used Cargo's `gate` profile;
copying its executables to the differential script's `target/release` location
does not change that profile or establish release-build performance.

## Argument pointer correction

PR #30's macOS C gate rejected the assignment of a `const uint8_t *` to the
runtime's mutable byte-pointer field. `argv` supplies `char **`; the runtime
validates the bytes and copies them into an owned `String`. The correction in
`28ed726e6a55dd55f98222069c650bd923cc43b2` matches that representation.
It does not change string copying, UTF-8 validation or iterator state.

A fresh native compiler was built from that commit, native tree
`eb8b4a8f1c31a6bbbeaded61a01ddd8691f5dff1`, using the declared SDK and locked
offline dependencies. Its targeted qualification produced these results:

| Check | Result |
| --- | --- |
| All 47 previously failing callback fixtures, argument regression and finite arithmetic fixture | 49 native/C/Rust status and stdout comparisons pass |
| Actual erased C declaration checks | 2,884 checks across 476 emitted C units pass |
| Argument regression | Five argument lists cover empty strings, ASCII, precomposed and decomposed Unicode, CJK and emoji, with forward, reverse and interleaved iterator ends |
| Independent wider-integer formulas | All 65,536 signed 8-bit pairs, 65,536 unsigned 8-bit pairs and 3,072 shift cases pass |
| Separate undefined-behavior sanitizer | Freshly compiled C for the argument and finite arithmetic fixtures matches Rust's status, stdout and stderr with no UBSan report |

The targeted C builds reject discarded pointer qualifiers on GCC as well as
incompatible callback types. The preserved preceding runtime fails that check.
The checked-in callback qualifier now uses `-Werror`, because GCC and Clang
classify the same discarded-qualifier defect under different warning names.
An actual preceding-source run fails under the updated qualifier; the rebuilt
source passes its direct and exception-table controls. No warning is suppressed.

## Scope

All results above executed on GNU/Linux x86_64. They do not replace macOS
execution, the complete address/leak sanitizer gate, real daemon qualification,
general mixed-program semantic integration, package qualification or the final
paired performance campaign. The existing local LeakSanitizer restriction
remains recorded separately; these UBSan results do not relabel it as a pass.

The full combined runs stay attached to their earlier native tree. The pointer
correction has its own rebuilt-source controls. Subsequent protocol changes or
shared-service changes require their own evidence. No finite collection of
tests proves every Rust program or every arithmetic domain correct.

## Subsequent protocol continuation

The terminal-exchange change in `64686b9c74606b3b25039e0415b12538480d0fc5`
produces native tree `80a66542b8dc348b036e2974ab2e6e0c42a433fe` when combined
with the source above. An independent verification compiled both complete native
binaries with `cargo check` under the declared SDK and ran all 21 tests in the
shared protocol module. Both checks passed. The
[transport record](evidence/native-final-20261119/transport-verification.json)
preserves their exact source and output identities.

The tests cover partial writes, missing or malformed replies, incorrect build
identities, invalid lengths and UTF-8, and explicit pre-execution build refusal.
They exercise the exchange logic with an in-memory peer. They do not establish
a real Unix socket exchange, rebuild a release package, or transfer the earlier
full differential runs onto the changed protocol source.
