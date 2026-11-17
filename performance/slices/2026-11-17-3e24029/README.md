# Completed public upstream-slice campaign

The corrected public-slice recorder **completed** in [run 44187580063, source-built
job 134408105819](https://github.com/glopyglerky/lustr/actions/runs/44187580063/job/134408105819).
The original GitHub Actions artifact is [14376438585](https://github.com/glopyglerky/lustr/actions/runs/44187580063/artifacts/14376438585).
The byte-identical [original ZIP](original-qualification.zip), its
[source/artifact manifest](manifest.json), 1,547 original files, all 288 timed
observations and all 18 comparisons are retained here. It contains public
fixture and tool evidence, not an arbitrary private-project campaign.

The PR head was `3e24029f75ee9837f3493be8900f7899e2b52e5f`. GitHub Actions
actually checked out merge commit `5711bb91b9a9fde3343b345c6ec23632de597741`,
tree `f083723dbf0e1c078306febd755c22aa2115161d`. The measured compiler
identities are bound inside the original report and command records. This is
**not** an additional benchmark of the earlier `6bf6e3e` candidate, and
merely publishing the results on a descendant branch does not remeasure it.

## Measured results

Twelve paired observations per scenario on a Linux x86-64 hosted runner,
AMD EPYC 9V74, four visible logical CPUs. Medians are milliseconds. Ratios
are paired geometric reference/candidate estimates, not ratios of medians.

| Work | Cargo/rustc median | Lustr standalone-entry median | Ratio [95% interval] |
| --- | ---: | ---: | --- |
| Cold check | 290.058 | 192.505 | 1.516 [1.475, 1.571] |
| No-op check | 10.409 | 35.689 | 0.293 [0.288, 0.298] |
| Cold build | 391.988 | 348.570 | 1.153 [1.100, 1.231] |
| No-op build | 10.669 | 38.231 | 0.283 [0.278, 0.288] |
| Compile plus run | 11.521 | 39.771 | 0.294 [0.287, 0.302] |
| Runtime | 0.800 | 1.099 | 0.742 [0.709, 0.780] |

This is a *pruned/adapted upstream goport-util-small slice* and a standalone
`lustr build/check FILE.rs` entry. It is **not** the mixed Rust/Lust Cargo
route, not an unchanged ts-rust workspace, not a source-edit benchmark and
not a representative runtime-throughput result. The no-op entry goes through
the native daemon, but its measured 35–38 ms overhead is much greater than
pristine Lust's 1.565 ms no-op check and 4.029 ms no-op build on this fixture.
The entry validates Cargo target selection with a separate metadata command;
the campaign does **not** isolate that command's cost. Profile that boundary
before attributing the entire difference or implementing unsafe cache shortcuts.

The original report also contains all pristine → integrated and pristine →
entry comparisons, p95s, execution order, profiles, selected host facts,
verified runtime oracle and exact original command streams. No slower sample
was deleted. The separate first attempt at `6bf6e3e` remains [failed with
zero timings](../2026-11-17-6bf6e3e/README.md), not relabeled as a pass.

## Independent archive replay

From this repository's root:

```sh
python3 tools/check-upstream-slice-archives.py
```

That check verifies the original ZIP digest and size, safe entry names, source
and run identities, every retained file/command/output hash, timings, table,
and runtime/build records. It executes **no archived command or compiler**.
The complete source-built job and original runner also passed their own replay;
this offline check does not establish another compiler qualification.

The SHA-256 of the original 590,978-byte ZIP is
`6ff694d3dd324e62421200cb0d546f563a7a45f78fea35f5f7bef82233ce8605`.
