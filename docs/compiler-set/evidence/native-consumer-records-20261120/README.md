# Native consumer record boundary

This evidence qualifies a research boundary from the existing native object
compiler: an owned preparation, complete records for its admitted scalar
declarations and bodies, and the actual emitted ELF definitions. It does not
qualify genuine Rust metadata, the public retained compiler, full Rust coverage,
or a speed comparison.

The source base is `c8218b7fc0f657314c9e9c59775091b53508c6b0`. The protected
`compiler/native` tree remains
`bf933c382fd99e52dd2586ebc46f0d0aceaa71a8`. No upstream source, pin, product
feature, product dependency, or public compiler route changed in this slice.

## What the records establish

`native_object::prepare` performs the existing native parse, collection, body
checking, lowering, borrow checking and C generation once. Its immutable value
owns the source, original token stream, program, generated bodies and accepted
body records. The original working directory and parameter-name spans come
from that preparation; metadata preparation does not lex or parse again.

The private completion token joins all collected source functions to all
accepted bodies. It uses original source visibility, actual native identities,
checked expression resolution, lowered direct-call edges, and generated
source-function references. Completion refuses any unrepresented obligation.
Its current scope is one ASCII, flat `no_std` source containing ordinary safe
nongeneric `(u64, ...) -> u64` functions with the represented expression bodies
and applicable declaration/lint conditions. This is not a replacement for the
full Rust declaration, privacy, liveness or lint machinery. An integration
refusal leaves the operation owned by Lust.

The emitted inventory comes from the actual generated object. Every accepted
source function must have its definition present, including private helpers.
Nonlocal backend helper symbols are recorded separately. The object parser
admits one ELF symbol table and rejects duplicated nonlocal definitions inside
that table; it does not conflate a separate dynamic symbol table with another
definition.

## Exact attempts and results

| Attempt | Result | Evidence meaning |
| --- | --- | --- |
| Initial preparation check | Cargo check passed | Earlier refactor checkpoint; not the final build |
| Build v1 | Build passed; source changed during the build | Explicitly unqualified source-to-binary correspondence; retained unchanged |
| Build/run v2 | Build passed; control run exited 101 | The const fixture referred to a const that the existing bare-object path could not resolve; original failure retained |
| Build/run v3 | Build passed; all 17 named controls passed | Stable final source, actual native objects, independent caller with 131,076 result assertions |

The v2 fixture assumed bare-object const resolution that was not present. The
final control instead keeps the const declaration and uses an otherwise
supported body. It proves the intended missing metadata-definition obligation
is refused. No failed compilation is counted as success, complement ownership,
or missing test coverage.

The positive fixture has two public functions and a private helper. The actual
native graph includes the public-to-private call. An independent pinned Rust
**C ABI caller** runs both functions over 65,536 small values plus two boundary
values, checking 131,076 results. Its stdout is exactly
`native-body-results=131076\n`, stderr is empty, and exit status is zero. This
caller does not yet consume Rust metadata.

The 17 controls also cover exact inventory and visibility, preparation surviving
a later source edit, duplicate object symbols, abandonment without emission,
invalid private bodies, duplicate declarations, invalid types, unrepresented
lint/signature/definition scopes, good-output preservation, and valid recovery.

The final qualifier executable SHA-256 is
`80e35e1e4b1ba34b2c699c26ff8f82f3f4eea0f6a8f135b217f2931c51305a54`.
The failed v2 executable has a separate identity; neither binary is committed.

## Records and replay

`verification.json` contains source and tool identities, the archive digest,
and each attempt's status. `inventory.json` lists the exact size and SHA-256 of
every original file, including four omitted compiled executables.
`records.tar.gz` preserves the other 102 original files byte-for-byte, including
both source snapshots, original command/environment receipts, compiler and
program streams, exact test sources, generated C and small object controls.
The archive digest is
`667ae970624cb7e975b2d28bca2b0374206ef11d9822aba504f4753d2a11fe92`.

The build receipts include a broad source-file inventory taken before and after
the build; it includes repository fixtures that are not compiled into the
qualifier. `verification.json` separately binds the six changed implementation
and qualifier files to the final build receipt.

The exact replay arguments and environment are in
`native-consumer-build-v3.json` inside the archive. With the pinned SDK and the
declared TinyCC/Clang installations, the central commands are:

```sh
cargo build --manifest-path crates/lustr-native-core/Cargo.toml \
  --locked --offline --jobs 2 --target-dir "$QUALIFIER_TARGET" \
  --config "build.build-dir=\"$QUALIFIER_BUILD\"" \
  --config profile.dev.debug=0 --config profile.dev.incremental=false \
  --features native-qualification --example native-consumer-qualification
"$QUALIFIER_TARGET/debug/examples/native-consumer-qualification" \
  "$NEW_EVIDENCE_DIRECTORY" "$PINNED_SDK/bin/rustc"
```

`CARGO_HOME`, `RUSTUP_HOME`, `RUSTC`, `LUST_TCC_HOME`, `PATH`, and
`LD_LIBRARY_PATH` must select the exact tools recorded by the receipt. Output
directories must be new. The Rust compiler is used to build the compiler and
as an independent qualification caller, never as a consumer-body fallback.

## Retained integration and cost still required

The existing research object backend is a one-shot Clang path. It is not the
public retained TinyCC path. Its completed records must be connected to the
ordinary compiler by capturing fresh check/lower/emission facts and joining
valid retained entries with exact definition/body/header/read-set generations.
`PreparedRequest::checked_functions()` alone is only the current worklist;
it cannot stand in for that complete inventory.

The new metadata capture is opt-in. Existing bare-object and artifact calls
skip its expression-resolution, IR-call and generated-use record walks.
Unrepresentable generated slot classes become absent metadata proof rather
than rejection of an existing valid object/artifact request.

The research capture performs work proportional to checked expressions, IR
blocks and generated edges, followed by the bounded obligation walk. It does
not repeat body checking, but the capture and conversion cost has not been
benchmarked. Ordinary retained integration must avoid rerunning all bodies
through this research path. No near-Lust, no-overhead or overall speed claim
follows from these controls.
