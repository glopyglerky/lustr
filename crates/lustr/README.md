# Retained Cargo planning helpers

This crate retains command parsing, argument preservation, configuration and
planning helpers from the former compiler coordinator. The coordinator's
compiler execution has been removed. Its command-line entry points accept help
and version queries and refuse compilation without invoking another compiler.

Lustr's only compiler direction is the
[exclusive coverage contract](../../docs/compiler-set/exclusive-coverage-contract.md).
The compiler uses Lust for everything Lust covers and imports only the upstream
implementation required for the remaining Rust work. The public compiler lives
at the repository root. This crate does not supply full Rust compilation or a
second way to choose a compiler.

## Current interface

The library retains parsers and plans because they preserve useful Cargo and
operating-system argument behavior. A plan is data. It does not establish
compiler ownership, execute a build or qualify an implementation.

The legacy executable names `lustr`, `cargo-lustr` and `cargo-lust` remain
refusal entry points for old callers. Installing this crate does not install the
Lustr compiler. `build`, `check`, `run`, `test`, toolchain selectors and engine
selectors return failure. A refusal must not launch Cargo, rustc, rustdoc,
rustup, Lust or a caller-supplied compiler wrapper.

Run the current crate tests from the repository root:

```sh
cargo test --manifest-path crates/lustr/Cargo.toml --locked --offline
```

The [retired acceptance disposition](../../docs/compiler-set/retired-coordinator-acceptance.md)
records which earlier tests depended on the removed execution path and which
argument, planning and independent oracle checks remain useful.

## Retained research

The [backend records](backends/README.md), [benchmark records](bench/README.md)
and fixtures preserve source identities, failures, ABI/unwind probes and
measurement methods. A complete rustc frontend with a different code generator
is not Lustr's selective complement. Results from those experiments cannot
qualify the current public compiler.

The complete [former instructions](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/README.md)
remain at their original revision for reproducing historical evidence. Their
compiler-selection commands do not describe current behavior. Continue current
work through [PR #33](https://github.com/glopyglerky/lustr/pull/33) and the
[single completion checklist](../../docs/compiler-set/completion-checklist.md).
