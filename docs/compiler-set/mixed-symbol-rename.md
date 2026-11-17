# Mixed Cargo object renaming in process

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

These are component correctness checks. A fresh complete source-built Cargo
qualification and paired campaign must establish any end-to-end speed change.
