# Native Cargo unit tests

The public compiler forwards Cargo's `--test` request as a retained test
session. Lust checks and lowers the original `cfg(test)` source. Its object
emitter binds the selected test functions to the installed Rust libtest
runtime. The adapter is built from `tools/native-test-runtime.rs` with the
pinned construction SDK. See the libtest source and license record in the
[complement inventory](../../../../upstream/complement-imports.json).

The installed compiler contract exercises an unchanged default-profile
binary using `cargo test --no-run` and `cargo test`. It checks the test list,
filtering, the ignore reason, expected panic matching, normal output
capture, shown output, and the exit status from a deliberately wrong panic
expectation. It also verifies that the ordinary program entry is not run.
The development candidate passed the seven entry tests and all 33 installed
compiler contract tests.

The observed fixture uses Linux x86-64 GNU, edition 2024, unwind, no dependent
crates and unit-returning tests. A subsequent development package also passed
ordinary `cargo test --lib --no-run` and `cargo test --lib` for a library with
a custom target name and source path. Cargo supplies the source and crate name
for this target and explicitly allows the pinned compiler's
`dead_code_pub_in_binary` lint, whose default level is already Allow.

The default-profile binary harness exercises the native `Result` and
`ExitCode` return forms. An error result prints its original Rust Debug
representation. The pinned libtest runtime reports nonzero termination as a
failed test and exits with 101. Both the pinned Rust and Lustr artifacts
produced the same output for the failing result and explicit exit-code cases.
The default-profile unwind gate separately exercises destructor order,
`catch_unwind`, resumption, and a joined thread's panic payload.

The next development package also selected and executed a dependency-free
integration test from `tests/behavior.rs`. Cargo built the normal application
and supplied its `CARGO_BIN_EXE_*` value to the independent test crate.
The installed gate exercised `cargo test --test behavior --no-run`, followed
by the actual test and a separate run of the original application.

General `Termination` implementations, dependent integration targets,
dev-dependencies, doctests, custom harnesses, and captured backtrace parity
remain open.
No complete-work speed comparison was run.
