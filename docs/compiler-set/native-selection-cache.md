# Reusing public-entry Cargo selection

The public native entry previously ran Cargo metadata, and Cargo pkgid when a
package selector was supplied, before every retained native request. Pristine
Lust's retained compiler did not pay that public-entry preflight cost. This
candidate reuses a successful selection for a bounded class of requests.

The optimization does not reuse the native resolver's graph as the public
selection. Those queries have different working-directory and filtering
semantics. It retains the exact metadata and optional package ID produced by
the public entry's original Cargo commands, then runs the original source,
package, target-name, target-kind and uniqueness filter on that data.

## Admission

All of these conditions must hold:

- The request is offline. It names a `Cargo.toml` with an explicit empty
  `[workspace]`, one local package, no dependencies and no build-script target.
- The manifest uses the admitted literal grammar. That grammar accepts known
  package, target and built-in profile sections with simple single-line
  strings, booleans and integers. Workspace members, inherited fields,
  dependency sections, custom syntax and unknown keys take the original path.
  Cargo still parses and validates the first request.
- The package lockfile already exists. Caller-directory ancestor Cargo configs
  and Cargo-home configs are absent. Both config filenames, errors, symlinks
  and formerly absent paths are checked again on each request.
- The host is Linux x86-64 with the pinned Cargo and reference Rust compiler.
  The source package contains its `lib/lustr-selection-tools.txt` receipt.
  Before storage, the cache verifies the recorded SHA-256 values of Cargo,
  the full compiler, reference compiler, rustdoc and packaged shared libraries.
  Cargo and the reference compiler must also match the frozen source pin.
  Rustup proxies, scripts, unknown compiler tools, custom dynamic-loader
  settings and unrecognized Cargo environment settings do not use the cache.
- The normal Cargo package-cache lock is an accessible regular file. Cache
  reuse cannot hide its replacement by a directory or an inaccessible path.

These restrictions deliberately exclude general dependency graphs and
multi-package workspaces. Such requests still run the existing Cargo selection.
They are not declared unsupported by Lustr.

## Freshness

Every candidate lookup records and compares:

- Exact native arguments, caller working directory and an opaque SHA-256 of
  the full environment. Raw environment values are not stored in the cache.
- Manifest and lockfile contents, the manifest's path binding and the requested
  source's canonical path binding.
- Discovery locations for automatic binary, library, example, test, benchmark
  and build-script targets. Directory entries and nested `main.rs` bindings
  invalidate an old answer when a target appears or disappears.
- Every target-source canonical binding in the cached Cargo output. Changing
  another target's symlink can make a formerly unique source ambiguous.
- Packaged executable and shared compiler-library identities, including inode,
  device, size, modification/change timestamps, ownership, mode and resolved
  symlink destination. The launcher itself is part of this identity.

The first successful population also hashes the tool bytes against the
source-package receipt. A private certificate retains that verification under
the complete tool identities and receipt contents. Different selection keys
can use the same certificate while those identities remain unchanged. The
first population cost is separate from the steady-state cache-hit cost and
must be reported; it is not hidden inside a warm-speed claim.

Normal source package construction now invokes the final installed launcher
with `--lustr-prime-selection-tools` after placing all tools and writing the
receipt. This uses the same actual byte verification and stores a private seed
under `lib/lustr-selection-verified`. A new project's cache can copy that
certificate only if the live launcher, tool and library identities and receipt
still match. This moves verification cost into package construction. A missing,
relocated, changed, corrupt or differently owned seed falls back to actual byte
verification in the project cache; it cannot authorize unknown tools.

Selection does not read Rust function bodies. A body edit therefore retains a
valid target selection and still reaches the native compiler's existing source
invalidation and checking. This cache does not store successful compilations,
skip semantic analysis or decide whether changed Rust is valid.

Before storing a new successful result, the implementation captures selection
inputs again and requires the before/after snapshots to agree. A newly created
lockfile or changed input prevents storage on that request. Failed Cargo
commands and failed unique selections are never cached. A corrupt, oversized,
ineligible or stale record takes the original Cargo path.

As with the entry selection contract, these checks assume the compilation inputs
are not concurrently mutated during one request. Ordinary edits between
requests are observed. File identities are filesystem stamps, not a claim of
protection against a malicious owner able to replace both the compiler and its
cache.

## Storage and controls

Records live in `cargo-selection-v1` below `LUST_CACHE_DIR`, or below the same
source-adjacent `.lust-cache` directory used by the native client when that
variable is absent. The selection directory must be private and owned by the
current user. Records use mode `0600`, an integrity digest, bounded reads and
atomic replacement. Symlinked or foreign/shared records are not accepted.

`LUSTR_NO_SELECTION_CACHE` disables this optimization. `LUST_NO_CACHE` also
disables it, preserving cold benchmark isolation. Optional
`LUSTR_SELECTION_DIAGNOSTICS` names a file that receives one `hit`, `miss` or
`ineligible` record per selection. Normal stdout and stderr are unchanged.
Instrumentation is excluded from timed measurements.

## Qualification

The production selector can be exercised without compiling an application by
the `tools/profile-native-preflight.rs` helper derived from PR #22. It now also
exposes the production package-verification operation for the seed gate. This
is a selection-only seam. It cannot establish end-to-end compiler performance.

The [independent review evidence](warm-entry-selection-evidence/README.md)
preserves the initial candidate's real stale-success failure, its exact source
snapshot, and the corrected selector's completed 132-invocation, 250-assertion
gate. The [followup review evidence](warm-entry-selection-followup-evidence/README.md)
binds the seed gate's 38 invocations and 85 assertions to its exact source and
the repeated 132-invocation selector gate to the identity-observation change.
The seed gate checks verified initialization, malformed or shared seeds,
package relocation, changed receipts, and the same unknown-tool counterexample.
Repeating a gate after an implementation change verifies that
revision; its invocation count is not added as new coverage.

The warm comparison must use public Lustr commands, the exact earlier public
launcher, pristine Lust and rustc through Cargo on the same runner. The two
Lustr launchers must share identical native compiler/runtime and SDK files.
Retained-state diagnostics, actual cache hits, invalid-code rejection, output
status and runtime output remain required. Cold and warm results must remain
separate, with all samples and regressions retained.

The native source tree, its retained state, the pristine Lust reference and the
imported Rust compiler are unchanged by this optimization. It does not qualify
the separate native runtime defects recorded by PR #24, or claim speed parity
before the full comparison has executed.
