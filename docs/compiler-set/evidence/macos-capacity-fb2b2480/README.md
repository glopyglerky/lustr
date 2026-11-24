# macOS runner acquisition failed before execution

Published PR head: `fb2b24802180321d3bbc534dee7083fb2a385d6b`.
The verified CI source receipt names merge
`5c733f1468b576432f57d48b69590a6982032fc9`. Both commits have tree
`afcc2cc8a565a8058c91f428e81e465d82896f0f`.

| Job | Recorded outcome |
| --- | --- |
| [CI macos-arm64, run 44546561737/job 135638574284](https://github.com/glopyglerky/lustr/actions/runs/44546561737/job/135638574284) | Cancelled before acquiring a runner. The public job annotations report unsuccessful hosted-runner acquisition and three macOS arm64 capacity notices. |
| [Planning helpers macos-26, run 44546561681/job 135638575181](https://github.com/glopyglerky/lustr/actions/runs/44546561681/job/135638575181) | Same acquisition failure and capacity notices, plus an internal server error. |

Both were created at `2026-11-19T14:41:53Z` and completed as cancelled at
`2026-11-19T14:56:54Z`. Each has `runner_id=0`, an empty runner name and no steps.
The API's `started_at` field equals creation time; it does not establish runner
or compiler execution. Both job-log requests returned `BlobNotFound`.

`annotations-observed.json` preserves the exact public UI messages and verified
URLs. The annotations were read by expanding each job's annotation panel after
the connector exposed the check records but could not retrieve annotations.
No sign-in, cancellation, rerun, source change or runner-label change was made.
`cancellation-details.json` preserves the selected raw job/check fields and log
retrieval failures. The later UI observation includes an additional internal
server error on the planning job, beyond its earlier four-annotation API count.

`source-receipt.json` retains the source/event fields, original receipt hash and
locally verified provenance ZIP hash. Trigger-actor labels are omitted; they do
not identify a cancellation actor. `source-identity.json` records the exact
signing helper, patch, workflow and odd-path fixture hashes for this source.

This is a hosted runner acquisition/capacity failure before execution. There is
no Darwin signing, failed-publication preservation, odd-path, retained-process
or full-matrix pass from these jobs. The prior actual macOS odd-path failure
remains preserved in [its original record](../macos-odd-path-62da158b/README.md).
A future real macOS execution is still required.
