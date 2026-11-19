# Native Cargo build-script layout

## Reader correction

The candidate reader supports both layouts of declared Cargo source
`c07ac2c4d12237effc4b94f4543769b421790045`. It reads the legacy `output` file or
the new `run/stdout` file beside `OUT_DIR`. A missing or ambiguous saved output
fails the request; it cannot become a successful build script with empty
configuration and environment directives.

Cargo's `root-output` records where the script originally ran. It is not an
equality certificate for the current directory. The reader follows the exact
[Cargo parsing and relocation rule](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/custom_build.rs):
rewrite the old output directory in each directive value to the current one.
It also matches Cargo's whitespace handling and ignores non-UTF-8 diagnostic
lines. Recognizing the new `build_script_build` executable name restores the
actual Rust compiler identity to the build-script record.

The record version advances to `LBS5`, so earlier records that may contain
silently lost directives cannot qualify reuse. An explicit compatibility-layout
setting participates in the setup key. The ordinary unchanged-record path
still checks its existing inputs without invoking Cargo or reading the new
saved-output paths.

The [repair record](evidence/native-cargo-layout-repair-20261119/verification.json)
preserves the original native failure: the existing real-Cargo transition test
received empty cfg/env lists instead of the script's actual values. An isolated
qualification includes the exact build-script and probe modules and unchanged
native hashing and Cargo command/path helpers. All six tests pass under both
layouts, including the existing script/source/input/output edit sequence with
its assertions unchanged. A separate reviewer moved an actual Cargo target
directory and verified that the corrected reader reproduces Cargo's remapped
directives byte for byte. The review's earlier equality-check objection and its
counterexample remain preserved.

The first full local native test build was terminated by SIGKILL before tests
ran. Its output is retained; the cause has not been established. The component
passes are not a substitute for a completed full compiler build or hosted gate.

CI now records the inherited Cargo layout choice without forcing the old one.
The additional qualifier discovers the real native test executable through
Cargo JSON and executes the existing build-script transition test once with
Cargo's default layout and once with its explicit compatibility setting. It
retains the binary hash and both original outputs. These checks require the
exact declared Cargo source before using its private compatibility setting.
Other Cargo-probe and general frontend-integration work remains distinct; this
repair makes no performance claim.

## Earlier compatibility qualification

The earlier native build-script reader read Cargo's saved directive output
from the `output` file beside `OUT_DIR`. That is the existing documented
[compatibility-layout contract](native-driver.md), and the same setup requirement
was already identified by the
[complete native warm qualification](complete-native-warm-comparison.md).

The declared Rust SDK contains Cargo source
`c07ac2c4d12237effc4b94f4543769b421790045`. Its default new build-directory layout
breaks that reader assumption. In the current checked-request driver
qualification, the complete 133-test group passed 132 tests and failed
`driver::buildscript::tests::script_reruns_when_what_it_read_changes`: it observed
empty cfg and environment lists rather than the build script's actual output.
With the explicit compatibility setting, all 133 passed. The original outputs
and their hashes are preserved in the
[source-bound record](evidence/native-cargo-layout-20261119/verification.json).
The [request qualification](evidence/checked-request-ownership-20261119/verification.json)
retains the exact compiler, source and other limits of those runs.

At published source `0170b9834a15f104f303d4497f93a00773564bd0`, both native CI
jobs required the exact Cargo source identity before using its private
compatibility switch. When no explicit value was inherited, they set
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1` for the native qualification.
An explicit inherited value remained unchanged. Each job retained the actual Cargo
identity, chosen value, and whether the value was inherited or defaulted.

That earlier change qualified the compatibility layout; it did not implement
the new layout. The source-bound historical result remains valid for its
recorded scope. The reader correction above replaces the default override while
preserving the original test assertions, failures and sanitizer requirements.
