# Mixed compiler completion qualification

This candidate extends PR7 from
`2add988c6397ac5175fcdf89a3aac17d359bf283`. It preserves the pinned Lust
reference, the matched full Rust implementation, and the existing Cargo graph.
It is not an admitted release. Hosted mixed correctness and paired measurement
execution now have completed evidence. The complete native gate, native macOS
execution and performance acceptance remain separate requirements.

[Retained work disposition](retained-work-disposition.md) records what was
carried forward from older PRs and preserves the unresolved Darwin base-library
investigation and historical performance evidence.

## Completed hosted evidence

### Latest completed source-built campaign: `3e24029`

The [fresh PR13 source-built run 44187580063, job 134408105819](https://github.com/glopyglerky/lustr/actions/runs/44187580063/job/134408105819)
passed source-built mixed Cargo correctness, the profile recorder, the ninth
fixed paired campaign (648 timings, 36 comparisons), a normal-profile
unchanged-project campaign (120 timings, 5 comparisons), and the corrected
pruned public upstream-slice campaign (288 timings, 18 comparisons).
It measured actual merge checkout `5711bb91b9a9fde3343b345c6ec23632de597741`,
tree `f083723dbf0e1c078306febd755c22aa2115161d` from PR head
`3e24029f75ee9837f3493be8900f7899e2b52e5f`.
The [original fixed artifact](../../performance/archives/2026-11-17-3e24029-qualification.zip)
and [original public slice](../../performance/slices/2026-11-17-3e24029/README.md)
are retained and replayed; the newest actual-project result does not yet
have the prior campaigns' authenticated private-original recovery.
Body-edit builds favor Lustr 1.046× [1.036, 1.055] on the small mixed fixture,
and rejection favors Lustr. Seven other Cargo modes are slower, so this is
not general Cargo acceleration. PR13's native macOS job was cancelled before
execution. PR14's subsequent native macOS job executed and failed two daemon
dependency tests because they invoked Linux `ldd` on macOS; the successor
change uses `otool -L` and `.dylib` on Darwin. Fresh hosted qualification is
required before accepting that fix or calling the compiler release-ready.

### Preceding recorded candidate: `6bf6e3e`

The [source-built job in run 44179667793](https://github.com/glopyglerky/lustr/actions/runs/44179667793/job/134380277750)
records head `6bf6e3e3384a99fbaeab2c66aa7c97bc73231996`, actual merge checkout
`5a3c5666a16d46629a4787e37ca9b20e65720ccc`, and source tree
`02652f033e84c0957dd2b99eb562db8171289e50`. The package removes external
symbol validation/renaming processes and the intermediate Cargo coordinator
startup. Standard Cargo commands and aliases now route through Cargo correctly.

Target layouts, the unchanged failed-link negative control and recovery, mixed
graph/edit/rejection checks, retained metadata, mixed libraries and all sixteen
entry/runtime tests passed. The [original phase profile](../../performance/2026-11-17-6bf6e3e-profile.md)
and [complete paired campaign](../../performance/2026-11-17-6bf6e3e.md) passed.
The latter retains all 648 timings and 36 comparisons. Body-edit Cargo builds
favor Lustr at 1.079× [1.070, 1.088]; rejection also favors Lustr, while seven
other modes remain slower. The [actual coordinator campaign](../../performance/projects/2026-11-17-6bf6e3e/README.md)
completed all five scenarios, with 120 timings and authenticated recovery of
600 originals / 510 streams. Default-profile project work still shows overhead.

The enclosing job **failed** its separate public-slice recorder before timing.
The [original zero-sample failure](../../performance/slices/2026-11-17-6bf6e3e/README.md)
remains retained. Diagnostics were enabled on the client but absent from its
daemon. The recorder correction passes 27 tests, including the actual upstream
statistics reader/renderer, and the independent `3e24029` run above supplies
fresh hosted execution. That does not convert the original failed attempt
into a completed campaign.

Seven other non-macOS Compiler set jobs passed. macOS compatibility was cancelled
before any step; the separate current full native Linux gate remains pending.
Its two-worker setting preserves the same corpus, sanitizer settings and
execution deadlines. Original earlier native failures remain below. Current
source review and completed measurements do not establish a release pass.

### Preceding recorded candidate: `5a3f493`

The [source-built job in run 44111326777](https://github.com/glopyglerky/lustr/actions/runs/44111326777/job/134137398335)
passed at PR head `5a3f4937d94e8f028dd85385ea65d51fcfe35a3b`, actual merge
checkout `b3393a4a2907228516546cd34f1a5c60aaf6aaaa`. It passed the unchanged
baseline regression requirement, mixed graph/edit/rejection checks, retained
metadata checks, mixed-library qualification and full entry/runtime contract.
Its first completed [schema-2 campaign](../../performance/2026-11-17-5a3f493.md)
adds matched standalone rustc and retains all 648 timings and 36 comparisons.

The [original v2 profiling qualification](../../performance/2026-11-17-5a3f493-profile.md)
also passed: 26 compiler profiles, 15 qualification commands and 35 successful
native responses validate and replay without trace reconstruction. The
[unchanged coordinator campaign](../../performance/projects/2026-11-17-5a3f493/README.md)
completed all five scenarios and 120 timings, and its encrypted originals were
authenticated, recovered and reconciled. These are separate measurements with
separate publication and workload boundaries.

Eight non-macOS Compiler set jobs passed. The macOS compatibility job was
cancelled before any step executed. The [fresh full native gate](measurements/2026-11-17-native-linux-5a3f493/README.md)
failed after 5,527 ordinary passes and 5,526 sanitized passes, at the same
`unicode_props_hash` execution deadline as the preceding native gate. Its
original artifact and source identities are retained separately;
source-built success and performance recording are not an all-platform or
release pass. The failed `40d2ac3` native gate remains recorded below.

### Earlier hosted campaigns

The source-built Compiler set campaigns at `c70eb1c` and `afb17a8` completed all
nine jobs, including baseline stale-output reproduction, corrected recovery,
mixed Cargo execution, retained checks with fresh metadata, mixed libraries,
the fourteen-test entry/runtime contract and all four selected target-layout
comparisons. The latter are cross-target layout checks, not native execution.
See [run 44056401150](https://github.com/glopyglerky/lustr/actions/runs/44056401150)
and [run 44063047987](https://github.com/glopyglerky/lustr/actions/runs/44063047987).

A third source-built job at `048d824` also passed its correctness gates and
complete paired campaign in
[run 44079784399](https://github.com/glopyglerky/lustr/actions/runs/44079784399/job/134027063238).
The whole run is not an all-platform pass: its macOS Cargo-compatibility job
was cancelled, while the eight non-macOS jobs passed.

A fourth source-built job at `dec0ce2` passed the same correctness gates,
its separate instrumented phase qualification, host-record capture and complete
paired campaign in
[run 44083661006](https://github.com/glopyglerky/lustr/actions/runs/44083661006/job/134040390909).
Eight non-macOS jobs passed. The macOS compatibility job was cancelled before
starting; GitHub's annotations report unavailable hosted ARM64 runner capacity.
This is not an all-platform pass.

The concurrently updated base at `74e14e7` completed its source-built job and
fifth paired campaign in [run 44087680045](https://github.com/glopyglerky/lustr/actions/runs/44087680045/job/134054288275).
It has no host snapshot or phase profile. Its older Linux native gate also
passed; [the exact original archive](measurements/2026-11-17-native-linux-74e14e7/README.md)
retains its 5,526-case corpus and later checks without applying that result to
newer native source.

At `40d2ac3`, [the combined source-built job](https://github.com/glopyglerky/lustr/actions/runs/44096292176/job/134084163908)
passed all correctness prerequisites, including the new library-selection test
in the fifteen-test entry/runtime contract, and completed the sixth fixed
campaign. Its actual unchanged-project campaign and encrypted uploads also
passed. The job still failed its separate v2 profiling validation: concurrent
trace writes joined two request records. The [original failure, diagnostic
reconstruction and correction](../../performance/2026-11-17-40d2ac3-profile.md)
are retained. Seven other non-macOS jobs passed; macOS compatibility was
cancelled without execution because hosted ARM64 capacity was unavailable.
At that revision the corrected trace writer still required fresh source-built
execution. The `5a3f493` result above supplies that execution without changing
the original failure's status.

The six schema-1 campaigns preserve 540 timed values and 27 comparisons each.
Together with the three schema-2 results, the nine fixed campaigns retain
**5,184 timings, 270 comparisons and 17,088 command-output files**.
[PERFORMANCE.md](../../PERFORMANCE.md) reports the mixed Cargo regressions, exact
source identities, uncertainty and workload limits. The native rustc comparison
also retains its cold-build and rejection losses. [The original ZIP
archives](../../performance/archives) preserve the full qualification outputs
as well as timing data. Measurement completion does not establish speed
preservation.

The three actual coordinator campaigns retain another **360 timings and 15
comparisons**. Each recovered and reconciled 600 original files and 510 output
streams. All use normal dev/test debug=2 and timestamp-forced root rebuilds;
none is a native acceleration or body/signature-edit campaign.

The earlier `afb17a8` [native CI run](https://github.com/glopyglerky/lustr/actions/runs/44063047989)
passed Linux but failed macOS after 5,522 passing differential cases and four
failures. Those observed failures motivated the [portable fixture and Unix
address correction](native-differential-portability.md). The correction and
separate mixed profiling work have their own exact-source CI. The latter
passed in the fourth source-built campaign. Its complete native Linux gate
passed in [run 44083661066](https://github.com/glopyglerky/lustr/actions/runs/44083661066),
with 5,527 ordinary and 5,527 sanitizer differential cases and zero failures in
either run. The [original artifact and scope](measurements/2026-11-17-native-linux-dec0ce2/README.md)
also retain the later suites. The macOS ARM64 job was cancelled before starting
with a hosted capacity error. Linux success is not native macOS qualification
for the correction.

The subsequent combined `40d2ac3` Linux native gate
[failed at sanitized program execution](measurements/2026-11-17-native-linux-40d2ac3/README.md).
Its ordinary differential suite passed all 5,527 cases; the sanitized repeat
passed 5,526 and timed out while executing `unicode_props_hash` at the unchanged
10-second deadline. Despite the diagnostic's wording, GCC linking had succeeded.
Fuzz, benchmark smoke and the final completion check were not reached. The
original ZIP and all reached suite counts remain retained. Its exact native
subtree also belongs to `5a3f493`; source equality does not convert that failure
into a passing qualification. The fresh candidate has a separate full CI run.

The [single bounded local diagnosis](measurements/2026-11-17-unicode-sanitizer-diagnosis/README.md)
instead failed with a LeakSanitizer fatal error. It used separately identified
local debug binaries and preserved the original sanitizer settings and limits.
It did not reproduce the hosted timeout or produce a passing sanitizer result.
The cause remains unresolved; neither selected source equality nor this local
blocker establishes a host-load explanation.

## Correctness changes

Edited native objects are immutable until a complete successful compiler
invocation publishes their new certificate. A failed relink cannot overwrite
the object referenced by the previous certificate. The qualification requires
the unchanged baseline to reproduce the specific stale-output error and the
candidate to execute the recovered program correctly.

Eligible mixed libraries use Rust's actual archive writer and metadata encoder,
including Cargo's split metadata policy. Linked libraries always receive fresh
Rust analysis and metadata; executable link recipes are not reused for them.
Requests needing embedded bitcode remain on the full backend. The library
qualification checks unchanged Rust consumers after hiding producer source,
body edits, rejection, metadata, proc macros, no_std/alloc and LTO fallback.

The source constructor now owns and seals its expanded library copy, installs
the required support links and preserves the expanded core's original inode.
It verifies the result through the runtime's actual seal validator. The full
existing compiler contract remains required, including retained source-library
edits, package tools, panic/drop behavior and Cargo process ownership.

The subsequent [native library-selection correction](native-library-selection.md)
preserves an explicit `--lib` request across native process startup. It covers
all six Cargo library kinds and rejects a bin-only package. Its recorded local
qualification has 22 direct source states and an actual Lustr-entry regression
with 52 command-output pairs. The full source-built contract also includes this
new test; it passed in the combined `40d2ac3` and `5a3f493` source-built contracts.

## Earlier local evidence and its limits

This table preserves the original local preparation record. Its pending items
were subsequently executed by the hosted campaigns above; the local socket
restriction itself was never counted as a compiler regression.

| Check | Result and scope |
| --- | --- |
| Pinned compiler builds and format checks | Changed owned compiler and qualification tools compile; no daemon execution implied. |
| Failed-link baseline reproduction | Originally blocked locally because Unix socket creation was denied. All eight hosted correctness campaigns subsequently reproduced the required baseline failure and passed corrected recovery; the sixth job's later profiling failure remains separate. |
| Source-library package seal | Correct packages made from current and exact baseline source pass. Source mutation, added input, bad links and substituted core inode fail; restoring the package restores its identity. |
| Target synchronization layout | Matching SDK supplies 26 independent size/alignment values for Linux x86_64, Darwin x86_64, Darwin aarch64 and Windows GNU x86_64. Baseline fails the three foreign targets; corrected native frontend passes all four and rejects a deliberate false assertion. |
| Target pool retention | The native unit test passes all nine transitions among Futex, Darwin and Windows models, including body overlays and incompatible-pool rejection. |
| Darwin runtime frontend | Six inherited fixtures pass checks for both Darwin architectures. This is not native linking or execution. |
| Linux runtime preservation | All six affected inherited differential filters pass with the exact CI Rust 1.99.0 oracle: nine cases, zero failures. Two differences against Rust 1.102 were separated as reference-version formatting changes. |
| Darwin descriptor-path adapter | The exact C body passes five isolated buffer/error cases with AddressSanitizer and UndefinedBehaviorSanitizer. Real Darwin syscall behavior still requires macOS. |
| no_std WebAssembly execution | Through the unchanged baseline's matched full Rust component, the new module executes its exported operations with correct results and no runtime imports. The final candidate's full contract repeats this gate. |
| Single-file native test command | The old entry incorrectly forwarded the file command to Cargo. The corrected entry passes the public runtime gate: successful native test, upstream failed-test status 101, and one persisted side effect from one failed test. The unchanged baseline native core was used for this local check. |
| Paired performance | Eight fixed hosted campaigns completed, including two with matched native-rustc lanes. Their raw samples, comparisons and regressions are preserved in the performance report; general speed acceptance remains open. Three actual-project campaigns add fifteen comparisons under normal dev/test defaults. |

See [target layout qualification](native-target-layouts.md) for the target
oracle and Darwin corrections, and [mixed compiler qualification](mixed-compiler.md)
for request boundaries and performance methodology.

## Independent source review

Separate automated reviewers performed the Standards and Spec reviews. Both
reviewed the baseline-to-candidate compiler and qualification changes. Spec
review found the missing source-library package component; the constructor
repair was independently re-reviewed and the finding closed. No remaining
blocking source finding was reported.

Both reviewers also checked the exact six-file Darwin runtime patch
`a4dc1d648ea2fc52ec2985411b263a74bd8c71c4711f70c0a9e4e38d0885fbf7`
(SHA-256), including its C buffer contract and target-specific Rust/libc
semantics. Together these reviews cover the protected integrated native tree
`2a1098b47fd9bb9ac85150afc3ac42de76b27445`. The separate constructor repair has
root tree `c7a8e2e96dded79901b29c075a3791de941e77f9` before the Darwin patch.
These are automated source reviews; they do not substitute for execution or
performance evidence.

The follow-through received separate automated Standards and Spec reviews of
the native library selector and its retained failures, the project campaign at
`857a5fc93a8251902651ae63218fb0b73c720b29`, authenticated-retention helper at
`e67c8fc96c48bb06f2c9274f9dfa19634e6d41f2`, and combined workflow at
`4f502f4e258fd90b917bcea451a56eb183fa1ae7`. Review found and closed two library
selection regressions, a public-export schema gap and loss of failed-run private
evidence. The final declared native tree is
`cb797ce8ce82192b87b3835267fc223c295dba70`; the pristine reference is unchanged.
Independent checks included 15 project-export tests, three ownership/lifecycle
tests, 12 actual OpenSSL tests, 16 workflow controls and complete replay of the
four archived fixed-fixture campaigns. No blocking source finding remained.
Those helper checks did not establish complete candidate execution or recovery.
Both subsequently executed at `40d2ac3` within the scope above. Actual authenticated
recovery and source-bound reconciliation verified 600 retained files, 510 raw
streams and all five public project comparisons. The [recovery receipts and
limits](../../performance/projects/2026-11-17-40d2ac3/README.md) explicitly separate
the locally rebuilt replay from unavailable original compiler/runner binaries.

Both reviewers also passed the finer phase-attribution range ending at
`fe18584021d9a9dbea02a6ea1f19aa5c295dbddd`. Review caught variable profiling
text in failed native responses, which would invalidate the daemon's diagnostic
comparison and discard retained state. The corrected emitter preserves those
error bytes exactly; the client records unavailable emitter detail explicitly.
Twenty-two reporter tests pass, and replay of all 26 historical profiles leaves
their existing fields unchanged. Remote emitter durations remain separate from
the client clock, and unmeasured exchange time is explicitly unattributed.
The [original project and profiler source bundle](measurements/2026-11-17-project-profiler-source/README.md)
preserves the reviewed commits and the failed profiler predecessor. This review
does not replace the final combined candidate's source-built execution.

While PR11 was in preparation, its PR9 base advanced to
`74e14e7a14cb283423465f665adf7288e2088a88`. The reconciliation preserves the
base's separately tested temporary-input helper, component-only copy experiment
and queued native-CI policy, together with PR11's profiler and project-evidence
checks. Temporary relinking uses `mixed_link_input::stage`; persistent retention
keeps copy, synchronization and only then recipe-path replacement. The existing
profiling spans surround those operations. Eight actual helper tests, 22
profiling tests, 16 workflow controls, the exact source guard, matching-nightly
component typecheck and formatting check pass after reconciliation. These local
checks do not replace the new combined source-built campaign.

The later trace-framing correction received independent Standards review.
The production-helper regression uses eight concurrent processes and fails the
original writer; all three fixed writer tests and 23 strict profiler tests pass.
The validator remains strict, and original failed native diagnostics, cache
behavior and durability operations are unchanged. The original failed v2 run
and its separately verified reconstruction remain distinguishable.

The first actual project's public and encrypted archives were independently
reviewed against GitHub's original digests and byte inventories. The public
archive checker at that point covered both retained campaigns: 240 timed values and ten
statistics. It does not decrypt or freshly requalify private originals. Each
campaign also has its own completed authenticated recovery and source-bound
reconciliation receipt. The recovery instruction preserves original file modes
explicitly, retaining the first campaign's initial extraction failure instead
of weakening integrity checks.

## Hosted admission gates

The compiler-set workflow builds optimized matched components from source,
reproduces the baseline regression, executes mixed Cargo and library
qualifications, and runs the complete compiler contract. It then measures
pristine Lust, integrated Lust, the lustr entry and matched standalone rustc on
equivalent native work, plus original Rust and mixed lustr on the same Cargo
graph. Cold, no-op,
body edit, signature edit, check, check no-op, check body edit, rejection and
compile-plus-test observations remain separate.

Every mode retains raw samples and identities; the report preserves slower
results and uncertainty. Its small corpus does not establish arbitrary-project
speed. The original native Linux and macOS gates remain unchanged in substance
and retain logs and binary identities. Successful candidate source-identity
checks do not grant release admission.

The first [hosted source-built run](https://github.com/glopyglerky/lustr/actions/runs/43963000944/job/133635056742)
built the optimized candidate and passed all four target-layout checks. It
stopped before runtime qualification because Cargo found the unchanged
baseline checkout inside the product's parent workspace. The workflow now
copies both comparison checkouts into separate temporary roots; neither
comparison's manifests or compiler source are changed. The failed run and
its artifact remain retained, and do not count as the required baseline
regression reproduction.
