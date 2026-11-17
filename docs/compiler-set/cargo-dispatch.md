# Direct Cargo entry

The packaged `lustr` entry calls the shared Cargo coordinator through
`cargo_lust::run_cli_with_default_root`. Previously it replaced itself with `cargo-lustr`,
which then selected and entered Cargo. The new path removes one executable
startup from builds, checks, tests, no-op commands and planning. Both standalone
Cargo extension binaries remain available and use the same coordinator.

The root package adds a local dependency on the existing dependency-free
coordinator. The coordinator remains independently installable and gains no
dependency on the native compiler. Argument parsing, explicit engine selection,
package-root selection, OS strings, command-scoped settings and Unix process
replacement remain in the same implementation. Cargo still owns the build graph,
artifact freshness, compiler execution and application exit status.

The coordinator's existing prefix parser applies the packaged root only when
the user has not selected an explicit engine, root or `+toolchain`. Arguments
after the Cargo command, including application arguments after `--`, remain
untouched. Previously an all-argument scan could mistake an application's
`--engine` argument for a compiler selection and silently use ambient Cargo.
The root also recognizes the coordinator's `--engine=value`, `+toolchain` and
frontend-job prefix forms. This uses the same parser and validation as the
standalone extensions, without parsing the command twice.

This change does not broaden native eligibility, change a project's debug
profile or bypass compiler analysis. It removes an entry cost; only a fresh
complete benchmark can establish its effect on Cargo or native command latency.

## Regression and qualification

`cargo_entry_runs_without_a_separate_coordinator_executable` copies the actual
entry into a runtime whose path contains a space and whose `cargo-lustr` binary
is deliberately absent. Real matching Cargo, rustc and rustdoc remain present.
The test checks that planning runs no build, a successful check produces a real
Cargo artifact, a repeated check reports that artifact fresh, a program's exit
code 23 survives, and invalid Rust still fails with Cargo's code 101 and its
type error. Plans additionally verify that application options cannot change
the compiler selection, while explicit engine and `+toolchain` options do.

The predecessor entry fails this test when it tries to execute the absent
coordinator. The changed development entry passes with the matching Rust SDK.
That local test qualifies this dispatch behavior, not a complete rebuilt native
package or a speed result. The source-built workflow executes the same test in
the full entry/runtime contract before measurements.

After building the complete package using the existing
[source toolchain instructions](mixed-compiler.md#build-and-qualify), run:

```sh
cargo test --features source-native-runtime --locked --offline \
  --test compiler_contract -- --ignored --exact \
  cargo_entry_runs_without_a_separate_coordinator_executable
```

Use the source-builder's matching `LUSTR_CORE_ROOT`, `LUSTR_LUST_ROOT`,
`LUSTR_NATIVE_BUILD_OUTPUT`, `LUSTR_TEST_PACKAGE` and new `LUSTR_TEST_OUTPUT`
directories, as in the full contract step. Old package or compiler identities
cannot qualify this source.
