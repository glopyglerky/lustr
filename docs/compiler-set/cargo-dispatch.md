# Direct Cargo entry

## Direct compiler entry inside Cargo

The assembled mixed toolchain now provides `bin/rustc-cargo` as a direct link
to the existing mixed Rust/Lust driver (`bin/rustc-full`). For Cargo invocations
only, the coordinator sets `RUSTC` to that direct compiler. Previously every
Cargo rustc command first entered the public `bin/rustc` link to `lustr`, which
then executed the same mixed driver. Eliminating this redundant executable
entry leaves the actual compiler, checked native-object integration, Cargo
metadata, dependency graph, diagnostics and compiler flags unchanged.

`bin/rustc` remains the public Lustr entry. Direct single-file commands still
have the original Lust-supported fast path. A packaged toolchain without
`rustc-cargo` retains its previous `bin/rustc` route, so existing BlueFe
sysroots and standalone coordinators do not require a new binary. The toolchain
assembler verifies that the direct Cargo link and `rustc-full` resolve to the
same compiler executable; the coordinator's public plan test covers selection
and fallback. The existing source-built mixed Cargo, metadata, macro, no_std,
link-recovery and paired performance campaign must qualify the new path.

This removes one process entry per Cargo compiler invocation. It does not
remove Cargo's own orchestration cost, make unsupported bodies eligible for
native generation, or establish an end-to-end speed gain until the matched
campaign completes. Preserve the original slower measurements.

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

The entry also recognizes the packaged Cargo's standard subcommands, including
`metadata`, `clean`, `tree`, `bench`, `doc`, `rustc` and `rustdoc`, and its built-in
`b`, `c`, `d`, `r`, `t` and `rm` aliases. Cargo receives their arguments unchanged
and owns alias expansion. Previously these commands fell through to the full
Rust compiler as source arguments. For example, `lustr metadata --offline`
failed because rustc does not accept Cargo's `--offline` option.

Explicit native commands such as `lustr build source.rs` retain their existing
selection priority. A direct `.rs` path, compiler options or an unknown bare
name retain their compiler interpretation. The executable named `rustc` always
interprets its arguments as compiler input, even when a source file is named
`clean`. For a custom Cargo alias, select the coordinator explicitly, for example
`lustr --bluefe-root /absolute/package verify-entry`. This avoids reading Cargo
configuration or guessing whether an unknown name is a source path or an alias.

This change does not broaden native eligibility, change a project's debug
profile or bypass compiler analysis. It removes an entry cost; only a fresh
complete benchmark can establish its effect on Cargo or native command latency.
The completed combined measurement is recorded below.

## Regression and qualification

`cargo_entry_runs_without_a_separate_coordinator_executable` copies the actual
entry into a runtime whose path contains a space and whose `cargo-lustr` binary
is deliberately absent. Real matching Cargo, rustc and rustdoc remain present.
The test checks that planning runs no build, a successful check produces a real
Cargo artifact, a repeated check reports that artifact fresh, a program's exit
code 23 survives, and invalid Rust still fails with Cargo's code 101 and its
type error. Plans additionally verify that application options cannot change
the compiler selection, while explicit engine and `+toolchain` options do.
The same actual-entry test now parses real Cargo metadata JSON, exercises the
`b`, `c`, `r` and `t` aliases, executes one passing test, and verifies that
`clean` removes the built executable. An explicitly selected custom alias stays
under Cargo's configuration handling. Direct Rust metadata emission and a
`rustc` invocation with a source named `clean` check the compiler boundaries.

The original predecessor entry fails when it tries to execute the absent
coordinator. The later command-dispatch predecessor fails the new metadata
case with rustc's `Unrecognized option: 'offline'`. The corrected development
entry passes the full expanded test with the matching Rust SDK, and all three
root-library tests pass.
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

## Hosted complete-package result

The expanded direct-entry regression passed inside the sixteen-test contract
in [run 44179667793](https://github.com/glopyglerky/lustr/actions/runs/44179667793/job/134380277750).
The same source-built package completed the fixed paired Cargo/native campaign
and the normal-profile coordinator project campaign. See [PERFORMANCE.md](../../PERFORMANCE.md)
for every timing and regression. Its body-edit Cargo win does not prove the
size of this entry change alone. The job remains failed because its separate
public-slice recorder failed before timing, with the original failure retained.
