# Fresh macOS jobs failed to acquire runners

These are observations of published head
`7c3d095fa8971a4c38aa0edf62e8fcfd7a51589a`. They do not reuse the cause recorded
for the preceding `fb2b2480` attempt.

| Current job | Public annotations observed |
| --- | --- |
| [CI macos-arm64, run 44553094710/job 135661771391](https://github.com/glopyglerky/lustr/actions/runs/44553094710/job/135661771391) | Hosted-runner acquisition failed after multiple attempts; three notices describe macOS arm64 capacity constraints. |
| [Planning helpers macos-26, run 44553094694/job 135661771685](https://github.com/glopyglerky/lustr/actions/runs/44553094694/job/135661771685) | The same acquisition failure and three capacity notices, observed on this job's own page. |

Both jobs were created at `2026-11-19T15:17:23Z` and cancelled at
`2026-11-19T15:32:24Z`. Both have no steps, `runner_id=0` and empty runner names.
The API's creation-valued `started_at` field does not establish execution.
Neither current annotation panel showed an internal-server-error notice.
Both current job-log requests returned `BlobNotFound`.

`annotations-observed.json` preserves the exact current messages, verified URLs
and observation time. `cancellation-details.json` retains the selected raw API
fields and log errors. The connector exposed the check records but not their
annotation endpoint, so the public annotation panels were expanded in a browser.
There was no sign-in, cancellation, rerun or runner-label change.

Both original workflow provenance ZIPs were downloaded and their complete
SHA256 values matched GitHub metadata. `source-receipts.json` retains their
identities and source fields, omitting trigger-actor labels that cannot identify
a cancellation actor. Both receipts identify merge
`66963315e9736ce9f151ce8ccac248d35a01d048`, whose tree equals the published head:
`2a6a51f099bdcdb74871fdc2e65a4df87aa0c867`. `source-identity.json` binds the
signing, workflow and odd-path inputs to that source.

This attempt supplies no Darwin signing, failed-publication, odd-path or retained
compilation result. The original actual [macOS odd-path failure](../macos-odd-path-62da158b/README.md)
remains preserved. Real platform execution is still required.
