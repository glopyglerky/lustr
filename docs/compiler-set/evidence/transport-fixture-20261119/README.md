# Complete transport-frame fixture repair

The first hosted run of published `fb2b24802180321d3bbc534dee7083fb2a385d6b`
failed the root-test/package-contract step. Its job was still running the
required retained campaign when this local diagnosis was made; the final
hosted log is separate evidence.

The existing ordinary test reproduces the failure on unchanged published
source: the fixture reader consumes 260 of 262 bytes. The native protocol now
encodes an explicit-target field, but the independent peer still counted the
old frame. The complete encoded frame contains ten fixed fields, feature names,
the displayed source path, and environment key/value pairs.

The repair updates only the test peer. It consumes the entire frame and checks
that the explicit-target byte matches the expected selector state. The ordinary
control now checks both states and rejects the opposite state. Full cursor
consumption, wrong build/source/output and truncated-frame rejection remain
required. The existing real socket fixture expects the unchanged direct-source
request's false state; response-status and terminal-failure checks are unchanged.

The focused test failed before the correction and passed afterward. The exact
commands, curated build environment, before/after test source, unchanged
production protocol, stdout and stderr are indexed by `verification.json` in
`reproduction.tar.gz`. The command uses the declared SDK and the recorded local
partial-package build inputs. It does not create a socket or establish a fresh
hosted package-contract pass. No production compiler, protected native source,
pin or canonical integration patch changes in this repair.
