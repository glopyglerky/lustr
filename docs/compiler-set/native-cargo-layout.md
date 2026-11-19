# Native CI Cargo layout qualification

The native build-script reader currently reads Cargo's saved directive output
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

Both native CI jobs now require the exact Cargo source identity before using its
private compatibility switch. When no explicit value is inherited, they set
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1` for the native qualification.
An explicit inherited value remains unchanged. Each job retains the actual Cargo
identity, chosen value, and whether the value was inherited or defaulted.

No test assertion, failure result, source fixture, sanitizer check or compiler
semantic flag is removed. This change qualifies the currently supported layout;
it does not implement the new Cargo layout. A conflicting explicit setting can
still fail native qualification and is not silently replaced.
