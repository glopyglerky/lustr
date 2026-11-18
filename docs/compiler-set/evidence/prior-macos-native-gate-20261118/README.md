# Earlier macOS native sanitizer failure

The macOS Arm64 native gate failed on an older PR #16 source revision. This remains a platform qualification issue. It is separate from PR #17's later Cargo process-test temporary-directory failure, and no root cause is assigned here.

## Exact evidence

- [CI run 44236489070](https://github.com/glopyglerky/lustr/actions/runs/44236489070), attempt 1, [failed macOS native job 134573766624](https://github.com/glopyglerky/lustr/actions/runs/44236489070/job/134573766624).
- PR head `2bbf4f540ac908926448dcda3735063025fcc277`; actual checkout `48b0fd76aa8a6eff01cba72694669533e10924e8`.
- Original [artifact 14404285074](https://github.com/glopyglerky/lustr/actions/runs/44236489070/artifacts/14404285074), 44,365 bytes, ZIP SHA-256 `273093c38d4429f63d8268f53d44859080672babab55c993df6731e6151e9775`.
- Retained host records identify Darwin 25.6.0 on Arm64, Rust 1.99.0 and Clang 23.1.5.

The exact 6,021-line log records these differential-suite results:

| Run within the native gate | Passed | Failed |
| --- | ---: | ---: |
| Ordinary differential suite | 5,527 | 0 |
| Sanitizer differential suite | 3,309 | 2,218 |

All 2,218 `FAIL` lines record an incorrect function-type diagnostic with pointer type `void (*)(void)`. The log SHA-256 is `9713fd9bf02f10b261557320aaec8468e24e53a09b33c409cad07deb437bc78d`. These observations identify the failed gate and recurring diagnostic. They do not establish whether the compiler, generated code, sanitizer instrumentation or another component caused it.

The `compiler/native` tree is `12ae35ee30f81d11f55b184134f8a0c4a977a496` at both this older head and its actual checkout, and at corrected-gate PR #16 head `34d847da3d90c3c2b253246a8ffdfdeb901c56b0`. This older source does not contain the later native-object reuse, eligibility-guard or process-fixture changes. The failure should remain visible alongside their separate qualification results.

## Original retention and review limits

The exact downloaded ZIP is retained in the [encrypted archive](encrypted/private-evidence.tar.gz.cms), with its [receipt](encrypted/receipt.json), [status](encrypted/status.json) and [source identity](identity.json). The ZIP includes the original native-gate identity, full log and produced-binary hash records. All original bytes are preserved. The artifact does not contain the compiled toolchain package for a new independent execution or binary rehash.

The unchanged repository helper used the pinned public recipient certificate. A deterministic reconstruction of the review archive matches its plaintext and manifest hashes. No private key was used, and the ciphertext was not independently decrypted. The wrapper's coordinator status is `not-observed` because it preserves a failed native job, not a coordinator campaign; it does not override the original failure.

No compiler source or sanitizer gate was changed, weakened or rerun for this evidence review. There is no timing or performance claim, and this report does not establish macOS native qualification.
