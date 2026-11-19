# Current measured-cost assessment

`tools/assess-current-performance.py` implements the performance dimension of
the [current policy](practical-performance.md). It preserves the frozen
historical assessors and their original decisions. No current complete compiler
campaign is supplied by this change.

```sh
python3 tools/assess-current-performance.py \
  --plan plan.json \
  --samples samples.jsonl \
  --evidence-root /absolute/path/to/recorded-evidence \
  --output /absolute/path/to/new-assessment
```

The output directory must be new and separate from the evidence directory.
Exit `0` means the measured overall speed criterion passed. Exit `2` means it
was slower or inconclusive. Exit `1` means the input or evidence was rejected.
Every result has `release_admission=false`. Neither an input Boolean nor a
performance pass authorizes release or establishes a complete Rust compiler.

## Statistic and decision

Fix each workload case and its integer frequency before measurement. For pair
block `i`, sum the complete public-request durations:

```text
R_i = sum_j frequency_j * stock_rustc_seconds_(i,j)
L_i = sum_j frequency_j * lustr_seconds_(i,j)
speedup = sum_i R_i / sum_i L_i
```

The tool resamples complete pair blocks 10,000 times with fixed SplitMix64,
rejection-sampled indices and the plan's seed. Every replicate uses the same
ratio of summed elapsed costs. Its percentile interval uses indices
`floor(0.025 * 9999)` and `ceil(0.975 * 9999)`. The performance criterion passes
only when the lower 95% bound exceeds one. An equal-time result or interval
crossing one is inconclusive.

This is a weighted workload total, not a geometric mean of per-case speedups.
A fast tiny case cannot conceal more time spent on a large case. Fixed
frequencies also prevent a rare improvement from concealing frequent losses.

Each case retains its separate comparison, mean, median, empirical nearest-rank
p95 and maximum. The near-stock comparisons preserve the existing per-case
geometric mean of within-pair ratios and its paired bootstrap interval. That
statistic is never the overall workload decision. Successful supported check/build cases
with lifecycle `warm-no-op` use the existing 10% engineering target and also
require a demonstrated per-case Rust/Lustr speedup. Successful complementary
check/build cases compare with stock Rust using 5%. Other modes retain their
measurements without inventing a numeric target. These target results are
separate from the overall decision. A near-stock miss
remains visible even when the actual total is faster. Conversely, satisfying
both targets does not establish that the total is faster. No absolute-latency
allowance overrides a failed relative near-stock comparison.

Use frequency zero only for a declared diagnostic comparison that is outside
the actual workload. Its results remain in the report. In particular, do not
substitute an isolated emission measurement for a complete Cargo artifact
request. A supported comparison with stock Lust must request equivalent work.
If Lust cannot produce that artifact, qualify the matching supported operation
separately and retain the extra artifact service in the full public request.

## Input contract

JSON rejects duplicate keys and nonfinite numbers. Every file reference is
exactly `{"path": "relative/path", "sha256": "64-lowercase-hex-digits"}`.
Referenced files must resolve inside the evidence directory. The tool checks
all declared bytes and rechecks their hashes before returning.

The plan uses schema `lustr-current-performance-plan-v1` and these fields:

| Field | Required meaning |
| --- | --- |
| `scope` | Concrete workload and qualified context being compared. |
| `pairs` | 24 through 4096 complete paired blocks. Complete all engine-order cycles, multiples of six when supported cases exist and otherwise multiples of two. These are assessment design bounds, not a universal statistical guarantee. |
| `seed` | Unsigned 64-bit bootstrap seed fixed with the plan. |
| `sampling_order` | `rotated-cases-balanced-engine-permutations-v1`. |
| `timing_boundary` | `public-command-start-to-completion`. Include dispatch, loading, transport, required compilation, artifacts and publication. |
| `compilers` | Maps `lustr`, `rustc`, and stock `lust` when used to exact identities described below. |
| `cases` | Ordered, nonempty workload case list described below. |
| `qualification_records` | List of supplied record file references. Empty means none was supplied. Their existence is not a qualification pass. |

Each compiler identity has exact 40-character Git `revision` and `tree` values
and an `artifacts` map. It declares `entry_kind` as `direct-compiler`. The map
must contain `entry`. Include every compiler
library, helper, runtime and other executable dependency required by the measured
request under named roles. The assessor hashes declared files. It cannot infer
that an undeclared dependency was used.

Version 1 supports a direct compiler command executed with the evidence root as
its working directory. Its `argv[0]` must be `./` followed by the declared entry
artifact's relative path. The execution receipt also identifies that exact
entry file and hash, with `working_directory` equal to `.`. A matching plan and
receipt that time a different executable while naming an unrelated compiler
inventory are rejected. Arbitrary Cargo and wrapper chains are outside this
version's capture contract; listing their files does not qualify the chain.

Each case has a unique `id`, a declared `class` of `supported`, `complementary`
or `mixed`, a nonnegative integer `frequency`, an ordinary `expected_exit`
status, a `request` file reference and `commands`. The latter maps each engine
to complete literal argv. Supported cases need all three engines. The other
classes need `rustc` and `lustr`. At least one case has positive frequency.
These class labels record the proposed ownership, not proof of that ownership.

The request file contains nonempty `operation`, `target`, `profile` and
`lifecycle` strings, a `flags` object, a nonempty `source_inputs` list of file
references, and an `artifacts` list of unique output roles. All compared engines
bind the same request digest. A successful build must retain its requested
artifact evidence. Different implementations may have different literal argv;
the reviewer must verify that flags, semantics, source, outputs, cache state and
work are equivalent. Declaring the same digest cannot prove that by itself.

The assessor exports `sequence(plan)` for capture tools. Within each pair,
rotate the case order by the pair index. For each case, enumerate permutations
of `rustc, lustr` or `rustc, lustr, lust` and select the permutation at
`pair % permutation_count`. Samples must appear in that exact order. A missing,
duplicated or reordered observation fails assessment.

Each JSONL sample is exactly:

```json
{"pair": 0, "case": "case-id", "engine": "rustc", "receipt": {"path": "receipt.json", "sha256": "..."}}
```

The execution receipt has schema `lustr-current-performance-execution-v1`,
`plan_sha256`, `pair`, `case`, `engine`, `compiler_identity_sha256`,
`request_sha256`, `argv`, positive finite `seconds`, `exit_status`, `signal`,
`timing_boundary`, `working_directory`, `execution_entry`, `stdout`, `stderr`
and `artifacts`. The signal must be null
and the exit must equal the predeclared expected status. Both streams are file
references. Artifacts map exactly the requested roles to retained file
references. Reusing one execution receipt for multiple observations fails.

`plan_sha256` hashes the plan file's original bytes. Compiler identity hashes
use the tool's `serialized(identity)` representation, sorted keys, two-space
indentation and a terminal newline. A referenced qualification record must
bind `candidate_revision`, `candidate_tree` and `compiler_identity_sha256` to
the same candidate. Other qualification content is retained but not interpreted
as admission. The report lists every supplied record identity.

## What the evidence does and does not establish

Hash binding detects changed plans, source inventories, artifacts, commands and
records. It does not prove that the plan predates measurements, that the
execution occurred, that a duration is truthful, or that the captured program
behaved correctly. Those require independent capture and qualification review.
The paired interval is conditional on this workload, host and an appropriate
independent block sampling design. P95 is an empirical observation, not a tail
confidence bound. Parallel builds require actual complete-request measurements;
their overlapping internal spans cannot be summed as serial costs.

Full Rust correctness, exclusive Lust ownership, selective imports, real
metadata, downstream artifacts, retention and advertised platforms remain
separate requirements. A file with `full_coverage=true` cannot discharge them.
This tool does not claim that a complete compiler, current matched performance
campaign, universal equivalence or a releasable package exists.

## Regression controls

Run `python3 tools/test-assess-current-performance.py -v`.

The controls use explicitly synthetic files and timings. They execute no
compiler. They test the arithmetic and reject missing measurements, altered
commands, partial-phase timings, stale qualification identities, changed helper
binaries, missing output artifacts, failure statuses, interrupted requests and
invalid durations. The adversarial cases include a favorable geometric mean
with a slower total, both near-stock targets satisfied with a slower total, and
an overall speedup that still misses the near-Lust target.
