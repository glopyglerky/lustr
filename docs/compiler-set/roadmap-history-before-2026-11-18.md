# Historical roadmap snapshot

This file preserves the document before the explicit architecture clarification
of 18 November 2026. It is not the governing plan. Follow
[architecture-plan.md](architecture-plan.md), [native-contract.md](native-contract.md),
[design.md](design.md) and [roadmap.md](roadmap.md). Statements about earlier
requirements or completed work below retain their historical scope.

---

# lustr delivery roadmap

1. Inspect the existing combined entry and packaged runtime. Reproduce the
   specific earlier failures and preserve their evidence.
2. Deliver matched Lust, full Rust implementation, Cargo, sysroot and runtime
   components through lustr, with explicit identities and licenses.
3. Qualify full Rust, dependencies, build scripts, proc macros, no_std,
   rejection, panic/drop and actual Cargo artifact behavior.
4. Verify that supported work reaches Lust, retains its incremental path,
   and preserves its performance in the integration.
5. Continue GitHub/default-branch delivery without losing PR7 or experiments.

Imported/copied Rust implementation is allowed. Semantic-image expansion is
retained research and is not the assumed route to full-language delivery.
See the [current contract](native-contract.md).

## Current acceptance and priority, 18 November 2026

The current goal is close-to-Lust supported work plus near-stock ordinary full
Rust, as defined in [the practical criteria](practical-performance.md). Substantial
speed gains outside Lust's subset are not required. The latest normal-profile
project medians range from 0.06% faster to 1.04% slower than matching Rust.
Keep the separate mixed opt0/debug0 regressions visible. Complete native runtime
and platform qualification before publishing an installable package, and
investigate the remaining public-entry cost without risking working semantics.
The [current snapshot](../../performance/2026-11-18-43a7e07/README.md) supersedes
the older status below; it does not rewrite its original evidence.

## Cargo performance delivery

The latest complete paired campaign at `6bf6e3e` measures a body-edit Cargo
ratio of **1.079× [1.070, 1.088]** and faster rejection. Seven other Cargo modes
remain slower, and normal-profile project builds still show overhead. See
[the full results](../../PERFORMANCE.md) and
[current qualification](completion-qualification.md).

1. Completed: qualify the package after replacing symbol-tool subprocesses
   and the intermediate Cargo coordinator executable. The source-built package
   passes mixed correctness and sixteen entry/runtime tests while preserving
   libraries, build scripts, proc macros, metadata, rejection, link-failure
   recovery and panic/drop behavior.
2. Completed: retain the full paired native/Cargo campaign, separate profile,
   normal-profile project evidence and updated README. Eight fixed campaigns
   preserve 4,536 timings and 234 comparisons; three project campaigns add
   360 timings and fifteen comparisons. Each result retains exact source,
   original output, uncertainty and regressions.
3. Next: reduce cold/signature/check overhead and broaden Cargo gains. The fresh
   body profile records about 8 microseconds in symbol validation and 15 ms in
   linking; the former process cost is no longer the dominant observed span.
   Preserve full Rust metadata and complete linking when selecting the next
   change. The separate public-slice recorder is corrected after its original
   zero-sample failure and still requires fresh hosted execution.
4. Extend actual Cargo coverage to unchanged larger projects and real source
   edits. Keep normal debug profiles visible. The current mixed-object path
   does not support default debug=2 generation, and retained literal-edit reuse
   does not cover arbitrary Rust edits.
5. Complete native Linux and macOS qualification and release review. Preserve
   previous failed evidence and state the exact conditions of each new run.

The [historical native-only roadmap](native-roadmap-history.md) remains available
for research provenance. Its old source status and reconstruction requirements
are superseded; it is not the delivery plan.
