# Cargo metadata still requires rustc

The pinned Cargo executable tries to run `rustc -vV` for the public slice's
metadata request. Removing rustc from the selection proof would hide a real
failure of that request. The cache policy remains unchanged.

The probe used Cargo revision `c07ac2c4d12237effc4b94f4543769b421790045`, installed
with the pinned Rust `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` SDK. Its Cargo
SHA-256 is `da29c2a9ff7f85bda5587e9c974e05c9e2a7196377e54466f6e9d6f9e00b2530`.
`cargo-version.stdout`, `plan.json` and `identity.json` preserve those identities.

## Executed requests

Each row ran once with its own initially empty Cargo home, target directory,
build directory and temporary directory. `CARGO_CACHE_RUSTC_INFO=0` disabled
independent rustc-information reuse. The metadata argv matches `full::select`
at product revision `3c378a11d60149a3ec99fa49ea1b336e9406fca8` and contains no
`--no-deps` or `--filter-platform` option. The final row adds a platform filter
as an explicit control.

| Request | RUSTC and RUSTDOC | Exit status | Result |
| --- | --- | ---: | --- |
| `metadata --format-version=1 --manifest-path ... --offline --locked` | Existing source-built wrapper and SDK rustdoc | 0 | Complete JSON metadata |
| Same metadata request | Deliberately nonexistent paths | 101 | Cargo attempts `deliberately-missing-rustc -vV`; the OS returns ENOENT |
| `pkgid --manifest-path ... --package slice --offline --locked` | Existing tools | 0 | Selected package ID |
| Same pkgid request | Deliberately nonexistent paths | 0 | Identical package ID |
| Metadata with `--filter-platform x86_64-unknown-linux-gnu` | Deliberately nonexistent paths | 101 | Same rustc-version failure |

The original public request has no package selector, so its selection path does
not run `pkgid`. That successful countercheck cannot justify omitting rustc
from the metadata request's input proof.

The source fixture is byte-identical to
[`warm-entry-selector-spans-evidence/fixture`](../warm-entry-selector-spans-evidence/fixture).
Its manifest declares a dependency-free `slice` binary and an empty workspace.
Every source digest appears in the predeclared plan; all fixture bytes remained
unchanged after these five observations. This is a dependency counterexample,
with no timing or compiler-performance claim. It does not establish every input
consulted by Cargo or prove a separate rustdoc-only dependency result.

## Source explanation

In the exact Cargo source,
[`build_resolve_graph`](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/ops/cargo_metadata.rs)
constructs `RustcTargetData` on the full metadata path, including an empty
platform-filter list. The public entry requests that path. The independent
[`pkgid` implementation](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/ops/cargo_pkgid.rs)
does not supply an alternative metadata proof.

The two retrieved Cargo files and their licenses are retained in `source/cargo`.
The public entry's exact `src/full.rs` is retained in `source/lustr`.

## Evidence and replay

Run `python3 verify.py` from this directory to check the fixed five-command
schedule, command/environment separation, raw stream digests, Cargo identity,
fixture identity and results. This verification does not execute Cargo again.

`runner.executed.py` preserves the executed runner after the documented path
projection. `plan.json` predates execution, and `report.json` retains each
command, exit status and stdout/stderr digest. Public files replace only the
task workspace prefix with `<DIAGNOSTIC_WORKSPACE>`. The report keeps both the
original and projected stream hashes.

`originals.cms` preserves the exact unprojected runner, plan, report, fixture
manifest/lockfile, version streams and command streams. It uses the repository's
existing evidence-recipient certificate, CMS AES-256-GCM and RSA-OAEP SHA-256.
The certificate and archive identities are in `identity.json`. No recipient
private key is part of this bundle.
