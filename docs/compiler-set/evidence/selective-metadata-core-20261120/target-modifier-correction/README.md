# Preserve the explicit target-feature modifier

The first native candidate's root metadata decoded successfully with the pinned
original reader. A normal Rust caller then failed in the original crate reader
because the candidate had omitted its synthetic target-feature modifier record.
The failed artifact and complete caller log are preserved by the joined native
consumer qualification record; this correction does not relabel that attempt.

Original `rustc_interface::util::add_target_modifiers` always inserts a synthetic
`TargetFeatures` record. Original `Options::gather_target_modifiers` retains it,
including when its value is empty. In the admitted default x86-64 Linux
hard-float context, the only x86 feature marked as a target modifier is
`soft-float`, which is incompatible with that ABI. There are no requested CPU
or target-feature overrides in this completed scope.

The unchanged original reader applied to the already compiled fixed reference
prints one explicit empty target-feature modifier. Its exact source, artifact,
tool, command and stream identities are in
[original-reader.json](original-reader.json). No additional source compilation
was needed to obtain that result.

The encoder now writes the original `TargetModifier` field order and
`OptionsTargetModifiers::TargetFeatures` discriminant with that actual empty
value. This corrects a missing context record; it neither disables Rust's
compatibility check nor changes the candidate's request or backend ABI. Wider
feature, CPU and target contexts remain unrepresented. Normal Rust consumer
acceptance and runtime results are separate joined qualification gates.
