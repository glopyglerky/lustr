# Public native-entry preflight profile

The public entry repeats a measurable Cargo target-selection step before it
contacts Lust. On this Linux x86-64 component profile, that selection took
**12.124 ms median** without a package selector. A separate variant with
`--package slice@0.0.0` took **23.455 ms median**. This identifies work worth
removing from a retained request. It does not measure the fraction of a previous
hosted entry result that this work caused, and it establishes no speedup.

No compiler product source or routing changed. The diagnostic helper includes
the actual parser, target selector and foreground child owner from source
`34d847da3d90c3c2b253246a8ffdfdeb901c56b0`. It performs target selection only.
There is no application compilation, native daemon request, generated-program
execution or complete entry measurement. Cargo may execute compiler information
probes while resolving metadata; the separately retained warm trace reported
cached compiler information.

## What was measured

The [existing public-slice runner](https://github.com/glopyglerky/lustr/blob/34d847da3d90c3c2b253246a8ffdfdeb901c56b0/tools/run-upstream-slice-benchmark.py)
uses this native check request, with analogous build arguments plus `-o`:

```sh
lustr check <fixture>/src/slice_main.rs \
  --manifest-path <fixture>/Cargo.toml --bin slice --offline --locked \
  --target x86_64-unknown-linux-gnu \
  -C panic=unwind -C overflow-checks=on -C debug-assertions=on
```

**That original request has no `--package` argument and does not run
`cargo pkgid`.** The package-selector rows below are a separately identified
diagnostic variant. They cannot explain the original no-package entry result.

All 43 archived fixture files, including the manifest, lockfile and source,
were copied unchanged and remained byte-identical after the profile. Their
hashes are in [identity.json](identity.json). The source is the existing
`goport-util-small` fixture with its recorded dev `opt0/debug0` settings, not a
new application or reduced source slice.

| Component | Samples | Median ms | p95 ms | Maximum ms |
| --- | ---: | ---: | ---: | ---: |
| `cargo metadata`, owned child | 24 | 12.386454 | 19.413503 | 24.252021 |
| Complete `full::select`, no package selector | 24 | 12.123610 | 17.586036 | 19.697448 |
| `cargo pkgid`, owned child, explicit package | 24 | 11.061824 | 19.658105 | 21.575540 |
| Complete `full::select`, explicit package | 24 | 23.455190 | 42.638208 | 90.960555 |

Each phase was measured once in each of 24 retained rounds. The four phase
orders were shuffled with recorded seed `5787764639354406728`. Two preceding
warmup rounds, eight helper invocations, remain recorded and are excluded from
the table. The 96 measured helper invocations contain 120 Cargo subprocesses,
because selection with a package invokes both metadata and pkgid. No measured
sample was discarded, including the 90.961 ms observation. The p95 is the
nearest-rank value at `ceil(0.95 * n)`.

Timing starts inside the helper after the original native argument parser.
It includes the selected component's process supervision, output handling and
wait. Complete selection also includes JSON decoding, canonical source matching
and target validation. It excludes helper process startup and argument parsing.
Metadata and pkgid rows use the same Cargo command, compiler environment and
child owner as the selector. They are separate observations, so subtracting
their medians from the selection median would not be a paired attribution.

The exact helper, Cargo, mixed compiler and reference Rust hashes are retained.
The compiler components come from an earlier explicitly bounded composition,
whose entry-related source is byte-identical to the inspected PR16 source.
That composition lacks the native shared library and is not a complete package.
This diagnostic uses only its Cargo and compiler-information path. It does not
promote that composition to a qualified native runtime.

## What the trace establishes

The separate `CARGO_LOG=trace` metadata command succeeded with the same fixture
and controlled environment. Cargo reported an existing rustc-information cache
and cache hits. The observed trace therefore does not support blaming repeated
compiler-information queries for this warm selection cost. The uninstrumented
samples remain separate from this trace. No syscall-complete process trace was
collected, so this is not a claim about every possible subprocess.

The historical hosted entry results used another execution environment. These
component medians must not be subtracted from those 31–32 ms results or combined
with their 8.6–8.7 ms Cargo medians to claim an attributable share. A future
optimization needs a same-host, source-bound whole-entry before/after campaign.

## Why the existing native graph is not a direct replacement

The entry's [full::select](https://github.com/glopyglerky/lustr/blob/34d847da3d90c3c2b253246a8ffdfdeb901c56b0/src/full.rs)
runs unfiltered metadata in the caller's directory and lets Cargo resolve an
explicit package specification. It requires exactly one package, target name,
kind and canonical source match.

The retained native [Cargo graph loader](https://github.com/glopyglerky/lustr/blob/34d847da3d90c3c2b253246a8ffdfdeb901c56b0/compiler/native/src/resolve/cargo.rs)
already reuses watched graphs. Its metadata command runs in the manifest's
parent directory, supplies feature and platform filters, and its normal unit
selection uses its own package-name/manifest rules. Sharing that existing graph
is a promising direction, but substituting it for the entry check would change
the validation contract.

A real Cargo counterexample is retained in
[qualification.json](qualification.json). The same selector and fixture return
exit 2 from a caller directory containing malformed `.cargo/config.toml`, then
succeed when invoked from the manifest's directory, which does not inherit that
configuration. This demonstrates the directory distinction without changing
the fixture. An optimization must preserve caller-directory configuration,
Cargo package-spec resolution, target uniqueness, locked/offline/frozen errors
and the existing output artifact on rejection.

The next integration should expose or retain the exact validated selection in
the existing native lifecycle, with explicit Cargo command/environment identity
and observed-input invalidation. Manifest/path dependency edits, target
discovery, symlink changes, lockfiles, Cargo configuration, tool identities and
offline source availability need real transition tests. No new independent
metadata cache, `--no-deps` substitution or validation bypass is introduced by
this diagnostic.

## Verification and retained evidence

The exact production selector passed the original fixture and versioned
package request, rejected a missing target and missing versioned package with
exit 2, and retained the real Cargo diagnostics. Metadata and pkgid component
commands succeeded. The directory counterexample returned the expected 2/0
outcomes. These are focused diagnostic checks, not native compilation gates.

Two initial direct-rustc helper builds failed while resolving Cargo's split
`.rlib`/`.rmeta` dependency artifacts. An isolated Cargo build of the unchanged
helper succeeded with the pinned Rust SDK, and its locked offline recheck
succeeded. All four build results are retained. Product sources, source guards
and benchmark assertions were unchanged.

- [report.json](report.json) retains every measured duration and summary.
- [samples.json](samples.json) retains all 104 helper observations, output,
  status, order and normalized arguments. Absolute paths are replaced with
  stable role markers; this is an explicitly transformed public projection.
- [identity.json](identity.json) binds input source, fixture, binary and lockfile
  hashes and records the failed and successful helper builds.
- [qualification.json](qualification.json) retains the focused acceptance,
  rejection and caller-directory checks.
- [encrypted-originals/receipt.json](encrypted-originals/receipt.json) identifies
  the authenticated archive of unmodified raw records, full Cargo trace, build
  logs, frozen helper/dependency inputs and fixture. The established public
  recipient certificate was used. No private key was accessed or decryption
  attempted. Compiler binaries and build/cache state are excluded.

The helper is [tools/profile-native-preflight.rs](../../../../tools/profile-native-preflight.rs).
Build it as a standalone Cargo binary with edition 2024 and exact dependency
`serde_json = "=1.0.154"`, pointing the binary path to that source file. Set
`LUST_BUILD_HASH` from the selected native build receipt for the included
protocol and use the matching Rust SDK. It neither starts nor communicates with
the native compiler. A fresh external Cargo project avoids modifying the
product manifest or incorrectly wiring split dependency artifacts by hand.

Its invocations are:

```sh
profile-native-preflight selection <absolute-package-bin> <native-arguments>
profile-native-preflight metadata <absolute-package-bin> <native-arguments>
LUSTR_PROFILE_PACKAGE=slice@0.0.0 \
  profile-native-preflight pkgid <absolute-package-bin> <native-arguments>
```

For the optional full-selection variant, append `--package slice@0.0.0` to the
native arguments. The pkgid diagnostic's explicit environment value must match
that selector. Use a new output directory, the recorded controlled environment,
and the same fixture for each phase. Raw records contain local paths and must
be reviewed or retained through the existing encrypted-evidence helper before
publication.
