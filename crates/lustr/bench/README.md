# Historical coordinator benchmarks

The runners in this directory record experiments on the former executable
coordinator. That execution path is retired. The current crate's command-line
entry points refuse compilation, so these campaigns cannot produce current
Lustr qualification by selecting `rustc`, `cranelift`, `lust` or a BlueFe root.

The [complete original benchmark instructions](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/bench/README.md)
retain the exact setup, workload, flags, matching rules, statistics and limits.
Use the recorded source revision when reproducing that historical behavior.
Preserve failed samples and original results.

The argument-log parser, matched workload fixtures, output comparisons and
statistical methods may be reused where they fit a current experiment. Each
new campaign must identify the actual compiler and emitted artifacts. A
planner's selected-engine label is insufficient.

Current acceptance compares Lust-covered work with matched stock Lust and
complementary work with matched stock rustc, separately. The
[performance policy](../../../docs/compiler-set/practical-performance.md)
and [exclusive coverage contract](../../../docs/compiler-set/exclusive-coverage-contract.md)
control those comparisons. A green replay of old measurements does not qualify
the current compiler or establish near-stock performance.
