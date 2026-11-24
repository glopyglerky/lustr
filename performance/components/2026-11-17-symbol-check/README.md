# Local symbol-check component experiment

This measures one operation on one actual TinyCC object. It does not measure
compiler startup, source analysis, Cargo, linking, synchronization, an application
or a complete native request. It cannot supply a whole-compiler speed claim.

Both operations inspect the same closed 64-bit scalar object. The reference
uses the previous production command, `nm --undefined-only --format=posix`,
and checks successful exit plus empty stdout. The candidate reads that object
in process, validates its bounded format, rejects undefined symbols and checks
the expected export. The candidate performs additional structural checks.

| Local operation | Median milliseconds |
| --- | ---: |
| Original nm process, including startup/capture/wait | 1.400631 |
| In-process read and symbol validation | 0.008848 |

There are 64 pairs and 128 retained timed values. Pair order alternates, so each
operation runs first 32 times. No timings or warmups are excluded. Filesystem
and executable caches were not cleared; the preceding compiler/tool checks
left them warm. This is a shared Linux x86_64 workspace, not the historical
hosted compiler runner.

The geometric mean of paired nm/candidate ratios is 181.967650, with conditional
paired-bootstrap interval [157.873509, 210.142279]. These large component ratios
must not be presented as compiler ratios. Separate medians do not define the
paired estimate. The new check does not explain the older hosted 31 ms `nm`
span; this host's reference median was about 1.4 ms.

## Preserved inputs and replay

- [Every timed value](samples.csv) and [derived statistics](summary.json).
- [Exact source/tool/object hashes and selected host facts](identity.json).
- [Fixture source](fixture.c), its original [TinyCC object](fixture.o), and the [empty original error stream](stderr).
- [Frozen measured reader](source/crates/lustr-native-core/src/mixed_symbols.rs) and [probe](source/tools/check-mixed-symbols.rs).

The frozen source bytes match the recorded hashes. The TinyCC object is retained;
the compiled probe, Rust compiler, TinyCC executable and GNU nm are identified
by hashes but are not included. Build locations are omitted from the public
command projection; commands and tool hashes make that omission explicit.

The [replayer](../../../tools/replay-mixed-symbol-measurement.py) verifies source
and input hashes, all pair identities and actual CSV execution order, medians
and statistics. Its regression test rejects shuffled rows even if their digest
is recomputed. It uses
Python's MT19937 `random.Random(20261117)`, 10,000 resamples and
`choices(log_ratios, k=64)`. Each draw samples a complete pair's log-ratio.
Each resampled mean is exponentiated; interval endpoints linearly interpolate
the sorted values at `(10000 - 1) * p`, for p = 0.025 and 0.975.

From the repository root:

```sh
python3 tools/replay-mixed-symbol-measurement.py performance/components/2026-11-17-symbol-check
```

Rebuild the frozen probe in a new output location:

```sh
cd performance/components/2026-11-17-symbol-check/source
rustc --edition=2024 -O tools/check-mixed-symbols.rs -o /absolute/new-symbol-probe
cd ..
/absolute/new-symbol-probe bench fixture.o lustr_mixed_0123456789abcdef /absolute/nm 64
```

Use the recorded compiler and symbol-tool identities for a matched rerun. A new
host, process state or source produces a separate experiment. Original observations
must remain unchanged. This experiment's 128 values are outside all complete
compiler campaign totals.
