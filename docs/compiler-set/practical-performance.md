# Practical performance and release criteria

The goal is a combined compiler that preserves Lust's large gains on supported
work and keeps ordinary full Rust close to the matching stock Rust compiler.
Large speed gains outside Lust's subset are not required. Correctness has
priority over removing a fraction of a millisecond.

These budgets apply alongside [the governing integrated-compiler plan](architecture-plan.md).
They do not authorize a coordinator, language selector or whole-program
compile/fail/retry architecture. Current compatibility-path measurements remain
useful evidence for those paths; meeting a budget is not completion of the
required native/complementary compiler boundary.

These are engineering budgets for deciding when to stop tuning. They are not
claims of identical performance, universal Rust correctness or one speed ratio
for every project. The budgets were chosen after inspecting the existing
measurements. Historical observations can be described against them, but that
does not turn an old strict-parity failure into a prospectively qualified run.

## Supported warm work

For each matched warm check and build, allow up to **10% overhead or 0.5 ms
added latency**, whichever permits the larger bound. Preserve the original
strict-parity result alongside this practical decision. Require the same
source, requested configuration, complete commands and actual correctness
gates. No averaging a slow operation away with a faster one.

A new complete-package campaign must replay its full evidence first. Its
practical assessor uses the existing paired ratio interval for the relative
budget and a separately reported paired-bootstrap interval for median added
latency. The upper added-latency bound must be at most 0.5 ms, or the lower
Lust/Lustr ratio bound at least 1/1.10. Both warm operations must also have a
Rust/Lustr ratio interval entirely above one. These intervals are conditional
on the repeated workload and host, not a guarantee for another machine.

At the retained PR #26 snapshot, warm public check adds 0.309 ms by the medians,
and warm build adds 0.206 ms. They fit the absolute budget descriptively.
That broad campaign is separate from the complete-package before/after gate;
no new causal gain or independent-package qualification follows from it.

Cold work, body/signature edits, rejection, priming, tails, program runtime and
compile-plus-run/test remain visible. The allowance for warm integration work
does not excuse a correctness failure or silently relax those other results.

## Ordinary full Rust

Use unchanged projects, dependencies, requested features and normal profiles.
Imported Rust implementation must supply complementary semantics and genuine
metadata/artifact services through the integrated boundary. Historical tables
also measure transitional packaged-compiler paths and must retain that scope.
Do not lower debug information, change panic behavior or substitute an easier
program to make the comparison favorable.

Use **5% relative overhead or 0.5 ms absolute overhead** as the initial
near-stock review budget for each ordinary full-Rust operation. Report paired
intervals and each operation separately. A result outside that budget needs
investigation, rather than a claim that Lust gains elsewhere cancel it.
No large speedup is required on this route.

The retained normal-profile coordinator project has median changes from
0.06% faster to 1.04% slower across its five scenarios. That fits the near-stock
direction for this measured project. It does not establish all projects or
platforms. A no-op may be Cargo Fresh, while the forced rebuilds change source
timestamps rather than contents.

The separate opt0/debug0 mixed fixture still loses on seven of nine operations.
Those losses are not covered by the ordinary project's near-stock result.
Keep them in the README and complete report. Further mixed acceleration should
address a demonstrated substantial cost without weakening semantics.

## Investigating the remaining fraction of a millisecond

The existing actual selector profile measured 285.341 microseconds, with
224.921 in input capture and 52.539 in record loading. Toolchain observation
is a nested 120.885 microseconds within capture. These are inclusive spans;
adding them double-counts work. The profile used a different package/host from
the public benchmark, so it cannot be subtracted from command latency.

Test selector validation, process/loader startup and integrated native work
separately on a matched host. Compare complete selected values with real fresh
Cargo. Keep diagnostics outside timed commands. Preserve configuration,
permissions, compiler/library identity, target discovery and cache invalidation
checks. The existing profile does not identify formatting as the source of the entire gap.

Upstream's default `next` was still the pinned `240e08d9` in the November 18
audit. Relevant statics/codegen/cache speed work is already present. Open
upstream proposals concern correctness or diagnostics, not an established
unimported speed improvement. Current library selection already adapts upstream
PR #11 with additional target-kind checks.

## Publication

Publish an installable package only after the exact delivered source and
components satisfy the [architecture](architecture-plan.md#completion-and-publication)
and pass the relevant runtime, rejection, Cargo, dependency, proc-macro,
panic/drop, invalidation and platform qualification. Retain source identities,
notices, relocation results and actual engine routing. The passing 16-entry
contract suite is useful evidence, not a proof for every Rust program.

The [native main C type repair](native-main-type-repair.md) is now ported onto
the current native tree with bounded cold tests. The older warm Result/ExitCode
findings remain unassigned, and current sanitizer/daemon qualification remains
open. The original candidate's [five cold Linux cases](main-return-smoke.md)
pass but do not establish retained transitions.
The [single-file check fix](full-check-metadata.md) also requires qualification
in the exact source-built package. Native macOS qualification is incomplete and
the primary native daemon has no Windows adapter. A package must name the
platform actually qualified; coordinator compatibility is a separate claim.

Meeting a practical speed budget does not override these conditions. Keep the
existing candidate as the working basis, preserve failures and published
experiments, and avoid broad compiler changes for negligible timing gains.
