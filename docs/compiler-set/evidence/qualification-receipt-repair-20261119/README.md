# Qualification receipt repair

The two selector fixture producers now keep exact tool-receipt bytes inside
their owned output packages. The preceding
[hosted capture failure](../native-cargo-hosted-7c3d095f-20261119/README.md)
correctly rejected two receipt symlinks whose referents were outside the
captured record tree. Its
[original artifact](https://github.com/glopyglerky/lustr/actions/runs/44553094694/artifacts/14546870100)
remains available separately, with ZIP SHA256
`4d7965327d6466f092c704992abca9baf950b8548d5e115225d5d2b5c4b4988a`.
That original failed result has not been replaced by a passing one.

## What changed and what was checked locally

Only `test-native-selection-cache.py` and
`test-native-selection-seed.py` changed for this repair. Each projection copies
`lustr-selection-tools.txt` byte for byte and verifies that the copy is a
regular file. Existing tool and library handling remains intact. The cache
fixture renames the receipt around its FIFO control and restores the same
bytes. The seed fixture verifies receipt bytes after relocation and proves
that its intentional receipt mutation leaves the input and relocated copies
unchanged.

All earlier selection, rejection, timeout, certificate and cache-state
assertions remain. An independent read-only review found no blocking issue.
The packager, production code and workflow were unchanged by this repair.

Before the workspace became unavailable, the two actual selector-component
gates completed using an existing source-matched profiler and package:

| Local gate | Invocations | Passed assertions |
| --- | ---: | ---: |
| Selection cache | 156 | 297 |
| Selection seed | 38 | 87 |

The independent review checked both result reports and every recorded
stdout/stderr hash. Input profiler, tool and receipt hashes matched before and
after execution.

The unchanged capture helper, SHA256
`3363ca74981e99ecc48689dfe46c4cf8f16b5aa6c29dc396599532fffb9b056e`,
then reported `capture_complete=true` for these local fixture outputs. Its
inventory contained 499 entries, with 498 archive members. Capture review
checked 442 regular files containing 1,049,374 bytes. The independent review
also verified the archive and inventory hashes and all three regular receipt
members against their live bytes. The seed package's own receipt intentionally
retained its invalid-source mutation; the input and relocated receipts retained
the original bytes.

## Evidence availability and source reconstruction

The original local raw archive was 46,157 bytes, with SHA256
`7bd374419d45b35dd92a1f1424e13647536c1d3489174098f78f5a660932b99d`.
That archive and the separate normalized 403-record review archive were
**not published**. Neither is currently retrievable after the workspace outage,
and this directory does not provide either archive. The local reports and
streams cited above are also unavailable in this session.
[verification.json](verification.json) preserves the observed counts, checks
and byte identities.

The exact reviewed producer source was reconstructed as GitHub blobs and read
back. Both full SHA256 values match the source reviewed before the outage:

| Source | Git blob | SHA256 |
| --- | --- | --- |
| Selection cache | `29bd738167996011cbb8edd971acbf7f72269ea8` | `31c1316f42bd6d743c9ea8f8a9d1684700f32210406f60cb7ac3dfced13f16f5` |
| Selection seed | `2250e67c4fb82e28768df4a1b394a0930c712b85` | `3fd50ea6421249f153923674add201a1fc842b263834352e28b5cbd1761e3e14` |

A fresh hosted run must establish and retain the repaired capture result.
This record adds no new Rust compiler qualification, hosted capture success,
Darwin result or performance claim.
