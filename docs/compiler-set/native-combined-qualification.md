# Combined native correctness qualification

The combined native source at `af0a281f2d1fae04e8ebfd5aa2c4872b20245c04`
passed all **5,540 ordinary differential cases**, with zero failures, against
Rust source `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. This includes program
execution, panic output, invalid-program rejection and valid Rust intentionally
outside the native subset. The ordinary harness, oracle, output comparisons,
normalization and deadlines were unchanged.

The [verification record](evidence/native-combined-20261119/verification.json)
binds each run to its native source tree, actual compiler binary hashes, Rust
compiler identity, script hashes and complete fixture/library manifest hashes.
The full manifests and raw outputs remain in the private evidence. Every
externally loaded platform-library file was compared byte for byte with the
listed source and then frozen. The compiler and library hashes remained
unchanged through each run.

## Results and preserved failure

| Source and route | Passed | Failed | Unsupported |
| --- | ---: | ---: | ---: |
| `af0a281f2`, full ordinary corpus | 5,540 | 0 | 0 |
| `af0a281f2`, complete unwind corpus, TCC | 64 | 1 | 0 |
| `af0a281f2`, complete unwind corpus, GCC exception tables | 64 | 1 | 0 |
| `ba10b745c`, complete unwind corpus, TCC | 65 | 0 | 0 |
| `ba10b745c`, complete unwind corpus, GCC exception tables | 65 | 0 | 0 |

Both initial unwind failures were `hash_capacity_overflow`. The program's
stdout, panic message, recovery behavior and status matched Rust, but native
reported the obsolete dependency path
`/rust/deps/hashbrown-0.17.1/src/raw.rs:36:40`. The declared SDK reports
`/cargo/registry/25cdd57fae9f0462/hashbrown-0.17.1/src/raw.rs:36:40`.
Both original failing cases and the exception-table route's emitted C were
preserved.

The pinned
[Rust bootstrap source](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/bootstrap/src/core/builder/cargo.rs)
enables Cargo's `trim-paths` behavior for the standard library. Its exact
[Cargo implementation](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/trim_paths.rs)
remaps registry dependencies through the registry source identity. The two
retrieved files were checked against their Git blob hashes. The repaired
runtime uses the virtual path observed in the matching SDK. No collection
algorithm, panic condition, recovery path, test or comparison mask changed.

A fresh native compiler built with that correction passed every unwind fixture
through both TCC's explicit propagation and GCC/g++ exception tables. The
compiler's only source delta from the preceding combined runtime was the
hash-table source-location literal and its explanatory comments. The full
ordinary result remains attributed to its earlier exact source; it is not
relabeled as a later-source run.

## Rejection and hook regression

The ordinary pass includes the new invalid-program fixture for
`PanicHookInfo::can_unwind` without the required unstable feature. The original
native source accepted it, while matching Rust rejected it with E0658. The
repaired native source rejects it with L0104 in both check and build, preserving
the internal flag and hook runner. The fixture remains in `tests/fail`.

The complete unwind corpus includes the new partial-move destruction controls
and the final cleanup-hook regression. An abort during a double panic must run
the required final hook and must never become a successful catch. The existing
recovery, cleanup, ownership and poisoned-state tests also remain present.

## Boundaries

These are Linux x86-64 native correctness results. The native compilers were
built with Cargo's `gate` profile and copied unchanged into the existing
harness's required `target/release` location. That path does not turn them into
release-profile builds. The C exception-table route used GCC/g++ 14.2.0 with the
existing harness flags; it is not a Clang sanitizer qualification.

The complete source must still pass its independently required hosted native,
sanitizer, daemon and integrated-compiler gates. These finite tests do not prove
all Rust programs correct, complete the integrated architecture, establish a
performance gain, or admit an installable package. Earlier failures and
historical performance comparisons retain their original source identities.
