# Verified unchanged-project campaign at `40d2ac3`

The real coordinator campaign passed in [run 44096292176, job 134084163908](https://github.com/glopyglerky/lustr/actions/runs/44096292176/job/134084163908).
Its original encrypted reports were subsequently recovered, all original files
were verified, and all five public comparisons reproduced. The original job
still failed its separate profiling step: concurrent trace records were joined
together. Passing this project campaign does not convert that job into a pass.

## Results and exact work

This measures the unchanged `crates/lustr` package through matched Rust and the
assembled primary Lustr entry. Normal dev/test defaults, debug information level
2, features and panic behavior stay intact. This profile is outside the current
mixed-object boundary; these are complete-project compatibility/overhead results,
not evidence of native acceleration. See the [campaign protocol](../../../docs/compiler-set/coordinator-performance.md).

Twelve matched pairs per scenario follow two excluded warmup pairs. Ratios are
the geometric mean of **matched Rust time / Lustr time**. Values below one favor
matched Rust; they are not ratios of the displayed medians.

| Scenario | Matched Rust median (ms) | Lustr median (ms) | Paired ratio [conditional 95% interval] |
| --- | ---: | ---: | --- |
| Clean dependency build | 523.001 | 530.715 | 0.9868 [0.9797, 0.9931] |
| Primed root build | 152.428 | 156.120 | 0.9820 [0.9682, 0.9959] |
| Primed root check | 58.046 | 61.243 | 0.9564 [0.9432, 0.9696] |
| No-op build | 10.017 | 11.046 | 0.9125 [0.8759, 0.9506] |
| Compile plus test | 4961.250 | 5041.113 | 0.9860 [0.9802, 0.9922] |

All five intervals favor matched Rust on this repeated project/host. These are
exploratory individual intervals, with no multiplicity correction or general
equivalence claim. The [original public report](public/README.md) includes p95s;
[samples.json](public/samples.json) includes every measured timing, actual order,
excluded warmups, flags, cache/daemon receipts, gates, compiler/source identities
and original evidence digests. [timings.csv](timings.csv) is a derived lossless
projection of the 120 timed command values, retaining pair order and stream hashes.

Primed scenarios force the root by changing only the `src/lib.rs` timestamp.
They are neither body nor signature edits. Cold work clears owned build/native/
mixed/temp state after observed daemon shutdown, while OS page caches and
dependency sources remain warm. The project has no registry dependencies.
Compile plus test executes its normal test command, including existing nested
builds. Both lanes pass initial/final executing-test and E0502 rejection gates
in every scenario. The project test gate reports **40 passed and 9 explicitly
ignored** tests; no existing ignore was removed or counted as a pass.

The [host record](public/host.json) reports a Linux x86-64 GitHub runner with
Intel Xeon 6975P-C, four visible logical CPUs/affinity 0–3 and 16,464,297,984 bytes
of physical memory. The campaign uses two Cargo jobs. Frequency-governor and
cgroup CPU/memory limits are unreported. No process CPU, RSS, energy or separate
application runtime is measured. Host sharing and a small sample count limit
tail and cross-machine inference.

## Source and artifact identity

| Identity | Value |
| --- | --- |
| PR head | `40d2ac3f0b71abaa1e73cd366d8c43f8eb4d4581` |
| Actual merge checkout | `3b4b0bd67401085129d26be1afbd20e96c9b9d02` |
| Repository tree | `78318c4b01f8eb6f9518ea331efe69f0c457b1ce` |
| Coordinator Git tree | `3f72ccd2f1c5f83f952e2a43564a046a32d75305` |
| Runner Git tree | `2ac3fa9d6f6c281e1c7e36e3fe63d52691868efc` |
| Matching full-Rust revision | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Original public artifact | `14336054692` |
| Original public ZIP SHA256 | `666c19ddfaa87b197419f419735e3806b0562ec2facfbd6cb4b2bdde768bea32` |
| Original encrypted artifact | `14336433890` |
| Original encrypted ZIP SHA256 | `9562ee5fec44311d7a278885b74889b163a2d4196468a33584399c6f47a7fa03` |

The [retained source bundle](../../sources/2026-11-17-40d2ac3/README.md) preserves
the exact merge object even after the live PR merge ref moves. The public
projection's six files are copied byte for byte, including their original
SHA256SUMS. Both original artifact ZIPs are also preserved byte for byte under
[`artifacts/`](artifacts/), beyond GitHub's reported 15 February 2027 expiry.
[manifest.json](manifest.json) binds the committed files; the original API
metadata and job state at completion are retained separately.

## Actual recovery and replay

Authenticated decryption succeeded with the dedicated private recipient key.
The archive/manifest verifier then checked **600 original files totaling
9,757,580 bytes**, including five reports and **510 raw command-output streams**.
No original plaintext or private key is published here. The ciphertext remains
inside `artifacts/encrypted-original.zip`; [retention/receipt.json](retention/receipt.json)
binds its exact hash, archive hash, recipient identity and encryption parameters.

The [reconciliation result](recovery/result.json) freshly hashes every restored
original, verifies the frozen source against the recorded Git tree, and loads
the exporter from that recorded revision with replacement refs disabled. All
five cases match, the public samples/README reproduce byte for byte, the other
public receipts agree semantically, and the existing Rust replay reproduces all
five statistical comparisons.

The first extraction used a Python tar data filter that changed 61 ordinary
0664 modes to 0644. The verifier correctly rejected it with `private-file-mode`.
The [initial failure](recovery/initial-extraction.json) is retained; the
documented `tar --same-permissions` extraction into a new private directory
preserved the original modes and passed complete reconciliation. No original
bytes or integrity checks were changed to obtain a pass.

The [recovery receipt](recovery/receipt.json) and [local replay build receipt](recovery/replay-build.json)
identify the tools used. The replay comes from the same runner source subtree
and matching Rust SDK, but its binary hash differs from the original CI replay.
The executed reconciler bytes also match reviewed revision `3a5b3192`, preserved
in the [recovery source bundle](../../../docs/compiler-set/measurements/2026-11-17-project-profiler-source/README.md).
The absent original compiler, runtime, constructor and runner binaries were not
freshly hashed or requalified. Exporter runtime identity was not recorded by
campaign version 1 and remains null. Recorded identity consistency and original
CI provenance are retained without claiming a new binary-to-source build proof.

For public statistical replay, build the existing replay example and run:

```bash
cargo run --manifest-path crates/lustr-bench/Cargo.toml --release --locked \
  --example replay -- performance/projects/2026-11-17-40d2ac3/public/samples.json
```

Full recovery also needs the separate private recipient key and the verified
source revision; use the [recovery instructions](../../../docs/compiler-set/coordinator-performance.md#recovery).
Replaying evidence does not execute a new compiler or rerun the campaign.

CI also runs `tools/check-coordinator-performance-archives.py` with the existing
replay executable. It checks every committed file hash, the exact public and
encrypted archive members, their GitHub artifact identities, the public checksum
manifest, ciphertext binding, all 120 CSV timings and the five comparisons.
That public check does not decrypt or freshly requalify the private originals.
