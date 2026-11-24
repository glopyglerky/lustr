# Native library-selection qualification

The corrected native compiler passes **22 matched Cargo/native cases**:
valid, invalid and recovered library source for seven layouts, plus a bin-only
package that must reject `--lib`. The actual Lustr-entry regression also passes,
with **52 child-command output pairs** retained. Successful and recovered Lustr
checks must report `lust` / `upstream-process`, so a full-Rust fallback cannot
hide a broken native path.

The correction is [described and attributed here](../../native-library-selection.md).
It adapts Jeff McDonald's upstream Lust selector while preserving all six
library kinds and rejecting a nonexistent library target. This is a local
x86_64 Linux correctness qualification using development-profile native builds.
It does not measure speed, execute the fixture applications, qualify library
artifact interoperability, or admit a compiler release.

## Exact inputs

| Input | Identity |
| --- | --- |
| Baseline revision | `dec0ce2e3a68b82cac16eae238fc51b16e534e5e` |
| Baseline native tree | `0c196f3b8f0803cbf3985cc2f4be687129831e26` |
| Qualified source revision | `b4ae69f676eaf89ae3e4ab087aa5906307b9cb2a` |
| Qualified native tree | `cb797ce8ce82192b87b3835267fc223c295dba70` |
| Full Rust source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` (`nightly-2026-11-09`) |
| Cargo | `1.102.0-nightly (c07ac2c4d 2026-11-06)` |
| TinyCC source | `43c7708b85681a2fd4451c8a541af4494a8919b2`, unchanged source with the existing PIC build constructor |

The [baseline report](baseline-report.json) and
[candidate report](candidate-report.json) contain exact client/core, Rust,
Cargo and runner SHA256s. Each report retains 54 commands: 22 native checks,
22 Cargo oracle checks, seven Cargo metadata queries and three identity queries.
Both use `LUST_NO_DAEMON=1` and `LUST_NO_CACHE=1`. The selected SDK's directory
comes first on `PATH`, ensuring that native metadata discovery invokes the
same Cargo binary as the explicit oracle. The runner checks that compilation
does not modify its recorded inputs.

## Observed results

Codes below are process exit codes. Cargo reports semantic and selection
failures as 101; native semantic errors are 1 and target-selection failures 2.

| Case | Baseline native | Corrected native | Cargo |
| --- | ---: | ---: | ---: |
| Hyphenated package: valid | 2 | 0 | 0 |
| Hyphenated package: invalid | 2 | 1 | 101 |
| Hyphenated package: recovered | 2 | 0 | 0 |
| Renamed library beside invalid package-named binary: valid | 1 | 0 | 0 |
| Renamed library: invalid | 1 | 1 | 101 |
| Renamed library beside invalid package-named binary: recovered | 1 | 0 | 0 |
| Explicit `rlib`: valid / invalid / recovered | 0 / 1 / 0 | 0 / 1 / 0 | 0 / 101 / 0 |
| `staticlib`: valid / invalid / recovered | 0 / 1 / 0 | 0 / 1 / 0 | 0 / 101 / 0 |
| `cdylib`: valid / invalid / recovered | 0 / 1 / 0 | 0 / 1 / 0 | 0 / 101 / 0 |
| `dylib`: valid / invalid / recovered | 0 / 1 / 0 | 0 / 1 / 0 | 0 / 101 / 0 |
| `proc-macro`: valid / invalid / recovered | 0 / 1 / 0 | 0 / 1 / 0 | 0 / 101 / 0 |
| Bin-only package requested with `--lib` | 0 | 2 | 101 |

The proc-macro fixture uses a private helper. It exercises actual Cargo target
selection and compiler checking without claiming execution of a procedural
macro. The final corpus preserves the previously accepted library kinds.

## Entry regression and retained first failures

[The entry report](entry-report.json) binds the actual Lustr executable, native
client/core and compiled regression-test binary. The focused qualification
assembly contains those source-built executables and links to the matching
pinned SDK. It is not the complete source-built distribution. The existing
`compiler-set.yml` entry-contract step also runs this ignored test against its
complete constructed toolchain.

The regression checks both direct `lust` and `lustr`, forces native process
startup without a daemon or response cache, rejects invalid source, checks
recovery, and refuses a missing library. The original baseline test failed on
the hyphenated-library selection. A direct Lustr baseline probe also showed
that the renamed-library case succeeded only after native rejection and
full-Rust fallback.

The evidence archive retains the first six-case discovery runs, initial setup
errors caused by a missing Cargo executable on `PATH`, and their corrected
invocations. It also retains two independent review findings: the initially
carried-forward selector accepted a bin-only package, and its first strict
revision rejected four other valid library kinds. Actual metadata, baseline,
intermediate and corrected outputs remain separate. Initial discovery used the
ambient Cargo shim; the final baseline/candidate reports above pin Cargo on
`PATH` explicitly and are the matched qualification.

## Replay and evidence integrity

Build native Lust from the desired committed checkout with the pinned SDK and
TinyCC installation, keeping build output outside the source tree. Then run:

```sh
python3 docs/compiler-set/measurements/2026-11-17-native-lib-selection/runner.py \
  --lust /absolute/native-build/debug/lust \
  --cargo /absolute/matching-sdk/bin/cargo \
  --rustc /absolute/matching-sdk/bin/rustc \
  --repository /absolute/source-checkout \
  --output /absolute/new-evidence-directory
```

The runner requires a new output directory and a committed native source tree.
It retains each command's stdout/stderr and status even when qualification
fails. Run the same runner against the preserved baseline to reproduce the
original failures. Its baseline and candidate runner SHA256 is
`1f4da031f1dab2af90439592bff0507b5aa5f982833b5af04cc9ee2e37185b1f`.

[`raw-evidence.tar.gz`](raw-evidence.tar.gz) retains 975 files covering the
discovery, failed firsts, matched corpus, entry-regression output, source fixtures
and available build receipts. [`evidence-manifest.json`](evidence-manifest.json)
binds every archived member's size and SHA256, plus the archive itself. Compiler
binaries are identified by hash and remain build outputs.

These are original isolated Linux-executor records from generated public
fixtures, not previously published hosted-CI logs. They retain generic executor
paths such as `/workspace/scratch/...` and `/home/agent`. Publication review
found no private project content or credential material. The separate policy
for arbitrary private-workspace benchmark reports is unchanged.

Archive SHA256:
`80b1f376b3c3b2421250ac82151946ea0e81498fbeb91ffe1bc5f8894e33d272`.

The exact locally qualified Git revision and both intermediate review failures
are preserved in [`source.bundle`](source.bundle), with size and SHA256 in
[`source-bundle.json`](source-bundle.json). Publishing the combined follow-through
creates a different outer commit; it does not rewrite the original source
identity in these records. The bundle requires the already published baseline
`dec0ce2e3a68b82cac16eae238fc51b16e534e5e`. From a clone containing that baseline:

```sh
git bundle verify docs/compiler-set/measurements/2026-11-17-native-lib-selection/source.bundle
git fetch docs/compiler-set/measurements/2026-11-17-native-lib-selection/source.bundle \
  refs/heads/lustr/lib-target-correction-20261117:refs/remotes/evidence/native-lib-selection
git show b4ae69f676eaf89ae3e4ab087aa5906307b9cb2a:compiler/native/src/resolve/cargo.rs
```
