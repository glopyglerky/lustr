# Process fixture ownership: hosted qualification, 18 November 2026

**All three process tests pass on macOS, Ubuntu and Windows with the repaired fixture allocator.** This is a test-ownership and process-behavior result. Full-source/general CI and the separate native macOS sanitizer finding remain outside this qualification.

[PR #19](https://github.com/glopyglerky/lustr/pull/19) changes only `crates/lustr/tests/process.rs`. Both existing real process/Cargo tests and their assertions remain unchanged. A process-wide atomic serial and exclusive directory creation establish owned fixture roots; the deterministic regression uses two live owners with the same timestamp.

## Exact source

| Role | Identity |
| --- | --- |
| PR head | `b73c83de4775044cc0df79af1d6385645b598cd4` |
| Base | `34d847da3d90c3c2b253246a8ffdfdeb901c56b0` |
| Hosted merge checkout | `d04acbb0cc5614b7e3367d36e9511474d3629bf5` |
| Head and merge tree | `de493c5fdfa93c783250400560fb04c90787c44f` |
| Tested file SHA-256 | `d8415c12e8d00f7636f7713a97b37ce420ee5722847d44c20d8ada065fe99171` |
| Compiler set run | [44252424546](https://github.com/glopyglerky/lustr/actions/runs/44252424546) |

## Original red/green evidence

Using the actual allocator with the old ownership behavior, the fixed-timestamp regression failed with Cargo exit 101 because cleanup removed the second live Cargo target. The corrected source passed all three process tests with two concurrent test threads. The final tested source matches the published file exactly. Local checks used Rust 1.99.0 on Linux; the hosted jobs below used Rust 1.99.1.

The earlier [PR #17 process-test failure](https://github.com/glopyglerky/lustr/actions/runs/44243040376/job/134595232256) remains failed. A shared-directory collision is probable in that failure, but was not directly observed because the successful test root was not logged. The deterministic regression independently confirms the allocator defect. That Cargo process-test incident is separate from older native compiler/sanitizer findings.

## Hosted process tests

| Platform | Exact job | Process result |
| --- | --- | --- |
| Cargo compatibility (macos-26) | [134626044688](https://github.com/glopyglerky/lustr/actions/runs/44252424546/job/134626044688) | 3 passed; 0 failed; 0 ignored; 0 filtered |
| Cargo compatibility (windows-latest) | [134626044910](https://github.com/glopyglerky/lustr/actions/runs/44252424546/job/134626044910) | 3 passed; 0 failed; 0 ignored; 0 filtered |
| Cargo compatibility (ubuntu-24.04) | [134626045254](https://github.com/glopyglerky/lustr/actions/runs/44252424546/job/134626045254) | 3 passed; 0 failed; 0 ignored; 0 filtered |

Each job executed these tests successfully:

- `cargo_alias_can_select_release_without_enabling_cranelift`
- `process_contract_preserves_arguments_io_failure_and_toolchain_routing`
- `repeated_timestamps_keep_scratch_ownership_separate`

The original workflow command and default Rust test-harness concurrency were unchanged. No workflow was manually dispatched or retried. Both broader workflow runs were still in progress at the recorded observation; these process results do not imply their completion.

## Preserved originals

The [qualification record](qualification.json) contains reviewed public counts, source hashes and exact original log digests. The [identity record](identity.json) binds the source, workflow observation and archive. The [encrypted snapshot](originals-encrypted/private-evidence.tar.gz.cms) retains all original local red/green receipts and streams, tested sources, exact hosted logs, source/workflow bindings and the prior process-test failure without rewriting their bytes.

The archive contains 34 original files. Its [receipt](originals-encrypted/receipt.json) binds plaintext SHA-256 `85eb8395bc33045fc34c09f35b1f36332e646ac1eafe666269ed25ee3c9e619f` and ciphertext SHA-256 `2a8b26e1da2056a709889db0f3137027a1c5c6fe912dafa680450b9d51e18330`. The unchanged repository helper used the pinned public certificate. No private key was accessed, and the ciphertext was not independently decrypted. A deterministic reconstruction reproduced the plaintext archive and manifest digests.

The generic helper [status](originals-encrypted/status.json) reports the coordinator campaign as `not-observed` because this envelope contains a process-test evidence snapshot, not a coordinator benchmark. The successful process-test qualification is recorded separately. All public files are allowlisted projections; raw paths and full logs remain encrypted.

This evidence-only branch leaves the PR #19 source head unchanged. It makes no compiler performance, complete native qualification or merge-admission claim.
