# Selection initialization and identity follow-up review

This evidence supplements the original selection-cache review. It records two separately reviewed source changes and one bounded Cargo input investigation. These are selector and initialization checks; they are not native compiler execution or end-to-end timing qualification.

| Source or investigation | Exact identity | Completed result |
| --- | --- | --- |
| Package verification seed | `a0f2b79ee70952e8227a18663cb0aae2e9e7aa11`, tree `bd08959de47e01064da5637c6932a9de85f2ee40` | 38 recorded invocations, 85 assertions |
| Reuse executable identity observations | `1aff7f7331d1d4293d3f5c74c740318c9686b030`, tree `7553100fe259c3a45dcb5640fe9dd50c3f0ed2dd` | 132 selector invocations, 250 assertions |
| Malformed Cargo credential files | Pinned Cargo SHA-256 `da29c2a9ff7f85bda5587e9c974e05c9e2a7196377e54466f6e9d6f9e00b2530` | Four commands including setup; both malformed-file cases exactly matched baseline |

The source changes are provided as patches. Helper build receipts, helper hashes, test-source hashes, and result hashes identify what actually ran. These local commit identities name the reviewed changes before their later composition or publication; a later commit needs an explicit tree or source mapping. The source patches do not establish such a mapping by themselves.

## Seed controls

The actual production `prime-selection-tools` function created a private tool certificate. A subsequent project obtained the same certificate and retained its selection. Corrupt JSON, checksum corruption, shared file permissions, and shared directory permissions forced safe fallback without repairing or trusting the supplied seed. Relocation obtained a different certificate bound to the new live identities.

Replacing Cargo with a real native delegate whose behavior depends on an external file failed seed creation and never cached a selection. After the external file appeared, both the candidate and uncached Cargo rejected with identical status and stderr. A changed source receipt also refused initialization and forced fresh selection.

This test invokes the production initialization function through the selector helper. Its package projection deliberately excludes a constructor's existing seed. It therefore verifies the function and its consumption, but does **not** prove that a complete source-package constructor invoked the installed `lustr` launcher. That requires the separate hosted constructor proof.

## Identity reuse controls

The follow-up reuses the canonical path and file-descriptor metadata already obtained by the native-executable check, rather than collecting the same executable identity twice. Native image reads and actual file opens remain in place. The existing selector gate passed on the exact new helper, including executable inode replacement, execution-permission changes, and the external-state native delegate.

No stat-only image-admission optimization is included in either reviewed change. A possible effect of changed supplementary-group access was raised as a source-level consideration for that separate idea, not as an executed failure in these results.

## Credential-file investigation

In a fresh standalone, dependency-free workspace, the pinned Cargo ran `metadata --format-version=1 --manifest-path ... --offline --locked`. The investigation placed malformed TOML first in the isolated Cargo home's `credentials.toml`, then in legacy `credentials`. Each request exited 0 with stdout and stderr byte-identical to the clean baseline. No missing invalidation was demonstrated for this admitted request shape. The probe did not read any pre-existing credential file and does not make a claim about requests involving dependencies, registries, or network authentication.

## Inspect or repeat

Run `python3 verify-evidence.py` to check the public inventory, stream hashes, completed counts, and outcomes without executing a compiler.

The supplied test tools accept the actual selector helper, matching source-package binary directory, and a new external output directory:

```sh
python3 test-native-selection-seed.py --profiler "$SEED_HELPER" --package-bin "$PACKAGE/bin" --output "$NEW_SEED_RESULTS"
python3 test-native-selection-cache.py --profiler "$STAMP_HELPER" --package-bin "$PACKAGE/bin" --output "$NEW_STAMP_RESULTS"
```

The helper construction commands and exact source inputs are retained in each set's build receipt. Rebuild from the corresponding source using the repository's `build-selector-gate` recipe. The binaries are not included here. The tests require the real package tool receipt and matching pinned toolchain; missing-receipt refusal would not exercise certificate reuse.

Paths in public reports and stream archives use stable role replacements. Raw hashes remain separate from projected stream hashes. Unmodified originals are retained with the repository's authenticated encryption helper and existing public recipient certificate; no private key was accessed. Repeated gate counts are not additional coverage and should not be aggregated with earlier runs.
