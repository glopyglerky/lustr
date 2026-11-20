# Retained metadata construction correction

## Original failed attempt

PR #33 source `cdec0f7a75cc2fe0d1dadbed689b5ae8e9d9e62e` failed both the
[Compiler set workflow](https://github.com/glopyglerky/lustr/actions/runs/44686636539)
and the broader [CI workflow](https://github.com/glopyglerky/lustr/actions/runs/44686636436).
Their actual merge checkout was `955a266f17a5732ad455ffc6c550c18ad1833100`.
Its tree `7a923a1653e83e68678c665bfeccd1252fb0dbe0` equals the published PR source tree.

The native compiler used a nonexistent `FnDef.span` at two sites. The retained
metadata caller also applied `?` to an I/O publication result in a function
returning `String` errors. These are source compilation failures.

The metadata job stopped before either the original one-shot or the new retained
candidate campaign. It therefore observed zero candidate runtime assertions and
zero candidate traces. Eleven evaluator controls passed before construction,
but those controls do not qualify the compiler. The other eight Compiler set
jobs passed; the complete workflow failed. The broader Linux and macOS jobs
stopped at the same native compilation errors, and their later dependent checks
did not produce qualifying results.

[first-failure.json](first-failure.json) binds the source, jobs, original record
hashes and artifact upload digests. Original construction and evaluator logs are
copied from the downloaded metadata artifact without text editing. The full
original job logs and artifact archives remain retained separately. The job-log
hashes identify the GitHub connector's decoded text re-encoded as UTF-8; artifact
hashes identify the original ZIP bytes and match GitHub's returned upload digests.

## Qualification boundary

This directory initially records the failed source only. Authored fixes, authored
histories and older-source passes are not a successful result for a corrected
candidate. The retained metadata campaign must construct and run on the corrected
source, with its ordinary Rust readers and consumers, real edit and failure
histories, and complete candidate execution traces.

Full Rust coverage, public compiler integration, compiler-wide `no_std`,
transport independence of the installed compiler and matched speed remain
separate obligations. This bounded research campaign cannot establish those
claims.

## First runtime attempt after construction repair

Source `1944f65fa7bee550da4343ddf86ea8325afee754` built the corrected
metadata caller and passed the existing one-shot scalar campaign. All 27 retained
evaluator controls passed. The new retained campaign then failed at the `insert`
archive check, before that inserted function reached its ordinary Rust consumer.
The seven preceding consumers completed 1,835,064 actual assertions. These are
partial results from an overall failed campaign, not its full expected count.

The actual history-session trace accounted for the observed request intervals,
including zero artifact-tool calls in both observed no-op requests. Later histories,
recovery, fresh sessions, SDK refusals and final complete verification had not run.
[runtime-failure-1944.json](runtime-failure-1944.json) records this exact boundary,
the verified upload digest, candidate binary identity, original logs and independent
checks of original consumer results and trace bytes.

The failure exposed an evaluator format error. The actual GNU archive index has
eight complete symbols followed by the one alignment NUL emitted by the pinned
LLVM writer. [archive-format-review.json](archive-format-review.json) binds that
source and the original index bytes. Correction
[`1573da06`](https://github.com/glopyglerky/lustr/commit/1573da06d44b6ed20483a0735ba12f10e2c17cd9)
reads the declared name count, then admits only the specified alignment. It keeps
nonempty and unique names, real member offsets and invalid-tail rejection. Three
new control methods accompany the parser change. Compiler source and the complete
required history and consumer outcomes remain unchanged. The corrected evaluator
and complete retained campaign still require exact-source hosted execution.

## Native diagnostic expectation correction

Source `b5b55b7813843956062fc40180a759c1b4569541` passed all 30 evaluator
methods and advanced through 13 successful consumers, totaling 3,670,128 actual
runtime assertions. Its retained campaign failed at the fourteenth observed
transition, `invalid-private`. The native type error, source path, line, column
and span were correct. The authored exact-text expectation omitted the driver's
normal final summary, including the attempted source path and diagnostic count.

[runtime-failure-b5b55b78.json](runtime-failure-b5b55b78.json) records the failed
metadata job, original artifact and candidate identities, observed histories,
consumer checks and actual trace boundaries. All four observed no-op intervals
and the invalid request contained zero artifact-tool calls. Later recovery,
fresh-session and SDK refusal histories had not run. No whole workflow conclusion
is inferred from this one completed job.

[native-diagnostic-review.json](native-diagnostic-review.json) verifies that the
candidate's `errors` function is byte-identical to pinned upstream Lust. The
preserved source and stderr show the exact expected native behavior. Correction
[`3fa48e5a`](https://github.com/glopyglerky/lustr/commit/3fa48e5a936f990531d92e9168923964f532e72b)
requires the complete original diagnostic, retaining exact equality and the
existing error-code, source-span and phase checks. Four negative subcases reject
a missing summary, wrong count, wrong path and extra text; these are additions to
an existing method, so the suite still has 30 methods.

Compiler source, the 29-request history and the 25-consumer inventory are
unchanged. The corrected evaluator and complete campaign require their own
hosted execution. Partial passes from the failed source are not qualification of
the new source.
