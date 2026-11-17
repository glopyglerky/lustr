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

## Cargo performance delivery

The latest complete paired campaign at PR13 head `3e24029` (actual merge
checkout `5711bb9`) measures a body-edit Cargo ratio of
**1.046× [1.036, 1.055]** and faster rejection. Seven other Cargo modes
remain slower, and normal-profile project builds still show overhead. See
[the full results](../../PERFORMANCE.md) and
[current qualification](completion-qualification.md).

1. Completed: qualify the package after replacing symbol-tool subprocesses
   and the intermediate Cargo coordinator executable. The source-built package
   passes mixed correctness and sixteen entry/runtime tests while preserving
   libraries, build scripts, proc macros, metadata, rejection, link-failure
   recovery and panic/drop behavior.
2. Completed: retain the full paired native/Cargo campaign, separate profile,
   normal-profile project evidence and updated README. Nine fixed campaigns
   preserve 5,184 timings and 270 comparisons; three authenticated recovered
   project campaigns add
   360 timings and fifteen comparisons. Each result retains exact source,
   original output, uncertainty and regressions.
3. Next: reduce cold/signature/check overhead and broaden Cargo gains. The fresh
   body profile records about 8 microseconds in symbol validation and 15 ms in
   linking; the former process cost is no longer the dominant observed span.
   Preserve full Rust metadata and complete linking when selecting the next
   change. The corrected public-slice recorder now passes a fresh source-built
   hosted campaign with 288 timings and 18 comparisons. Its standalone entry
   is slower than matching Cargo for no-op check/build, despite faster cold
   checks/builds on the pruned slice. Separately profile per-request Cargo
   metadata target selection, native-daemon request and Rust fallback before
   changing correctness-sensitive routing or caching.
4. Extend actual Cargo coverage to unchanged larger projects and real source
   edits. Keep normal debug profiles visible. The current mixed-object path
   does not support default debug=2 generation, and retained literal-edit reuse
   does not cover arbitrary Rust edits.
5. Complete native Linux and macOS qualification and release review. Preserve
   previous failed evidence and state the exact conditions of each new run.

The [historical native-only roadmap](native-roadmap-history.md) remains available
for research provenance. Its old source status and reconstruction requirements
are superseded; it is not the delivery plan.
