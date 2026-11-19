# Independent review of the exclusive native cleanup

Review date: 19 November 2026.

Build-source commit IDs below identify frozen local inputs. The corrective PR
records the published source-tree equivalence; documentation and evidence added
after construction do not transfer a pass to changed compiled inputs.

**Disposition: the concrete cleanup findings are closed for the reviewed source and focused test scope.** This is a bounded independent review of the compiler entry, shipped native helpers, package constructor, Cargo discovery and relevant CI changes. It is not approval of a complete Rust compiler, complete Lust integration, selective-import minimality or compiler-speed claims. The reviewer made no product source edits or GitHub writes.

## Standards review

The cleanup removes the competing whole-compiler execution paths from the product. `src/full.rs`, the executable mixed driver and the full/mixed constructors are deleted. The retired coordinator's public entry points refuse compilation; its remaining pure planning APIs are research. The native-only constructor does not install the old full compiler, rustc-reference, compiler driver or LLVM runtime. Bootstrap SDK use is a separately identified build input.

`src/main.rs` treats a native compiler diagnostic, nonzero exit, signal, library failure or terminal daemon reply as the result of that request. There is no status-text classifier followed by another compiler. Public `run` compiles once and executes the program only after success. The actual native test command and explicit-target native C release variant remain distinct from the donor's rustc oracle and delegated release/profile commands.

The product feature is explicit in both native build manifests. Its boolean participates in the native build hash used by protocol/cache/image identities. Both executable helpers must answer their own exact policy and complete build identity before root construction accepts the native receipt. A guarded client cannot stand in for a stale unguarded core. Product builds without the source-native-runtime feature or without the feature-bound receipt are rejected.

The Cargo information service has a deliberately narrow supported surface. It reports partial-native Lustr identity, actual native host configuration and supported executable filenames. It emits truthful unsupported-type diagnostics rather than fabricated library or Rust metadata filenames. Source-dependent name/artifact queries require an explicit crate name, and nonempty stdin is rejected before output. Full Cargo feature resolution remains Cargo's responsibility; it is not replaced by an invented local feature algorithm.

The selection cache now depends on the actual Cargo utility, owned query entry and rustc alias rather than deleted full-SDK artifacts. Its receipt/schema changes invalidate prior assumptions. Constructor seeding and content/identity checks remain. Historical warm campaigns are explicitly evidence replay, with their original source identities and failures. Their replay does not qualify current timing.

## Spec review

The reviewed correction is consistent with exclusive native ownership for the currently admitted product requests. Direct product helper entry cannot recover the donor's rustc oracle, release/profile delegation, compiler probe wrapper or build-script replay. The product compile excludes those implementations where necessary; rejecting at the outer CLI alone was insufficient.

The whole goal is still unfinished. Requests needing unintegrated host compilation, procedural macros, dependency graphs, Rust metadata/library emission and other complementary services are refused honestly. These refusals must remain integration gaps. They do not redefine Lust's true supported subset, prove that those cases require a different compiler, or satisfy full Rust coverage.

The retained semantic/ABI/import research is useful only as research until it can supply the required shared or complementary services through the same owned compilation. No full Rust frontend is admitted ahead of native work. A structural import inventory is evidence of declared source disposition, not a proof that a dependency closure is minimal or semantically complete.

No fresh matched speed qualification accompanies this cleanup. Correctness tests, debug compiler construction and historical timing replay cannot establish near-stock Lust or near-stock rustc speed. The acceptance work still requires matched programs, flags, artifacts, target, native-versus-complement ownership, cold/warm state and failure behavior, with explicit overhead budgets and retained regressions.

## Findings and resolution

| Finding | Corrective boundary | Review result |
| --- | --- | --- |
| Native error/status could trigger alternate compiler or semantic replay | Remove full execution and diagnostic/status retry from the product entry | Source reviewed; named process controls cover terminal status and signal propagation |
| Unmatched native CLI passthrough exposed oracle/release modes | Recognize genuine native forms separately; reject delegated forms | Native test preserved; oracle and delegated release/profile refused |
| Direct shipped native helpers bypassed the root policy | Feature-bound no-delegation implementations and exact helper identity | Both actual packaged helpers independently rejected oracle, delegated release and internal probe mode |
| Internal `LUST_PROBE_FAKE_DIR`, including empty value, executed an arbitrary command | Reject its presence before argument processing, cache access or daemon connection; exclude probe module in product | Original successful bypass retained; actual guarded helper refusals observed |
| Cargo/build-script cache could replay old external compilation | Reject unsupported graph/build-script work before probe/cache/replay; exclude replay implementations | Late build.rs addition independently rejected; no ambient compiler/tool spy executed |
| RUSTC tool steering changed compiled `env!` values | Apply metadata tool bindings to the Cargo child only | Actual standalone and Cargo artifacts preserve the caller's RUSTC value |
| Cargo safe one-package compilation failed on rustc information queries | Implement narrow native information responses using the pinned Cargo parser contract | Actual full metadata and nested-feature/implicit-library fixture pass |
| Name query silently ignored source attributes/input | Explicit crate-name admission and empty-input requirement | Source-dependent requests return status 2 with empty stdout |
| Selection cache permanently ineligible after full SDK removal | Native-only receipt, owned query identity, matching alias and constructor seed | Revised source reviewed; final cache/seed/compact reports complete with 773 passing assertions across 277 recorded subprocess invocations |
| Stale helper pairing could pass a client-only prefix check | Exact policy/full-hash query from both helpers | Actual final package helpers return the same complete feature-bound hash |
| New package could not run old whole-package timing harness | Keep old campaigns as explicit historical evidence replay | Current workflow intent reviewed; no new performance claim |
| Cargo layout setting leaked into semantic environment | Remove Request/client/child/daemon injection and keep Cargo controls outside semantic environment | Corrected in `cd775d996dde91c0c2f98c38fcc72e2ede2ffebf`; original fixture and cache independently pass the absent/custom environment matrix |

The last finding was reproduced independently after construction: with the caller's layout variable absent, the standalone library route emitted `unset`, while the safe Cargo process route emitted `1`. The nested feature results remained correct (42 with defaults, 0 without defaults), which is why a build-success-only test would have missed the leak. The original reproduction is retained. After correction, the same fixture/cache produces `unset` when absent and `0` when explicitly set, through both the standalone library and Cargo process routes. RUSTC and feature results remain correct. The bounded audit found no other new integration-side mutation of `Request.client.env`: the native cold core reads its inherited process environment, retained requests carry their original captured environment, and compiler transport settings stay outside that request snapshot. Existing donor behavior beyond this integration boundary was not exhaustively requalified.

## Source and runtime evidence

The pristine Lust reference remains `240e08d91d4aed0299639f6f11fd522912b248f9`, tree `0ab1b09815802894e6078de0af59273cbdf46cb0`. The guarded integrated native tree is `a5223b8045f75660281342bf547769fc1b098725`, with a separately recorded integration patch and candidate qualification status. The correction does not relabel the integrated native source as pristine.

The clean-source debug constructor completed from `e0304bc388a2e22d2d3c7a316692ac0e02856c0f`. Its guarded native identity is `b48307d68f2f2b4e`. Both actual packaged helpers independently returned exactly the feature-policy marker and that complete hash. The final public entry was separately rebuilt from clean commit `cd775d996dde91c0c2f98c38fcc72e2ede2ffebf`, with the constructor's native helpers and library reused byte-for-byte because their source did not change. The rebuild manifest records that distinction, and the earlier package is preserved. This is an exact entry rebuild with unchanged native components, not a claim of a second complete constructor execution.

| Reviewed artifact, with source scope above | SHA-256 |
| --- | --- |
| Public Lustr entry before final environment correction | `7bdfd6944d4c69d8ba458b3eacd8de00823adaea65a53fdaeb8dfa562df8657b` |
| Corrected public entry from `cd775d996` | `e17d6411c1c7c0c48ad5991fd1d1b0ae253977106ff36028309d2bd39754820d` |
| Native client | `a6c29f4003791c351c56edd0ebcd56d7846b90f93c753345ab06c877271f03a8` |
| Native core | `bc2b0573ebbaa43f48b441afd58dee01e82c11e43e03a23df251133c0eb2603d` |
| Native compiler library | `1a2b0e1efb6d67791c4d3a44838b9d4141e9f3e9bb291f14e4e1ddbba06827d2` |

Independent observations on the constructed package and corrected entry:

- Truthful verbose version and native cfg queries succeed. Both source-dependent query refusals produce no stdout.
- Pinned real Cargo produces full metadata with the actual nested feature closure.
- The safe one-package fixture includes an implicit local library. Real Cargo reports features `default`, `inner` and `outer`, and both library and binary targets. Native artifacts print 42 with nested default features and 0 without defaults, and preserve the caller's custom RUSTC string.
- The corrected entry passes six real compilations with matching artifact executions: standalone, Cargo default features and Cargo no-default-features, each with the layout environment absent and explicitly set to 0. The original fixture/cache is reused; no ambient tool sentinel executes.
- Direct native client and core calls refuse the rustc oracle, delegated release and empty internal probe environment, all with status 2.
- Adding a build script after the successful safe graph is refused by the actual native core.
- Ambient Cargo/rustc/build/check/test sentinels record no invocation during these owned native requests.
- The structural source guard passes and all six inventory tampering controls are rejected. Those are structural controls, not semantic or performance proofs.

The implementation owner's final entry report was independently read and checked against the corrected entry/helper hashes. It contains **56 passed controls and one unavailable retained-daemon control**, with actual native library success and terminal diagnostic failure distinguished from synthetic process witnesses. Its runner SHA-256 is `c1e94d10975748167390b9b04e3ed12e423b2198ef55f2d734a77a1cca3e5ced`. The reviewer did not claim to have independently rerun all 56 controls.

The final selector reports also complete without failed assertions: cache 294, seed 85 and compact selection 394. All use profiler SHA-256 `5933a97c67b183db48b3a9fc1fd2b5324a6c62577adf55e96272c0e8c79827f3`; their 277 recorded subprocess invocations include setup and paired cached/fresh observations. The earlier failing lock-corruption and package-cache-lock expectation reports remain preserved. The two changed expectations reflect actual `cargo metadata --no-deps` behavior: it does not parse the package lockfile or acquire that package-cache lock. Their byte/identity changes must still invalidate the selection cache, and cached/fresh complete results must agree. They are not evidence that later native compilation can ignore a corrupt lockfile.

The Cargo query contract was checked against the pinned primary source, [`rust-lang/cargo@c07ac2c4d12237effc4b94f4543769b421790045`, target_info.rs](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/build_context/target_info.rs). Its unsupported-crate-type parser consumes no filename for a type reported unsupported. The implementation uses that supported protocol rather than claiming rlib or metadata artifacts.

## Explicit limits and disposition

Local Unix socket operations fail with executor `EPERM`, so this review contains no successful local retained-daemon execution claim. Hosted CI requires the real daemon control path; an unavailable local socket is not counted as a pass. Existing donor image behavior, the full differential corpus, Darwin runtime, explicit-target release performance and complete source import closure were not requalified by these focused checks.

The review's no-delegation claim concerns owned `lustr`, its `rustc` alias and guarded native compilation helpers. The package still uses a stock Cargo utility for metadata with child-only compiler binding; direct arbitrary Cargo invocations require a separate explicitly stated package contract. Do not broaden the claim to every possible externally configured tool invocation.

**Standards result:** no unresolved concrete finding remains in the reviewed cleanup scope at `cd775d996dde91c0c2f98c38fcc72e2ede2ffebf`. **Spec result:** this is a bounded correction of product ownership and truthful failure behavior; complete Rust replacement remains unimplemented. This review does not approve all Lust-supported cases, source minimality, speed targets or release readiness. No product architecture or full-compiler approval is inferred from the focused passing checks.
