# Performance snapshot for PR #26

This snapshot retains the measured results at `43a7e07d` and their original
evidence. Supported warm work is close to Lust and substantially faster than
the matched Rust reference on the measured native workloads. **The performance
goal is to retain Lust's subset gains while ordinary full-Rust compilation stays
close to stock Rust.** Large gains outside the Lust subset are not required.

The unchanged coordinator's normal-profile medians range from 0.06% faster to
1.04% slower than matched Rust. Those results fit the near-stock direction for
this workload. Fixed Cargo body edits and rejection improve, while seven other
fixed Cargo modes regress. Those opt0/debug0 regressions remain separate from
the normal-profile result and are not classified as near-stock. No universal
overall ratio or guarantee for every Rust configuration follows from these
campaigns. Release qualification remains pending on correctness and platform
requirements.

No compiler, application or timing campaign ran while this snapshot was made.
The original benchmark tools replayed the retained evidence and reproduced the
published dashboard JSON, Markdown and checksums byte-for-byte. Its source
identity is historical; later documentation or code must not inherit its timings.

## Read the results

- [Exact original dashboard](dashboard/README.md) has 29 operation rows for
  Lustr, pristine Lust and the matched Rust reference. It retains all missing
  measurements as `NT`.
- [All 59 original comparisons](comparisons.json) retain the original paired
  geometric ratios, 95% bootstrap intervals, medians, p95, raw paired ratios,
  seeds and test statistics. This includes comparisons against the integrated
  native client that are not separate columns in the dashboard.
- [Public numerical timings](raw-timings.json) retain all 648 fixed-workload
  and 288 slice observations. [Project samples](project/samples.json) preserve
  the further 120 measured timings, 20 excluded warmups and actual order.
- [Unchanged-project report](project/README.md) preserves the normal dev/test
  profile and its five outcomes. No debug-information reduction is used there.
- [Summary for tools](summary.json) identifies host, compiler bytes, profiles,
  paired intervals, coverage and limits. [Audit receipt](audit.json) binds the
  original artifact digests and byte-identical dashboard replay.

Ratios above one favor the named candidate. Paired geometric ratios are not
ratios of medians. Intervals are conditional on this host and repeated workload;
individual exploratory comparisons are not adjusted for multiplicity. No
cross-run or cross-host before/after benefit is inferred.

## Source, tools and host

| Role | Identity |
| --- | --- |
| Published PR head | `43a7e07d32929309fc6e402d7bca6b5673e186a2` |
| Actual merge checkout | `8a7af2e1e2c21743d14dad45c83283232e2bc102` |
| Complete tree, identical for both revisions | `853e22bc16d1a9d96c00369f13a8fec267162dfc` |
| Pristine Lust | `240e08d91d4aed0299639f6f11fd522912b248f9` |
| Full Rust | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Rust / LLVM | `1.102.0-nightly`, `23.1.5` |
| Native host | `x86_64-unknown-linux-gnu` |
| Hosted runner | Ubuntu 24.04, image `20261111.127.1` |
| CPU | Intel Xeon 6975P-C, four visible logical CPUs |
| Cargo jobs | Two |

[Compiler set run 44337186349](https://github.com/glopyglerky/lustr/actions/runs/44337186349)
passed all ten jobs. [The original runner receipt](runner-source/runner-source.json)
binds the run, merge checkout, workflow and PR head. Compiler components use
release builds; workload profiles remain distinct. Fixed native/Cargo fixtures
use dev opt0/debug0 with normal unwind/checks. The unchanged coordinator uses
its original dev/test defaults, including debug=2. Exact tool hashes and all
profile descriptions are in `summary.json` and the original evidence.

Affinity was observed, not pinned. CPU frequency and host load were not held
constant. Cold cases clear their owned compiler/build state; OS page caches are
not flushed. No process CPU, aggregate memory or energy claim is made.
Standalone rustc no-op invokes the compiler; Cargo no-op may be Fresh. These are
different workloads. The public slice is adapted supported upstream code and
does not represent an unchanged complete upstream application.

## Executed compilation coverage

The Linux source-built package passed all 16 public entry/runtime tests and the
mixed Cargo graph, build-script, procedural-macro, core/alloc no_std library,
metadata, library/linking, edit/invalidation, rejection and failure-recovery
gates. The entry contract executed no_std WebAssembly in an independent runtime
and exercised panic/drop behavior. The unchanged coordinator's executing tests
reported 41 passed and nine existing explicit ignores, with matching initial
and final tests and structured borrow-error rejection in both lanes.

Cargo coordinator compatibility passed on Linux, macOS and Windows. Native
mixed runtime execution in this campaign remains Linux x86_64. Four target
layout comparisons each checked 26 size/alignment facts and semantic rejection;
foreign linking and execution are separate obligations. Complete native macOS
qualification remains open and the primary native daemon is not a Windows
runtime.

Full Rust uses the identified upstream Rust implementation. These executed
cases do not prove every possible Rust program or platform. The normal debug=2
project lies outside current mixed native-object eligibility. Its measured
results do not establish general native acceleration.

The separate [complete-package run 44337186487](https://github.com/glopyglerky/lustr/actions/runs/44337186487)
failed baseline qualification before timing. This snapshot supplies no
controlled before/after gain for PR #26's no-change materialization change.
[PR #24](https://github.com/glopyglerky/lustr/pull/24) separately retains a native
main function-type repair and unresolved Result-error transition behavior.
Neither the warm-speed budget nor these passing Compiler set jobs resolves that
correctness/platform work. Ordinary full-Rust compilation should remain close
to stock Rust; this snapshot does not establish that every mixed configuration
meets that direction.

## Original artifact retention

The following are the exact original ZIP byte identities from the run. Their
names and hashes refer to downloaded originals, not rewritten public projections.

| Artifact | GitHub artifact ID | SHA-256 |
| --- | --- | --- |
| [coordinator-project-originals-encrypted-1](https://github.com/glopyglerky/lustr/actions/runs/44337186349/artifacts/14444452602) | `14444452602` | `7dc757a10b7c69fe964156796020a15b3b8a89deb2c34f9d3edac6fefa415e20` |
| [coordinator-project-performance-1](https://github.com/glopyglerky/lustr/actions/runs/44337186349/artifacts/14443883811) | `14443883811` | `d48037291f7ce3b3b5e4f6b68cfe18e5b0f84050dc536ef59e9572738c6d710b` |
| [runner-source-44337186349-1](https://github.com/glopyglerky/lustr/actions/runs/44337186349/artifacts/14442271569) | `14442271569` | `4c057b33ada72c2011626617531e565a151608ee209f18fef4f75df9cc3ab05d` |
| [source-built-mixed-qualification-1](https://github.com/glopyglerky/lustr/actions/runs/44337186349/artifacts/14444082989) | `14444082989` | `64cb6a4fed94a12b6fc69a4f1c32d7c575617938a441094e33b534b7eb1eefa7` |
| [three-compiler-performance-1](https://github.com/glopyglerky/lustr/actions/runs/44337186349/artifacts/14443254589) | `14443254589` | `c74d551cc63b56f07fa6c570e4338cbb59d494bf74da8fd394d9f3573250039c` |
| [upstream-public-slice-1](https://github.com/glopyglerky/lustr/actions/runs/44337186349/artifacts/14443288234) | `14443288234` | `e15ce3dce10c667f827b112c87d0abbfd21c04ae01776093652f3326391e1a61` |

This public snapshot contains sanitized numerical samples, comparisons,
source/compiler identities and the original public dashboard/project exports.
Encrypted archives and original private command streams are not included in the
public commit. The exact original artifact URLs and hashes above remain the
inputs for a complete replay; retain those originals separately before Actions
artifact expiration. The original archives were retained privately during this
review, without disclosing their contents through the public repository.

## Reproduce the evidence review

From the repository root, check the published snapshot and exact dashboard:

```sh
cd performance/2026-11-18-43a7e07
sha256sum -c SHA256SUMS
python3 replay/report-three-way-performance.py --replay dashboard
```

This verifies the public snapshot and deterministic rendering without compiler
execution. Public timing projections support numerical inspection but do not
replace the full command/stream/source qualification records.

The complete original replay additionally needs the original artifact ZIPs.
Retrieve them through the workflow links above, verify their digests against
`audit.json`, and extract the source-built, public-slice and project ZIPs into
distinct private directories. The frozen original renderer calls the original
fixed/slice replayers and validates the unchanged-project export:

```sh
python3 replay/report-three-way-performance.py   --source-revision 8a7af2e1e2c21743d14dad45c83283232e2bc102   --fixed-root /private/path/source-built   --slice-dir /private/path/public-slice   --project-public /private/path/project   --fixed-outcome success --slice-outcome success --project-outcome success   --output /private/path/new-dashboard
```

Use a new output directory. Its three files must match `dashboard/` exactly.
The [frozen replay identities](replay/identities.json) bind every copied tool to
PR #26. No original renderer or measurement source was modified for this snapshot.
