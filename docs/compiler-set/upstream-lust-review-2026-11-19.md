# Lust preservation evidence for Lustr's single compiler direction

The [exclusive coverage contract](exclusive-coverage-contract.md) governs implementation. This report records bounded upstream evidence and candidate mechanisms; it does not qualify the complete compiler or authorize every mentioned component as an import.

Reviewed 19 November 2026. This is source and historical-evidence review, with current upstream identity and Actions status refreshed through GitHub. No compiler was built or benchmarked in this review. Historical timing numbers below are donor claims or retained Lustr observations, identified separately.

## Identity and actual review boundary

The canonical donor is [t3dotgg/lust-lang](https://github.com/t3dotgg/lust-lang). GitHub currently identifies `next` as its default branch, at [`240e08d91d4aed0299639f6f11fd522912b248f9`](https://github.com/t3dotgg/lust-lang/commit/240e08d91d4aed0299639f6f11fd522912b248f9), tree `0ab1b09815802894e6078de0af59273cbdf46cb0`. This matches Lustr's existing `upstream/lust.pin`. The separate `main` branch is the earlier prototype at `65215d74458bb677ea65b520efe1529023a8e52f`; using its README or implementation as the current donor loses substantial later work.

Reviewed the current upstream branch list, all fourteen current issue/PR records, the four open PRs, the complete workflow-file history returned by GitHub, the ten `scripts/ci.sh` history entries returned by GitHub, and selected implementation/history paths. Source review focused on retained sessions, checked-body reuse, patching, dependency reads, the public release route, test entry points, and several measured improvements and reverts. This is not a claim to have semantically reviewed every upstream commit, every compiler function, or all transitive dependencies.

Lustr's existing [upstream benchmark audit](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/upstream-benchmark-audit.md) already documents the benchmark scope and several measurement hazards. Preserve it as historical evidence. It does not supply current integrated-complement qualification.

## The mechanisms that actually produce the gain

| Mechanism | Inspected evidence | Required integration behavior |
| --- | --- | --- |
| Retain parsed and checked state | [`src/driver/session.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/session.rs) owns AST, stable identities, generated bodies, glue, slots and object state per crate selection. | Lustr must keep this state across requests. A new compiler process and full frontend pass per edit defeats the central mechanism. |
| Patch only an eligible edited body | [`src/driver/patch.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/patch.rs) reparses an edited body against retained resolver state. Its first obligation is equivalence to a fresh resolve. Macros, outside reads, hidden types, const bodies and item changes require more work when their proofs do not hold. | Preserve this fast path, including its refusal conditions. Rejecting a fast-path optimization is not authorization to switch the whole program to another compiler. |
| Recheck the affected dependency closure | [`src/driver/compile.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/compile.rs) distinguishes body, signature and full-resolve edits. [`src/driver/graph.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/graph.rs) records bindings, signatures, impl sets, method probes, layouts and failed lookups with normalized fingerprints. | Imported complementary services must expose precise dependencies in this same compilation. Broadly marking every body dirty on every complementary change destroys reuse. Omitting a dependency permits wrong code. |
| Stable function slots and small rebuilds | The [upstream daemon explanation](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md#how-the-daemon-works) and session source describe a base library, stable call slots and changed-unit overlays. | Preserve stable identities and exact ABI/layout ownership across native and complementary bodies. Do not regenerate or relink unrelated work only because another implementation supplies one missing feature. |
| Function parallelism and cheap development emission | [Upstream's speed explanation](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md#why-it-is-fast) identifies parallel checking/borrow checking/emission, a reduced semantic problem and TinyCC in development builds. | Measure equivalent requested profiles. Replacing TinyCC or adding a complete rustc frontend has a cost; a renamed executable does not preserve the speed mechanism. |
| Avoid repeated construction of unchanged release units | [`53554783eeabd6bf57f25cb4d13f93e202e3feb2`](https://github.com/t3dotgg/lust-lang/commit/53554783eeabd6bf57f25cb4d13f93e202e3feb2) enables B1 unit reuse by recorded reads. [`55b444749c939c0470fc1e9f4a719f7b37d54d62`](https://github.com/t3dotgg/lust-lang/commit/55b444749c939c0470fc1e9f4a719f7b37d54d62) extends it to the release statics pass. | Preserve dependency-specific identities and the checking oracle. Keep memory cost and dirty-case regressions in the evidence. |

The donor's headline advantage is mostly an edit-loop advantage. Its pinned README reports 23.3 ms versus 1,386 ms for warm body checks in one corpus, and 18.8 ms versus 4,742 ms in another. Its cold comparisons are about 2.4–2.6 times, and reported signature-edit tails reach seconds. These are published upstream experiments on named, adapted corpora, not measurements of Lustr or arbitrary unchanged Cargo applications. The same README reports materially slower generated-program execution in the measured TinyCC development mode. Compilation, generated-program execution and compile-plus-test time must remain separate measures. [Source](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md#numbers).

## History that should govern the integration

| Historical change | What it establishes | Lustr consequence |
| --- | --- | --- |
| [`2511b824`](https://github.com/t3dotgg/lust-lang/commit/2511b82454f63d7a5fb0dca6a42963067ae8e9a1), cold benchmark repair | Daemon autostart and disk caching had made purported cold rows warm. The correction disables both for fresh-cold measurements and names cached-cold separately. | Freeze lifecycle and cache state explicitly. No-op, restart, cold and retained-body timings cannot be interchanged. |
| [`86ac7cbe`](https://github.com/t3dotgg/lust-lang/commit/86ac7cbe3a0c1d7794fe6ef2d700ccf80bd87f06), external macro input invalidation | Included-file content and environment values became tracked expansion dependencies after stale reuse was possible. | Complementary macros and build scripts need exact observed-input dependencies; source files alone are insufficient cache inputs. |
| [`ba15b727`](https://github.com/t3dotgg/lust-lang/commit/ba15b7271a99928e5b4abcf6427b9c6ba3a24c5d), stale-reuse repair | Session-local module numbering, incomplete trait buckets and untracked recursion limits caused warm false accepts. | Cross-engine identifiers must be stable, semantic and generation-bound. Record negative lookups and policy limits as reads. |
| [`c537579b`](https://github.com/t3dotgg/lust-lang/commit/c537579b77d26297dc9dfb69644e54d17f77651f), missing type and hygiene reads | Nested aliases, const array lengths and macro hygiene produced warm/cold output differences. The fix tracks names inside types and normalized hygiene. | A shared type representation alone does not prove correct incremental invalidation. Adapters must record all semantic reads. |
| [`93e57035`](https://github.com/t3dotgg/lust-lang/commit/93e57035ed84f607e9b6402a6bf9a8370acffc0b), precise drop constraints | Reading full layout closed a correctness hole but rechecked too many bodies. A dedicated drop-check constraint fingerprint retained correctness with less invalidation. | The best boundary follows the actual semantic question. Use narrow shared queries, rather than a whole-program rustc pass or broad structure hash. |
| [`1fca7814`](https://github.com/t3dotgg/lust-lang/commit/1fca781447632713a151733243259b3bbffc5972), shared read answers | Two reuse walks recomputed many of the same fingerprints. One request-scoped answer table evaluates each indexed key once. | Share semantic results, not duplicated frontend execution. Imported services should join the current request's dependency graph. |
| [`89974f0d`](https://github.com/t3dotgg/lust-lang/commit/89974f0d41d045a1c0af73909ea2788cad39ebac), optimization reverted | A lazy well-formedness construction change was reverted after paired replay showed no gain. | Plausible complexity arguments and smaller code paths do not establish a speed improvement. Retain A/B evidence and remove unhelpful changes. |
| [`5a6f97c1`](https://github.com/t3dotgg/lust-lang/commit/5a6f97c1f5272239404ebd1c4cc1534ecc8a8e0d), early monomorphization reverted | A change had passed extensive checks, but later edit histories exposed order-dependent type naming between one-shot and retained-image compilation. | Require longer mixed histories, deterministic identities and recovery states. A successful cold build or first edit does not qualify the retained compiler. |
| [`abe8fa6a`](https://github.com/t3dotgg/lust-lang/commit/abe8fa6a659e1544f753de87bd80c881fe878f0a), generic-reuse oracle | Existing oracles had missed stale values retained by generic reuse. A dedicated oracle rechecks retained concrete work and compares emitted representations. | Verification must cover every reuse mechanism. An independent checker belongs in qualification, rather than an unconditional second production frontend. |
| [`b3954002`](https://github.com/t3dotgg/lust-lang/commit/b39540025bae9bd9162c5c1891aac777b61b012c), generated edit histories | A seeded generator adds 51 families spanning types, macros, layouts, traits, consts and codegen-only reads, with edit and reversal states. | Reuse these families as differential acceptance cases for the native/complementary boundary. Preserve unsupported classifications; do not count them as fast successful compilations. |

The source already contains these changes. Reimporting a current donor snapshot does not newly obtain their gains; an integration must avoid discarding them.

## CI design and what its green/red status means

The current [.github/workflows/ci.yml](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/.github/workflows/ci.yml) has Linux x86-64 and macOS ARM64 jobs, pinned TinyCC construction/cache and Rust 1.99. Both call [scripts/ci.sh](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/ci.sh). Its file history starts at [`21fa7e4f`](https://github.com/t3dotgg/lust-lang/commit/21fa7e4f0891f21b36ba7c937b88093bbcabc288). Later script changes add kernels, macros, crates, client behavior, stable slots, warm/cold/rustc scenarios and port-tool tests.

The gate builds, checks diagnostic registration, runs ordinary differentials, macro/dependency/kernel/crate tests, retained edit scenarios, stable-slot tests, sanitized differentials, available fuzzers and benchmark smoke. The performance smoke checks comparison shape and outputs. [scripts/speed-row.sh](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/speed-row.sh) explicitly allows poor speed and deletes its temporary JSON. This does not establish an enforced near-stock performance budget or durable performance evidence retention.

Refreshed [run 39780031006](https://github.com/t3dotgg/lust-lang/actions/runs/39780031006) is a push run of exact upstream head `240e08d9`, attempt 1, and remains failed. The [Linux job](https://github.com/t3dotgg/lust-lang/actions/runs/39780031006/job/119828625752) log records the `unicode_props_hash` sanitized case timing out, with 5,525 passes and one failure. The [macOS job](https://github.com/t3dotgg/lust-lang/actions/runs/39780031006/job/119828626209) failed linking on `_personality`. These are donor observations, not proof that the corresponding Lustr failures have the same cause or that later Lustr platform repairs fail.

The four current open upstream PRs concern a README for the older `main` prototype, library-target resolution, clone/Copy bounds and diagnostic cast edits. They are not evidence of an omitted general speed patch. Lustr already records its adaptation of the library-target change. Issue #14's broad shared-library proposal is a contributor's proposal, not an established causal explanation for all rustc cost. Preserve that distinction.

## Mathematics and implementation obligations

Let a compilation request include source, compiler release, edition, target, feature set, profile, operation, dependency graph and required artifacts. Syntax alone does not determine coverage. Split the semantic and emission work into supported Lust work `L`, complementary work `C`, and shared prerequisite services `S`. Each operation has one owner. Shared representations and queries do not authorize a second implementation to recompile Lust-owned bodies.

For an idealized sequential equivalent-work model,

\[
T_{\mathrm{lustr}} = T_L + T_C + T_S + H,
\]

where `H` is integration overhead after assigning shared work only once. If fraction `p` of stock elapsed cost can be replaced by work with speed ratio `s`, complement cost stays at stock and normalized extra overhead is `h`, then

\[
\mathrm{speedup} = \frac{1}{(1-p)+p/s+h}.
\]

This is a cost model, not a prediction for parallel builds or a measured result. `p` is a measured time fraction, not a fraction of Rust syntax or lines of code. With parallelism the dependency DAG's critical path and resource contention matter. A full rustc frontend before every native request increases `h`; speed lost there cannot be recovered merely by emitting a few bodies through Lust afterward. A narrow code-generation substitution can therefore pass many correctness tests while missing the requested architecture and speed goal.

For retained checking, let `D(delta)` be the transitive set of affected queries and bodies for edit `delta`. The useful target is approximately

\[
T_{\mathrm{edit}} = T_{\mathrm{observe}} + T_{\mathrm{patch}} + T_{\mathrm{validate\ reads}} + \sum_{v\in D(\delta)} T(v) + T_{\mathrm{emit/link}}.
\]

The implementation must make `D` complete for correctness and precise for speed. The drop-check history is a concrete example of improving that precision without deleting a needed proof.

Source importing is a dependency-closure problem. Starting with complementary capability roots `R_C`, compute the least closure `I = R_C union deps(I)` under a declared build configuration and documented interface cut. Every imported file/module must be justified by a root, dependency or unavoidable shared service. This closure is minimal only relative to that chosen decomposition. Rust's compiler crates are not partitioned along the Lust language subset, so subtracting files by keyword or taking every `rustc_*` crate is not a valid complement calculation. If the closure drags a duplicate supported compiler stage into production, improve the boundary before calling the import compliant. The complete unchanged Rust source stays in blueFe for provenance, extraction and reference qualification.

One additional conflict requires explicit handling. The pristine Lust public CLI contains a release command that delegates to Cargo/rustc, although later native release-driver machinery also exists. See [`src/main.rs`](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/main.rs) and the README release instructions. Keep pristine source as a reference, but do not expose that delegation as a silent Lustr production escape from exclusive ownership. Qualification may run a stock oracle separately. A native compiler fault must remain a failure; it cannot be relabeled as missing capability and retried with rustc.

## Acceptance implications

1. Establish a capability/operation ownership manifest and assert actual engine use. Supported native cases must execute with no stock compiler selected or launched. Keep a distinct independent oracle for comparison.
2. Import through a pinned file/module manifest with provenance, licenses, feature configuration and a dependency path back to a missing capability. Refuse complete Rust source vendoring and unowned duplicate compiler stages.
3. Preserve the donor source guard and explicit integration patch. Qualify supported native workloads against the same pristine Lust, not an old prototype or a different source/profile.
4. Run accepted and rejected edit histories through fresh, retained, restarted and mixed requests. Include cross-boundary layout, generics, macros, lifetimes, drops, panic, metadata, const evaluation and negative lookup changes.
5. Measure each operation independently against the proper reference. The [current performance policy](practical-performance.md) requires relative overhead evidence for new near-stock claims, using 10% for supported warm work and 5% for ordinary complementary work. The former 0.5 ms alternate rule remains historical; it could admit a large relative slowdown on a fast baseline. These are review assumptions, not evidence of success. Cold, signature, rejection, tails, memory, application runtime and compile-plus-test results remain visible.
6. Persist raw observations, exact candidate/oracle artifacts and actual checked-out source. Smoke tests, skipped jobs, timeouts and successful coordinator output never establish the full integrated compiler contract.
