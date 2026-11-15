# Native preview 2026.11.15-1

This pre-alpha preview packages unchanged upstream Lust, bounded native Lustr
library additions and the official unmodified Rust nightly compatibility SDK.
The [Lustr release](https://github.com/glopyglerky/lustr/releases/tag/native-preview-2026.11.15-1)
and [blueFe release](https://github.com/glopyglerky/blueFe/releases/tag/native-preview-2026.11.15-1)
carry identical assets. Full native Rust, Rust library metadata, general native
Cargo routing and required full-project speed remain incomplete.

## Source and delivery

The implementation source is Lustr
`60f9d8b2375d2980a46aa15879c737a8f55a43f6`. Later release documentation does not
change its compiled implementation. Complete corresponding native application
source, offline locked dependencies, original notices, exact TinyCC source and
the Rust recipient relink helper accompany the runtime in `share/bluefe-source`.
The complete Lust tree remains upstream `240e08d91d4aed0299639f6f11fd522912b248f9`,
tree `0ab1b09815802894e6078de0af59273cbdf46cb0`. Its tracked fixture caches remain
to preserve that tree; newly generated local targets and private recipient proof
outputs are excluded.

The full Rust compiler is official **unmodified** `nightly-2026-11-09`, source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, rustc 1.102.0-nightly / LLVM 23.1.5.
It is not a fork-produced Rust compiler. The root native entry bytes match the
qualified v26/v28 runtime; preview assembly adds source, notices and evidence.
Experimental rustc-dev metadata and withdrawn TLS/prefork paths are excluded.

Download all release assets, verify `SHA256SUMS`, and extract the archive into
a new versioned directory. Preserve hardlinks and the complete directory tree.
Use its absolute root; no global installation or default-toolchain change is
required:

```sh
BLUEFE_ROOT=/absolute/bluefe-lustr-native-preview-2026.11.15-1
"$BLUEFE_ROOT/bin/lustr" build program.rs -o application
"$BLUEFE_ROOT/bin/lustr" check program.rs
"$BLUEFE_ROOT/bin/lustr" --bluefe-root "$BLUEFE_ROOT" doctor
CARGO="$BLUEFE_ROOT/bin/cargo" RUSTC="$BLUEFE_ROOT/bin/rustc" \
  RUSTDOC="$BLUEFE_ROOT/bin/rustdoc" "$BLUEFE_ROOT/bin/lustr" test --locked
```

The archive's README describes routing, host dependencies and relinking.
`source-components.json`, `payload-manifest.json`, `platform.json`,
`archive-audit.json` and `qualification.json` bind the actual delivered bytes
and acceptance scope. Unsupported requests retain explicit full-Rust behavior;
an executed application or test is never retried through another compiler.

## Qualification and limits

The extracted runtime passed 17 unit entries and 13 actual-runtime integrations,
including native cold/retained library execution, rejection, non-Unicode bytes,
capacity-overflow recovery, unwind/Drop, source sealing, Cargo target selection,
foreground cancellation, real metadata, build scripts, proc macros, doctests,
Wasm linking, formatting and direct/Cargo Clippy rejection plus success.
The boolean lint is explicitly enabled because this pinned Clippy places it in
the pedantic group; initial fixture failures remain retained. These are bounded
integration checks, not universal Rust semantics or tool compatibility.

The payload's 32 ELF objects were inspected individually. Qualified host is
Linux GNU x86_64 on AerynOS 2026.08. Observed requirements include GLIBC 2.39,
GCC_4.2.0, libgcc, zlib and **both** `/usr/lib/ld-linux-x86-64.so.2` and
`/lib64/ld-linux-x86-64.so.2` interpreter paths. System compiler/linker, headers,
startup files and development libraries remain required for native linking.
Other hosts and Wasm execution are unqualified. All seven broader public
platform gates remain separately held.

The full source export matches the implementation commit. Frozen-byte review,
raw normalized archive headers, safe names/links, complete extracted manifest
and real core hardlink checks accompany the runtime evidence. Privacy scans
found no configured private prefixes. Seven heuristic findings were accounted
for by unchanged official SDK strings and scanner literals in supplied source;
this is not proof that arbitrary credentials can never occur.

Original official SDK notices remain unchanged. Exact VCS supplementation
recovered 22 files for 14 declaration-only packages. Of the four original-text
absence cases, three are unshipped test inputs. Shipped Clippy's
`quine-mc_cluskey` has exact manifest/blob provenance and explicitly labelled
standard MIT terms, without an invented original holder, year or notice file.
The kit records this declaration-based acceptance and the dirty-base limitation
for `spanned`; no universal legal-compliance certification follows.

The recipient recipe rebuilt every native consumer after a passive TinyCC
VERSION edit, regenerated source/runtime identities and passed the prior
17+12 runtime suite. Failed builds invalidated the completion marker and normal
retry regenerated identical consumer digests. This does not qualify arbitrary
TinyCC changes, atomic installation or interruption/crash exhaustiveness.

## Performance remains bounded

The archive contains the unchanged nine-sample campaign's raw 1,428 events,
summary, source snapshots, traces and outputs. Its method distinguishes cold,
repeated, body/signature/type/macro edits, rejection and compile-plus-run, on a
shared host with original and matching-producer pristine baselines. Repeated
native requests improve on the bounded fixtures; cold/edit/rejection regressions
remain. These samples do not establish native Cargo no-op/test, whole-project
speed, whole-toolchain memory safety or a never-slower guarantee. See the
[native compiler contract](native-contract.md) for the unfinished requirement.

Earlier published previews retain their original source, assets, measurements
and rollback role. Their rustc-based speed results do not transfer to this
native preview.
