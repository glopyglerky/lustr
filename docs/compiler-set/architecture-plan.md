# Governing plan: one Lustr compiler

**Status: required architecture, reaffirmed 18 November 2026. The current
development candidate does not yet implement the complete boundary.**

This is the delivery plan for Lustr. Read it before changing compiler routing,
proposing a coordinator or selector, interpreting a benchmark as completion,
or preparing a package release. The [compiler contract](native-contract.md)
and root [AGENTS.md](../../AGENTS.md) carry the same requirement. Historical
experiments and implementation descriptions do not override it. Continue with
this direction in later sessions; the user does not need to reauthorize it.

## Required product

Lustr is one compiler intended to replace rustc for the user's Rust builds.
It uses Lust for the work Lust supports and imports upstream Rust implementation
for the complementary work Lust does not support. Both participate inside
Lustr's compilation of the unchanged program, including programs that combine
the two sets of features.

Importing Rust implementation is explicitly allowed. Reimplementing all missing
Rust semantics independently is not required. The imported implementation must
serve the complementary work through the integrated compiler boundary; shipping
a whole-program rustc backup behind an error-driven retry is not completion.

Preserve Lust's existing frontend, retained state, body-edit behavior and fast
code generation for supported work. Do not make every supported request pay for
a complete second Rust frontend merely to decide that Lust can handle it.

## Rules that must survive implementation changes

1. **Assign work from capabilities inside the compiler.** Use actual compiler
   representations, resolved identities and supported semantic/ABI contracts.
   A failed whole-program Lust compilation must not be the normal capability
   test. There is no separate language-routing selector in the target design.
2. **Keep supported work on Lust.** A mixed program must retain Lust participation
   for its supported work. Sending the entire program to stock rustc because
   one feature needs complementary support does not meet the requirement.
3. **Use imported Rust for the complement.** Reuse upstream implementation,
   metadata machinery and necessary shared semantic services with exact pins
   and licenses. Their presence does not justify duplicating the complete
   supported compilation or treating a packaged stock compiler as the product.
4. **Compile one program with consistent meaning.** Share or faithfully bridge
   definition identities, types, layouts, calling conventions, generic instances,
   drops, unwinding, symbols, metadata and dependency state. Produce genuine
   artifacts that Cargo and dependent Rust crates can consume.
5. **Separate unsupported work from compiler failure.** An unsupported capability
   is an ordinary internal planning result before emission. A compiler crash,
   emitter failure, invalid object or linker failure remains a failure; it must
   not silently become successful compilation through a second compiler.
6. **Preserve the user's program and build.** Do not rewrite applications into
   Lust's subset, change features/profiles/panic settings, forge metadata, or
   retry a program that has already executed.

These rules do not require an unsafe keyword scan. Macros, name resolution,
trait selection and types can affect whether an operation is supported. The
capability contract must come from the implementations and their checked
representations, not guesses from source spelling.

Cargo can still schedule its real dependency graph, build scripts and proc
macros. Small command adapters may preserve its CLI and artifact conventions.
Those are supporting infrastructure, not a substitute for the integrated
compiler. Cargo package/target lookup is also distinct from language capability
dispatch; the existing approximately 0.34 ms target-selection measurement does
not establish a language-selector design or justify one.

## What exists today

The following are source findings, not descriptions of the completed target.

| Existing path | Actual behavior | Gap from this plan |
| --- | --- | --- |
| Standalone `lustr build/check FILE.rs` and eligible simple rustc-style requests | [`src/main.rs`](../../src/main.rs) tries a whole native request; native status 1 can enter the packaged Rust path. | Error-driven whole-program retry remains. It is transitional compatibility. |
| Ordinary Cargo project commands | The Cargo adapter reaches the imported Rust driver and bounded mixed backend directly. These commands do not generally first try a complete native build. | A coordinator forwarding work does not itself implement the required compiler boundary. |
| Current mixed functions | [`mixed_main.rs`](../../crates/lustr-semantic/src/mixed_main.rs) performs Rust analysis and selects a limited set of nongeneric root scalar functions using real HIR bindings and Rust/C ABI queries. Linked native participation is constrained to Linux x86_64, opt0/debug0 and represented bin/rlib requests; metadata-only checks can retain normal debug profiles. | This is substantially narrower than Lust's supported language subset, and cold mixed work still performs whole-crate Rust analysis. |
| Native object handoff | [`mixed_backend.rs`](../../crates/lustr-semantic/src/mixed_backend.rs) omits successfully supplied native definitions from LLVM and lets imported Rust emit the complement and real metadata. | This is useful existing composition, not complete automatic partitioning. |
| Admission and generation | The [first typed admission boundary](native-admission.md) checks represented scalar IR, request and ABI facts before C generation, retains the checked program into emission, and distinguishes complementary work from compiler failure. Cold and retained callers propagate actual failures. | This covers the existing scalar bridge only. Full Lust capability coverage, broader shared semantics and exact-source hosted qualification remain required. |
| Consistent Rust semantics | The [Unicode audit](unicode-semantics.md) identifies a build-host-dependent split between native constant evaluation and checked-in runtime data. The imported Rust source and native upstream contract name different Rust versions. | Align native and complementary semantics with one declared Rust source and shared upstream data before admitting the affected operations. A changed version string cannot repair this. |

The native phase observer exposes checked state to observers. The first scalar
bridge now has an internal prepare/emit ownership boundary, while the general
native ABI still exposes whole-request status and output. No general API currently partitions the entire
Lust subset and the Rust complement before emission. Existing capability reports
are descriptive inventories, not that missing executable contract.

These gaps explain why passing standalone speed measurements and compatibility
tests is not proof that the requested architecture is finished. The current
candidate remains a useful basis; its demonstrated fixes and measurements stay
preserved with their original scope.

## Implementation sequence

### 1. Expose the actual capability boundary

Inventory Lust's supported operations, semantic contexts, target/profile
requirements and runtime/ABI requirements from the pinned implementation.
Connect each represented boundary to existing accepted and rejected fixtures.
Record what the shared bridge represents separately from what Lust alone can
compile. A partial bridge must not quietly redefine Lust's subset.

Define explicit internal outcomes for admitted native work, complementary work,
invalid Rust and implementation failure. Make admission usable before object
generation. Keep checked state and dependency reads alive between admission and
emission rather than parsing/checking the same unit again.

### 2. Build the shared compilation boundary

Reuse the existing real symbol, FnAbi, object, metadata and retained-state work.
Define who owns semantic facts and how each implementation consumes them. Import
upstream Rust services where Lust lacks the required support; do not restart a
feature-by-feature independent reconstruction of Rust.

A prepare/emit interface or narrower representation bridge can be an intermediate
step. It is not completion if it still sends mixed programs through a second
whole-program compilation. Review the actual work performed, not module names.

### 3. Integrate native and complementary work in one request

Extend the compiler boundary so known supported work reaches Lust and the
remaining work reaches the imported Rust implementation automatically. Compose
their outputs with real Rust linkage and metadata. Preserve supported work
across changes to complementary code when its recorded semantic inputs allow
reuse. The public Lustr entry must use this integrated path; an error-driven
native-first retry must not remain its normal architecture.

### 4. Qualify the actual architecture and performance

Use unchanged supported, complementary and mixed projects. Record actual
per-unit/phase participation, not just a successful executable or requested
engine label. Test cold, warm, body/signature/type/trait/macro/dependency edits,
invalid Rust, panic/drop, source changes across the boundary, and genuine Cargo
artifacts. Confirm no ordinary failed native emission is used to discover the
complement and no required supported work silently moves to stock compilation.

Preserve near-Lust performance on supported work. Ordinary complementary Rust
should remain close to matched Rust performance; large gains there are not
required. Small supported-path overhead is acceptable. Keep every timing and
regression visible under the [practical performance policy](practical-performance.md).

## Completion and publication

Architecture, correctness and speed are separate conditions. A completed task
or release claim must identify the source and evidence for all applicable
conditions. An installable package presented as the requested full replacement
must satisfy this plan, not merely wrap enough existing executables to compile
the test corpus.

Before calling the compiler complete, evidence must establish:

- Lust participation across the supported coverage being claimed.
- Correct complementary and mixed compilation through the integrated boundary.
- Genuine Cargo/dependency/metadata behavior and relevant target/runtime checks.
- Retained fast-path behavior without ordinary whole-program compile/fail/retry.
- Source-bound speed results meeting the practical goals without concealing
  slower operations or substituting an easier profile.

Correctness repairs to transitional paths can still be worth retaining. They
must be labeled as such and must not become a new delivery architecture by
default. Do not interpret a timing allowance, a successful coordinator build,
or permission to import upstream Rust as permission to replace this plan.

## Guidance for the next contributor

For each proposed implementation change, explain which compiler boundary it
advances, which work actually stays in Lust, which work uses imported Rust, and
which semantic state is reused. State any remaining duplicate analysis or
whole-program retry explicitly. Do not ask the user to restate this architecture
or silently narrow the goal to a coordinator, selector, fallback wrapper, or the
current scalar-only bridge.
