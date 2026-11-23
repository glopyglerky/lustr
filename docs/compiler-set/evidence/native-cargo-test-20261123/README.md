# Native Cargo binary unit tests

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
crates and unit-returning tests. General `Termination`, integration targets,
dev-dependencies, doctests, custom harnesses, and captured backtrace parity
remain open. No complete-work speed comparison was run.
