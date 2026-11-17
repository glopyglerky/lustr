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

Each ZIP is byte-identical to the original GitHub Actions artifact. The
[manifest](campaigns.json) binds its run, artifact, actual checkout/tree,
archive/raw-data hashes and derived-file hashes. It is retained in Git so
Actions expiration does not remove the original qualification record.
The archives include all 3,536 recorded command-output files across the two
campaigns. They do not contain the entire compiler binary toolchain.

These are the separate `lustr-mixed-bench` fixture campaigns produced and
already published by this public repository's GitHub Actions workflow. Review
of both complete artifacts found public fixture/source/build records and
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
  performance/archives/2026-11-17-afb17a8-qualification.zip \
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

After a new completed campaign, download its **original ZIP**, verify its
Actions artifact digest, and run the reporter against it. Keep the ZIP as a new
dated archive, generated CSV/table as new files, and add its exact identities
to the manifest through a reviewed PR. Run both committed-data checks. Preserve
older files. Do not replace an older report with new timings, infer host facts
for a historical run, or publish a partial/failed campaign as a completed one.

The [upstream audit](../docs/compiler-set/upstream-benchmark-audit.md) lists the
additional larger-workspace, edit-family, runtime and resource campaigns needed
to match the breadth of Lust's own studies. Those remain unmeasured here.
