# Public imported-generation refresh

Implementation base: `3507c74d492a48b27523ff60724eb1ce1d826db9`.

The public owner now validates replacement scalar metadata/archive inputs before
updating its facts. The native session compares exact input identities/bytes,
refreshes declarations and replays semantic reads while retaining unrelated
checked bodies. Imported disk-cache restoration remains unavailable. Dependency
changes discard a cached executable so new archive code is actually relinked.
Malformed inputs fail the request before replacing the previous imported facts.

The implementation reuses the existing retained projection proof's invalidation
mechanism (`driver/compile.rs`) for the public path. It preserves the native
frontend, checker and emitter. No additional Rust source or compiler dependency
is imported. The declared Rust revision remains
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`; the existing metadata decoder owns
artifact validation. The read-replay design follows the pinned native read-set
mechanism and the red/green invalidation model discussed in the existing
[Rust review](../../upstream-rust-review-2026-11-19.md).

`tools/test-public-import-generation.rs` uses an unchanged default-profile Cargo
workspace, a renamed scalar dependency, actual Cargo request capture/replay,
provider body/type edits, malformed input and repair. Pinned Rust independently
consumes the native provider metadata. A wrapper forwards each Cargo request
once; it cannot select an alternative compiler after failure. The recorder
removes its own wrapper setting before forwarding; the product's refusal of
configured external wrappers is unchanged. Initial execution on `bc267683`
passed cold/default Cargo checking, a real no-op compiler request, independent
metadata reads, provider body/type edits and repair, and renamed-definition
failure/repair. It then failed the malformed-input repair control: the daemon
reset the session on status 3 despite import validation having left native
state untouched. The repair distinguishes those input failures from compilation
failures and panics, which retain their original terminal-state handling.
Its installed-path rerun **passed on `3eae63d837195cf454e9ad8b611bc805bf11a47c`**,
including malformed-input repair with zero checked consumer bodies. The public
entry, native client and core hashes for that constructed package are respectively
`f4023aa0beb7be997cf4967f405171b9c0040f2b6bf24ac1147911f287e8ca8c`,
`346cbf940bfa31269a5eab24889a8ae2c69d8a6b70bf5cd7d351da2af9765c92`
and `002de34eb958d8de014e162617cf9a4224f8f8edd5cb6385535460d8f8f7480d`.
The qualifier source hash at that execution was
`b37e3358d59dec4407d701bbe915fdcc9c5bd19d86551fe20ecd10dd2401116b`.

The first offline build failed on
an absent cached dependency; the first source build caught one stale field
name, which was repaired. Original logs remain retained with the runtime owner.

The first recorder attempt hit the existing external-wrapper refusal. A later
evaluator assertion assumed Cargo preserved stale output before invoking its
compiler; Cargo can delete it. The corrected control independently checks
direct-request preservation and requires any output remaining after a failed
Cargo check to have the preceding valid bytes. Both failures are retained.
The native compile test group recorded 32 passes and three missing-child
failures; all nine affected mixed-artifact tests then passed with the pinned
reference compiler supplied. These are bounded controls, not full coverage.

## Associated-item predicates

The same continuation independently reproduced a native false acceptance:
`trait Family { type View where Self: Copy; }` with a non-Copy implementing
provider was accepted by the prior native checker, while the pinned Rust
reference rejected the assignment with E0277. The collector had discarded
the associated-item where clause.

Native collection now retains associated-item type/const parameters, universal
lifetimes, region facts and where predicates under stable item and parent
definition identities. They stay separate from the trait's environment. The
ordinary native impl-obligation checker consumes the item's predicates; trait
fingerprints include them, and retained span shifting covers the new records.
No new complementary implementation is imported. **Three focused tests pass**:
invalid and valid/conditional ordinary providers, actual binders/predicates of
a still-rejected GAT, and trait fingerprint changes. This preserves the GAT
declaration gate rather than claiming normalization/WF/region-service admission.

A subsequent native region slice retains regioned item predicates and checks
them against the actual impl header. In particular, erasing a reference must
neither accept a short borrow under a `'static` requirement nor reject a real
`&'static T`. The native region checker now consumes those requirements using
its existing header frames, declared and implied facts, and trait-candidate
region constraints. **Five focused tests pass** on this updated source.
The earlier named-lifetime-trait fixtures exposed the existing separate
associated-type gate; their valid cases remain unqualified, and the original
failed test logs are retained. That gate and the GAT gate remain in place.
All 32 retained patch tests passed before this subsequent region change; they
are not transferred as a post-change result.

The installed qualifier now also requires three valid and three invalid ordinary
associated-item cases to match pinned Rust, and a separate abort/debug-disabled
linked dependency edit to change actual program output while reusing native
body checks. Its final-source execution is pending at this checkpoint. The
linked control does not establish default-profile build/run or Rust debugging.
Existing broader-campaign count expectations now require zero native work for
provider-only body edits while still requiring changed metadata/archive work.

The constructed `56441e16` package passed the complete scalar check history and
all six ordinary predicate differential cases. Its first linked control failed
before compilation because Cargo requested an unrepresented incremental-link
context. The separate linked fixture now declares `incremental = false` along
with its original abort/debug-disabled profile. The default-profile check
fixture is unchanged. This failure remains separate from the passing checks.

The next native consumer check reproduced a second false acceptance: a generic
`T::View` use omitted the associated item's `Self: Copy` condition. Pinned Rust
rejected it with E0277; the preceding installed compiler accepted it. Native WF
now consumes the ordinary item's trait predicates at its projection. Bound
provenance is retained so the trait declaration checks each associated bound in
that item's environment without turning its conditions into supertraits.
**Six focused tests pass** after the repair. Its initial test exposed the scope
mistake and remains retained; the repaired valid cases and invalid consumer
pass their independent expectations. Full projection outlives/environment
checking and the selective upstream GAT service remain open.

## Final bounded qualification

[Verification](verification.json) binds compiler source
`d2edc3a3d2e504c52577d9dbdae5eb4795ff2065`, actual package executable hashes,
qualifier bytes, operation statuses, timings and original stream hashes.
The final installed scalar Cargo history passes, including invalid-provider
publication preservation, changed signatures, missing definitions, malformed
inputs and repair. Actual no-op/compiler-repair requests retain unrelated bodies.
Four valid/invalid ordinary predicate pairs match pinned Rust. The separate
same-workspace abort/debug-disabled/non-incremental linked fixture produces
`63`, `63`, then `127`; the provider edit rechecks zero consumer bodies and
actually relinks the changed dependency. This is not default-profile linking.
Six focused tests, all 32 final-source retained patch tests and 29 existing
evaluator controls pass.

The first cross-workspace linked fixture exposed the existing profile-context
refusal; that scope remains unqualified. Its source and failure are retained.
The broader traced campaign was not run because this environment has no
strace; no fresh process-trace qualification is inferred from these checks.
Production GAT service/provider metadata, default-profile build/run/debug/test,
full replacement and performance qualification remain open. No release is made.

GAT provider validation, selective upstream normalization/WF/region service,
wider metadata and full Cargo/debug/runtime qualification remain unfinished.
This slice makes no near-stock or full-replacement performance claim.
