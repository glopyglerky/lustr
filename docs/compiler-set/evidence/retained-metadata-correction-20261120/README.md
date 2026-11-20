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
