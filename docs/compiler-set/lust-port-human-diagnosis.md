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
exact-test execution passed at `fc94bc2b892a7e34e3133d4c6fb1e19a2e3ee8b0`.
Its retained human report contains seven files, three findings across active,
target-specific and test-only scope, one parse error, and two occurrences of
`attrs.unsupported`. Four other integration tests were filtered in this
diagnostic invocation. The scanner executable SHA-256 remained unchanged across
the logging change:
`5cb69d5e5d797afd3da06657f4d90fa4347ea89a60b430d79da628e4e22dc7b6`.

The [portable evidence bundle](evidence/lust-port-human-2026-11-18/README.md)
retains the original hosted failure, the one unchanged local success and the one
instrumented local success. No run replaces another. The native integration
declaration remains explicit because this test lives under the protected native
source tree. Candidate source checks do not admit a release.

The original hosted artifact is `14437914129`, SHA-256
`0093ad3c49171757f0ca9c6e8cbf884216677338eac786b5b19439b0f8d16478`.
The decoded full job log is 167361 UTF-8 bytes, SHA-256
`83177df25fb74c79539c2d5197bc16c0c0223c7d0b06fbd5e20b6a8ac720a140`.
The original failure, local success, exact compiler/source identities and any
instrumented outcome must remain together in the evidence.

The hosted failure remains unresolved. This change makes its missing report
observable if it happens again; it is not a scanner correction or an assertion
relaxation.
