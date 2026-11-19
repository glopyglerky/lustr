# Exclusive coverage cleanup

This correction follows the [canonical contract](exclusive-coverage-contract.md)
merged through [PR #31](https://github.com/glopyglerky/lustr/pull/31). It removes
conflicting compiler routes. It does not deliver the unfinished full Rust
complement or establish a new performance result.

## Why the implementation drifted

The earlier default instructions mixed native expansion, strict speed demands
and a compatibility coordinator. The later integrated plan still explicitly
allowed the complete upstream compiler dependency graph. The public entry then
used native exit status and diagnostic text to decide whether to retry through
other implementations. Ordinary Cargo paths could enter the full driver
directly. The scalar mixed backend selected native work after whole-consumer
Rust analysis.

Those are source findings at the preserved
[PR #32 head](https://github.com/glopyglerky/lustr/tree/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3).
See its [public entry](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/src/main.rs),
[full compiler path](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/src/full.rs),
[mixed driver](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/crates/lustr-semantic/src/mixed_main.rs),
and the [superseded import exception](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md).

Compatibility, correctness and speed tests of those paths remain evidence for
their exact sources. They did not demonstrate exclusive Lust ownership or a
selective complementary implementation. The new contract makes those distinct
acceptance obligations explicit.

## Code disposition

| Earlier active path | Correction |
| --- | --- |
| Public status/diagnostic-driven native retry | Removed. A native failure remains a failure. Unintegrated requests report the missing implementation. |
| `src/full.rs` | Full Cargo/rustc compilation and publication removed. Necessary Cargo target selection is retained in `src/cargo_selection.rs`. |
| Executable Auto/Rustc/Cranelift/Lust coordinator | Compiler execution retired. Planning helpers remain independently testable. The root compiler no longer depends on the coordinator. |
| `lustr-mixed` whole-program driver | Executable target and launcher removed. Useful semantic, symbol, object and ABI components remain available for necessary independent research. |
| Full SDK package assembler and runtime relinker | Removed. Their exact source remains in Git history. |
| Full mixed-toolchain package builder | Replaced with `tools/build-native-toolchain.rs`, which builds a partial native compiler without a bundled whole Rust compiler. |
| Raw native-helper oracle, delegated release and hidden Cargo compiler probes | Excluded from the product helper build. Genuine native operations and native `ReleaseCc` remain available. |
| Whole-driver package success used as completion evidence | Replaced by explicit ownership/routing tests and retained direct shared-service qualification. Historical measurements keep their original scope. |

Full Rust source remains in blueFe. The new
[`upstream/complement-imports.json`](../../upstream/complement-imports.json)
records the existing partial identifier-normalization and declared-SDK Unicode
reuse. It also discloses compiler-private research dependencies that are not
admitted into the product. It deliberately records `full_rust_coverage=false`.

The [structural guard](../../tools/check-exclusive-coverage.py) checks known
retired routes, manifest/source identity, dependency provenance and packaging
boundaries. It refuses a complete-replacement release. Its scope is structural;
the inventory does not prove semantic completeness, global source minimality or
speed. Process controls and actual compiler tests remain separate evidence.

Every shipped native compiler helper uses the `exclusive-native` product
feature. The feature participates in the real native build hash, which binds
daemon, disk-cache and image identities. The entry constructor verifies the
policy and full build identity reported by both actual helper binaries; a
guarded client receipt cannot authorize an unguarded core. The pristine donor
remains a separately identified reference. See the
[helper boundary](exclusive-native-helpers.md).

Cargo metadata uses the packaged Cargo utility and a read-only native compiler
information service. Queries report the partial compiler honestly and obtain
target facts from the native implementation. They do not run stock rustc or
claim unavailable Rust artifacts. Child-command tool binding preserves the
program's semantic environment. Selection receipts describe the actual native
package so removed SDK files cannot silently disable warm selection reuse.

The packaged standard Cargo utility is used internally for metadata and package
selection. Direct invocation of that utility retains Cargo's normal caller
compiler selection; it is not the current qualified Lustr Cargo compilation
frontend. The no-delegation controls cover owned compiler entries and native
compiler helpers. Normal full Cargo compilation remains integration work, and
an external compiler selected through direct Cargo cannot count as Lustr coverage.

## Corrections found during independent review

The review checks the complete package and nested compiler requests, not only
the top-level executable. It identified several problems in early cleanup
drafts before publication:

- Standalone native `test` and `run` must remain reachable after retiring the
  coordinator.
- Native release emission and the upstream Cargo/rustc release delegation are
  different parsed operations. Removing one must not remove the other.
- Cargo feature/build-script/procedural-macro probes can launch a compiler under
  the native resolver. Unimplemented complementary host work must remain explicit.
- Rebinding the semantic client environment changes `env!` and `option_env!`.
  The cleanup must preserve program-observed inputs.
- A Cargo layout workaround also leaked into manifest, cold-child and daemon
  environments. A complete environment-boundary audit and absent/present-value
  controls are required, not only checks of variables named `RUSTC`.
- Dependency-free Cargo metadata requires a truthful read-only compiler-version
  query. It must not invoke stock rustc or invent Rust artifacts.
- Selection caching must describe the actual native package and Cargo inputs.
  Retaining requirements for removed SDK files would force repeated metadata
  processes and regress the warm path.
- Internal probe-mode environment variables and raw installed helper commands
  can bypass a guarded public entry. Every shipped compiler executable needs the
  product boundary.
- Cached build-script records can replay a compiler before a fresh Cargo probe.
  Product guards must precede cache consumption as well as fresh probing.
- Product/oracle features and both actual helper binaries need distinct, verified
  identities. Otherwise an old unguarded daemon or core can survive packaging.
- Historical warm-comparison harnesses require their original package layout.
  Pointing them at a different native-only package does not produce valid timings.

The source-bound verification record accompanying the final candidate records
which controls actually executed. Stub launch controls, actual native execution,
retained-daemon execution, source checks and performance qualification must never
be reported as interchangeable results.

## Verification of the corrected candidate

The [independent review](exclusive-coverage-review-2026-11-19.md) and
[complete verification record](evidence/exclusive-coverage-20261119/native-entry-verification.json)
close the concrete cleanup findings for frozen source
`cd775d996dde91c0c2f98c38fcc72e2ede2ffebf`. That is a local build-source identity;
the PR binds the published tree to the same compiled inputs. The native subtree
is `a5223b8045f75660281342bf547769fc1b098725`, product build identity
`b48307d68f2f2b4e`. The pristine Lust and declared Rust pins are unchanged.

| Evidence | Result and scope |
| --- | --- |
| Clean-source development-profile constructor at `e0304bc3` | Built native helpers, public entry, native compiler library, source seal, selection receipt and seed. |
| Corrected entry at `cd775d99` | Rebuilt the entry after the environment repair, reusing byte-identical native components whose source did not change. This is not a second complete constructor run. |
| Entry/helper controls | 56 passed; one retained-daemon control unavailable because local Unix socket operations return `EPERM`. Actual native library/process cases are distinguished from synthetic route/failure witnesses. |
| Independent actual execution | Safe Cargo metadata, nested feature and implicit-library compilation, absent/custom semantic environment, direct helper refusals and late build-script rejection observed; no ambient compiler spy executed. |
| Cargo selection/cache controls | Cache 294, seed 85 and compact selection 394 passing assertions; 277 recorded subprocess invocations including setup and cached/fresh comparisons. These qualify selection behavior, not compiler speed. |
| Source/import enforcement | Exact candidate native source/patch check and structural inventory checks pass; the complete-replacement release check deliberately refuses admission. |
| Optimized constructor | Earlier native-core library compilation was killed by `SIGKILL`. That attempt remains failed/incomplete, with no Rust compiler diagnostic establishing its cause. |
| Near-stock speed, full native differential suite and full replacement | Not established by this correction. Historical results retain their own exact sources. |

Hosted CI requires the real public retained-daemon control. The local unavailable
result is not a pass. Failed earlier selector expectations and the original
environment mismatch remain in the evidence; changing a test expectation must
be justified by actual Cargo behavior and still preserve invalidation controls.
The independent review records those distinctions.

## What remains to complete Lustr

The cleanup removes conflicting behavior and exposes the actual coverage gap.
A request that previously succeeded through a packaged stock compiler can now
report incomplete complementary support. That is an intentional correction of
the advertised execution path. It is not successful implementation of the
missing semantics.

The next required artifact service remains the
[genuine consumer-metadata continuation](https://github.com/glopyglerky/lustr/blob/cb0721d49be95ec3f34a113c2354bc9d2a38bbf3/docs/compiler-set/shared-consumer-metadata.md).
The encoder demands completed analysis before freezing definitions. Lustr needs
to supply the real native declaration, privacy, lint, definition and emitted
instance obligations through a narrower upstream interface. Unconditional
analysis success, fabricated metadata and whole-consumer Rust reanalysis are
prohibited.

General complementary semantics, supported mixed instances, genuine Cargo
artifacts, cross-boundary invalidation and platform coverage remain unfinished.
Each import must satisfy the necessity and ownership contract. A bridge gap
remains Lust-owned work awaiting integration.

No new near-Lust or near-rustc speed claim follows from this cleanup. Apply the
[existing performance policy](practical-performance.md) to a source-bound,
equivalent-work comparison of the actual integrated implementation when it is
ready. Preserve the prior measurements and every failed attempt.
