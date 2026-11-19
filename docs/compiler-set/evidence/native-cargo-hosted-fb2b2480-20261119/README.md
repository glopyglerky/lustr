# Hosted native Cargo outcome at fb2b2480

The guarded constructor and required retained campaign passed. The job overall
failed because an independent root-test transport fixture counted the old wire
frame. That failure prevented execution of the five installed-package contracts.
This record preserves both outcomes.

[Compiler-set run 44546561681, job 135638575172](https://github.com/glopyglerky/lustr/actions/runs/44546561681/job/135638575172)
checked out merge commit `5c733f1468b576432f57d48b69590a6982032fc9`. Its tree,
`afcc2cc8a565a8058c91f428e81e465d82896f0f`, exactly matches published head
`fb2b24802180321d3bbc534dee7083fb2a385d6b`. The original decoded complete job
log is stored as `job.log.gz`; its uncompressed and compressed hashes are in
`verification.json`. API job/source metadata is preserved separately.

## Actual results

| Check | Result |
| --- | --- |
| Guarded partial compiler construction | Passed |
| Exclusive public entry controls | 71 passed, 0 unavailable |
| Retained evaluator controls | All 55 executed and passed; no skips, including all five real pinned-oracle controls |
| Required retained histories | 349 unique histories passed across 1,744 versions; exact name set matches the published required inventory |
| Native Cargo process proof | Seven exact native/Rust program results and one compiler-request refusal passed using a real retained daemon |
| Capability result qualification | 17 exact program results and one matching test-harness result passed |
| Root library tests before abort | 3 passed |
| Root entry tests before abort | 35 passed; 1 intentionally ignored child fixture |
| Root compiler-contract target | 23 passed, 1 failed, 5 ignored |
| Explicit installed-package contracts | Not executed after the preceding all-target command failed |

The retained inventory also lists 13 unresolved entries. The successful qualifier
explicitly reports `full_retained_coverage=false`. The 349 required histories do
not establish all upstream histories, all Rust coverage or release admission.

The failing ordinary test was
`transport_peer_consumes_and_validates_the_actual_encoded_request`. It consumed
260 of 262 bytes and failed the existing complete-frame assertion. The exact
failure matches the separately preserved local reproduction. The subsequent
fixture-only repair is recorded under `../transport-fixture-20261119`; it does
not change the production wire or native compiler.

## Observed request process counts

| Request | Cargo processes | Tool-hash processes |
| --- | ---: | ---: |
| Fresh explicit package | 5 | 4 |
| Initial ordinary request | 0 | 0 |
| Unchanged request | 0 | 0 |
| Body edit | 0 | 0 |
| Feature selection | 3 | 0 |
| Feature recovery | 0 | 0 |
| Existing explicit package | 2 | 0 |

The spy recorded 38 permitted information queries. The test required the same
initial core process identity before and after each request. Its literal
PID/start/executable tuple and compiler binary hashes are recorded in the remote
report, not printed in the job log; this local record does not invent them.
Generated-program and independent-oracle artifact hashes present in the job log
are retained in the parsed result rows. Process counts are functional evidence
of retained planning reuse, not matched latency measurements.

## Artifact retrieval limit

The full original artifact is still preserved on GitHub as
[exclusive-entry-and-shared-services, artifact 14541962757](https://github.com/glopyglerky/lustr/actions/runs/44546561681/artifacts/14541962757).
Its reported size is 584,645,008 bytes. The GitHub download tool rejected it
because its limit is 536,870,912 bytes. The exact error and metadata are retained.

GitHub reports ZIP SHA-256
`114076578e1fecc2dae1c47fc14591bed14e0c973b1e8d0bebf552829a244402`.
That value is **server-reported, not independently verified locally**. No full ZIP
or complete omitted-member hash inventory was downloaded. Per-command original
stdout/stderr/trace files, complete constructor receipts and the original daemon
identity remain in the remote artifact. The compressed complete job log and
parsed results here do not substitute for those original streams. The artifact
metadata reports expiry on 17 February 2027.
