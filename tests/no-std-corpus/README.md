# Native no_std acceptance corpus

This unchanged project is an acceptance workload, not a coverage claim. It
combines core's prelude, const generics, fixed arrays, pattern matching,
incremental state, checked slicing, borrowing and explicit optional allocation.
The optional freestanding Linux x86_64 binary uses its own entry and panic
handler, a real library dependency and a direct exit syscall. Inspect its ELF
dependency closure before running it. Stock compilation is an independent
oracle only; native compilation must produce and consume the real library
artifact through the same Cargo graph.

The dev/release panic policy is abort, declared in this manifest. Preserve the
semantic/profile inputs between engines and retain native failures/artifact
absence. Rejection is not a speed baseline. Core-only and optional alloc need
separate evidence. Other targets and full no_std remain open until qualified.
