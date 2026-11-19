# Imported code emission belongs to the build continuation

The shared integer service previously invoked `llc` from
`Provider::validate_checked`. That violated the native preparation boundary:
both a `Check` request and a build that would be abandoned after preparation
could execute the adapter compiler before returning a `PreparedRequest`.

The [frozen original witnesses](evidence/shared-import-emission-20261119/before-fix.json)
demonstrate both failures against unchanged provider and native driver sources.
The qualifier added only an action choice. With an intentionally invalid LLVM
option, both requests returned status 3 with the real `llc` argument diagnostic.
Neither reached the prepared-request callback. Existing output bytes survived,
but preparation had already executed code generation. The original harness
patch, source hashes and executable hash are preserved.

## The repaired ownership boundary

Semantic validation now checks only the installed facts. After the checked
program and lowered arenas have transferred successfully, the provider moves
one owned code plan into the native continuation. The plan holds immutable
archive object bytes, LLVM input and exact tool identities. Its type is
`Send + 'static`; it carries no borrowed Rust compiler arena. The concrete plan
and ABI module are not cloneable, and taking the plan twice is an error.

`Check` returns without executing that plan. Dropping an unused prepared request
also drops the plan. An admitted `Build` consumes it after all semantic and
complementary-return branches, before native object generation and linking.
The adapter module returns validated owned object bytes directly; there is no
lazy `OnceLock` accessor that can run an emitter during validation.

The tool identities, exact LLVM argument modes, object validation, imported
archive closure and last-successful-artifact rules are unchanged. Failures
during plan transfer or actual emission are terminal compiler failures. The
ordinary native path has no imported plan and allocates none. This change does
not implement retained imported generations or public compiler routing.

## Source-bound qualification

The [clean qualification](evidence/shared-import-emission-20261119/verification.json)
rebuilt the executable from:

| Identity | Value |
| --- | --- |
| Source commit | `5f8db03a0931bda306232ad3d645bee0c1c174cc` |
| Complete source tree | `1f5e81e817a7fc157b3005bdfb4fdcd73bd01215` |
| Native source tree | `211fc0c51ae2a69ac83395fdadb31caf6ffa869b` |
| Qualifier SHA-256 | `a1a4e3d4299d09299777b2677e506fcba04260a88615bfe880c5e869f745cfa7` |
| Rust source | `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` |

All **29 process/compiler/runtime controls** passed. The
[case inventory](evidence/shared-import-emission-20261119/campaign.json) and
[original outputs](evidence/shared-import-emission-20261119/outputs.json) record:

- Normal checks and abandoned prepared builds succeed, create no object files,
  and preserve a previous executable or leave an absent output absent.
- Checks and abandoned builds also succeed with each deferred missing-tool,
  failed-tool, malformed-output and empty-output control. These same controls
  fail an actual build, so they distinguish skipped emission from ignored errors.
- Invalid native source is rejected before LLVM in build, check and abandoned
  preparation modes. Matching Rust rejects the same source with E0308.
- Four actual adapter emission failures, an already-transferred plan and an
  object-publication failure return status 3 and preserve prior output bytes.
- A fresh build recovers after failures, including when its dependency snapshot
  pathname is removed after preparation.
- Native and matching-Rust execution each check all 65,536 signed-byte operand
  pairs against a wider integer sum followed by reduction to `i8`. This is a
  complete finite arithmetic slice, not a proof of all compiler semantics.

The source-bound run used `tools/qualify-shared-upstream.py --emission-only`.
It did not repeat the earlier 120-control semantic campaign. The default hosted
launcher runs the existing campaign followed by the new lifecycle campaign.
The earlier 120-control result retains its original source identity and scope.

An independent reviewer also replayed both exact original integer-consumer
witnesses, including their frozen dependency archive, against the repaired
development executable. Both succeeded with 30 planned adapters, no object
files and unchanged prior output bytes. That
[independent record](evidence/shared-import-emission-20261119/independent-after.json)
keeps its distinct development executable hash; it is not substituted for the
clean source-bound executable above.

Full current-source native/platform qualification, consumer Rust metadata,
general semantic integration, imported reuse, package admission and performance
qualification remain separate work. No new latency claim follows from this
correctness repair.
