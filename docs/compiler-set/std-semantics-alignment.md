# Native standard-library alignment with the declared Rust source

The candidate native compiler declares Rust source
[`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`](https://github.com/rust-lang/rust/tree/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494).
Its observable standard-library behavior must use that source. Historical
Rust 1.99 comparison results retain their original compiler and oracle.

The [source-bound verification record](evidence/std-semantics-alignment-20261119/verification.json)
records a failing complete ordinary differential run and the subsequent focused
repairs. The initial run passed 5,432 cases and failed 98, using the unmodified
ordinary differential harness. Every failed case was preserved. A focused run of
the standard-library repairs passed 232 invocations covering 212 distinct
fixtures. That focused result is not a complete corpus pass, sanitizer result,
performance claim, or claim that the integrated compiler is complete.

## Panic locations and observable library results

The implementation previously retained Rust 1.99 source locations and messages
while the candidate oracle used the declared source above. The repair updates
source identities, locations and changed messages from the actual pinned source.
It also follows changed `track_caller` behavior for signed division/remainder and
range validation. Callers that preserve their own internal location, including
`Vec::drain`, `VecDeque::drain` and `Vec::extend_from_within`, retain that location.
The source investigation verified the Git blob identities of 99 retrieved old and
new upstream files; those identities are in the record.

The permanent `std_declared_source_locations` fixture prints hook locations to
stdout. The ordinary harness compares the complete Rust source identity there,
so its existing stderr normalization cannot hide an obsolete source hash. The
original native source fails this fixture; the repaired source passes. Three
Duration panic assertions now require the exact messages in the declared
[`core::time` implementation](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/time.rs).
The assertions remain present.

Other repaired observations are `Rc`/`Arc` type names after their upstream module
move, Unix `ExitStatus` debug formatting, and the invalid append/truncate error
from `OpenOptions`. These are checked against matching Rust outputs, including
existing debug-formatting and process fixtures.

## All OpenOptions flag combinations

The permanent `std_open_options_flag_matrix` fixture enumerates all 64 settings
of `read`, `write`, `append`, `truncate`, `create` and `create_new`, each for an
existing and a missing file. It checks 128 outcomes, including successful file
lengths and the kind and text of every error. The native result matches the
pinned compiler byte for byte; both programs exit successfully with empty
stderr. The original native source differs on 16 outcomes.

The fixture acquires an exclusively owned temporary directory using `create_dir`
and retries only an `AlreadyExists` result. It removes only its own files. The
red control explicitly loads the archived original platform library: an old
compiler binary alone is insufficient because that part of the library can be
loaded from its source tree. An earlier control that accidentally used the
updated library is preserved and excluded from the red result.

## The final cleanup panic hook

Rust runs the panic hook for the final `panic in a destructor during cleanup`
panic before aborting. The pinned
[`core` entry](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/core/src/panicking.rs)
sets `can_unwind=false` and forces no backtrace; the
[`std` dispatcher](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/std/src/panicking.rs)
runs the hook and then aborts. The native runtime had printed a fixed default
message without calling that hook.

The repair supplies the cleanup payload and source location to the installed
hook, preserves the no-backtrace setting when a custom hook forwards to a taken
default hook, and aborts after it returns. In explicit propagation mode it takes
the second pending exception out of the generated hook's propagation path while
preserving the thread's panic count. Otherwise generated hook code returns
before doing its work. The first exception-table-only repair exposed this
additional defect when tested through the source-built TCC route; both failures
are preserved.

The permanent `panic_cleanup_hook` fixture observes the final hook, forwards it
to the default hook, and ensures the catch cannot recover. Direct source-built
native controls compare the abort status, stdout and stderr with the pinned
compiler with `RUST_BACKTRACE=0` and `1`. A separate nested-hook control requires
an immediate abort if that final hook itself panics. Only the OS thread number
in a default panic header is normalized in these focused comparisons.

Private controls also check the hook's `can_unwind` value for normal panics under
both strategies and for the final cleanup panic. Rust needs its unstable feature
attribute for this API. At the runtime-qualification revision, native exposed
the method but rejected that attribute. Those controls disclose the input
difference and are not ordinary differential fixtures or a claim of
feature-attribute support. The ordinary fixture uses stable APIs and identical
source for both compilers.

That investigation also found a false-acceptance defect: native accepted a
program using the unstable accessor without its feature attribute, while the
pinned Rust compiler rejected it with E0658. The bounded follow-up removes the
public accessor while preserving the runtime flag and internal hook runner.
The permanent `tests/fail/panic_hook_can_unwind_unstable.rs` fixture requires
native check and build to reject it with L0104. It remains an invalid-program
fixture; it is not reclassified as complementary work.

## Qualification boundaries

The local native executables use the `gate` Cargo profile. For the existing
script they are copied unchanged into its required `target/release` location;
that directory name does not make them release-profile builds. Exception-table
checks use GCC/g++ and unchanged emitted C, with separately identified composed
runtime checks for the partial-drop repair. These checks do not substitute for
the hosted Clang sanitizer gates or native macOS qualification.

The failing full run and the focused green run remain distinct evidence. The
combined source must pass the complete ordinary corpus and the independent
platform and integration gates before a release claim is made.


## Combined source follow-up

The [combined native qualification](native-combined-qualification.md) records
5,540 passing ordinary cases on the combined source and complete 65-case
unwinding passes in both execution modes after the separately identified
hash-table dependency-location correction. Earlier failing and focused results
above retain their original source identities.
