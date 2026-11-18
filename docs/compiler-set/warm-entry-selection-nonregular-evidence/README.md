# Nonregular input correction and descriptor path review

The selection cache could hang before Cargo ran when either its Cargo executable path or its tool receipt was replaced by a FIFO. Both failures were reproduced with real pinned Cargo in private package projections. The corrected selector rejects these nonregular inputs before opening them, then preserves Cargo's ordinary result.

This is selection correctness evidence. It does not qualify native runtime behavior or establish an end-to-end speedup.

## Observed failures and corrections

| Stable input transition | Original cached-path request | Same uncached request | Corrected request |
| --- | --- | --- | --- |
| Replace projected `bin/cargo` with a FIFO | Timeout at 3 seconds; no stdout or stderr | Exit 2, `Permission denied (os error 13)` | Exact uncached exit and error; cache ineligible |
| Replace projected `lib/lustr-selection-tools.txt` with a FIFO | Timeout at 3 seconds; no stdout or stderr | Exit 0, selected target unchanged | Exact uncached target and exit; cache ineligible |

The original Cargo and receipt probes each contain four recorded commands: real Cargo setup, successful selector priming, the changed candidate request, and the uncached reference. Only owned projection paths were changed; the supplied toolchain remained intact. The source, helper identity, raw stream hashes, projected streams, and timeout limits are preserved in `original-cargo-fifo` and `original-receipt-fifo`.

The regular-file correction passed the expanded selector gate: **140 invocations and 265 assertions**. The gate also checks that restoring Cargo regains a hit and that the previously rejected native Cargo delegate cannot gain a cache entry or hide its external-state error. Its result is in `regular-file-fix/results.json`.

The earlier 132-invocation gate did not cover these FIFO transitions. Its historical result remains valid for those executed cases; it did not establish the absence of this later-discovered defect.

## Exact source identities

| Purpose | Revision | Tree | Selector helper SHA-256 |
| --- | --- | --- | --- |
| Original failing helper | `1aff7f7331d1d4293d3f5c74c740318c9686b030` | `7553100fe259c3a45dcb5640fe9dd50c3f0ed2dd` | `82a6e270dc70d780d25838252c8050234fb06e5e957581863de965586c7e015a` |
| Regular-file correction | `e997dde3361d9a17e46727f331832211ce8a0d05` | `3dafe1c9d972fca573c21c304500b3b0e983e5b6` | `4c9f61c8826097c1afd5808241e05ed4a1a3a6e15f7035c03c5a416c9eafa481` |
| Separate descriptor-path candidate | `ddf84bc779d4aef0b866708624940eed5815ae1a` | `ee13a019e62c3140817deb3232eb329ef80cb272` | `badee2554f4d6f5bae77c02338200f874a8d8ed97ef8d56651d1928c32ab00d2` |

The original `selection_cache.rs` has SHA-256 `c6fc392e1a46980292fb4813a6fadba8a0c96d2686301ced2b1dcad38981c3f2`. Its bytes also match revision `3d2d4c17d3da929b0b1c0a871bd7128e58e21578`. The original helper's frozen source closure is retained in `failed-source`, including a per-file hash inventory. Each corrected set includes its exact selector source and build receipt. The patches show the separate product changes and the added regression controls.

The expanded regression script was committed separately as `e4134d66b916dccb1ac12e9639a0b1bce45b1e6a`; its SHA-256 is `a1d018d05702bcf5ac06bf8024e17914ca05e2f8931c1a95eae2c2df9791963b`. Later composition or publication needs an explicit source mapping; a different commit identifier alone is not evidence that a helper was rebuilt.

Both corrected product revisions preserve the native/runtime source closure of PR 21 revision `28a2842635ee994db492b43f8c956eddce57a803`. `identity.json` records the checked paths and their Git object identities.

## Scope of the source changes

The regular-file fix follows symlinks with a metadata query and declines caching unless the target is a regular file. It retains the subsequent actual open, read, and descriptor metadata check. A FIFO or device therefore cannot block the cache-specific read before normal Cargo selection. The existing assumption that inputs remain coherent during a request still applies; the change does not claim protection from concurrent path replacement between observations.

The separately reviewed descriptor-path candidate opens each executable and obtains its resolved Linux path from the live descriptor. Ambiguous or deleted procfs paths, absent procfs, and direct-open failures use the original canonicalize-and-open route. Native image reads, readability checks, full identity stamps, receipt validation, and byte verification remain in place. The same regular-file guard precedes either opening route.

That candidate also passed **140 invocations and 265 assertions**. Its filesystem unit checked ordinary files, symlink chains, relative paths, hard links, a literal deleted-suffix filename, denied access when applicable, missing paths, rename, and unlink. The exact unit source and result are retained. This correctness clearance does not establish a performance benefit; component measurements are a separate record.

## Reproduce or inspect

Run the archive verifier without executing a compiler:

```sh
python3 verify-evidence.py
```

The included regression tool takes a helper built from the exact production selector and a matching package containing real pinned Cargo, Rust tools, libraries, and the source-bound tool receipt:

```sh
python3 test-native-selection-cache.py \
  --profiler "$SELECTOR_HELPER" \
  --package-bin "$SOURCE_BUILD/toolchain/bin" \
  --output "$NEW_RESULTS_DIRECTORY"
```

The helper can be rebuilt from a clean source checkout using the repository's recipe:

```sh
python3 "$SOURCE/tools/run-warm-entry-comparison.py" build-selector-gate \
  --repository "$SOURCE" --built "$SOURCE_BUILD" --output "$NEW_HELPER_BUILD"
```

`SOURCE_BUILD` must include the matching `toolchain` directory and `NATIVE-BUILD-RECEIPT.txt`. The helper build receipts preserve the executed source inputs, dependency versions, native identity, and compiler commands. No compiler binary is included in this archive.

The regression tool creates its own package projection and bounded FIFO controls. On the original failing helper it stops at the Cargo-FIFO timeout and records failure. The independent receipt-FIFO failure and its exact request are retained separately, since a fail-fast full suite cannot continue past the first timeout.

## Public edit-gate review

A separate benchmark continuation, `34c86339b653e965f43e28b04b1274a7611b4011`, adds untimed public-entry cache-hit checks through an ordinary body edit, a type error, and restoration. The source holds one daemon and binds valid runtime results to each stage's successful build. Its exact-source artifact replay passed 28 controls, including rejection of missing edit proof and a restored stage reusing the earlier runtime command. The review receipt and streams are included in `public-edit-gate-review`.

Those 28 controls validate the harness and replay rules. They do not mean the actual public compiler composition ran locally; that execution remains a hosted gate.

## Evidence handling

Public paths use stable role replacements. Original stream hashes and projected stream hashes remain distinct. Unmodified selected originals are retained with the repository's authenticated encryption helper and existing recipient certificate; no private key was accessed. The manifest covers the complete public file set. Repeated gates on separate candidate sources must not be added together as if they were additional coverage.
