# Selective Rust reuse and the Lustr performance contract

The [exclusive coverage contract](exclusive-coverage-contract.md) governs implementation. This report records bounded upstream evidence and candidate mechanisms; it does not qualify the complete compiler or authorize every mentioned component as an import.

Research observation: 19 November 2026. This is a source and history review, not a compiler qualification. No compiler or benchmark workload was run for this review.

## What the upstream evidence establishes

Lustr can reuse real Rust compiler components without importing all of Rust. Upstream already maintains that kind of boundary for rust-analyzer. The right next step is to adapt those reusable components to Lustr's retained semantic state, using them only for operations Lust does not supply. Linking the complete `rustc_driver` and calling it an imported complement does not establish that boundary.

The full source is already preserved in blueFe. At observed blueFe main `d3c5da6f3d9d097600c9b461064c76cbb345d4ef`, GitHub's tree API reports identical `compiler`, `library`, and `src` tree IDs to upstream Rust pin `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. The compiler tree is `9bb98adca40010656ac057f2c4103cf72f5cdc6f`; the library tree is `f48d6700750a1dc11b2f8f1df91a26dee3ef13fb`; the src tree is `8042d9063cce1688a1f0e2f945c6da48f9568645`. This verifies tracked source preservation under Git's object integrity assumptions. It does not prove that a binary was built from those sources or that submodules are populated.

Sources: [blueFe pin](https://github.com/glopyglerky/blueFe/blob/d3c5da6f3d9d097600c9b461064c76cbb345d4ef/upstream/rust.pin), [blueFe tree](https://api.github.com/repos/glopyglerky/blueFe/git/trees/d3c5da6f3d9d097600c9b461064c76cbb345d4ef), [upstream pinned tree](https://api.github.com/repos/rust-lang/rust/git/trees/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494), [preservation contract](https://github.com/glopyglerky/blueFe/blob/d3c5da6f3d9d097600c9b461064c76cbb345d4ef/docs/bluefe/upstream-preservation.md).

The freshly observed upstream Rust main was `b0b600668bb9b4a897e2299b0a44289d42276545`. Researching it does not authorize changing blueFe's protected pin or mixing semantic versions in one Lustr invocation.

## The actual selective reuse seam

Rust's current [shared solver design](https://github.com/rust-lang/rust/blob/b0b600668bb9b4a897e2299b0a44289d42276545/src/doc/rustc-dev-guide/src/solve/sharing-crates-with-rust-analyzer.md) explains that rust-analyzer supplies its own representations and query database while using Rust's generic type IR and trait solver. `Interner` specifies the concrete types and semantic context. `InferCtxtLike` and `SolverDelegate` supply inference operations. The solver does not need to own a complete rustc frontend.

The document lists `rustc_abi`, `rustc_ast_ir`, `rustc_index`, `rustc_lexer`, `rustc_next_trait_solver`, `rustc_parse_format`, `rustc_pattern_analysis`, and `rustc_type_ir` as reused components. That is evidence of existing modularity, not a proposed blanket Lustr import. A component whose required operations are already supplied by Lust is excluded unless an independently documented complementary obligation requires a smaller part of it.

At the existing blueFe Rust pin, a manifest-level, non-nightly path-dependency closure rooted at `rustc_next_trait_solver` contains these eight Rust crates:

| Crate | Why it appears in that dependency closure |
| --- | --- |
| `rustc_next_trait_solver` | Reusable solver algorithms over supplied interfaces |
| `rustc_type_ir` | Generic types, constraints and solver context interfaces |
| `rustc_type_ir_macros` | Derives used by the type IR |
| `rustc_abi` | ABI and layout representation dependencies of type IR |
| `rustc_ast_ir` | Shared representation types |
| `rustc_index` | Typed indices and collections |
| `rustc_index_macros` | Index derives |
| `rustc_hashes` | Stable hashing used by the ABI representation |

The relevant dependency edges explicitly disable default features where needed. `rustc_next_trait_solver` and `rustc_type_ir` enable broad nightly compiler dependencies by default, so leaving defaults on defeats this smaller boundary. `rustc_type_ir_macros` and `rustc_index_macros` have no default nightly feature. `rustc_abi`'s comments explicitly identify stable builds for rust-analyzer as a supported use of its reduced dependency configuration.

Sources at the protected donor pin: [solver manifest](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_next_trait_solver/Cargo.toml), [type IR manifest](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_type_ir/Cargo.toml), [ABI manifest](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_abi/Cargo.toml), [AST IR manifest](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_ast_ir/Cargo.toml), [index manifest](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_index/Cargo.toml), [hashes manifest](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_hashes/Cargo.toml).

This is a researched candidate boundary, not an implemented complete complement or a verified minimal source import. Its third-party dependencies, build-time derives, licenses, tests, referenced external files, target configurations and the actual Lustr adapter still need qualification. Traits, lifetimes, normalization, coherence and metadata must all retain their real semantics. A new adapter must not turn unresolved constraints into acceptance. The adapter must not invoke the solver for Lust-owned operations just to choose a route.

## Why the complete driver is the wrong import root

The current upstream [driver manifest](https://github.com/rust-lang/rust/blob/b0b600668bb9b4a897e2299b0a44289d42276545/compiler/rustc_driver_impl/Cargo.toml) directly depends on the parser, expansion, resolver, HIR analysis, MIR construction, metadata and `rustc_interface`. The [interface manifest](https://github.com/rust-lang/rust/blob/b0b600668bb9b4a897e2299b0a44289d42276545/compiler/rustc_interface/Cargo.toml) pulls in borrow checking, trait selection, MIR transforms, monomorphization, query implementation and code generation. A single shared library can therefore contain almost an entire compiler even if Lustr's own source tree is small.

Removing the `rustc` executable while retaining the whole driver is not a source-minimal complement. Similarly, disabling LLVM emission for selected definitions only removes the final emission work. It does not remove the Rust frontend analysis already performed on those definitions. Such a bridge can provide valuable ABI experiments and genuine crate metadata, but it cannot be promoted as proof of the required exclusive Lust path.

The audited [PR #32 source](https://github.com/glopyglerky/lustr/tree/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3) shows exactly this distinction. [`crates/lustr-semantic/src/mixed_main.rs`](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/crates/lustr-semantic/src/mixed_main.rs) calls `rustc_driver::compiler_entrypoint`, uses `after_expansion` and `after_analysis`, and imports `rustc_interface`, `rustc_middle`, metadata and codegen libraries. Its `mixed_backend.rs` suppresses codegen for selected native items. Its own architecture plan correctly admits that cold mixed work still performs whole-crate Rust analysis. This observation is about that source identity; refresh the active head before applying changes.

## A precise ownership model

Define a requested compilation context as source and expanded definitions, dependency identities, Rust edition and semantic version, target and ABI, optimization/debug settings, panic strategy, build-script observations and proc-macro results. A language feature name alone is not a context.

Let `U(q)` be the necessary semantic and emission operations for unchanged request `q`. Let `L(q)` contain operations covered by the pinned Lust implementation with that context. Let `C(q) = U(q) \\ L(q)`. The intended ownership is:

1. Every operation in `L(q)` has Lust as its implementation owner.
2. Every operation in `C(q)` has a justified imported or integration owner.
3. No required operation is omitted and no compilation operation has two executing owners.
4. Explicit shared representation and metadata services have one owner and consume real checked results. They do not recompute a Lust-owned semantic operation under another name.
5. The capability contract is computed before emission from compiler facts. Compiler failures and invalid programs do not redefine `L(q)` or turn into complementary work.

This model permits shared identifiers and types. It forbids compiling the same native function through a complete second frontend merely because the file also contains an unsupported feature. A partial bridge must track its gaps separately from Lust's actual coverage. Failure to represent supported Lust work in the bridge is an integration defect, not permission to relabel that work as the complement.

The partition is finer than files or crates. A generic source function may produce several monomorphized instances, each with different contextual obligations. Upstream's [monomorphization documentation](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html) describes instance collection and CGU partitioning. Lustr must own concrete instances consistently with Rust symbol and metadata identities. Rust ABI, drops, unwinding, const evaluation and cross-crate generics are not satisfied by concatenating object files.

## The dependency mathematics

Source selection is a constrained dependency-closure problem, not subtraction of directories named after language features.

Let `G = (V, E)` describe donor components and their required dependencies under an explicit feature/target configuration. Let `M` be the chosen roots that implement missing operations. Let `B` be donor dependencies replaced by verified Lustr interfaces. Starting with `D_0 = M`, iterate:

`D_(n+1) = D_n union {v : some u in D_n requires v, and v is not supplied by B}`.

For a finite dependency graph, this reaches a fixed point after at most `|V|` additions rounds. Traversing adjacency lists computes a fixed graph's closure in `O(|V| + |E|)`. That calculation establishes dependency completeness for the modeled graph, not semantic correctness or global minimality across all possible refactorings.

The objective can be stated as minimizing imported bytes, maintenance cost and request overhead subject to full complementary semantics, explicit licenses, dependency closure and exclusive Lust ownership. A verified adapter can cut a dependency edge. Renaming a whole compiler or allowing the linker to discard unused routines does not meet the source-selection objective.

For each admitted import, record the donor commit, exact paths or symbols, source hashes, license notices, missing capability, required dependencies and features, existing Lust overlap excluded, semantic adapter, observable caller and tests. An entry with no real caller remains research, not delivered coverage. A donor refresh must recompute the closure and review newly enabled features.

## The timing mathematics

For a matched Lust-supported request, write `T_lustr = T_lust + H`, where `H` includes dispatch, validation, conversion, loading, transport, duplicate work and publication. A tolerated fractional overhead `epsilon_L` requires `H <= epsilon_L * T_lust`. If stock Lust takes 0.2 ms, a 0.2 ms wrapper doubles latency despite being small in absolute time. A constant process or full-driver initialization cost is especially damaging to warm body edits.

For complement-only work, write `T_lustr_C = T_rustc + H_C - S_C`, where `S_C` is actual work saved. Near-stock performance requires `H_C - S_C <= epsilon_C * T_rustc`. No speed gain on this set is required. Copying stock source does not prove the inequality because build configuration, initialization and adapters may differ.

For mixed work, a useful serial accounting identity is `T_mixed = T_L + T_C + T_shared + T_bridge + T_link`. Overlapped work needs measured critical-path accounting; summing CPU time does not establish wall time. A whole Rust analysis followed by Lust emission adds duplicate work before the bridge can save anything.

If fraction `p` of baseline time is replaceable by Lust and Lust accelerates that work by factor `s`, with normalized extra overhead `h`, the idealized speedup is `1 / ((1-p) + p/s + h)`. This is conditional on equivalent work and the serial decomposition. For example, with `p=0.5`, `s=100`, and `h=0.05`, total speedup is about `1.80`, not `100`. Once Lust-owned work is fast, startup, bridge and metadata costs dominate. Therefore retain Lust state, invoke complementary services lazily, avoid a complete second frontend, and measure each request class separately.

For incremental work, retain an explicit dependency graph across both implementations. Edits invalidate affected operations and their dependents. Reuse is valid only when every relevant semantic input matches. Rust's [red-green design](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html) explains why checking changed inputs and comparing results can stop unnecessary propagation. Repeated calls to a driver are not proof that its semantic context survived.

## CI and performance history worth preserving

The current [Rust workflow](https://github.com/rust-lang/rust/blob/b0b600668bb9b4a897e2299b0a44289d42276545/.github/workflows/ci.yml) computes its matrix using `src/ci/citool` and [jobs.yml](https://github.com/rust-lang/rust/blob/b0b600668bb9b4a897e2299b0a44289d42276545/src/ci/github-actions/jobs.yml). PR jobs, merge jobs, try builds and performance try builds differ. PRs exercise selected checks; merge CI covers much broader host and target combinations. Try builds can skip tests to produce artifacts sooner. A green try build or a successful compiler bootstrap is therefore not interchangeable with application correctness or compiler latency evidence.

Rust's [performance process](https://rustc-dev-guide.rust-lang.org/tests/perf.html) uses compiler commit comparisons and a maintained corpus, with fresh and incremental configurations. These design lessons belong in Lustr's acceptance, alongside upstream Lust's workload-specific differential and timing gates.

Selected history was inspected, not the entire Rust history:

| Evidence | Mechanism or failure | Lustr consequence |
| --- | --- | --- |
| [PR 151509](https://github.com/rust-lang/rust/pull/151509) | A parallel incremental query could be marked green and red concurrently, writing duplicate metadata. The fix uses atomic compare/exchange for red marking too. | Shared retained state needs tested single-owner transitions and race handling. Parallelism cannot be assumed correct from a sequential gate. |
| [PR 160005](https://github.com/rust-lang/rust/pull/160005), [commit 94a8720](https://github.com/rust-lang/rust/commit/94a8720a7778de52ca0cd68b8a6a056064edc048) | Profiling found excess copies. Changing two `Vec` fields to `ThinVec` reduced a frequently moved pair from 136 to 104 bytes, avoiding an observed code-generation threshold. Size assertions preserve the gain. | Preserve donor data-layout decisions and assertions when adapting shared types. Count conversions and copies across the boundary. This does not establish the same threshold on every target. |
| [PR 165913](https://github.com/rust-lang/rust/pull/165913) | Repeated MIR statement insertion was replaced by merging, changing the stated worst-case cost from `O(existing * new)` to `O(existing + new)`. | Inspect algorithmic costs of edits and conversions. Cache/bridge work can erase fast body-edit gains. The complexity improvement does not itself establish an end-to-end speedup for Lustr. |
| [PR 161236](https://github.com/rust-lang/rust/pull/161236) | CI job downloads became parallel to shorten result reporting. | Faster CI feedback is distinct from faster generated compilers. Keep both measurements named correctly. |
| [Issue 165953](https://github.com/rust-lang/rust/issues/165953), [commit 748c5ea](https://github.com/rust-lang/rust/commit/748c5ea2835fb9b10e654baa55ac2f9d29bc0666) | New Windows ARM runner tooling broke stage1 compilation; reverting unrelated compiler changes did not fix it. CI introduced a targeted older MSVC setup. | Record runner image, C toolchain and linker identities, and preserve platform failures instead of blaming compiler architecture without evidence. |

Rust's [16 November 2026 announcement](https://blog.rust-lang.org/2026/11/16/enabling-parallel-frontend-on-nightly/) adds a current baseline hazard: nightly started enabling two frontend threads by default with `nightly-2026-11-14`. The post distinguishes single-crate gains from whole-graph gains and lists remaining reproducibility issues. Compare matching versions and explicit thread counts. Do not transfer those published gains to blueFe or Lustr.

## Acceptance that follows from the goal

- Preserve the full pinned Lust coverage inventory and tests. The bridge's current supported rows do not define that inventory.
- On purely Lust-supported work, observe zero invocations of a competing Rust semantic compiler or emitter. Include ordinary commands, Cargo units, warm edits and mixed-program native instances.
- Make complement selection a typed pre-emission outcome. Native compiler, transport, object or linker failures must remain failures.
- Track per-phase and per-instance ownership. Each generated native symbol must be emitted exactly once, and all required symbols and real Rust metadata must be present.
- Prove import provenance and dependency closure mechanically. Reject whole-tree vendoring and undescribed broad driver dependencies. Do not claim dead-code elimination removes source-policy violations.
- Test boundary semantics with unchanged programs: traits, projections, lifetimes, macro expansion, generic instantiations, const/runtime agreement, layout, panic/drop/unwinding, dependencies and genuine Cargo artifacts.
- Measure matched pristine Lust, integrated Lust, public Lustr entry, and stock Rust separately. Use fixed source/toolchain identities, explicit threads and identical flags. Preserve cold, no-op, body edit, signature/type/trait/macro/dependency edits, rejection and compile-plus-test results independently.
- Adopt explicit overhead budgets before sampling. Report paired distributions and uncertainty, raw failures and per-workload regressions. An aggregate gain must not hide a failing Lust-owned class.
- Keep correctness, ownership and speed as separate release conditions. A source-cleanup PR can enforce direction while the complete complement remains unfinished; it cannot claim full rustc replacement from documentation or guards alone.

## Review limits and next executable slice

This review inspected selected manifests, the reusable solver architecture, current CI definitions and six relevant upstream changes/failures. It verified blueFe's protected top-level source tree identities. It did not inspect every Rust crate, all CI logs, every historical PR or all target semantics. No performance claim is based on a new local run.

The immediate established gap is the [consumer-metadata completion service](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md), with its [original failing dependency probe](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/evidence/shared-metadata-dependency-20261119/verification.json). The genuine encoder reaches `analysis(())` to freeze definitions; current native checked state does not yet satisfy all declaration, privacy and lint obligations. Import or adapt that narrow service to consume actual native proofs and exact emitted instances. Do not return unconditional analysis success, fabricate typed results or an empty instance graph. The generic trait-solver seam above remains researched reusable upstream design, not a competing task that replaces this continuation. Keep Lust's actual compiler and retained state as the owner and require a minimal justified manifest for each concrete imported service. Keep incompatible whole-driver experiments in history and remove their alternate compiler launchers. Independent bounded mechanism/oracle qualification may retain necessary compiler-private dependencies with explicit disclosure. Expand that boundary using upstream implementations rather than reconstructing every missing Rust feature or replacing the native path with a new coordinator.
