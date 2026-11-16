# Mixed compiler completion qualification

This candidate extends PR7 from
`2add988c6397ac5175fcdf89a3aac17d359bf283`. It preserves the pinned Lust
reference, the matched full Rust implementation, and the existing Cargo graph.
It is not an admitted release. Hosted runtime, performance and native macOS
results remain required.

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

## Local evidence and its limits

| Check | Result and scope |
| --- | --- |
| Pinned compiler builds and format checks | Changed owned compiler and qualification tools compile; no daemon execution implied. |
| Failed-link baseline reproduction | Pending hosted execution. The local environment rejects Unix socket creation, so its failed attempt is infrastructure evidence, not the required regression. |
| Source-library package seal | Correct packages made from current and exact baseline source pass. Source mutation, added input, bad links and substituted core inode fail; restoring the package restores its identity. |
| Target synchronization layout | Matching SDK supplies 26 independent size/alignment values for Linux x86_64, Darwin x86_64, Darwin aarch64 and Windows GNU x86_64. Baseline fails the three foreign targets; corrected native frontend passes all four and rejects a deliberate false assertion. |
| Target pool retention | The native unit test passes all nine transitions among Futex, Darwin and Windows models, including body overlays and incompatible-pool rejection. |
| Darwin runtime frontend | Six inherited fixtures pass checks for both Darwin architectures. This is not native linking or execution. |
| Linux runtime preservation | All six affected inherited differential filters pass with the exact CI Rust 1.99.0 oracle: nine cases, zero failures. Two differences against Rust 1.102 were separated as reference-version formatting changes. |
| Darwin descriptor-path adapter | The exact C body passes five isolated buffer/error cases with AddressSanitizer and UndefinedBehaviorSanitizer. Real Darwin syscall behavior still requires macOS. |
| no_std WebAssembly execution | Through the unchanged baseline's matched full Rust component, the new module executes its exported operations with correct results and no runtime imports. The final candidate's full contract repeats this gate. |
| Single-file native test command | The old entry incorrectly forwarded the file command to Cargo. The corrected entry passes the public runtime gate: successful native test, upstream failed-test status 101, and one persisted side effect from one failed test. The unchanged baseline native core was used for this local check. |
| Paired performance | Runner compiles and its statistical/order tests pass. Actual optimized observations and conclusions remain pending. |

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

## Hosted admission gates

The compiler-set workflow builds optimized matched components from source,
reproduces the baseline regression, executes mixed Cargo and library
qualifications, and runs the complete compiler contract. It then measures
pristine Lust, integrated Lust and the lustr entry on equivalent native work,
plus original Rust and mixed lustr on the same Cargo graph. Cold, no-op,
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
