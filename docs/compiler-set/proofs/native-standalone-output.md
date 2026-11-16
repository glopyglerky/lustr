# Native semantic persistence and standalone output

Status: explicit integration candidate, unadmitted. The canonical integration
declaration still describes the empty admitted patch. The normal source guard
must reject this candidate's worktree changes until exact correctness, review,
performance and integrated-tree admission are complete. Qualification builds
must bind the complete proposed patch and source; they do not promote it.

The focused Lust interface separates `OutputKind` from semantic persistence.
`Session::new` retains its original defaults: a persistent session selects the
session image; a one-shot session selects standalone output. `set_output`
invalidates compiled objects and overlays on a layout change, while preserving
the native frontend, semantic read sets and retained programs. No language
implementation or resolver is copied into the integration.

Standalone output builds the complete base-form object set, then invokes the
existing native linker once. It creates no libbase or thin executable. The
existing `link_fn_units` determines the actual function and glue object closure;
the owned caller no longer reconstructs that closure from group counts. The
in-memory run shortcut is restricted to standalone output. Session-image output
and the original constructor's callers retain their existing link paths.

The owned shared entry selects standalone output and sends the original request
directly to the driver. It still forces fresh objects, disables disk objects,
validates the runtime generation and input context, and runs changed-input
semantic checks. Original collection provenance and conservative rejection
remain unchanged. A transformed AST is retired after its request.

Required evidence includes supported and complement runtimes, edits and
rejections with recovery, generic/drop/unwind behavior, standalone ELF closure
after compiler exit, and original session-image behavior. Exact source review
and optimized equivalent-work performance remain separate gates. Removing two
links is a structural mechanism, not a measured speed claim. Full native Rust,
typed crate producer/consumer artifacts, Cargo composition and release profiles
remain open. Rust bootstrap, TinyCC/C, mimalloc and the host platform remain
explicit dependencies.

The Rust `native-output-layout-qualification` example composes the actual
native compiler sources. Its constructor-default and live image → standalone
→ image sequence includes populated overlays and rejection/recovery after
both transitions. It executes each distinct output once after retiring the
native session, verifying original images still load their retained bases.
The helper is explicitly bootstrapped; its applications use the native driver.
