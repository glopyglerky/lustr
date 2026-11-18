# Lust-port fixture collision: fixed interleaving evidence

The same-process fixture collision is reproduced by one forced interleaving.
A one-line fixture-name correction passes that same ordering and the full
ordinary tool tests. The historical hosted schedule was not observed.

| Lane | Exact source | Result |
| --- | --- | --- |
| Preserved baseline | `8191c173d1f5036c1cbaf84a50cb2dddbc88076a` | Human scanner exits 0 with one unreadable file, zero findings, one read error; the original `attrs.unsupported` assertion fails. Unknown-package assertion passes. |
| Corrected candidate | `ca032fe45fff0931dab84958296a5e761fae68b2` | Human scanner exits 0 with seven files, three findings and one parse error; both original assertions pass under the same ordering. |
| Ordinary candidate | The same candidate, with all diagnostic overlays removed | Eight unit tests and five integration tests pass. The final execution source equals the 145-file candidate tool source. |

The product correction changes only the unknown-package test's fixture label
from `human` to `unknown-package`. The scanner, original test assertions, and
fixture contents are unchanged. Both actual test functions run in threads in
the same process; no artificial PID substitution is used.

The exported scanner pauses after real Cargo metadata and root discovery,
before the original module walk. The sibling fixture helper deletes its own
directory and waits before rebuilding it. This guarantees the human scan
runs inside that deletion window. All barrier paths, process IDs, discovered
roots and complete scanner outputs are retained. The diagnostic scanner's
SHA-256 is identical between lanes:
`15b523b56c7feb09c220cd2c516dc486fcc0fe515baea00e0e05bdf7bf817a3f`.
The ordinary candidate's scanner and two test executables have separate
receipts derived from Cargo's actual emitted executable records.

All three planned executions ran once. The runner's final postcheck then failed:
it incorrectly expected zero file records. The existing walker keeps the
unreadable root as one empty file and records the read error. This report is
still successful and has no findings, so it reaches the original failed
assertion exactly as intended. The executed runner, its traceback and its
`incomplete` report remain unchanged. Corrected replay is pinned to that runner's
hash and candidate revision and checks these same records; no execution was
repeated to obtain a different result.

The local corrected replay rehashed all seven preserved executable copies.
Their 158 MB of payloads remain local and are omitted from this compact archive.
Portable replay verifies their recorded identities and Cargo/source/command
bindings; it explicitly reports that it does **not** rehash absent executable
payloads. The archive includes source snapshots, complete raw commands and
streams, all barrier records, the original and corrected runners, and the
candidate/default source-guard outcomes. Candidate source admission passes;
default release admission still refuses.

Both runs used Rust 1.99.0 commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4` and Cargo 1.99.0 build
`5f94df478`. Exact versions and executable hashes are retained. The earlier
hosted failure and two passing local observations remain unchanged in the
[prior bundle](../lust-port-human-2026-11-18/README.md).

Run `python3 -B replay.py` from this directory to check the compact archive.
This does not compile or rerun tests. Full native/platform qualification,
release admission and performance qualification are separate.
