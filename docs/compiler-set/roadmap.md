# Integrated compiler roadmap

The [architecture plan](architecture-plan.md) is the fixed product contract.
The [completion checklist](completion-checklist.md) is the single ordered work
list, with observable completion criteria. Do not create another roadmap that
replaces imported upstream complementary support with independent native Rust
reimplementation, a coordinator, or whole-program retry.

## Order of work

| Order | Deliverable | Completion evidence |
| --- | --- | --- |
| 0 | Authoritative contract and one recorded integration head | Corrected default instructions; unique existing fixes retained and duplicate work identified. |
| 1 | Full Lust capability and checked-state interface | Decisions use resolved facts before emission and cover Lust's actual support, with complementary needs and real failures distinguished. |
| 2 | Shared semantic continuation with imported Rust services | Supported requests retain Lust's frontend; mixed requests carry one authoritative semantic context across both implementations. No mandatory duplicate complete frontend. |
| 3 | General ABI/runtime/artifact composition | Supported and complementary definitions in unchanged programs exchange values, ownership and calls correctly and produce genuine Rust artifacts. |
| 4 | Retention across that boundary | Cold and warm answers agree across body, signature, type, trait, macro, dependency and configuration edits; failures do not publish stale output. |
| 5 | Normal standalone and Cargo entry | The integrated compiler replaces status/diagnostic-driven whole-program retries. Existing CLI/build behavior is preserved. |
| 6 | Exact-source correctness and package qualification | Declared Rust version and advertised platforms pass meaningful native, mixed, dependency, runtime and package gates. |
| 7 | Finite practical performance assessment and publication | Near-Lust supported work, near-stock ordinary complement, useful measured gains, clear losses and reproducible README evidence. Publish the qualified integrated package. |

Necessary correctness repairs and consolidation can proceed alongside the
semantic interface work. They do not require a new performance win. The
approximately 0.3 ms overhead is a bounded investigation, not a reason to keep
rewriting working compiler behavior.

## Reviewed state

Default `next` at `7cea76c9d3adc587b54ccff168fe07efafd40309` predates the scalar
mixed implementation. The [design checkpoint](design.md) distinguishes it from
the architecture-audited PR #29 candidate. PR #30 is a later correctness
continuation; its qualification does not follow automatically from PR #29.

Existing Lust compilation/retention, scalar checked-state admission, real
Rust metadata/object integration, Linux native gates and tested mixed failure
recovery are work to retain. They do not yet provide the general semantic
boundary required above. Read the [qualification checkpoint](qualification-review-2026-11-18.md)
and [performance checkpoint](performance-review-2026-11-18.md) before marking
anything done or rerunning a supposedly missing gate.

Historical coordinator, compiler-tuning and benchmark experiments remain
evidence for their original sources and workloads. They cannot close the
integrated-compiler tasks. The [previous roadmap](https://github.com/glopyglerky/lustr/blob/7cea76c9d3adc587b54ccff168fe07efafd40309/docs/compiler-set/roadmap.md)
is retained in history; its independent native expansion and compulsory
faster-results requirements are superseded.
