# Hosted qualification at 7c3d095f

All compiler and qualification steps in this constructed-package job passed. The job overall failed because the new supplemental packager refused two symlinked receipt records. Both outcomes remain visible.

[Compiler-set run 44553094694, job 135661771824](https://github.com/glopyglerky/lustr/actions/runs/44553094694/job/135661771824) checked out merge `66963315e9736ce9f151ce8ccac248d35a01d048`. Its tree `2a6a51f099bdcdb74871fdc2e65a4df87aa0c867` exactly matches published head `7c3d095fa8971a4c38aa0edf62e8fcfd7a51589a`. The native tree remains `bf933c382fd99e52dd2586ebc46f0d0aceaa71a8`.

## Actual results

| Check | Result |
| --- | --- |
| Guarded partial compiler construction | Passed |
| Exclusive public entry | 71 passed; 0 unavailable |
| Ordinary root targets | 93 passed, 0 failed: 3 + 35 + 24 + 31; two additional targets had no tests |
| Explicit installed-package contracts | All five passed, including real socket and terminal-status controls |
| Capability qualification | 17 exact programs and one complete test-harness comparison passed |
| Retained evaluator controls | All 55 executed and passed, no skips, including all five actual pinned-oracle controls |
| Required retained histories | 349 unique histories and 1,744 versions passed; exact required-name equality checked |
| Supplemental packager controls | Passed |
| Supplemental capture | Failed for the two receipt symlinks below |
| Supplemental and original full uploads | Both passed |

The ordinary root run intentionally ignored two child fixtures and the five installed-package controls. The latter five then ran explicitly and all passed. The retained inventory still contains 13 unresolved histories and reports `full_retained_coverage=false`. These results do not establish all upstream histories, all Rust, matched speed, Darwin execution or release admission.

The [complete original decoded job log](job.log), [349 parsed history rows](history-results.json), [exact published inventory](published-retained-inventory.json), [source/job metadata](source-and-job.json) and [verification summary](verification.json) retain their distinct scopes. The log is 565,216 UTF-8 bytes with SHA-256 `8574acbf5a9bcc8a9da5d68870c543adeeb1cad033f78ef1f0abf789066dc3da`.

## Actual retained process observations

The archived report records original core PID 6264 / start ticks 35433, 38 permitted information queries, seven exact program comparisons and one refused compilation-class request.

| Request | Cargo processes | Tool-hash processes |
| --- | ---: | ---: |
| Fresh explicit package | 5 | 4 |
| Initial ordinary request | 0 | 0 |
| Unchanged request | 0 | 0 |
| Body edit | 0 | 0 |
| Feature selection | 3 | 0 |
| Feature recovery | 0 | 0 |
| Existing explicit package | 2 | 0 |

The initial ordinary request followed fresh explicit-package setup. The qualifier checked the same original core process before and after every request. These counts establish functional reuse; they are not latency measurements.

Native build `a670fc9b9e695df6` used entry SHA-256 `013c580386e3dd4eafd536d3f057902e73f60344d7696f348169f37f17620800` and core SHA-256 `8f817887e0cbb3da16200ce37be09fcaa1408728fa8f5e38414c4d9704b66b5c`. The verification summary retains the library and SDK identities.

## Original incomplete capture

The [original supplemental ZIP, artifact 14546870100](https://github.com/glopyglerky/lustr/actions/runs/44553094694/artifacts/14546870100) is 13,603,715 bytes. Its SHA-256, verified locally against GitHub's digest before workspace access was lost, is `4d7965327d6466f092c704992abca9baf950b8548d5e115225d5d2b5c4b4988a`.

It contains `inventory.json` and `records.tar.gz`. Their sizes and hashes are recorded in [verification.json](verification.json). Independent checking verified all available members and 26,126 original retained-report stream hashes across 349 histories. The inventory explicitly omits 28 compiled regular files.

Capture remains incomplete because these two record paths are symlinks whose referent bytes were not read:

- `native-selection-cache/package/lib/lustr-selection-tools.txt`
- `native-selection-seed/relocated-package/lib/lustr-selection-tools.txt`

Both point to the original installed package receipt. The original `capture_complete=false` and independent verifier's overall false result remain unchanged. No integrity mismatch was found in the available bytes.

The [full original artifact, 14546231385](https://github.com/glopyglerky/lustr/actions/runs/44553094694/artifacts/14546231385) is 586,107,251 bytes, above the download tool's 536,870,912-byte limit. Its SHA-256 `5170926f6a099a752fd9d55477c93f627fa698996db210bbdbed6ec2bc7b1699` is server-reported only; no full download or local digest verification is claimed. Artifact metadata reports expiry on 17 February 2027.

## Preservation boundary

The workspace became unavailable before the local evidence bundle was committed. This directory was reconstructed from preserved original GitHub logs and metadata, exact source at the published revision, and previously recorded verification results.

Neither ZIP is copied into this directory. Original per-command records remain in the GitHub artifacts. The local extraction, independent verifier script/report and uncommitted local bundle were not recovered; the summary does not present them as published originals. Later source or fixture repairs cannot change this run's recorded failure or inherit its qualification.
