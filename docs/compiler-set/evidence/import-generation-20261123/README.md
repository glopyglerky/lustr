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
once; it cannot select an alternative compiler after failure. Execution is
pending at this implementation checkpoint. The first offline build failed on
an absent cached dependency; the first source build caught one stale field
name, which was repaired. Original logs remain retained with the runtime owner.

GAT provider validation, selective upstream normalization/WF/region service,
wider metadata and full Cargo/debug/runtime qualification remain unfinished.
This slice makes no near-stock or full-replacement performance claim.
