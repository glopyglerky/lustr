# Native eligibility guards: local evidence, 18 November 2026

**Status: draft candidate; native correctness and performance are unqualified.**

[PR #18](https://github.com/glopyglerky/lustr/pull/18) contains a small change to the existing mixed driver. Both before/after driver components compiled successfully from exact source with the pinned SDK. The first baseline mixed-runtime qualifier failed because this execution environment denied Unix socket creation. Further native qualifiers and all local timing campaigns stopped. This record preserves that failure; it supplies no speedup result.

## What changed

The existing native-selection loop rejected unsupported signatures only after walking represented HIR bodies and querying Rust and C ABI information. The candidate moves its unchanged scalar signature and parameter-count filter ahead of that work. It first applies the same root free-function conditions already required by the body validator. Every surviving definition still passes the existing body/binding validation and both actual ABI checks.

The codegen-unit hook now returns the original upstream codegen unit when the installed native-object set is empty. This avoids enumerating symbols in that case. When entries exist, the existing filtering and object integration continue unchanged. The candidate preserves zero-definition trace publication.

The source change is confined to [mixed_main.rs](https://github.com/glopyglerky/lustr/blob/be27c3f32725dd80e623af89034b15eeebde7e9a/crates/lustr-semantic/src/mixed_main.rs), [mixed_backend.rs](https://github.com/glopyglerky/lustr/blob/be27c3f32725dd80e623af89034b15eeebde7e9a/crates/lustr-semantic/src/mixed_backend.rs) and [mixed_objects.rs](https://github.com/glopyglerky/lustr/blob/be27c3f32725dd80e623af89034b15eeebde7e9a/crates/lustr-semantic/src/mixed_objects.rs). It does not expand the eligible language subset. Source review found no blocking issue; real native execution remains a required gate.

These guards affect work inside the mixed driver. A Cargo no-op can complete without running a compiler, so this change supplies no explanation or fix for coordinator-only no-op overhead.

## Exact source and component provenance

The full machine-readable mapping is in [identity.json](identity.json).

| Role | Identity |
| --- | --- |
| Historical remote baseline | `765e45c8befb6cf78f04db4a6e4935569567860e` |
| Local baseline commit | `18b8d1e55520ca739e8bccb081a08db8a99926fe` |
| Identical baseline source tree | `bd3fd98eebdcb9391b9714972fc2e20660349830` |
| Reviewed and built local candidate | `992fe781180e0ecbdd7ba9053f02b1b788816996` |
| Local candidate source tree | `d9ed7b6694a5517801fe4b2be2061bbda6173262` |
| Published parent containing the corrected evaluator | `34d847da3d90c3c2b253246a8ffdfdeb901c56b0` |
| Published three-file source commit | `be27c3f32725dd80e623af89034b15eeebde7e9a` |
| Published three-file source tree | `fdb6af7d1ea50c4b63fd8b6a217633979deedf22` |

The three baseline compiler blobs match the published parent exactly. Applying the three reviewed candidate blobs to that parent produces the published tree above. The locally built candidate binary came from the local candidate tree; it is not relabeled as a rebuild of the later published tree. The corrected evaluator changes are outside these compiler files.

Both optimized driver builds used the same pinned Rust source/SDK commit `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, the release profile, `full-core`, the locked offline dependency cache and one Cargo build job. The actual native-core build identity supplied to both was `dc82dab611c87e7e`. Their installed `-vV` output matched the original SDK output exactly.

| Mixed-driver artifact | SHA-256 |
| --- | --- |
| Baseline | `4fdde6c5f0fdb5e34ba00fc9da514592d50f5958a7b764c470affd49af3048d0` |
| Candidate | `2dcb154a130200055a13390df92fde0ae3da3d31d78bfb73e28e37c62e8a08c6` |

The original complete constructor stopped at native shared-library compilation after its compiler process received SIGKILL. A serial retry with unchanged inputs also received SIGKILL. A concurrently attempted pristine native build received SIGKILL as well. The cause of these signals was not established. The original constructor failure and each separate attempt remain retained. No additional native shared-library build was attempted after the serial failure.

The later manual composition used only the successful constructor stages and separately built mixed driver. The native shared library was absent. The baseline and candidate retained byte-identical native core, entry, Cargo coordinator and SDK components. Only `rustc-full` changed, with its relative `bin/lustr-cargo/rustc -> ../rustc-full` alias preserved. The expanded-core hardlink, source pins and artifact identities were independently checked. A successful identity check on this composition is not a complete-package construction or runtime pass.

## Qualification outcome

| Check | Observed result | Meaning |
| --- | --- | --- |
| Exact baseline and candidate optimized mixed-driver builds | Exit 0 for both | The three-file patch compiles with the pinned compiler API |
| Source pins, clean source state, installed bytes and SDK version equality | Passed | Bounded component provenance is established |
| Complete package constructor | Exit 101 at native shared library | Complete package unavailable locally |
| Existing baseline mixed Cargo qualifier | Exit 101 at native-participation assertion | Native runtime qualification failed |
| Baseline cold Cargo build and application execution within that qualifier | Exit 0; expected output `1083` | Rust fallback executed; native participation was not established |
| Candidate mixed, retained-check, metadata and library gates | Not run after baseline environment failure | Pending |
| Full standalone entry/runtime qualification | Not run; shared library absent | Pending |
| Before/after timing, fixed nine-mode and unchanged-project campaigns | Not run locally | No performance result or admission |

The failed baseline produced five projected native unit sources and zero native object files. Its native-selection trace recorded zero selected definitions. The daemon log contains 4,732 identical EPERM records. A minimal independent Python AF_UNIX stream-socket probe also failed with errno 1 during socket creation, before any bind or compiler code ran. This supports the environment-denied diagnosis. The exact daemon log SHA-256 is `4d46f6a9e72cbb990d481af699aed779dda9c3fb5dab0c66f1b7172748a9950f`.

The qualifier's native-participation assertion was left intact. The failed fixture and diagnostic were preserved; no permission workaround, altered fixture, fallback-only timing campaign or relaxed native assertion was used. Local host contention was not established, and no local absolute runtime is compared with earlier hosted measurements.

## Retained evidence

The public identity file contains a narrow projection of source identities, binary hashes, fixed outcome categories and exact original-log digests. It is an explicit projection, not a rewritten original report. For the initial complete-constructor failure, the digest labeled `original-constructor-outer-stderr` refers to the outer constructor stderr; the separate inner native-library stage has its own digest.

The unchanged repository helper `tools/encrypt-coordinator-evidence.py` archived the frozen local records through its `private/snapshot` input. The archive contains 106 files, including exact build receipts and logs, original qualifier outputs, the generated fixture and projected unit sources, source mappings and labeled composition recipes. Proposed full-package continuation recipes are marked unexecuted. Compiler binaries, build caches and unrelated host state are excluded.

- [Authenticated encrypted originals](encrypted/private-evidence.tar.gz.cms)
- [Archive and encryption receipt](encrypted/receipt.json)
- [Retention status](encrypted/status.json)

The helper checked the regular-file inventory, exact bytes, per-file hashes and archive manifest before encryption. It used the existing public recipient certificate with DER SHA-256 `69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f`. No private key was used during retention. The plaintext archive SHA-256 is `211610cce775657bf35c448d4b93a0e0dc337932f4c779746b95443f305df1fa`; the ciphertext SHA-256 is `a83f9d1bd460a23b5b7f012ae0935e37ad1ef1162c4cc95d418e8ed955904764`.

For review, the same unchanged helper reconstructed a deterministic plaintext archive from the frozen snapshot. Its archive and manifest hashes matched the pre-encryption receipt exactly. The ciphertext was not independently decrypted during this review; no recipient private key was accessed.

The helper's coordinator campaign and five scenario fields correctly remain `not-observed`. This archive is the native-eligibility local build/failure snapshot, not evidence that a coordinator campaign ran. The recipient's private key is required to decrypt the originals.

## Required next evidence

Hosted qualification must build the complete exact candidate package and pass the existing mixed Cargo, real metadata, retained-state, library and standalone entry/runtime gates. The unchanged nine-mode fixed-workload campaign and corrected unchanged-project campaign must retain every mode and any failures. A matched before/after comparison must establish whether these guards deliver a meaningful improvement while retaining the known body-edit and rejection behavior.

The candidate remains a separate draft. It does not include PR #17's cache experiment and is not ready for performance admission or merge on the basis of this local record.
