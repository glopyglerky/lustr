# Compiler qualification fixtures

These are reusable workload sources and independent oracle fixtures. A stock
Rust or Cranelift pass is evidence for that exact compiler invocation, not Lustr
coverage. Historical coordinator-based tests are retired; current product tests
must execute the [exclusive Lustr entry](../../../docs/compiler-set/exclusive-coverage-contract.md)
and check actual output, failures and ownership.

`full-rust` is a dependency-free Cargo workspace with three real crates. It
exercises an external procedural derive, a build script's generated Rust and
environment variables, async polling, generic associated types, const generics,
unsafe pointer access, a C calling convention, cross-crate calls, and a doctest.
The binary catches a panic and checks that a stack destructor ran. This catches
an accidental change from unwinding to aborting. A deterministic stderr trace
marks entry, the panic hook, and successful recovery. The retained independent
stock reference compiles and runs this workload. The historical optional
Cranelift comparison ran the final executable directly and compared stdout,
stderr and exit status with ordinary Cargo. That optional route is retired;
its old result is not a current Lustr or backend qualification.

Expected output with no application arguments:

```text
derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1
```

The `BENCH_PROBE` line in `full-rust/core/src/lib.rs` is an edit point for paired
benchmark copies. The binary casts its result to `u64`, so changing its return
type from `u32` to `u64` preserves the printed value. Benchmark code must edit a
private copy of the fixture, never the repository's sources.

`invalid-borrow` is an invalid Rust oracle fixture with rustc error `E0502`.
A candidate rejection must independently establish the intended borrow error;
compiler failure or an unrelated diagnostic cannot count as acceptance.

`subset/supported.rs` is a small Lust program with output `42`.
`subset/unsupported.rs` records valid Rust rejected by the pinned Lust fixture
with `L0405`. A new support decision needs the pinned capability rule and actual
semantic context, not that diagnostic alone. Current native failure cannot
select another compiler or turn a Lust-owned integration gap into the complement.

These tests establish agreement on this corpus. They do not prove arbitrary
Rust compatibility, arbitrary backend interoperability, or a compilation speedup.
