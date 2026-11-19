# Retired coordinator acceptance

Recorded 19 November 2026. The [exclusive coverage contract](exclusive-coverage-contract.md)
controls this correction. The coordinator crate remains historical planning
research. Its executables must refuse compilation; the integrated native entry
has separate ownership, correctness and performance acceptance.

## Failure found after route retirement

PR #33 removed coordinator execution, but ordinary Cargo test discovery still
found tests requiring it. The compiler-set workflow selected only `--lib`, so
the contradictory executable tests were outside that gate.

At published source
[`ec99578d`](https://github.com/glopyglerky/lustr/tree/ec99578d51c93674af3a1bcf9c182f4ad4b17d89),
running `cargo test --manifest-path crates/lustr/Cargo.toml --locked --offline
--test entrypoints` failed with exit 101. The stock control program had run and
returned 23. The retired CLI correctly returned 2, but the test required 23:

```text
tests/entrypoints.rs:67
left: Some(2)
right: Some(23)
lustr: the independent compiler coordinator has been retired; no alternate compiler was invoked
```

Restoring compiler delegation would make that obsolete test pass while violating
the product contract. This correction replaces the conflicting expectation.

## Current executable acceptance

`crates/lustr/tests/entrypoints.rs` now runs all three retired executables and
real Cargo extension lookup. A separately compiled tool witness records any
execution. The test verifies that witness works before placing it at compiler,
Cargo, native-helper and wrapper paths.

Compilation, engine selection, BlueFe selection, frontend selection, inspection
commands and unknown operations must return the retirement error without tool
execution or Cargo artifacts. Non-UTF-8 arguments receive the same refusal on
Unix. Only the standalone help and version commands succeed; appending a build
request cannot turn either into a successful compilation.

The current tests do not skip obsolete compiler requirements. They remove the
retired executor expectations and enforce its replacement behavior through real
processes. The Linux execution observed 62 refusal invocations, 12 successful
information invocations and two positive tool-witness controls. The standard
suite runs without `--lib` in Linux, macOS, Windows and minimum-Rust CI jobs.

## Source and fixture disposition

Original tests remain available in immutable source history. No branch or old
measurement was removed.

| Original source at `ec99578d` | Current disposition |
| --- | --- |
| [entrypoints.rs](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/tests/entrypoints.rs) | Replaced whole-project execution acceptance with actual retirement/refusal acceptance. |
| [process.rs](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/tests/process.rs) | Removed the obsolete executor process suite. Its argument/stdio/exit/alias and wrapper observations remain in history; current refusal tests verify that these old requests cannot invoke another compiler. |
| [frontend.rs](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/tests/frontend.rs) | Retained pure argument-boundary and byte-preservation planning tests. Removed CLI plan output and complete-compiler execution expectations. |
| [bluefe.rs](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/tests/bluefe.rs) | Retained historical plan representation and invalid-selection checks. Removed whole-project, cross-target and CLI execution assertions. |
| [qualification.rs](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/tests/qualification.rs) | Preserved independent stock-reference fixture execution, invalid borrow rejection, tests/doctests, variadic file ABI and real Cargo configuration-ordering checks. Removed routes through the retired coordinator, including optional native/Cranelift selectors. |
| [mixed_abi.rs](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/tests/mixed_abi.rs) | Preserved the fresh stock-reference ABI fixture, runtime/destructor/panic checks and evidence parser. Removed coordinator and optional Cranelift execution expectations. |

The unchanged fixtures retain build scripts, procedural macros, async code,
generic associated types, invalid borrowing, aggregate/enum/niche layout,
allocation ownership, cross-crate unwinding, thread-local state, atomics and
callbacks. Their passing stock-reference controls state expected behavior.
They do not establish Lustr implementation of those capabilities.

Cross-target operation, compiler-wrapper semantics, native external-read
invalidation and nested-build behavior remain required under the
[completion checklist](completion-checklist.md). Historical controls for the
retired executor cannot qualify those behaviors in the integrated compiler;
their preserved sources remain useful when qualifying the actual owner.

## Observed verification

On Linux x86_64, the full ordinary crate suite passed with the declared Rust
SDK `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`:

```sh
cargo test --manifest-path crates/lustr/Cargo.toml --locked --offline
```

The execution used separate generated-output directories. It observed 39
passing tests and zero ignored tests: 22 library tests, four BlueFe plan tests,
two retired-entry tests, four frontend plan tests, two ABI reference/evidence
tests and five reference qualification tests. The binary and documentation
test targets contained no tests. Formatting and whitespace checks passed.

This local execution used the declared nightly SDK. Rust 1.85, CI Rust 1.99.1,
macOS and Windows execution require their own hosted results. This is acceptance
cleanup, not full Rust coverage, a new native compiler build or a speed result.

## Retired mixed qualification tooling

The same audit found two obsolete tools outside Cargo test discovery:

| Original source at `ec99578d` | Current disposition |
| --- | --- |
| [test-mixed-workflow.py](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/tools/test-mixed-workflow.py) | Removed the unused test for the retired `mixed-source-build` job, whole-compiler constructor and general mixed qualifier. It could not validate the replacement native job and was no longer invoked by CI. |
| [profile-mixed-compiler.py](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/tools/profile-mixed-compiler.py) | Removed the live qualifier launcher and its required `rustc-full` sibling. Command execution now refuses before resolving tools or creating an output directory. All six parsing/rendering functions and schema constants remain byte-for-byte unchanged. |

The unchanged `tools/test-mixed-profile.py` still tests all historical schema
meanings, including the retired successful complementary classification. That
preserves the interpretation of old records; it does not restore the old
compiler route. Frozen reporters and original evidence under `performance/`
remain unchanged. The [phase-attribution document](mixed-phase-attribution.md)
now states this historical scope and points to the existing replay commands.

Focused verification passed: 36 profile parser tests; all four frozen replay
corruption tests; and the actual frozen replay of 130 original inputs,
26 profiles, 15 commands and 35 successful native records. The replay preserved
the original CI run's failed status and reproduced only its separately labeled
derived reconstruction. An actual retired CLI invocation returned status 1
without creating its requested output directory. These results verify evidence
preservation and tool retirement, not compiler execution or performance.
