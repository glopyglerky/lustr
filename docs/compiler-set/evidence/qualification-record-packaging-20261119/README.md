# Supplemental qualification records

The complete artifact from published `fb2b2480`, run `44546561681`, job
`135638575172`, is still retained by GitHub as artifact `14541962757`.
Its reported size is **584,645,008 bytes**, above the connector's
**536,870,912-byte** download limit. `remote-original-artifact.json` records
GitHub's declared SHA256. The archive was not downloaded or locally verified;
the download limit was not bypassed. No original artifact was replaced.

The next workflow also creates `exclusive-qualification-records`. The original
complete-upload step and its 26 paths are unchanged. The helper reads that exact
literal path block, avoiding a second list that could drift. Its checks, capture
and supplemental upload use `always()`; the original upload remains independent
of helper success and output. Both workflow event filters include the helper
and its tests.

`records.tar.gz` preserves original regular-file bytes, including binary-valued
stdout/stderr, hidden logs, process traces, sources and tool receipts. Only
recognized Linux ELF object/executable/shared-library, ar or Rust metadata files
of at least 1 MiB omit their payloads. Record names override this classification.
Smaller compiled files, unknown formats and ELF core dumps remain included.
Every original regular file, including omitted compiled files, has a size and
SHA256 in `inventory.json`. Overlapping roots do not duplicate entries. Missing
roots remain explicitly absent; they do not become compiler passes.

The capture uses descriptor-relative opens with no symlink following. Link
targets are preserved and hashed as link text. A symlink named as a record makes
capture incomplete instead of claiming its referent bytes were preserved.
Sockets, FIFOs and other non-regular objects are inventoried without reading
them. Changed inputs and unreadable roots make capture fail. Each entry is
checked across its read; this is not a coordinated cross-file snapshot or a
verified download of the separate complete artifact.

The supplemental archive and inventory have a 480 MiB download budget. If that
budget is exceeded, packaging fails explicitly, preserves its inventory and
leaves the original complete upload required. It does not truncate a stream or
silently drop additional records to fit the budget.

## Observed local controls

`verification.json` binds the helper, tests and workflow to **13 passing
packaging controls**. They exercise the real archive, inspect its members and
compare extracted bytes with the original inputs. The controls include an actual
host executable used only as packaging material, binary-valued streams, a core
dump marker, unknown binary data, hidden logs, raw filenames, external symlinks,
FIFOs, changed inputs, missing roots and capture-budget failures.

The retained bounded sample uses all 26 workflow patterns: eight regular files,
seven included without byte changes, and one omitted compiled file with its
exact hash. Its original files remained unchanged, and its external link was
not followed. The 577-byte sample archive and inventory are under `sample/`.
This is packaging evidence only. It establishes no compiler, retained-process,
Darwin or performance result. The next hosted supplemental upload remains to be
executed and retrieved.
