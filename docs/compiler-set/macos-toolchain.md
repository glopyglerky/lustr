# Pinned macOS sanitizer toolchain

The native macOS workflow acquires the official LLVM 23.1.5 ARM64 archive using
the reproducible installer and SDK selector already implemented in
[PR #23 source `ea1cd30`](https://github.com/glopyglerky/lustr/commit/ea1cd30c5e8425228a6e9a7c482e8790382f03f8).
The installer, ten controls, and package pin are restored byte for byte. The
[restoration record](evidence/macos-toolchain-restoration-20261119/verification.json)
binds those files and the upstream release metadata checked on 19 November 2026.

This deliberately selects 23.1.5. LLVM 23.1.6 exists, but its complete
[official release asset inventory](https://api.github.com/repos/llvm/llvm-project/releases/448854669/assets?per_page=100)
contained 37 assets and no macOS package when checked. The 23.1.5 ARM64 archive
and attestation remain available with the exact previously reviewed lengths and
SHA-256 values. The workflow uses a fixed pin; it does not discover a different
compiler version at runtime.

Before extracting or executing anything from the archive, the installer checks
its byte length and SHA-256 and verifies its signed provenance against the exact
upstream repository, source commit, tag, signing workflow, certificate identity
and OIDC issuer. It checks the extracted compiler's version and the location of
its runtime resources. The package stays in an owned temporary directory. The
LLVM package no longer comes from a floating Homebrew formula; Homebrew still
supplies the upstream test harness's command tools and the archive decompressor.

The restored SDK selector preserves an explicit `SDKROOT`, otherwise asks
`xcrun --sdk macosx --show-sdk-path` under the inherited developer selection. It
validates the resulting SDK and records SDK settings and the standard-library
header identity. It does not change the host's active developer installation.
The selected `SDKROOT`, compiler and symbolizer are passed to subsequent checks.

All existing clean-execution, ASan overflow, UBSan overflow, leak-detection and
native differential gates remain required. Both added unwind differential
routes also remain required; emitted-C unwinding uses the pinned package's
Clang and Clang++. Setup records and failure output are retained with the
native gate artifacts.

Ten installer controls pass locally, including corruption/truncation rejection,
provenance-boundary ordering, timeout output preservation, explicit SDK handling
and discovery without changing developer selection. Workflow parsing and shell
syntax checks pass. The earlier
[acquisition evidence](https://github.com/glopyglerky/lustr/blob/ea1cd30c5e8425228a6e9a7c482e8790382f03f8/docs/compiler-set/evidence/macos-sanitizer-toolchain-20261118/README.md)
and [SDK evidence](https://github.com/glopyglerky/lustr/blob/ea1cd30c5e8425228a6e9a7c482e8790382f03f8/docs/compiler-set/evidence/macos-sdk-selection-20261118/README.md)
retain their original source identities and limits. This restoration has not
itself rerun macOS acquisition or native execution; the new hosted workflow must
establish that result on its own source.
