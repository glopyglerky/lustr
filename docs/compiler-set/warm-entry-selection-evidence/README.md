# Independent Cargo-selection review evidence

This package preserves the first candidate's reproduced stale-success defect and the corrected selector's completed regression results. It qualifies the bounded reuse of Cargo target selection on the recorded Linux toolchain. It does not qualify native execution or establish an end-to-end compiler speedup.

## Source identities

| Role | Revision | Tree |
| --- | --- | --- |
| Unchanged public-entry baseline | `28a2842635ee994db492b43f8c956eddce57a803` | Baseline recorded in `identity.json` |
| Locally reviewed and executed correction | `53057e72f1853bf2ec005e5a798975b4f951044f` | `b568e8bf4d9970418b7c1f4ef2984ce391ffa5f5` |
| Published correction | `18fe6c3408e6d581961bcf7f2721f72ca74cfbfa` | `b568e8bf4d9970418b7c1f4ef2984ce391ffa5f5` |
| Subsequent regression-report completion repair | `6ac30dd13f1ce9cb1f6b34feacc0ff9f30951637` | Separate validation change; product selection code unchanged |

The published revision's tree was read through the GitHub API and compared with the locally executed tree. The different commit identifiers have the same source tree. The initial failing candidate was uncommitted; `failed-source/` contains its exact helper source closure, manifest and lockfiles, file hashes, source timestamps, and executed helper hash. Its parent was the baseline above. This frozen snapshot is evidence, not recommended product code.

The actual helper executed for the final reviewer run has SHA-256 `24d5fd47e2a96ff40ffed39ddd649475be78fd203cfbfa5f8443a37dbb3ebf68`. The independently built workflow helper has a different hash, recorded with its build recipe in `outcomes/workflow-helper-build.json`. Both include the production parser, selector, protocol, and child-process owner. Absolute build paths can affect binary hashes; rebuilt binaries need their own receipts.

## Reproduced defect and correction

The first candidate treated a native executable's file signature and unchanged filesystem identity as sufficient evidence that its behavior was fixed. The control installs a small real ELF Cargo delegate. Before an external sentinel exists, it executes the pinned Cargo. After the sentinel appears, it rejects the same request with exit status 2 and a fixed diagnostic. Its own bytes, filesystem stamps, request arguments, and environment remain unchanged.

The original run in `outcomes/first-elf-failure.json` records three successful priming requests ending in a cache hit. After the external change, the candidate again reports a hit and success; the uncached production selector exits 2 with `external tool state rejects metadata`. This is a real stale-success failure. An ELF header does not establish that a tool's inputs have been observed.

The correction admits only the source package's tool receipt, requires the fixed Cargo and reference-compiler pins, verifies the listed executable and library bytes, and binds a private verification certificate to the current receipt and complete live file identities. Replacing Cargo with the control never produces a cached selection. After the external change, the corrected candidate and uncached selector both reject with exactly matching status and stderr. A diagnostic of `miss` for an unknown native tool is expected: byte verification occurs before cache storage. The failed request is not retained.

This protects normal package updates, relocation, configuration changes, and accidental cache corruption. It does not claim a security boundary against the same account deliberately replacing both a trusted package and its private evidence.

## Executed results

| Evidence set | Observed result | Scope |
| --- | --- | --- |
| `outcomes/corrected-source/` | 132 selector invocations, 250 assertions, all passed | Exact local correction tree matching the published revision |
| `outcomes/workflow-replay/` | 132 selector invocations, 250 assertions, all passed | Independent build using the workflow's helper construction recipe |
| `outcomes/reporting-fix/` | 132 selector invocations, 250 assertions, completed with no error | Replay after the validation-only completion-report repair |
| `outcomes/reporting-negative-control/` | Failed status, success false, JSON parse error retained | A malformed helper cannot turn an interrupted suite into a passing report |

The repeated sets confirm their respective source and runner constructions; their counts must not be added together as new coverage. Each complete set contains 66 candidate/reference pairs. They compare exact exit status, the selected target or rejection, and Cargo stderr. The helper's elapsed duration is removed when comparing its structured selection result.

The transitions include source body edits retaining a hit; caller-directory and Cargo-home configuration; same-size, same-mtime lockfile corruption; target renaming and discovery; a new build script; symlink-created target ambiguity; package-cache lock failure; environment changes; malformed, checksum-invalid, and nonprivate cache entries; tool replacement and execution permissions; and the native delegate's external-state rejection. `tools/test-native-selection-cache.py` in the corrected source is the complete executable gate.

The public projection preserves separate raw and normalized stream hashes. Original stream hashes remain in `stdout_sha256` and `stderr_sha256`; normalized stream hashes use `public_stdout_sha256` and `public_stderr_sha256`. Each set's `public-streams.tar.gz` contains regular files with basename-only paths. Normalization replaces execution-directory prefixes with stable role names. It does not change exit statuses, diagnostics, assertion results, or source snapshots.

The initial incomplete regression-tool run is also retained in the encrypted originals. It aborted while inspecting a deliberately corrupted file belonging to another request context. Its old report format could incorrectly show success before completion. It is not a completed qualification. The reporting repair now requires the suite to reach its completion marker and records exceptions explicitly. The malformed-helper negative control exercises that failure path.

The additional counterexample records distinguish observed errors from non-errors: malformed caller configuration and a package-cache lock replaced by a directory cause real Cargo rejection; replacing the configured target directory with a regular file did not make this metadata request fail. The latter is retained as a negative investigation result, not reported as a fixed defect.

## Reproduce and inspect

First verify the published evidence without executing a compiler:

```sh
python3 verify-evidence.py
```

For a fresh reproduction of the original defect, provide the pinned `nightly-2026-11-09` SDK and a matching source package. The Cargo dependencies in the frozen lockfile must already be available for the offline build. Use new output directories outside the checkout:

```sh
python3 build-failed-profiler.py --sdk "$SDK" --output "$FAILED_HELPER_BUILD"
python3 reproduce-native-tool-change.py \
  --profiler "$FAILED_HELPER_BUILD/target/release/profile-native-preflight" \
  --package-bin "$PACKAGE/bin" --output "$FAILED_CONTROL" --expect stale
```

For the corrected source, use the repository's workflow construction recipe and the matched package build directory containing `toolchain/` and `NATIVE-BUILD-RECEIPT.txt`:

```sh
python3 "$REPO/tools/run-warm-entry-comparison.py" build-selector-gate \
  --repository "$REPO" --built "$PACKAGE_BUILD" --output "$CORRECTED_HELPER_BUILD"
python3 "$REPO/tools/test-native-selection-cache.py" \
  --profiler "$CORRECTED_HELPER_BUILD/target/release/profile-native-preflight" \
  --package-bin "$PACKAGE_BUILD/toolchain/bin" --output "$CORRECTED_TRANSITIONS"
python3 reproduce-native-tool-change.py \
  --profiler "$CORRECTED_HELPER_BUILD/target/release/profile-native-preflight" \
  --package-bin "$PACKAGE_BUILD/toolchain/bin" --output "$CORRECTED_CONTROL" --expect fresh
```

The supplied package must have its actual `lib/lustr-selection-tools.txt` receipt; otherwise a missing-receipt refusal does not reproduce the corrected byte-verification control. The portable scripts were prepared from the executed control and retain new results when run. Historical execution claims here refer to the preserved source hashes and original records, not to an unexecuted future reproduction.

## Original retention and boundaries

`encrypted-originals/` uses the repository's existing `tools/encrypt-coordinator-evidence.py` workflow and public recipient certificate. Its receipt identifies every original file through the encrypted manifest and binds the plaintext archive and ciphertext hashes. The original file inventory includes predecessor failures, raw streams, source snapshots, build receipts, fixture sources, and original executed review scripts. It excludes unrelated files and compiler binaries. No private key was accessed. Decryption requires the recipient's existing private key and is not needed to inspect the public projection.

The trusted recipient certificate DER SHA-256 is `69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f`; its SPKI SHA-256 is `7da67bcb676201917fcadb49ae8802f678eddfa734d3493e82d3a2cd735a9f96`. The retention helper's generic benchmark-scenario fields do not imply that those benchmarks ran in this selection-only review.

Local Unix-domain sockets were unavailable and the projected local package was not a complete native runtime qualification package. No native compilation or runtime admission is claimed from these selector tests. Recorded helper durations are component observations, not a paired end-to-end performance campaign. The separate hosted benchmark and native gates determine those results.
