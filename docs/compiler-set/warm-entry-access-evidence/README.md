# Effective tool-access correction

The cache now checks current effective execute access for every native
executable and effective read access for each included shared library.
The actual Cargo transition gate passed **154 invocations and 290 assertions**.
This is a separate correctness correction; no performance benefit is claimed.

| Revision | Role |
| --- | --- |
| 68395242e638abdde91ff73d14bde532583934a8 | Executed product/helper source |
| 33196f20e299e8b48764b0b5614b0c616fbe6f96 | Adds unreadable-library control; product/helper source unchanged |

The earlier cache observed file ownership, modes and other stamps, and opened
executables for reading. Those observations do not fully encode a process's
effective group memberships or access capabilities. The same readable file
could lose effective execute access without its metadata changing. This was a
source-level finding; a changed-credentials process transition was not
executed locally.

The correction uses Linux x86-64 faccessat2 with AT_EACCESS: X_OK for native
executables and R_OK for included library files, checked on their original
paths. Unsupported or failed calls make the optional cache ineligible and
preserve the original Cargo selection path. The direct syscall avoids older
libc faccessat emulation that omits ACLs. See the
[Linux manual](https://man7.org/linux/man-pages/man2/access.2.html).

Pre-open regular-file checks, actual executable opens, image checks, descriptor
metadata, tool byte verification, private cache ownership and input snapshots
remain. This does not prove that every later exec succeeds under every loader,
filesystem, security policy or concurrent mutation. No credentials or process
permissions were elevated or changed during the tests.

## Executed controls

The full selector suite still compares complete cached/fresh Selection values,
normal exit status and exact Cargo selection stderr. It retains the real
unknown-native-delegate/sentinel, malformed config, discovery, target alias,
manifest/lock corruption, changed tool, cache corruption and FIFO controls.

New controls show that a readable, read-only executable remains eligible,
removal of Cargo's execute permission makes the cache ineligible and preserves
Cargo's error, and removing read access from a private copy of an included
library makes the cache ineligible. Restoration reaches the original retained
behavior. The library copy is isolated; no chmod follows a symlink into the
caller's SDK. These file-permission transitions do not stand in for a
changed-credentials test.

An independent reviewer read the exact source and replayed all 154 command
streams and their complete Selection comparisons. The retained review receipt
states its scope. No native compiler or daemon qualification follows from
this gate.

## Replay

    python3 PATH_TO_THIS_DIRECTORY/verify-evidence.py

The replay verifies the full file inventory, frozen source/helper binding,
projected raw stream hashes, all 77 cached/fresh response pairs and the 290
assertions. Original path-bearing bytes and build receipts are encrypted under
the repository's existing recipient certificate. No private key was accessed.

For fresh execution, build the stated source's profiler with the source-bound
build-selector-gate recipe, then run:

    python3 tools/test-native-selection-cache.py --profiler HELPER \
      --package-bin MATCHED_PACKAGE/bin --output NEW_OUTPUT_DIRECTORY

The native source/runtime and the imported Rust pin are unchanged. The
canonical-parent path experiment and its inconclusive timing result are
retained separately and are not part of this correction.
