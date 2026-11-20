# Mixed Cargo object renaming in process

> Historical source-bound record. The object-renaming mechanism remains useful; the compiler fallback described below is retired.
> Follow the [active correction](active-correction.md) and
> [exclusive coverage contract](exclusive-coverage-contract.md).
> Earlier results and proposals below do not qualify current source or authorize
> a fallback, duplicate Rust frontend, runtime ownership probe or new install route.

The mixed Cargo driver renames its validated TinyCC export to the actual Rust
symbol without launching `objcopy`. Cold selection and retained body edits use
the same safe Rust helper. The original compiler analysis, ABI checks, metadata,
object selection and linker remain unchanged. A refused object or failed
publication keeps the existing full-compiler path before execution.

The helper transforms the exact bytes returned by the shared bounded ELF
validator. It appends a replacement symbol string table, preserving its
alignment, and updates only that table's offset/size and the selected export's
name offset. Instructions, data, relocations and symbol indexes are unchanged.
An existing replacement symbol is rejected. The complete result is validated
again. Only its exact expected Rust export may exceed the input format's short
name limits; total output remains bounded to 64 MiB.

Publication uses a fresh sibling file and a no-overwrite hard link. Readers can
only observe the complete output, existing destinations and symlinks are never
replaced, and the output never shares an inode with the input. Existing caller
durability barriers still apply. The temporary is removed on success or error.

`tools/test-mixed-symbol-rename.py` builds actual TinyCC objects and checks
shrinking, unchanged and growing names, including names longer than 4096 bytes.
GNU `objcopy` and `nm` provide independent symbol checks, and separately linked
Rust executables exercise both outputs. Tests also check unchanged original
bytes outside the three edited fields, malformed input, imports, collisions,
size limits, output ownership and failed publication.

These component checks are separate from the completed `6bf6e3e` source-built
qualification. That complete package passed mixed correctness, all sixteen
entry/runtime tests and the [paired campaign](../../performance/2026-11-17-6bf6e3e.md).
Its body-edit Cargo ratio is 1.079×; seven other Cargo modes remain slower.
The [original profile](../../performance/2026-11-17-6bf6e3e-profile.md) records
0.043122 ms in body-edit renaming. The measurements cover all three Cargo cost
corrections together and do not isolate this change's causal effect.
