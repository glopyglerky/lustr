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

The next deliverable is a qualified Cargo speed improvement from the existing
mixed implementation. The last completed campaign is still slower than matched
Cargo/rustc in eight of nine modes. See [the full results](../../PERFORMANCE.md)
and [current qualification](completion-qualification.md).

1. Qualify the current source-built package after replacing symbol-tool
   subprocesses and the intermediate Cargo coordinator executable. Preserve
   mixed libraries, build scripts, proc macros, real metadata, invalid-source
   rejection, link-failure recovery and panic/drop behavior.
2. Run the complete paired native and Cargo campaign, with profiling in a
   separate directory. Keep cold, no-op, body/signature edits, checks, rejection
   and executing tests separate. Record exact source and compiler identities,
   host facts, raw outputs, every sample, uncertainty and regressions.
3. Publish the original evidence and regenerated README/performance tables.
   A component timing, source guard or passing correctness test does not
   establish a whole-Cargo speedup. The matched stock Lust lanes separately
   measure whether integration preserves the supported native path.
4. Extend actual Cargo coverage to unchanged larger projects and real source
   edits. Keep normal debug profiles visible. The current mixed-object path
   does not support default debug=2 generation, and retained literal-edit reuse
   does not cover arbitrary Rust edits.
5. Complete native Linux and macOS qualification and release review. Preserve
   previous failed evidence and state the exact conditions of each new run.

The [historical native-only roadmap](native-roadmap-history.md) remains available
for research provenance. Its old source status and reconstruction requirements
are superseded; it is not the delivery plan.
