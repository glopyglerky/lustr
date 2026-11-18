# Explicit macOS SDK selection for the pinned sanitizer compiler

The first hosted PR #23 attempt acquired the exact LLVM 23.1.5 package and
verified its signed provenance, then failed while compiling the sanitizer
probe because `stdlib.h` was unavailable. No sanitizer executable ran in that
attempt. The narrow follow-up selects a macOS SDK for the job and records its
identity while preserving the caller's developer-tool selection.

## Preserved hosted failure

The original [macOS job](https://github.com/glopyglerky/lustr/actions/runs/44278447999/job/134712502807)
ran PR head `aa46db14eab3978a8618ffcb8862e6a4739e9560`, with checkout
`911dc503d931cf7e069f6f45d01ce15d6390e41a`.

- All seven installer controls passed. The complete archive acquisition,
  hash checks, signed provenance verification and exact-version assertion
  passed on the hosted runner.
- The unchanged capability gate invoked Clang 23.1.5. Both `probe.c` and
  `allocation.c` failed to compile with `fatal error: 'stdlib.h' file not found`.
  This is a setup failure before the four executing capability checks.
- Original artifact `14415244263` is 12,189 bytes with SHA-256
  `8c99d56095a34db6b33e031dd634a3004ef9cd87dbfa5b28bcfb59d8a011424c`.
  Its compiler hash is the same pinned binary used below. The original ZIP,
  complete job log, metadata and successful acquisition receipt are retained.

This failure remains distinct from the older native function-pointer failures.
Acquiring the compiler successfully did not establish that its host SDK was
configured for compilation.

## Selection and scope

`select_sdk` in [the installer](../../../../tools/install-macos-sanitizer.py)
uses a nonempty inherited `SDKROOT` unchanged. If none is supplied, it records
the result of `xcrun --sdk macosx --show-sdk-path`. That command inherits
`DEVELOPER_DIR` and the host's active developer selection. The selector rejects
a relative path, `/`, a nonexistent directory, a path containing a line break,
or a directory without `usr/include/stdlib.h`. An invalid explicit SDK is a
failure; it does not trigger a different SDK selection.

The existing job environment export gains `SDKROOT`. The installer records the
selection's origin, inherited developer-directory setting, SDK settings-file
hashes and standard-library header hash. The SDK remains a host dependency;
those fingerprints do not claim to hash the entire SDK. No global developer
selection or compiler installation is changed.

The [pinned LLVM driver source](https://github.com/llvm/llvm-project/blob/392dfff97d29b59d77e5cac6d4b6ef2585fe7117/clang/lib/Driver/ToolChains/Darwin.cpp)
uses a valid absolute `SDKROOT` as the default `-isysroot`, while preserving an
explicit command-line sysroot. Apple's [developer-tool selection documentation](https://developer.apple.com/documentation/xcode/configuring-command-line-tools-settings)
describes how `DEVELOPER_DIR` overrides the active selection for a command.

The archive and attestation pins, compiler/runtime binaries, workflow,
sanitizer flags, fault fixtures and individual test deadlines are unchanged.

## Actual component and capability verification

The changed selector was executed on macOS arm64 using the already verified
LLVM archive from the preceding acquisition qualification. It selected SDK
`macosx27.0` through `xcrun`. The downstream compiler and capability processes
received exactly the `SDKROOT` returned by that selector.

| Check | Observed result |
| --- | --- |
| Exact compiler SHA-256 | `a22ab66d53cf70dfee8544eabdeb36f075bf942861cb8d59f98a3fadc7a65c26` |
| Executed installer source SHA-256 | `3a298fa6b4a7d3229ac7f8487135082c66ffb284829ac66935f8519e0306c808` |
| Compiler driver arguments | Contain `-isysroot` with the selector's exact SDK path |
| Clean execution | Passed |
| ASan heap-overflow detection | Passed |
| UBSan signed-overflow detection | Passed |
| LSan leak detection | Passed |

The unchanged capability source SHA-256 is
`c4c430752b89d279121cb1218969c805cf5455bb8e24a23c5c653263da727960`.
[qualification.json](qualification.json) binds the source, compiler, SDK
settings, header, driver output and capability-output hashes.

This run exercised the changed `select_sdk` function and the four downstream
executing checks. It did not rerun the complete installer or reacquire the
archive. The original hosted attempt separately establishes the previous
complete acquisition pass. Neither result is a pass for the complete native
compiler suite, and the changed source still requires hosted CI.

The earlier local capability result did not record its complete SDK
environment. A later read-only inspection found no inherited `SDKROOT` or
include-path overrides, and the compiler's default driver arguments contained
no explicit sysroot. That later observation does not reconstruct the earlier
process environment or explain its header discovery. The new verification
binds SDK selection explicitly.

## Controls and retention

All ten installer controls passed locally and in independent review. The seven
existing corruption, provenance-boundary and timeout-retention controls are
unchanged. Three added cases check explicit SDK preservation, inherited
developer selection during discovery, and rejection of invalid SDK roots
without fallback. The explicit-path case also preserves a symlink spelling
containing spaces.

[private-evidence/receipt.json](private-evidence/receipt.json) binds the encrypted
original hosted artifact and log, current host observations, executed selector
source, SDK/driver/capability records, and local and independent controls.
Encryption uses the existing repository recipient certificate with SHA-256
`69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f`.
No private key was accessed and independent decryption was not performed.
Coordinator scenario fields are `not-observed` because this is SDK setup
qualification, not a benchmark campaign.
