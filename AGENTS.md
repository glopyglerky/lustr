# Work on this fork


## Governing architecture: read before implementation

Follow [the explicit architecture plan](docs/compiler-set/architecture-plan.md),
reaffirmed by the user on 18 November 2026. Lustr is one compiler replacing
rustc: Lust handles its supported work, and imported upstream Rust implementation
handles the complementary work inside the integrated compilation. Both must
compose in the same unchanged program. Preserve Lust's frontend, retained state
and fast body-edit path for work it supports.

Do not substitute a coordinator, a language selector, a packaged stock-rustc
backup, or whole-program Lust compile/fail/retry for this architecture. The
current standalone retry route and narrow scalar mixed bridge are transitional;
their tests and timings do not establish completion. Internal capability dispatch
must use actual compiler representations and semantic/ABI contracts before
emission. Do not use source-keyword guesses or require a complete extra Rust
frontend on every Lust-supported request merely to choose its implementation.

Importing upstream Rust source and internal services is explicitly allowed with
licenses, notices and pinned provenance. Independent reconstruction of all Rust
semantics is not required. Cargo adapters may preserve real graph, CLI and
artifact behavior; adapter work is not delivery of the compiler boundary.
Do not reinterpret a speed allowance or permission to import Rust as permission
to revert to a fallback/coordinator design. Do not ask the user to restate this
direction. Historical plans and measured compatibility paths do not override it.

## Current priority and ownership

Expose the actual Lust capability boundary, retain checked state through
emission, and connect imported Rust support for the complement through a shared
compilation boundary. Reuse the existing real ABI/symbol/object/metadata work.
Investigate previous failures before repeating an approach. The native
semantic-image experiments remain retained research, not a requirement to
reimplement every missing Rust feature independently.

Keep the pristine Lust reference and explicit integration guard. Imported Rust
must remain separately identified; no hidden changes to Lust or blueFe's
protected Rust pin. Read the current contract and entry contract before routing
changes. Preserve published PR7, unique source and failed evidence.

## Performance acceptance

Use [the practical performance criteria](docs/compiler-set/practical-performance.md).
Preserve large gains on Lust-supported work and near-stock behavior on ordinary
full Rust. Do not require big gains outside the Lust subset or exact Lust parity.
Keep historical strict results and every regression. Correctness and native
platform qualification remain independent release conditions. Investigate the
remaining public-entry cost with matched measurements before changing source;
never remove input validation to meet a sub-millisecond target.

## Product contract

Follow [the compiler contract](docs/compiler-set/native-contract.md). Qualify
actual native, complementary and mixed work inside Lustr. A public command
compiling a program through a stock compiler does not by itself meet that
contract. Never fabricate Rust metadata, rewrite applications into the subset
or conceal the implementation that produced an artifact.

## Correctness rules

- Current broad compatibility comes from the packaged Cargo/rustc implementation. Name that transitional dependency honestly. Do not claim it delivers the required integrated complement, or that this fork independently implements all of Rust.
- Never infer semantic compatibility from source keywords, a successful parser run, or a compiler accepting the source. Macros, traits, lifetimes, layouts, generics, drops, panic strategy, and crate metadata interact.
- Lust currently has no Rust rlib format. Do not place its output in Cargo's artifact graph as a Rust library or forge metadata or dep-info. Whole-target Lust compilation and a future verified semantic-IR bridge are separate routes.
- Preserve the project's source, features, target, profiles, Cargo flags, dependency versions, panic strategy, and build-script behavior. Adapter substitutions and reduced features require explicit qualification against the requested Cargo build.
- Never turn compiler errors into success. Never retry an executed program or test automatically through another compiler. Do not change panic behavior to obtain a passing or faster result.
- The stock rustup Cranelift preview failed the real panic-recovery gate. Build the pinned complement with its `unwinding` feature and use the matching compiler plus explicit backend path. A panic flag cannot supply backend code that was not compiled in. Preserve the recovery/drop gate.
- Keep stdout usable for Cargo JSON and program output. Pass OS arguments without a shell or lossy decoding. Preserve subprocess failures and signals.
- Scope compiler backend settings to the current Cargo invocation. Injected `CARGO_PROFILE_*_CODEGEN_BACKEND` environment variables leak into programs and tests and can break nested Cargo commands. Place generated Cargo configuration after the subcommand and before caller arguments so later package overrides preserve it. Retain the real configuration-precedence and nested-Cargo regression gates. Verify actual compiler flags when qualifying a backend; a requested plan or successful program alone is insufficient.
- Key any future cache by the compiler/runtime identities, complete semantic inputs, target/profile/features, build-script observations, and transitive dependency identities. An untracked input forces recomputation.

## Work layout and validation

`crates/lustr` is the independently installable compatibility coordinator, not the target compiler architecture. Its own build must not require tcc, nightly Rust, or registry dependencies. Required native compiler sources remain in `compiler/native/src/` and
`compiler/native/library/`, as described in the primary README.

Read `docs/compiler-set/architecture-plan.md`, `native-contract.md`, `design.md` and `roadmap.md` before changing routing or claiming coverage. The upstream `DESIGN.md`, `docs/ARCH.md`, `docs/PLAN.md` and archived design snapshots are historical. Check executable code and tests before treating any plan as implemented behavior. A release described as the requested full replacement must satisfy architecture as well as correctness and performance; coordinator success cannot waive that condition.

Use real integration tests for the Cargo path, including build scripts, proc macros, full-language constructs, rejection of invalid Rust, and observable runtime behavior. Keep the upstream differential gates for changes to Lust itself. Report which engines and platforms actually ran.

Measure cold builds, no-op builds, body edits, signature edits, rejection latency, and compile-plus-test time separately. Preserve raw samples and exact compiler identities. Report regressions. Do not transfer upstream Lust's speed ratios to this fork, Cranelift, or arbitrary Rust projects.

Keep changes reviewable on a branch. Do not change repository visibility, upstream licensing, or the user's other forks as part of compiler work.
