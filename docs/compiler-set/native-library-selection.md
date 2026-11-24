# Native library target selection

The no-daemon native process must select the library named by Cargo's target
metadata when `--lib` is requested. A package name is not necessarily its
library name: Cargo replaces hyphens with underscores by default, and `[lib]
can specify another name. A package-named binary must never substitute for that
library.

This correction adapts the focused selector from Jeff McDonald's
[upstream Lust commit `b1c95c0d5253ee60e286faaeada7c958f1e8b78e`](https://github.com/glopyglerky/lustr/commit/b1c95c0d5253ee60e286faaeada7c958f1e8b78e),
also retained in Lustr PR6's history. Other PR6 language extensions, routing and
qualification changes are not imported. The original commit and authorship
remain in Git history; the current integration patch declares the exact change.

## Failure and correction

At `dec0ce2e3a68b82cac16eae238fc51b16e534e5e`, native `--lib` looked up
the package name and fell back to a target of that name regardless of kind.
A valid `hyphen-lib` library exited 2 because its actual target was
`hyphen_lib`. A valid renamed `app_library` library beside an invalid `app`
binary exited 1 after checking the binary. Matching Cargo accepted both
libraries. Disabling both the native daemon and response cache reproduced
the failures, including after an invalid edit and recovery.

The Lustr entry validates the Cargo target before dispatch. Its daemon request
contains the corrected target name, but a new native process reparses the
original `--lib` arguments. The native resolver must therefore honor the same
kind-based selection. The correction selects only the package's library
target for `--lib`. Actual Cargo metadata identifies library targets as `lib`,
`rlib`, `dylib`, `cdylib`, `staticlib` or `proc-macro`; all six are accepted by
this selector. Other requested kinds retain their current name-based behavior.
Review of the initial carried-forward fix also found that its remaining
name fallback accepted a bin-only package as a library. The actual native
compiler returned 0 while Cargo rejected the request. The adaptation removes
that fallback for libraries and retains the failing first case.
The first strict revision accepted only `lib` and `rlib`, which rejected
previously accepted static, dynamic and proc-macro library targets. Independent
review reproduced all four missing kinds against the original native compiler,
the strict revision and Cargo. Those failures are retained, and the final
predicate recognizes every library kind already admitted by Lustr's Cargo
selection contract while still excluding binaries and other non-library targets.

## Qualification

The [reproduction runner](measurements/2026-11-17-native-lib-selection/runner.py)
records real native client/core hashes, the source tree, matching Rust/Cargo
identities, unchanged fixture hashes, commands, exit statuses and diagnostics.
It compares valid, invalid and recovered states for seven library layouts,
including all six kinds, with the actual Cargo `check --lib` oracle, and requires
a bin-only package to fail target selection. The runner retains each layout's
actual Cargo metadata. Every native request disables the daemon and cache. The
runner prepends the selected Cargo SDK directory to `PATH`, because native
metadata discovery invokes Cargo by name. The original failing evidence is
retained separately from the corrected run.

`native_library_selection_survives_process_startup` in
`tests/compiler_contract.rs` also executes the packaged native and Lustr entries,
requires the successful and recovered Lustr checks to record `lust` /
`upstream-process`, and requires invalid-library edits and a missing library
target to fail. This catches a Rust fallback that could otherwise hide failure
to preserve the supported native path.

The proc-macro fixture contains a private helper rather than a public ordinary
function, which Rust forbids exporting from that crate type. These are compiler
check and target-selection cases; they do not establish proc-macro execution,
library artifact compatibility or dynamic linking.

The [retained qualification](measurements/2026-11-17-native-lib-selection/README.md)
passes all 22 native/Cargo states on x86_64 Linux and the actual Lustr-entry
regression, including 52 captured child-command output pairs. The qualified
native source tree is `cb797ce8ce82192b87b3835267fc223c295dba70`. The focused
entry assembly uses the matching pinned SDK; it is not a complete distributable
package. The original full native, packaged, platform and performance gates
remain required. The source-bound integration remains a candidate and this
correction makes no release or speed claim.
