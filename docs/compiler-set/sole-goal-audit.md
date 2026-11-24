# Sole-goal corrections and actual result evidence

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

## Repository documentation and entry-point audit

The repository-wide review on 20 November 2026 UTC starts from published PR #33
source `ea26fe16636d390763257a19c19df3db992cdbcb`. It continues this audit and the
existing checklist. The [documentation disposition](documentation-disposition.json)
accounts for all 396 tracked Markdown files at that baseline with exact Git
blob identities, reviewed content hashes, authority, action and review scope.
Any audit Markdown added after the baseline is classified separately. It is a
documentation inventory, not another implementation plan.

The review found executable-direction conflicts outside the canonical plans:

| Live conflict | Documentation correction |
| --- | --- |
| Default-branch governing docs could be mistaken for merged compiler code | Startup instructions resolve PR #33 independently from `next`; closed PRs, old worktrees and preview tags are historical sources. |
| Mixed-symbol and scalar-integration notes described successful full-compiler fallback or public retry in present tense | Their source-bound bodies remain preserved under explicit historical scope. Current bridge gaps remain Lust-owned incomplete integration. |
| Benchmark and fixture guides still recommended coordinator engine selection, full-compiler package construction or new acquisition through removed lanes | Removed those live recipes and linked their immutable originals. Independent measurement and read-only replay remain distinct from product compilation. |
| Preview download/install instructions and a Windows coordinator recommendation looked current | Marked preview assets as historical and removed the active installation recipes. No complete replacement release is qualified. |
| Old proofs and performance tables used geometric summaries, an earlier patch state or an earlier candidate as current authority | Bound them to their original source and date; new claims follow paired total elapsed costs and the current declared integration patch. Original numbers, failures and archived decisions remain unchanged. |
| Broad compiler-private shared-service experiments could read as admitted complementary implementation | Marked their research scope and linked the real import inventory. No full-driver dependency is admitted by a passing fixture or metadata serializer. |
| Socket semantics and a `no_std` metadata interface could be confused with a portable compiler | Kept generated-program Unix APIs separate from compiler transport; the current compiler still uses `std` and Unix sockets. |
| Historical links targeted removed paths or headings | Corrected the live navigation while preserving original source/history links. |

Current product Cargo planning also needs an exact scope statement.
[`resolve/cargo.rs`](../../compiler/native/src/resolve/cargo.rs) excludes
`unit_features_via_check` from `exclusive-native`. Non-simple graphs and
custom-build targets use
[`cargo_plan::select`](../../compiler/native/src/resolve/cargo_plan.rs), whose
pinned Cargo unit-graph request performs no compilation and uses the sibling
Lustr information-query service. This is dependency planning, not a trial
compilation to choose an owner. The older donor probe description remains
reference text, not an admitted product route. Legacy whole-Rust build-script
output acquisition/replay refuses product use; the
[native host-script continuation](native-cargo-continuation.md) uses ordinary
Lust preparation and emission for its admitted host scripts. Procedural macros,
registry/git graphs and build-std remain explicit integration gaps; refusal
does not reclassify any of them as complementary coverage.

The disposition separates 117 repository guidance/checkpoint pages from 173
protected native/donor Markdown files, one upstream runtime README and 105
immutable evidence/measurement pages. Active contracts, entry guidance and
changed instructions received content review; the other repository checkpoints
received authority, navigation and conflict-pattern review. Protected donor
manuals and archived evidence received inventory/boundary verification, not a
fresh semantic audit of every historical sentence. They are unchanged.

Verification for this documentation change checked all 396 inventory content
hashes and all relative file and heading links in repository guidance.
All 279 protected donor/runtime/evidence Markdown files are byte-identical to
the baseline, and whitespace checks pass. The explicit review methods remain
recorded per document. It runs no compiler, generated program, benchmark
or hosted CI. It neither repairs all remaining compiler gaps nor transfers an
older test pass to new source. CI is required evidence, not a claim that current
repository settings enforce merge or release admission. BlueFe's moving head is
also distinct from Lustr's recorded Rust pin; the [source register](sources.md)
records that boundary without updating protected pins.

### Construction and evaluator follow-through

The [construction admission record](evidence/product-construction-audit-20261120/verification.json)
and [payload-retirement record](evidence/product-construction-audit-20261120/packaging-retirement-verification.json)
bind the subsequent source changes and actual controls. The constructor selects
the reviewed package/bin/lib targets and checks the effective source graph
before building compiler components. The checks reject unreviewed compiler-package
environment and Cargo configuration, manifest target overrides, generated
build-script modes and profile flag/backend injection. These checks
govern compiler construction, not a user's Rust request or its semantic flags.
The existing `--current-sdk` discovery query remains distinct from compilation.

Independent review found that the first draft still admitted Cargo's per-package
default/forced target keys. The
[before-and-after control](evidence/product-construction-audit-20261120/before-after-manifest-modes.json)
records the gap and correction, with separate product and metadata research
mutation controls. The later
[feature-closure correction](evidence/product-construction-audit-20261120/feature-closure-correction.json)
derives product ownership from the constructor's actual `source-native-runtime`
selection and rejects removal of its required `exclusive-native` edge. It does
not assume that product mode was separately selected. Ordinary reviewed
version/profile changes remain admissible. Actual constructor controls stop at explicit SDK
sentinels; they do not claim a rebuilt compiler package. The
[280-manifest disposition](evidence/product-construction-audit-20261120/manifest-disposition.json)
records source roles without calling every donor fixture a production import.

The retired Cranelift setup helpers, dead semantic full-core build branches and
unused expanded-source payload are removed. The latter's only active caller
was package construction: `src/native_expansion.rs`, five owned standard-library
adapter files and their expanded package layout had no public compiler or
bounded research caller. Original bytes remain in Git with recorded hashes.
The Lust subtree, both protected compiler pins and ordinary public compilation
semantics are unchanged by that removal.

The [CI/evaluator follow-through](evidence/ci-anti-regression-20261120/index.json) fixes a
different acceptance gap: an ordinary test
could panic with exit 101 yet retain a native `pass` label. The
[capability result guide](capability-result-qualification.md) now requires the
independent oracle's panic expectation, exact message/location and complete
pinned fixture inventory. Missing inputs in explicitly selected optional
measurement controls fail; ignored controls are not passing compiler evidence.
Retired mixed acquisition and qualifier entry points are removed or refuse
execution, while original evidence and read-only replay remain available.
Independent measurement-tool controls use explicitly identified Cargo/SDK tools
under the [measurement guide](benchmark-method.md). The index preserves initial
dependency/setup failures, an unqualified tiny study blocked by absent Linux
pressure observations, separately executed real Cargo functional checks and
unselected optional integrations. Comparing an archived native test record with
a fresh pinned oracle does not become a new candidate run. These cleanup and
evaluator results supply no full Rust coverage or new compiler-speed claim.

## Follow-through after 5cd40fd3

> Historical scope for this and the remaining sections: each record describes
> its own named source and execution date. References to a "current" native
> tree, admitted patch, blueFe/Rust equality or branch head apply only within
> that record. They are not assertions about the later repository-wide audit,
> a moving upstream branch or today's compiler. Use the verified integration
> source, [source register](sources.md) and declared pins for present work.

The next review starts from published
`5cd40fd313503d4a4fc935ee47ca7c05fce1471a`, tree
`114415d4cb1850a80fbc1451fce7bdb8ce9878c3`. The product fallback removals remain
intact in the inspected entry, loader, helper gates, daemon and constructor.
This review found three different ways success could still be overstated.

### A malformed daemon status could become success

The shared protocol decoder accepted every nonnegative `i32` status. The Unix
clients pass it to `process::exit`; a status such as 256 or 512 then becomes
zero as observed by the parent. The
[Rust API contract](https://doc.rust-lang.org/std/process/fn.exit.html#platform-specific-behavior)
documents the low-byte behavior on these hosts.

The decoder now accepts only the representable Unix range 0 through 255.
An ordinary or mixed malformed response fails the request without another
compiler attempt. Positive controls preserve ordinary errors and conventional
signal-style statuses. The installed-package socket control supplies 256, 512
and 257 through both real entries and requires terminal failure, preserved
prior artifact bytes, no success output and no endpoint removal.

The [source-bound evidence](evidence/daemon-status-20261119/verification.json)
retains the restored-defect control that fails because 256 was accepted,
focused protocol and public-entry execution, and the local socket `EPERM`
failure. Local entry/helper builds use an explicit synthetic test identity and
hashed reused dependencies. They are not a fresh native-core/library constructor.
Hosted construction and the actual peer exchange remain separate requirements.

Only `compiler/native/src/protocol.rs` changes inside the protected donor tree.
The pristine pin remains unchanged; the full declared patch and candidate
identity are updated. The correction tightens an existing integer conversion.
It adds no compiler pass, probe, allocation or child process. This is source
cost analysis, not a measured latency claim.

### Per-case speed claims could still conceal higher elapsed cost

The previous assessor used total time for the overall workload but geometric
ratios for each near-stock claim. A synthetic 24-request witness takes 24 time
units in Lust and 104.6 in Lustr while producing a favorable geometric ratio.
The previous near-Lust result passed despite about 4.36 times the elapsed cost.

Version 2 uses paired summed elapsed costs for each current near-stock claim
and its per-case Rust speed condition. It retains geometric ratios only for
diagnosis and displays mean and maximum latency beside p95. The witness now
has an inconclusive near-Lust interval and cannot claim that target. A separately
established overall win against rustc still passes. No new release blocker or
numerical target is introduced. The
[25-control record](evidence/current-performance-v2-20261119/verification.json)
contains synthetic assessor evidence, with no compiler execution or benchmark.
See the [assessment contract](current-performance-assessment.md).

### Upstream retained scenarios could silently lose accepted cases

The unchanged donor harness labels valid Rust rejected by both warm and cold
Lust as "unsupported today" and still reports that scenario as successful.
The [pinned Linux log](https://github.com/t3dotgg/lust-lang/actions/runs/39780031006/job/119828625752)
reports 363 of 363 scenarios as okay while also reporting 27 unsupported versions
across 13 histories. Its later sanitizer failure remains a failure; this is
evidence about the scenario verdict, not a passing upstream job.

The separate [retained-result qualifier](../../tools/qualify-retained-results.py)
uses the original fixture order, overlays and lifecycle controls while making
independent acceptance and result decisions. Its
[pinned inventory](../../upstream/retained-scenario-results.json) fixes known
accepted histories before candidate execution. It requires 349 histories and
keeps the 13 mixed-support
histories explicitly unresolved, without turning their observed rejection
counts into allowances for future regressions.

Public `lustr check` and `lustr build`, artifact production and program execution
are observed separately. Warm and cold native rejection cannot excuse valid
Rust. Actual output and exits must agree with the pinned independent Rust
oracle; the same owned daemon must survive the warm sequence. Original donor
scripts and fixtures remain unchanged. Qualification runs outside production
requests and does not measure compilation speed or select production ownership.

The [retained-result control record](evidence/retained-result-controls-20261119/verification.json)
records evaluator controls, actual pinned-Rust diagnostic parsing and the
earlier local package attempt. That package's daemon failed at socket creation
with `EPERM` before any version was qualified. The original attempt binds its
own runner and package identities; it does not qualify the final runner or the
current compiler package. CI runs the new qualifier immediately after its
guarded constructor and retains all reports, command streams and daemon logs.

## Direction that must survive this correction

The canonical ownership and release rules are unchanged. Preserve every
Lust-owned operation, import only the genuine complement, perform shared work
once, and release a complete correct replacement when its declared overall
workload beats matched stock rustc. Keep improving toward stock Lust afterward.
The default-branch instruction sync removes the older ownership wording,
absolute-latency loophole and subset-target-before-release ordering. A document
publication cannot qualify the still-incomplete compiler implementation.

Prefer `no_std` throughout compiler implementation and no required Unix socket
transport when retained speed and completion are preserved. This does not remove
support for compiling programs that use `std` or target-supported Unix APIs.
There is no qualified socket-free retained replacement or whole-compiler
`no_std` conversion in this change. The established next compiler work remains
genuine native declaration/privacy/lint/definition completion and the actual
emitted-instance graph feeding the metadata encoder, then justified selective
complementary services. These checks expose failures; they do not discharge
that missing implementation.

## Follow-through after a03b1e2d

The next review used published `a03b1e2d9787523c72efe628a72cb02afbfaa491`,
tree `a27edade00c0098ded2564d1a60f4e333208c0cb`. The earlier removal and
verification record below remains historical evidence for that correction.
The follow-through changes current contracts, qualification tools and CI.
It does not change the compiler's request path, protected Lust source or pins,
and it does not implement the still-missing general complement.

The audit found these remaining acceptance problems:

| Finding | Correction |
| --- | --- |
| The mathematical supported set depended on what the implementation "correctly covers," although defects were also required to remain Lust-owned | Define ownership by reviewed pinned capability rules applied to normal semantic facts. Correctness, defects and bridge gaps are separate obligations. A failed compilation cannot redefine the owner. |
| The checklist treated provisional near-stock budgets as release requirements despite the clarified first-release goal | Require a correct full replacement with measured overall acceleration. Keep stock-Lust matching as ongoing optimization and 10%/5% as separately named engineering claims. Report slower cases without holding a qualified faster full replacement for perfect parity. |
| Two passing subset budgets or a favorable geometric mean could conceal greater total elapsed time | Add combined-overhead accounting and a fixed-frequency total-time comparison with paired uncertainty. Assess near-stock ratios separately. |
| Supported capability probes could print wrong values and still exit zero | Preserve the upstream suite and add exact generated-program result comparison through public `lustr`, using the pinned Rust oracle only in qualification. Compare named test outcomes for the test fixture. |
| Root tests had local evidence but were not executed by the hosted constructor; installed-package controls remained ignored | Run all root targets and explicitly invoke all five package contracts in constructor CI. An unavailable socket is a failed/unqualified control, never a passing skip. |
| The only executable timing assessor replayed the retired historical rule | Add the [current performance evaluator](current-performance-assessment.md) with fixed workload costs, exact declared input/entry identities and failure controls. It assesses performance evidence only and always reports `release_admission=false`. |

The [capability-result record](evidence/result-proof-20261119/verification.json)
binds unchanged compiled inputs and the four reused debug-package binary hashes.
All 18 supported fixtures passed through the public entry: 17 generated programs
matched Rust stdout, stderr and exit status exactly, and the test fixture
matched four passing tests and one intentional ignore. The 87 root tests passed.
Four installed-package controls passed; the fifth failed before exchange because
`UnixListener::bind` returned `EPERM`. Its original exit 101 and output are
retained. The final evaluator's 15 failure controls passed. This is cold,
finite functional evidence, not a new complete build or speed qualification.

The [current assessor record](evidence/current-performance-assessment-20261119/verification.json)
records its own synthetic arithmetic and evidence-rejection tests. Those tests
execute no compiler and supply no benchmark result. A truthful full replacement
claim still needs the actual implementation, independent ownership/correctness
evidence and a complete matched campaign. Hash consistency cannot prove that
timings are truthful or that a workload was chosen before measurement.

The research refreshed the same Lust and Rust heads and retained the original
history findings. The [updated mathematics](exclusive-coverage-mathematics.md)
adds the missing connection between subset costs and overall time, and keeps
shared work counted once. This follows the same upstream lessons: preserve
checked state, narrow invalidation and request-generation-bound facts.

The next compiler work remains genuine native consumer metadata and its actual
emitted-instance graph, then the justified complementary implementation. A
new evaluator cannot discharge that missing work. `std` and Unix sockets remain
present under the completion-first portability priority.

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
