# Upstream Lust reference and explicit integration

Preserve the pristine pinned Lust reference and allow a small, explicit reviewed
integration diff. Focused interfaces inside Lust may share phase results and
retained state, eliminating duplicate parsing or rechecking. Preserve supported
semantics and speed. Imported Rust implementation is allowed as a separately
identified component; independent semantic reconstruction is not required.
The [native compiler contract](native-contract.md) remains the full-project goal.

## Pristine reference

[`upstream/lust.pin`](../../upstream/lust.pin) pins `t3dotgg/lust-lang` revision
`240e08d91d4aed0299639f6f11fd522912b248f9`, tree
`0ab1b09815802894e6078de0af59273cbdf46cb0`. Keep those original Git objects and
their provenance. The upstream default was observed at that revision; this is
not a claim about its current remote state.

The integrated tree lives at `compiler/native`. Its declared diff is currently
**empty**. No nonempty integration patch has been admitted. Existing
experimental owned paths and historical compatibility artifacts retain their
original scope; this policy does not promote them or relax blueFe's Rust pin.

## Admission

Before admitting a nonempty integration, retain:

1. The pristine revision/tree, a focused interface need, exact patch identity and
   expected integrated tree. Prefer existing phase, session and optimizing
   C/LLVM machinery over a duplicate frontend, type system, backend or daemon.
2. Acceptance, rejection, runtime, ownership, panic/drop and relevant edit/reuse
   evidence. Keep final rechecks when semantic inputs change and preserve firsts.
3. Independent review of that exact source and the proposed interface contract.
4. Appropriate performance evidence. Compare pristine Lust with the minimal
   integration, then unused-extension cost, then mixed/native-complement work
   against equivalent stock work. Preserve profiles, targets, features and input
   proofs. Separate profiling from timing and retain individual supported-path
   regressions. A necessary interface need not win an unrelated optimization
   benchmark. No zero-overhead or full-project gain is presumed.

Keep proven improvements and unique experiments. Replace a working path only
after its replacement is verified. No hidden overlay, arbitrary
drift, generated source substitution or unrecorded build-time patch is allowed.
Source/API compatibility and any promised prebuilt Rust-binary interoperability
are distinct obligations. Production full Rust may use the pinned Rust
implementation. Preserve its licenses and provenance and identify its role.

## Source guard

The dependency-free Rust guard verifies the pinned revision's actual tree and
the integrated committed tree (or staged tree with `--index`). It rejects scoped
worktree changes, unexpected staged changes and nonignored untracked additions.
It refuses assume-unchanged and skip-worktree flags within the protected scope.
Patch generation explicitly includes gitlinks regardless of local Git settings.
The declaration consists of:

- [`lust.integration`](../../upstream/lust.integration): reference revision/tree,
  patch blob identity, qualification-record blob identity and integrated tree.
- [`lust-integration.patch`](../../upstream/lust-integration.patch): the exact
  normalized full-index binary Git diff between reference and integrated trees.
- [`lust-integration.qualification`](../../upstream/lust-integration.qualification):
  matching identities, status and correctness/review/performance evidence references.

Blob and tree identities use the repository's Git object format. The guard
checks patch and qualification bytes with `git hash-object` without writing
objects. It independently regenerates the complete tree diff and compares exact
bytes. An empty patch requires the reference tree and `empty` qualification;
nonempty declarations require `reviewed` status and all three evidence references.
Changing the record or expected result without matching identities fails.

Evidence references are admission records, not executable proof. The guard
cannot establish that a named review or benchmark actually happened; the owner
must verify those results before declaring the integration reviewed. Synthetic
guard-test records qualify the guard only. Current empty integration does not
claim a new compiler performance result.

Build outside the source tree, then run from its root:

```sh
rustc --edition=2024 tools/lust-upstream-check.rs -o /absolute/cache/lust-upstream-check
/absolute/cache/lust-upstream-check
```

`--index` supports a staged exact restoration or integration before commit.
CI obtains the exact pristine commit in a separate disposable checkout and gives
the guard read access to those Git objects; the compiler source remains the
declared integrated tree. The guard verifies the commit's actual tree, so shallow
fork history alone is insufficient. CI also runs positive exact-diff and negative
drift tests. The guard does not
certify ignored outputs, external/submodule contents, compiler correctness,
publication clearance or universal source/secret absence. Retain unique changes
on failure; do not reset them to make the check pass.
