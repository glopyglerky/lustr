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
Its installed-path rerun is pending at this checkpoint.

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

GAT provider validation, selective upstream normalization/WF/region service,
wider metadata and full Cargo/debug/runtime qualification remain unfinished.
This slice makes no near-stock or full-replacement performance claim.
