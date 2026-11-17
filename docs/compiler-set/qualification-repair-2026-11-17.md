# Native qualification repair, 17 November 2026

This additive candidate follows PR9 at `baf8b35700b158bdf7fd2752e32d3b3b8ebf4aa9`.
It does not change the imported complete compiler, language coverage, timing
thresholds, benchmark corpus, requested application profiles or default branch.

## Observed failures

Compiler-set run 44046766887, source-built job 133914272305, passed mixed
execution, failed-link cache isolation, retained checks, fresh metadata,
source-hidden mixed-library consumption, the complete 14-test entry contract
and four target-layout comparisons. The performance probe then rejected
pristine Lust's `mode=bodies` result despite zero rechecked/checked bodies and
zero generated/compiled units. It still performed collection (`collect=4`);
this result must not be described as zero total work or a measured speed gain.
No paired timing campaign completed in that run.

Native CI run 44046767054 failed its macOS ARM64 link because process startup
referenced Linux's `personality` execution-domain syscall. This is unrelated
to Rust's exception/unwind personality routine. The front-image feature was
already documented and selected as Linux-only, but runtime environment checks
did not remove that unresolved reference from the macOS binary.

## Changes and scope

The benchmark now parses one real stats record and one real visits record,
requires a nonempty fixture and zero body/code-generation work counters, and
distinguishes direct reuse from retained bodies with collection work. Missing,
duplicate, malformed, conflicting and nonzero evidence fail. Raw diagnostics
and every existing timing/correctness gate remain intact.

Linux-only image entry points and their FFI declaration are compile-time gated.
Other hosts retain normal compiler/daemon operation and explicitly decline
fixed-address front-image re-execution. No fake syscall, panic-abort change,
linker suppression or runtime library substitution is introduced.

Twelve focused evidence tests executed successfully during preparation. The
existing eleven workflow-control tests also executed successfully. These are
harness tests, not whole-compiler or performance qualification. A non-Linux
native unit regression is included; actual macOS compilation and native CI,
Linux differential CI, and the paired performance campaign remain required.
The exact upstream patch and candidate tree identities are regenerated without
changing the pristine reference or granting release admission.
