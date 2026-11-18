# Preserve the missing lust-port human report

The hosted Linux native gate at run `44328101820`, job `134884752622`, failed
`scan_fixture::human_output_groups_findings` because its human output did not
contain `attrs.unsupported`. The assertion did not include that output, so the
failed report is absent from the saved log. The cause remains unassigned.

The executed merge `c3513db6bc32a57ec4ae48bc0e0c504582d3a5b5` and published
`ff9c0de9d2a2c89edd8df48e74e64a5c9483f232` have the identical full tree
`52ab272cbed5d88b7126d35da5a7f7ff2607cc24`. The existing qualification step had
passed 15 client and 387 native unit tests. The subsequent release build and
diagnostic registry check passed, then the tool suite passed eight unit tests and
four of five integration tests. Later differential and runtime gates did not run.

One bounded local run of the unchanged tool source, with the exact CI Rust
1.99.0 compiler and Cargo 1.99.0, passed all eight unit and five integration
tests. The original Cargo.lock was retained. This does not clear or explain the
hosted failure, and the test was not repeated until it passed.

The diagnostic change adds four lines to the failing test. It prints the actual
fixture path, process status, complete stdout and stderr before the existing
assertions. All fixtures, commands and assertions remain unchanged. Cargo will
show this diagnostic when the test fails; `--nocapture` also exposes it on a
successful diagnostic run. No scanner, formatter or compiler behavior changes.

The logging change has independent source review. One fixed instrumented
exact-test execution is pending, with its source recorded separately from the
unchanged run. The native integration declaration remains explicit because this
test lives under the protected native source tree. Candidate source checks do
not admit a release.

The original hosted artifact is `14437914129`, SHA-256
`0093ad3c49171757f0ca9c6e8cbf884216677338eac786b5b19439b0f8d16478`.
The decoded full job log is 167361 UTF-8 bytes, SHA-256
`83177df25fb74c79539c2d5197bc16c0c0223c7d0b06fbd5e20b6a8ac720a140`.
The original failure, local success, exact compiler/source identities and any
instrumented outcome must remain together in the evidence.
