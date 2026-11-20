# Remaining native entry cost

> Source-bound checkpoint. Statements about the current candidate, required
> follow-up and measured speed below refer to the named source and date. The
> [active correction](active-correction.md) identifies present implementation
> work; the [current performance policy](practical-performance.md) controls new
> claims. Historical successes, incomplete paths and failed results remain scoped
> to their original implementation.

The remaining selector cost is roughly three tenths of a millisecond on the
measured fixture. This review found no proven, low-risk change that removes all
of it while preserving the existing validation. This diagnostic made no
compiler or selector changes. The
[practical performance policy](practical-performance.md) governs acceptance of
the public compiler commands.

## Current measurement

On 2026-11-18, the existing
[`profile-native-preflight` helper](../../tools/profile-native-preflight.rs)
timed the production `full::select` call using the complete source-built PR26
package. It parses the actual native request before starting the timer and
serializes its evidence after stopping it. The fixture is the existing
[single-package slice](warm-entry-selector-spans-evidence/fixture).

| Selector mode | Samples | Minimum | Median | p95 | Maximum |
|---|---:|---:|---:|---:|---:|
| Cache enabled after three primes | 24 | 265.025 µs | **341.3595 µs** | 569.900 µs | 786.544 µs |
| Fresh Cargo selection | 24 | 36.392121 ms | **42.5706515 ms** | 76.802983 ms | 148.440904 ms |

The plan contained 53 commands: a fresh selection, three excluded cache primes,
24 alternating pairs, then a final fresh selection. Even pairs ran cache-enabled
first; odd pairs ran fresh first. Every measured observation is retained,
including the last pair's high values. The p95 is the nearest-rank observation
at `ceil(0.95 × n)`. Both fresh controls and all primes are excluded from the
distributions; their numeric values remain in
[`samples.json`](evidence/selector-overhead-20261118/samples.json).

All 53 commands exited successfully with empty stderr. Comparing every output
field except elapsed time found identical results throughout, including the
complete `Selection` manifest, name, package and target values. The helper,
recorded package input binaries, selection-tool receipt and all 43 fixture
files passed the original runner's before/after identity checks. The fresh
lane adds only `LUSTR_NO_SELECTION_CACHE=1` to the common environment. The timed
run did not enable a cache-hit counter, so its label is **cache enabled**.

The host exposed nine CPUs on an AMD EPYC 9V45 system, running Linux 6.18.57.
Other work on the shared host was paused for this bounded probe. CPU affinity,
frequency and operating-system scheduling were not controlled. The
[`summary.json`](evidence/selector-overhead-20261118/summary.json) records the
command, environment settings, host details, identities and validation scope.

## Exact identity

| Input | Identity |
|---|---|
| Product source | `43a7e07d32929309fc6e402d7bca6b5673e186a2` |
| Product source tree | `853e22bc16d1a9d96c00369f13a8fec267162dfc` |
| Native build hash | `f35b6e4b3f083e67` |
| Native build receipt SHA-256 | `c70412c9b47814037f15143a1665640eaf179e23b091f32fac1976a176396a79` |
| Helper binary SHA-256 | `f120808b3c9f5a2ab072a2d649b5c9c153b835e256ebf47059a0aa85845054a1` |
| Helper SDK | `nightly-2026-11-09-x86_64-unknown-linux-gnu` |

The helper includes byte-identical copies of the seven source inputs listed in
the summary. Its release build used the actual package's native build hash;
the native build output used by the helper matched the final package receipt.
This binds the observation to PR26, rather than a later documentation or
correctness-fix commit.

## What the source and earlier profile show

The [earlier selector profile](warm-entry-selector-spans-evidence/README.md)
measured a 285.341 µs median for `full::select`: 224.921 µs in input capture and
52.539 µs in record loading. Toolchain observation accounted for a nested
120.885 µs within capture. These are inclusive spans; adding them double-counts
work. The earlier evidence replay still verifies all 111 commands, 96 retained
frames, complete selection agreement and span distributions.

At the measured source, `full::select` captures once and attempts one record
load. A successful load returns before Cargo metadata is invoked. There is no
duplicate Cargo metadata call on that path. The capture validates environment,
manifest and lock contents, config absence, discovery paths, executable
identities and access, tool receipt and shared-library inputs. Record loading
checks integrity, current state and target bindings. Earlier changes already
reuse opened executable observations.

Small allocation or hexadecimal-formatting changes remain possible experiments,
but the evidence does not identify them as the cause of the full residual cost.
Removing the live input checks would change the correctness contract. No new
cache design or launcher-C reuse change is justified by this probe.

## Limits and evidence access

An initial syscall trace attempt failed with `PTRACE_TRACEME: Operation not
permitted`. It produced no syscall observations. That failed five-command
attempt is recorded separately; the subsequent timing plan explicitly omitted
tracing. No syscall count or category breakdown is claimed.

The current 341.3595 µs observation is the same order as the earlier profile,
but it used a different host and package layout from the published compiler
comparison. It cannot be subtracted from those compiler timings or establish
that selection causes their entire gap. This probe does not measure startup
alone, integrated versus pristine native compilation, or full Rust correctness.

The public evidence contains numeric observations, hashes and a sanitized
validation summary. Original plans, receipts, source copies and output streams
are retained privately. Their hashes identify them; hashes alone do not allow
public replay of omitted bytes. The public verifier checks the projected
arithmetic, order, source bindings and file integrity:

```sh
python3 docs/compiler-set/evidence/selector-overhead-20261118/verify-summary.py
```

It does not reexecute the compiler, rehash omitted binaries or independently
replay the privately retained selection outputs.
