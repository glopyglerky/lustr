# Sole-goal enforcement after d1328bc2

Reviewed 19 November 2026. This continues the single correction in
[PR #33](https://github.com/glopyglerky/lustr/pull/33). The audit input was
`d1328bc2df289835b7a48d72575a103154b3c9e3`, tree
`6713a608d50bc589232fecbabb498693af768103`. The
[exclusive coverage contract](exclusive-coverage-contract.md) remains canonical.

Lustr's only goal is full Rust compilation with Lust owning everything Lust
covers, plus only the necessary upstream implementation for the complement.
Keep supported work near matched stock Lust speed and complementary work near
matched stock rustc speed. Prefer no_std throughout and no required Unix sockets
when those changes preserve speed and do not delay compiler completion.

## Why the architecture regressed

The [earlier plan at f42649f4](https://github.com/glopyglerky/lustr/blob/f42649f4f02d170dc899ac70fcc048db2b91da73/docs/compiler-set/architecture-plan.md)
explicitly allowed the full upstream compiler dependency graph. The same source
described whole-crate Rust analysis before narrow native emission. Successful
compatibility builds and scalar composition then supplied evidence for a
different architecture from the required compiler. Removing an executable from
a package did not remove the broad compiler dependency or duplicate analysis.

Later corrections removed the normal compiler retries and distinguished bridge
gaps from the actual complement. This audit found the same conflict still
reachable through optional features, exported experimental ABIs, obsolete Cargo
examples and ignored tests. A separate timing rule also allowed small absolute
overhead to conceal large relative slowdown. The useful native correctness,
retention and ABI work remains; these conflicting routes and claims are retired.

## Executable corrections

| Finding | Correction |
| --- | --- |
| Cargo could combine `exclusive-native` with the diagnostic-driven inference or full-SDK research composition | The native library requires an explicit product or bounded qualification mode. Product/research unions fail before bootstrap compiler execution. |
| A build without explicit product mode could still expose the general product ABI | Only `exclusive-native` exports `lustr_native_v2`. A no-mode build is rejected. |
| Retained/session inference APIs could recompile after a diagnostic | Removed the inference/retry implementation, root feature, worker target, loader methods and general retained/session exports. The ordinary native path keeps its one `driver::compile` call. |
| Auto-discovered examples and ignored tests still expected removed full-compiler packages or retry behavior | Removed the obsolete launchers and expectations. Retained original fixture sources, current execution/argument/library/transport controls and historical evidence. |
| Source guards did not inspect effective product feature/dependency closure | Added reviewed package roots, lockfile edges and source kinds, local feature closure, and dependency features/defaults/optional settings. Added actual build-script admission controls before construction. |
| Upstream capability probes were absent from its ordinary CI call graph | Added the existing capability suite to current product-constructor CI and retained declarations, binary identities and output. Its finite coverage and exceptions remain explicit. |
| Historical benchmark language still suggested current compiler qualification | Marked the remaining independent comparators and old package schema historical. Removed the generic benchmark instructions using the retired coordinator. Algorithms, statistics and frozen samples stay unchanged. |

The bounded assertion printer and useful native object, phase, runtime,
publication and imported-definition qualifiers remain. The qualifier mode is
`native-qualification`; its broad `upstream-semantic` dependency remains
research-only and cannot be combined with the product. No general compiler
escape is retained under a different label.

The protected donor subtree remains
`e0d5f908d528ec2048c717ae8ad89b6c52dd4dae`, with donor build identity
`4fbb2e5c41854f34`. The original Lust frontend, checked-state behavior and
retained daemon were not changed in this audit. Both protected pins are
unchanged. A fresh GitHub tree comparison confirmed that blueFe's `compiler`,
`library` and `src` trees equal the declared Rust source; Lustr does not need a
duplicate full Rust checkout.

## Known support without trying compilers

Establish capability rules from the pinned implementation and qualification
outside user requests. Apply those rules to facts produced by normal native
compilation or its retained update, and keep the resulting checked state.
Do not add a scanner, trial compilation, second parser/checker or eager complete
complementary-driver initialization to choose ownership.

The upstream syntax scanner lacks resolved type information and full macro
expansion, and can count unknown constructs as supported. Its input schema also
differs from the current capabilities producer. It cannot decide production
ownership. Unknown evidence stays unknown. An advertised Lust operation with a
defect stays Lust-owned and requires repair. The
[upstream support and mathematics review](exclusive-coverage-mathematics.md)
records the exact sources, CI histories, limitations and cost deductions.

## Near-stock means a relative comparison

The former rule accepted 10% overhead or 0.5 ms added latency. A hypothetical
0.2 ms operation taking 0.7 ms would satisfy the absolute arm despite taking
3.5 times as long. That cannot establish near-stock speed.

The [current policy](practical-performance.md) requires the relative comparison
for new claims, using the existing engineering assumptions of 10% for supported
warm work and 5% for ordinary complementary work. Paired uncertainty and latency
tails remain visible. Absolute added latency is diagnostic, not an alternate
pass. Historical measurements, thresholds and decisions remain unchanged.

Assess each material integration cost during implementation. Final package
qualification is not permission to postpone all speed assessment. Include
dispatch, conversions, initialization, transport and publication in actual
request timings. Compare equivalent artifacts and retain separate supported,
complementary and complete-command results. No new speed result is claimed here.

## Verification and its limits

The [verification record](evidence/sole-goal-audit-20261119/verification.json)
binds compiled source trees/blobs, new entry/library hashes, unchanged donor
helpers and the [compressed receipts](evidence/sole-goal-audit-20261119/receipts.json.gz).
Publication copies normalize local paths and retain both original and stored
hashes. This is a focused assembled package, not a fresh complete constructor
or optimized-package qualification.

| Check | Observed result |
| --- | --- |
| New native library and public entry | Both debug builds passed. Among `lustr_native_*` symbols, the product library exports only `lustr_native_v2`. |
| Public-entry controls with scanner and alternate-compiler traps | 56 passed; one real-daemon case unavailable locally because Unix socket binding returns `EPERM`. |
| Root tests | 3 library, 33 main and 22 contract/protocol tests passed. One internal child fixture and five package-dependent cases were initially ignored. |
| Explicit package controls | Three passed: single test execution, 2,050 native arguments, and six library-selection cases with rejection/recovery and an independent Rust oracle. These use process mode. |
| Complete root test target compilation | Passed. Socket-specific cases were compiled but not executed locally. |
| Surviving bounded qualifiers | Native qualification all-targets check and upstream-semantic example check passed. These are compile checks, not new runtime qualification. |
| Dependency/source enforcement | 18 planted package/manifest/lockfile changes and six forged inventories rejected; benign manifest changes accepted. Full-replacement release correctly refused. |
| Actual compiled build-script admission | Three forbidden modes failed before any compiler child or linking directive; the allowed mode reached the controlled bootstrap sentinel. Actual Cargo builds also reject the removed feature and worker. |
| Existing upstream capabilities | 73 probes, 149 declared lines, zero failures on the exact unchanged donor helpers. Eight sections and four declared exceptions; not every capability combination. |

The initial attempt to test a removed feature used `cargo metadata --no-deps`,
which returned success. That assumption was inadequate, and the original result
is preserved. Actual Cargo build refusals replaced it. No failure was edited
into a pass.

Independent reviews found and closed the no-mode ABI escape, dependency-feature
guard omission and remaining auto-discovered launchers. Current-source hosted
construction, full native CI and matched performance need their own run
receipts. The prior head's passing checks do not qualify a new source by name.

## Remaining compiler work

This cleanup does not implement the general complement or finish integration
of all Lust-supported operations. Continue the existing genuine consumer-metadata
boundary: native declaration, privacy, lint/definition completion and the actual
emitted-instance graph must feed the encoder without whole-consumer Rust
reanalysis or fabricated metadata. Then connect the necessary upstream
complementary services through the same owned state and precise dependency reads.

The current compiler still uses std, native host facilities and Unix sockets
for its automatic warm CLI. Their migration is conditional on completion and
retained speed. No full Rust replacement, no_std compiler, socket-independent
retained compiler or near-stock speed qualification is asserted by this audit.
