# Hosted qualification checkpoint — 19 November 2026

Observed at **07:13 UTC**, for PR #30 head
`bd1d2d0067decc1771eed98884e64ff1fd12b3d4`. GitHub ran the pull-request merge
`3dc9ecbc4c70a03142e71bab0bc0492906fe2443` against PR #29 head
`a9fd4c6336b1e39c4f584f7da50a437c5c248a0d`. These identities come from the original
runner provenance and checkout logs. The accompanying
[verification record](verification.json) identifies the jobs and the SHA-256 of
privately retained original logs.

## Completed results

[Compiler set run 44476177032](https://github.com/glopyglerky/lustr/actions/runs/44476177032)
has completed these jobs successfully:

| Qualification | Job | Observed scope |
| --- | --- | --- |
| Checked semantic inspection | [135397195376](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195376) | Pinned nightly-2026-11-05 format/build and 12 semantic/artifact-interoperability tests. |
| Declared source integration | [135397195485](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195485) | Candidate source guard and seven source-guard controls. |
| Actual Lust subset | [135397195393](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195393) | Two explicit actual-Lust acceptance/rejection and external-read invalidation tests. |
| Cargo compatibility, Linux | [135397195284](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195284) | Independent compatibility coordinator and its process/argument/qualification tests. |
| Cargo compatibility, macOS | [135397195419](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195419) | Same compatibility scope on macOS. |
| Cargo compatibility, Windows | [135397195531](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195531) | Applicable coordinator tests on Windows; Unix-only cases are not claimed. |
| Minimum Rust | [135397195300](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195300) | Independent coordinator build and tests. |
| Cranelift with unwinding | [135397195003](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195003) | Four explicit Cranelift behavior tests, one bidirectional LLVM/Cranelift Rust ABI test, coordinator tests, and Cargo/coordinator unit qualification of the native compiler project. |

The Cranelift job used upstream backend source
`8edf4d040acb77048e04b52a379e93dccae888e6`, built with unwinding, and
nightly-2026-11-05 (`rustc 1.101.0-nightly`, `2d20b42b8`). It also completed its
existing compatibility-coordinator measurements and retained
[artifact 14507495716](https://github.com/glopyglerky/lustr/actions/runs/44476177032/artifacts/14507495716).
Those measurements concern that separate coordinator/backend route. They do not
qualify the integrated native compiler's latency or the unchanged-project mixed
campaign. This checkpoint makes no new general speed claim.

## Repaired qualification blockers

The preceding [run 44474781550](https://github.com/glopyglerky/lustr/actions/runs/44474781550)
at PR head `724e1d03e0c732b270e5734846e86899321a7a11`, runner merge
`94a29a611c11b44ff71af8d44f003de32d8f91fd`, stopped semantic inspection on one
rustfmt expression and source-built mixed qualification on an incomplete build
identity fixture. The fixture omitted both dependency lockfiles now required by
the production build helper. The repair copies the actual locks and checks that
changing either changes the identity, restoring either restores it, and removing
either fails with the required-lock diagnostic. The product requirement remains
intact. At the new source, formatting and the actual TinyCC symbol/rename controls
passed; source toolchain construction then began.

## Historical entry comparison

[Run 44476176976, job 135397179809](https://github.com/glopyglerky/lustr/actions/runs/44476176976/job/135397179809)
failed its applicability guard before package construction. The frozen entry-only
baseline `d57f485eeb2b638d1f95a0594294d11e8001fe8e` has different
`compiler/native`, `crates/lustr-native-core`, and `upstream/bluefe-core.pin` inputs.
The original same-runtime comparison therefore cannot qualify this source. Its
29 comparison controls, 11 diagnostic controls, and 16 applicability controls
passed. **No compiler build, runtime test, or timing ran in that comparison.**
The frozen baseline and the guard were preserved.

## Still pending at this checkpoint

The source-built mixed job
[135397195435](https://github.com/glopyglerky/lustr/actions/runs/44476177032/job/135397195435)
was building the optimized local toolchain. Its later shared-service, mixed
program, failure/recovery, public-entry transport, and performance results were
not available.

Both Linux and macOS native jobs were in the full native gate in each of
[runs 44474781566](https://github.com/glopyglerky/lustr/actions/runs/44474781566)
and [44476176941](https://github.com/glopyglerky/lustr/actions/runs/44476176941).
The former retains head `724e1d03e`; the latter retains head `bd1d2d006`. Both
heads have native directory tree `80a66542b8dc348b036e2974ab2e6e0c42a433fe`.
This directory identity does not erase the separate whole-source identities or
qualify changes to imported services outside that directory. Both hosts in both
runs had passed service-dependency controls, actual sanitizer capability probes,
portable process/IPC/SDK checks, and both Cargo-layout transition checks. Full
native, sanitizer corpus, exhaustive Unicode, daemon, and later runtime results
remain pending until their original logs report completion. No installable
package or complete integrated compiler is admitted by this checkpoint.

