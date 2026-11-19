# Hosted mixed qualification harness repair

The first hosted run of the expanded scalar admission source exposed two
qualification assumptions. Neither failing assertion demonstrated incorrect
generated program output on that source.

The Cargo graph qualifier still required five native definitions. The actual
trace contained six admitted definitions and six omitted LLVM definitions,
including the newly supported `nested::kernel`; its cold executable had already
produced the expected result. The current qualifier now requires that exact
six-path inventory. The unchanged historical compiler still admits five, so CI
builds a separate baseline qualifier with the explicit
`historical_mixed_baseline` compile-time expectation. That qualifier must reach
the original failed-link stale-output witness. The current qualifier cannot
accept the historical count, and the baseline cannot accept an unrelated failure.

The scalar qualifier's cold compilation and execution succeeded. Its next step
deleted only Cargo's published executable and expected another compiler request.
Cargo restored the executable from its intermediate artifact without invoking
the compiler, leaving the trace empty. An actual pinned Cargo/rustc control
reproduced this under both artifact layouts: published-file removal caused zero
crate compilations. Cleaning only the target package with `cargo clean -p`, then
running the same build, caused exactly one compilation with identical rustc argv
and identical relevant environment. The environment comparison excludes only
the four jobserver variables excluded by the existing mixed request key. The
resulting executable bytes also matched.

The scalar qualifier now asks the public command to perform that package clean.
It checks cleanup success and verifies the retained compiler certificates are
byte-for-byte unchanged. The subsequent build still must report retained Rust
object reuse with no full analysis, and its runtime differential remains intact.
The source, compiler flags, target/build directories and retained generations
are unchanged.

The [verification record](evidence/mixed-hosted-qualification-repair-20261119/verification.json)
contains the captured hosted trace, exact compiler identity, actual Cargo
controls and source hashes. Two trace regressions and 21 workflow guards pass.
The complete mixed worker campaigns still require the next hosted run; these
harness controls do not stand in for that compiler qualification or establish a
performance result.
