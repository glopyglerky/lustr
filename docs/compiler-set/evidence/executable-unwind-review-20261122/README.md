# Executable unwind review, 22 November 2026

The executable unwind change at `930b384a` is a useful continuation of the
existing native compiler. This review found no new compiler defect in the
inspected change or the additional runtime cases. It corrects stale status
documentation and makes the next semantic-service acceptance test concrete.
Ordinary default-profile compilation, Rust source debugging, general cross-crate
unwinding, full Rust coverage and complete-work performance remain unfinished.

This is a source-bound review of [PR #33](https://github.com/glopyglerky/lustr/pull/33),
which remained open and unmerged. It changes no compiler or evaluator source.
The [architecture plan](../../architecture-plan.md) and
[completion checklist](../../completion-checklist.md) remain the implementation
plan and sole work register. This evidence note does not grant release admission.

## Sources and original CI failure

| Identity | Value |
| --- | --- |
| Previous reviewed head | `bc8a7adb36fcae44f0bca686d503714646f21b5d` |
| Executable unwind implementation | `5c769ab5b36da48f8efd96bad88787b831840525` |
| Reviewed correction | `930b384ad03a26fe141d10be4e261bd82a60be76` |
| Reviewed source tree | `0de8622237295e2607b7ffed22359749c2e4bc1a` |
| Native integration tree | `8b535898c1a1df6d4c046b75289702c8bd7f6265` |
| Hosted merge checkout | `fc77881243e36849fe1d87b1d22ba572c0197900` |
| Pinned Rust source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |
| Declared SDK | `nightly-2026-11-09`, Linux GNU x86-64 |

The hosted merge checkout has exactly the reviewed source tree. The pristine
Lust commit `240e08d91d4aed0299639f6f11fd522912b248f9` and tree
`0ab1b09815802894e6078de0af59273cbdf46cb0` remain unchanged. The explicit
integration patch reproduces the new native tree, and the patch, qualification
and all 52 public source-manifest hashes agree. The three native files changed
are the checked emission path, the object cache and the TinyCC interface.

The first push failed. [Compiler set run 45083831224](https://github.com/glopyglerky/lustr/actions/runs/45083831224/job/137397777628)
stopped at the unchanged native publication/signing caller with E0061 because
`tcc::link` required four arguments while the caller supplied three. The
separate Linux/macOS workflow also failed at that qualifier. `930b384a` restores
the existing three-argument API and its previous options, with a separate
`link_with_frames` entry for the newly admitted route. This preserves the
original gate. The original decoded failure log is retained and hashed in
[verification.json](verification.json).

## Standards review

No documented-standard violation or material design smell was found in the
increment. Admission remains restricted to the actual Linux GNU x86-64 host;
unwind linking with Cargo dependencies stays unimplemented. The new disk-object
key distinguishes frame-enabled units. The semantic profile remains part of
the retained session identity. Ordinary and non-Linux TinyCC callers preserve
their previous options and publication helpers. The existing library DWARF
evaluator and its controls are byte-for-byte unchanged.

The change preserves one native compilation path. It adds no complementary
compiler import, second frontend, trial compiler or fallback. Structural source
preservation does not establish full language coverage or performance.

## Spec review and hosted evidence

No incorrect behavior was found in the represented executable increment.
The independent review downloaded [public artifact 14778558073](https://github.com/glopyglerky/lustr/actions/runs/45084610787/artifacts/14778558073)
from [job 137400268103](https://github.com/glopyglerky/lustr/actions/runs/45084610787/job/137400268103).
The 70,099,429-byte ZIP matches SHA-256
`137742eb4d514115bf258413dda5b255e4ebfc76b0d405e25c4b7bc76c1da586`.
All 13,122 captured source files match the reviewed checkout. The 749 recorded
stdout, stderr and trace files across 328 campaign commands match their hashes.
The source and native trees match independently of the status badge.

The executable fixture uses the actual Cargo request, default unwind strategy
and an explicit `debug=0` profile. Its cold and unchanged outputs are
`drop:leaf`, `drop:middle`, `caught`, `drop:outer`, in that order. After the edit,
the first line becomes `drop:edited`. Pinned Rust produces the same outputs.

| Executable history | Cargo status | Native bodies checked | Artifact result |
| --- | ---: | ---: | --- |
| Cold | 0 | 17 | New executable |
| Unchanged request | 0 | 0 | Identical executable; no observed artifact-tool work |
| Body edit | 0 | 1 | Changed executable with the edited result |
| Invalid edit | 101 | No successful work report | Last valid executable unchanged; no program execution |
| Repair | 0 | 1 | Same bytes as the valid edited executable |

The cold and edited executables each have 1,235 nonempty, disjoint FDE ranges
and a GNU_EH_FRAME segment. This is linked frame inventory evidence. The panic
test exercises Lust's existing explicit propagation and cleanup, so successful
`catch_unwind` alone does not prove system DWARF unwinding.

| Preserved hosted qualification | Observed result |
| --- | --- |
| Complete public library campaign | Pass; 16 original Rust consumers and 4,194,432 assertions |
| Existing library unwind history | Objects compiled: 3, 0, 1, 0, 0 |
| Profile semantics | 24/24 paired cases pass; no fallback observed |
| Library debugger walks | Six selected-DWARF walks pass on GDB 15.1 |
| Library corrupted-CFI controls | Both rejected by the same frame predicate |
| Evaluator controls | 29 public, 3 profile and 11 frame controls pass |
| Full Compiler set workflow | All ten jobs pass, including shared-service regressions |

The 4,194,432 assertion count belongs to the preserved library/metadata campaign.
It does not count Rust language features or include the new executable and
profile tests. The existing debugger gate qualifies its two native library
frames and Rust caller, not arbitrary executable frames or source debugging.

## Independent runtime follow-up

An independent actual-Cargo evaluator ran the constructed `930b384a` package
against pinned Rust, with fixed expectations before candidate execution.
Eight retained cases cover unwind, unchanged unwind, abort, unchanged abort,
return to unwind, a body edit, a type error and repair. Six further cases cover
panic payload/resumption, a caught panic inside `Drop`, partial initialization,
partial moves, terminal double panic and thread isolation. All 14 pass.

The evaluator distinguishes compilation from runtime. Expected terminal cases
first compile successfully, then run and terminate with SIGABRT. An arbitrary
compile failure cannot pass as an expected panic. Candidate process tracing
finds no unexpected compiler execution. The original source, package and failed
artifacts are preserved. [runtime-review.json](runtime-review.json) records
the exact evidence and work counters; the included reproduction harness is a
review tool, not an additional product route or a matched timing campaign.

## Independent executable frame observation

The separate [executable frame record](executable-frames.json) checks the
constructed package's edited unwind executable with GDB 17.2. At its first
stdout write, the evaluator records a live stack/RBP return-address sequence,
then disables every GDB unwinder except `dwarf2`. The resulting eleven native
frames through generated `main` match that independent PC sequence exactly.

Changing one `DW_CFA_def_cfa_offset` operand in a private executable copy leaves
the normal cleanup output, exit status and FDE count unchanged. The same
backtrace comparison then fails after frame zero. This distinguishes working
native CFI from the explicit panic propagation observed by the runtime tests.

After printing the required native frames, GDB cannot unwind the libc caller
beyond native `main` with only `dwarf2` enabled. That diagnostic and earlier collector attempts
remain in the evidence. The result qualifies this native frame sequence only;
it does not qualify the full system stack, original Rust names/types/locals,
other stopping points or a new installed CI gate. The existing library gate
retains its separate source, debugger and corruption controls.

## Next implementation decision

Keep ordinary Cargo/debug completion and the first genuine mixed semantic
service moving in parallel. Default Cargo dev behavior requests full debug
information. Completing that behavior requires original Rust source, type,
parameter and local information, with real breakpoint and stepping checks.
Frame ranges or generated-C locations cannot satisfy it.

The [associated-type review](associated-type-boundary.md) identifies the
specific missing operation as `Family::View<'a>`. Ordinary associated types
already have native implementations. The archived query preserves a non-static
upstream placeholder and observes changed dependency metadata, but it runs
through original compiler infrastructure and does not feed a result into Lust.
Its missing-impl branch also accepts any obligation error rather than proving
the specific rejection reason. The archived runs lack a complete binding of
executed arguments, SDK, binary and metadata inputs; this review reports their
source and logs without calling them independently replayed qualification.

Implement a selective typed service over the native definitions and inference
state, preserving binders, universes, constraints and semantic dependency reads.
The exact adapter and necessary source closure still need proof. The decisive
retained history changes the provider from `&u32` to `&u64` while leaving its
consumer unchanged. That consumer must fail for the correct type errors,
preserve its prior executable and recover after repair. Unrelated native bodies
must remain reusable. The review note specifies lifetime, negative-lookup and
stale-answer controls. Complete this boundary before another broad scalar
expansion; do not package the research driver as the complement.

## Hosted status and limits

At the status check recorded in `verification.json`, `930b384a` had 20
successful checks, no failed checks and one running macOS job. The broad Linux native job had passed. Both platform signing steps
had passed. This is an observation at that source and time, not a claim that
the entire matrix or a later head is green. The existing PR watch remains the
source of later hosted updates.

This review establishes bounded correctness and retained reuse, with no
measured complete-work speedup. No full-Rust, default-profile build/run/test,
Rust source-debugging, general cross-crate-unwind or release qualification follows.
Keep those obligations in the existing checklist and measure each material
semantic boundary as it lands.
