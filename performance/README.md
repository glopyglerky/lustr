# Performance evidence index

[PERFORMANCE.md](../PERFORMANCE.md) explains the measured results, methodology,
regressions and remaining coverage. The [README summary](../README.md#numbers-and-current-qualification)
keeps the headline table visible. Neither document claims general Rust speed
parity or release readiness.

## Complete historical campaigns

| Measured candidate | Full table | Every timed value | Original qualification artifact |
| --- | --- | --- | --- |
| `c70eb1c` | [27 comparisons](2026-11-17-c70eb1c.md) | [540 values](2026-11-17-c70eb1c.csv) | [Original ZIP](archives/2026-11-17-c70eb1c-qualification.zip) |
| `afb17a8` | [27 comparisons](2026-11-17-afb17a8.md) | [540 values](2026-11-17-afb17a8.csv) | [Original ZIP](archives/2026-11-17-afb17a8-qualification.zip) |
| `048d824` | [27 comparisons](2026-11-17-048d824.md) | [540 values](2026-11-17-048d824.csv) | [Original ZIP](archives/2026-11-17-048d824-qualification.zip) |
| `dec0ce2` | [27 comparisons](2026-11-17-dec0ce2.md) | [540 values](2026-11-17-dec0ce2.csv) | [Original ZIP](archives/2026-11-17-dec0ce2-qualification.zip) |

Each ZIP is byte-identical to the original GitHub Actions artifact. The
[manifest](campaigns.json) binds its run, artifact, actual checkout/tree,
archive/raw-data hashes and derived-file hashes. It is retained in Git so
Actions expiration does not remove the original qualification record.
The archives include all 7,072 recorded command-output files across the four
campaigns. They do not contain the entire compiler binary toolchain.

These are the separate `lustr-mixed-bench` fixture campaigns produced and
already published by this public repository's GitHub Actions workflow. Review
of these complete artifacts found public fixture/source/build records and
hosted-runner paths, with no private workspace content or credential material.
Their original bytes and hashes are preserved. The private arbitrary-workspace
`lustr-bench <spec.json>` reports described in
[benchmark-method.md](../docs/compiler-set/benchmark-method.md#report-schema-and-publication)
still require the documented sanitization before public use. This archive does
not change that rule or authorize publishing private reports.

From the repository root:

```sh
python3 tools/test-mixed-performance-report.py
python3 tools/check-mixed-performance-tables.py
python3 tools/check-mixed-performance-archives.py
python3 tools/report-mixed-performance.py \
  performance/archives/2026-11-17-dec0ce2-qualification.zip \
  --output /absolute/new-report-directory
```

The last command refuses an existing output directory. It verifies the input
before writing a report and never runs a command copied from a log. Corrupted
or incomplete original evidence fails verification. Historical table replay
does not test a newly built compiler.

## Record the next campaign

The [source-built workflow](../.github/workflows/compiler-set.yml) builds matched
pristine Lust, integrated Lust, packaged Lustr and the full Rust/Cargo reference.
All correctness prerequisites must pass before its paired measurements. It
preserves cold, no-op, body/signature edits, check variants, rejection and
compile-plus-test separately, with every slower sample.

The same workflow now captures selected host facts and performs phase profiling
in a separate qualification directory. Host facts are bound into new campaign
identities. Profiling is disabled for all paired samples. The full artifact
retains source/build receipts, commands, outputs, fixture files, profiles,
host records and failure information.

The first [completed phase report](2026-11-17-dec0ce2-profile.md) is linked
separately from its paired timings. Its original traces and host record are
inside the fourth qualification ZIP; their instrumented values are not pooled
with the 2,160 timed command values.

After a new completed campaign, download its **original ZIP**, verify its
Actions artifact digest, and run the reporter against it. Keep the ZIP as a new
dated archive, generated CSV/table as new files, and add its exact identities
to the manifest through a reviewed PR. Run both committed-data checks. Preserve
older files. Do not replace an older report with new timings, infer host facts
for a historical run, or publish a partial/failed campaign as a completed one.

The [upstream audit](../docs/compiler-set/upstream-benchmark-audit.md) lists the
additional larger-workspace, edit-family, runtime and resource campaigns needed
to match the breadth of Lust's own studies. Those remain unmeasured here.

## Unchanged project coverage

The separate [coordinator campaign](../docs/compiler-set/coordinator-performance.md)
runs this repository's actual coordinator package under its normal dev/test
profiles. Its five scenarios preserve test execution, declared cold/warm state,
compiler identity and raw timing values. A strict public projection omits raw
private output and environment paths while retaining their original digests.
This differs from the complete original ZIPs above. The workflow has been wired;
no project timing result is claimed before a real campaign completes.
The private full logs are separately encrypted to a pinned public recipient;
success and failure paths both attempt retention. Only the ciphertext and
allowlisted receipt/status may be published. The decryption key stays private,
and actual recovery must be verified before the retention record is complete.
