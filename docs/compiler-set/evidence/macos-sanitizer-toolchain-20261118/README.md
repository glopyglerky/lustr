# Reproducible macOS sanitizer compiler acquisition

The subsequent [hosted acquisition and SDK-selection report](../macos-sdk-selection-20261118/README.md)
records the first PR #23 run: the exact package acquisition passed, then the
sanitizer probe could not find the host's standard-library header. It also
records the separate SDK setup correction and its bounded verification. The
execution and helper hashes below describe this original acquisition stage.

The [PR #21 macOS setup job](https://github.com/glopyglerky/lustr/actions/runs/44264239099/job/134664704136)
installed Homebrew Clang 23.1.6 and then failed its required 23.1.5 assertion.
Native compiler tests did not run. Updating a floating Homebrew formula cannot
reliably obtain the exact previously selected release.

The separate candidate based on PR #21 source
`28a2842635ee994db492b43f8c956eddce57a803` acquires the official LLVM 23.1.5
macOS ARM64 archive in an owned temporary directory. It checks its exact byte
length and SHA-256, verifies its signed upstream provenance, and only then
extracts or executes it. The exact 23.1.5 compiler assertion remains enforced.
No global compiler installation or host configuration changes are required.

## Pinned upstream inputs

The [official release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-23.1.5)
and [release API](https://api.github.com/repos/llvm/llvm-project/releases/432551565)
identify the archive and its accompanying attestation bundle. The repository's
[pin file](../../../../tools/macos-sanitizer-toolchain.json) records these inputs.

| Input | Identity |
| --- | --- |
| LLVM source | `392dfff97d29b59d77e5cac6d4b6ef2585fe7117` |
| Source ref | `refs/tags/llvmorg-23.1.5` |
| Archive | `LLVM-23.1.5-macOS-ARM64.tar.zst` |
| Archive bytes | `870686518` |
| Archive SHA-256 | `503de30ef90311e0b143623801b19af2250adac6d69faed65ad0051170202e3b` |
| Attestation SHA-256 | `961b20e89b54952add75ab7a3a565de1db1e300b546cceb091ade31afe21d671` |

The observed signing certificate names the reusable
[`release-binaries.yml` workflow](https://github.com/llvm/llvm-project/blob/392dfff97d29b59d77e5cac6d4b6ef2585fe7117/.github/workflows/release-binaries.yml).
The installer verifies that exact certificate identity, the GitHub Actions OIDC
issuer, repository, source ref, source digest and signer digest using
[`gh attestation verify`](https://cli.github.com/manual/gh_attestation_verify).
The caller `release-tasks.yml` and signer `release-binaries.yml` have distinct
roles; the policy binds the actual signer.

## Actual macOS verification

The pinned package was downloaded, verified and extracted on macOS ARM64.
The exact compiler returned version 23.1.5. Its SHA-256 is
`a22ab66d53cf70dfee8544eabdeb36f075bf942861cb8d59f98a3fadc7a65c26`.
[receipt.json](receipt.json) retains the symbolizer and all 22 packaged Darwin
runtime hashes as relative package paths.

The existing `tools/test-native-sanitizers.py` ran unchanged and passed all four
executing capability checks:

| Check | Required behavior | Result |
| --- | --- | --- |
| Clean program | Exit zero without a sanitizer diagnostic | Passed |
| Heap overflow | Nonzero exit with the ASan heap-buffer-overflow diagnostic | Passed |
| Signed overflow | Nonzero exit with the UBSan signed-integer-overflow diagnostic | Passed |
| Leaked allocation | Nonzero exit with the LSan leak diagnostic | Passed |

The gate retains `-fsanitize=address,undefined`,
`-fno-sanitize-recover=undefined`, `detect_leaks=1` and the original deadlines.
It clears inherited LSan and UBSan options. Its unchanged source SHA-256 is
`c4c430752b89d279121cb1218969c805cf5455bb8e24a23c5c653263da727960`.

A second actual provenance verification deliberately supplied the wrong source
digest while preserving the archive, bundle, repository and signer policies.
It returned exit 1 specifically because the certified source digest differed.
No extraction or compiler execution follows a rejected provenance check.

Seven local installer regression checks passed, including same-size corruption,
truncation and symlink rejection, stopping a bad archive before any provenance
or extraction command, preserving prior failure output before a retry, and
retaining both partial output streams when a real subprocess times out.
The workflow YAML parsed and the existing sanitizer capability step remains.
[qualification.json](qualification.json) records the results and original
output hashes.

The actual successful macOS execution used installer SHA-256
`6d3be0240e356a8d0413977f85afa24564dc3b184549cf280fe549bfa26edfd6`.
Review subsequently corrected command-timeout evidence retention. The published
installer preserves partial stdout/stderr before propagating a timeout and
records an explicit exit, signal and timeout status for each completed command
attempt. Its acquisition, provenance, extraction and compiler-selection logic
is unchanged. The new timeout regression fails against the executed predecessor
with a missing retained stdout file and passes with the repair. The final
installer and test identities are in [timeout-retention.json](timeout-retention.json).
The final helper has not been reexecuted on macOS during this bounded review;
its changed timeout path was checked through the actual subprocess regression.

## Preserved setup attempts

Two unsuccessful attempts preceded the final installer:

- The first combined mutually exclusive `gh` identity flags. GitHub CLI rejected
  them before extraction. The final policy uses the exact certificate identity
  with the independent signer/source digest checks.
- The second verified provenance but the host's default 128 MiB zstd decoder
  limit rejected the archive's 1 GiB window. The final installer gives only the
  archive decompressor an explicit 1 GiB memory limit. This does not change
  compiler memory settings, sanitizer options or test deadlines.

The original source, stdout/stderr, receipts and retrieval records for all three
attempts are retained under the repository's already pinned public encryption
certificate in [original-records-encrypted](original-records-encrypted/receipt.json).
The full upstream archive is referenced by its verified public identity rather
than duplicated in this evidence directory. No private key was accessed and
independent decryption was not performed.

## Qualification boundary

This evidence establishes exact compiler acquisition and the four sanitizer
capabilities on the observed macOS host. Hosted CI on the candidate and the full
native differential suite remain separate requirements. It does not repair or
waive the older incompatible function-pointer failures. Compiler source,
sanitizer source, differential fixtures, timeout values and performance
thresholds are unchanged by this acquisition candidate.
