# Complete Linux native gate at `dec0ce2`

The Linux x86-64 job in [native CI run 44083661066](https://github.com/glopyglerky/lustr/actions/runs/44083661066/job/134040390485)
completed successfully. `native-gate-original.zip` is the original GitHub
artifact, preserved byte for byte. [manifest.json](manifest.json) binds its
archive and member hashes to the exact PR head, checkout, repository tree and
native subtree. The checkout was GitHub's merge commit; it had the same tree
as the published PR head.

| Identity | Value |
| --- | --- |
| PR head | `dec0ce2e3a68b82cac16eae238fc51b16e534e5e` |
| Checkout | `dbacce1d14c8df60427eeca8dc7865156c98dcb4` |
| Repository tree | `c29f3bda282c0870c163303d865abb9be81bd066` |
| Native tree | `0c196f3b8f0803cbf3985cc2f4be687129831e26` |
| Artifact | `14333541823` |
| Original ZIP SHA256 | `63a3df69632cbf1b4a08e55b26b2ef3aedda79c156245db77b61f6d2fc8263f6` |

## Executed scope

The original gate log records 5,527 successful differential checks with zero
failures: 2,229 pass cases, 3,140 rejection cases and 158 deliberately rejected
valid-Rust cases. The `SANITIZE=1` run repeats that case set with zero failures;
it sanitizes generated C for accepted pass cases. This is not 5,527 separately
sanitized compiler binaries. The daemon edit scenarios report 363/363 successful checks and 27
unsupported versions. The client suite reports 89 passes, crate fixtures 85,
macro-dependency fixtures 13, and `lust-port` fixtures 13. Kernel, slot, fuzz
and benchmark-smoke steps also completed. The separate CI step for the Unix
address C models passed with ASan/UBSan.

The 58 macro-corpus passes are **Rust-reference results**: the gate invokes
`scripts/test-macros.sh` without its optional `LUST` setting. They do not
establish support for all those macros in native Lust. The kernel step also
defaults to Rust/Cargo reference mode when `LUST` is unset; its 43 successful
checks are not 43 native Lust kernel executions. The macro-dependency suite is
a separate native test.

The benchmark smoke uses one repetition. Its original values remain in the
log, including slower results, but it is neither the paired performance
campaign nor evidence for a general speedup. Use the separately retained
[matched campaign](../../../../performance/2026-11-17-dec0ce2.md) for measured
comparisons and uncertainty.

The native gate uses Rust 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`,
LLVM 23.1.1) as its reference. The mixed compiler campaign uses its separately
pinned SDK; these toolchains are not interchangeable reference identities.

## Qualification boundary

This result qualifies the complete Linux gate for the recorded native tree,
including the portable Unix-address fixtures. It does not qualify the later
library-selection or finer profiling changes. Those require a new combined
candidate run.

The macOS ARM64 job was cancelled before execution. GitHub reported that a
hosted runner could not be acquired and that ARM64 capacity was constrained.
No macOS execution, all-platform pass or release admission follows from this
Linux result.

To inspect the retained bytes:

```bash
sha256sum native-gate-original.zip
unzip -p native-gate-original.zip native-gate-identity.txt
unzip -p native-gate-original.zip native-gate-builds.sha256
unzip -p native-gate-original.zip native-gate.log
```
