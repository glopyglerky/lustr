# Historical workflow control repair

[Historical evidence job
135661773915](https://github.com/glopyglerky/lustr/actions/runs/44553094553/job/135661773915)
failed in its workflow assertion. The native compiler-set job now has a valid
supplemental qualification upload as well as the original selector evidence
upload. The assertion unpacked every upload in that job as a single value and
raised `ValueError: too many values to unpack` before checking its contents.

The failed checkout was `66963315e9736ce9f151ce8ccac248d35a01d048`, with exactly
published head `7c3d095fa8971a4c38aa0edf62e8fcfd7a51589a`'s tree
`2a6a51f099bdcdb74871fdc2e65a4df87aa0c867`. [hosted-source.json](hosted-source.json)
and [the complete original log](job-135661773915.log.gz) preserve the failure.
The preceding 43 comparison, 8 startup and 14 assessor controls passed; the
applicability suite had 15 passes and one error. Subsequent historical replay
steps were skipped and remain unclaimed.

The corrected [control](../../../../tools/test-warm-applicability.py) selects
the exact `native-entry-and-services` job and requires exactly one
`exclusive-entry-and-shared-services` artifact upload. Other artifact uploads
can coexist. Every prior assertion remains: the actual constructed package,
selector build outcome, cancellation guard, daemon requirement, retention after
failure, all three raw evidence directories and no `continue-on-error`.

The original failure reproduces locally. After the repair, all **19** applicability
controls pass. Nine mutations verify rejection of missing, renamed, duplicated
or misplaced uploads, a duplicated job, each required evidence directory moved
to an unrelated upload, and removal of `if: always()`. An independent read-only
review found no blocking scope or acceptance issue in the tested source.

[verification.json](verification.json) binds the before/after commands, source
hashes, original compressed stdout/stderr, counts and review identity. The
native source tree remains `bf933c382fd99e52dd2586ebc46f0d0aceaa71a8`, and the
integration patch blob remains `d3777126382c8c592e4130cdc67abf9efe6b2bb8`.
No product source, workflow, fixture or historical record changed. This local
proof runs source/evidence controls and establishes no compiler or speed claim.

The separate [mixed performance reporting
run](https://github.com/glopyglerky/lustr/actions/runs/44553094603) succeeded on
the same published tree. Its reporting and historical replay results retain
their existing scope. This repair has not yet received a fresh hosted result.

Compressed streams preserve their original bytes. [SHA256SUMS](SHA256SUMS)
records file hashes; `verification.json` also records uncompressed stream
hashes. The checksum list excludes itself and this README.
