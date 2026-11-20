# Mixed LLVM / Cranelift ABI contract

> Independent Rust-backend research fixture. Both compiler lanes use rustc;
> neither implements the exclusive Lust owner or the selective complement.
> The retired coordinator/backend package must not be rebuilt as Lustr from
> this guide. The [current contract](../../../../docs/compiler-set/exclusive-coverage-contract.md)
> governs product work.

This path-only workspace checks calls between separately compiled Rust crates.
`mixed-abi-contract` defines shared types; `mixed-abi-provider` implements the
provider; `mixed-abi-app` implements callbacks and runs assertions. Boundary
functions use `#[inline(never)]`. Both backends use the same pinned rustc
frontend and its ordinary LLVM-built standard library.

The current integration test runs only an explicitly identified stock Rust
reference and validates fresh compiler evidence. It does not execute a Lustr
compiler or the retired optional Cranelift route. The original experiment built
a fresh ordinary Cargo baseline and these two mixed configurations:

| Configuration | Contract | Provider | App and callbacks |
| --- | --- | --- | --- |
| LLVM baseline | LLVM | LLVM | LLVM |
| Cranelift consumer | LLVM | LLVM | Cranelift |
| Cranelift provider | LLVM | Cranelift | LLVM |

Callbacks cross back into the caller, so each historical mixed configuration executes
calls in both directions. The reverse configuration also tests each crate's
implementation with the other code generator. The gate reads actual `cargo -vv`
compiler commands, checks every crate's selected backend, rejects absent fresh
compilations and requires the pinned rustc commit. A failed backend load,
compiler invocation, link, execution or assertion fails the gate. No retry
through another backend occurs.

The executable and Cargo test harness assert the following behaviors. Which
compiler configuration actually ran must come from its source-bound evidence:

- Rust-layout aggregates passed and returned by value, including a `u128`,
  fixed arrays, `String`, `Vec` and an optional nonzero integer;
- generic monomorphizations of distinct integer types, const generics, and a
  function-pointer callback returning a generic aggregate;
- trait-object method calls and dynamic destructors in both directions;
- present and absent nonzero, owned-pointer and borrowed-pointer niches, plus
  enum variants containing owned allocations;
- allocations created by one crate and grown or freed by the other;
- panic recovery after unwinding through the other backend's stack frames,
  with exact destructor order for both panic origins;
- atomic compare/exchange plus concurrent updates through both crates;
- distinct thread-local values and thread-exit destructors in both crates;
- a `repr(C)` aggregate passed and returned through an `extern "C"` callback.
  This last check uses Rust-defined C calling conventions, without a separate
  C compiler or allocator.

Run the fixture without a network or registry dependencies:

```sh
cargo run --manifest-path crates/lustr/fixtures/mixed-abi/Cargo.toml --offline --locked
cargo test --manifest-path crates/lustr/Cargo.toml --offline --locked --test mixed_abi
```

The [original backend setup and optional invocation](https://github.com/glopyglerky/lustr/blob/ea26fe16636d390763257a19c19df3db992cdbcb/crates/lustr/fixtures/mixed-abi/README.md)
remain historical. A future independent backend comparison must supply its own
explicitly pinned SDK/backend and source-bound evidence; it must not reconstruct
the retired Lustr coordinator package or count rustc fixture success as product
coverage.

`LUST_TEST_ABI_EVIDENCE=/absolute/directory` retains raw stdout, stderr and exit
status for each compilation, direct execution, test command and compiler identity
probe. Each run gets a unique directory. A `passed.txt` receipt is written only
after every assertion succeeds; failed runs retain their failing command output.

`LUST_TEST_CARGO` may select an explicit Cargo binary for the retained independent
reference. The original optional backend gate required an explicit backend; its
environment and setup remain in the immutable historical recipe. The existing
source does not qualify a new LLVM/Cranelift pair, native Lust rlib interoperability
or compatibility for every Rust program or platform.
