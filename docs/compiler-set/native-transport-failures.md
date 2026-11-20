# Native daemon failures at public entries

> Source-bound checkpoint. Statements about the current candidate, required
> follow-up and measured speed below refer to the named source and date. The
> [active correction](active-correction.md) identifies present implementation
> work; the [current performance policy](practical-performance.md) controls new
> claims. Historical successes, incomplete paths and failed results remain scoped
> to their original implementation.

The native client and the primary Lustr entry now keep ambiguous daemon
exchanges terminal. A partially written request, a disconnected peer, a lost
reply, a malformed frame, an old unclassified response or an unexpected
compiler identity returns failure. The entry cannot turn that result into a
cold native compile or a Rust retry.

The explicit `OtherBuild` reply remains distinct. The daemon produces it while
reading the request header, before admitting any compilation. A missing listener
or that explicit refusal can therefore start a new daemon. Other connection
errors, including permission denial, are failures. Startup errors and a readiness
deadline also remain visible instead of silently selecting another compiler.

## Wire and ownership checks

Both ordinary and mixed requests use the same exchange implementation. The
reader bounds the fixed response header, checks the status without integer
truncation, verifies the expected compiler identity and response kind, and reads
exactly the declared body lengths. It grows buffers only for bytes received;
a claimed enormous length followed by EOF cannot force an allocation of that
length. Invalid UTF-8 is rejected because the producer's response fields are
Rust `String` values. An ambiguous old reply no longer unlinks its endpoint.

The source change is
`64686b9c74606b3b25039e0415b12538480d0fc5`. The combined native tree is
`80a66542b8dc348b036e2974ab2e6e0c42a433fe`. The
[independent native record](evidence/native-final-20261119/transport-verification.json)
records compilation of both native binaries and 21 protocol tests. Root library
and entry tests also passed, with 3 library tests and 37 entry tests; the existing
child-process fixture remains invoked by its owning test.

## Actual public-process regression

The [public-process record](evidence/public-transport-20261119/verification.json)
compares the old and new public entries with the same real native compiler and
valid source. Socket creation is unavailable in that executor, so it supplies
an actual connection-permission failure.

| Entry | Previous behavior | Corrected behavior |
| --- | --- | --- |
| `lust` | Returned success and replaced the preexisting output after the connection failed | Returns status 3 and preserves the output bytes |
| `lustr` | Returned success, replaced the output and recorded cold-process compilation | Returns status 3, preserves output, and records no fallback compilation |

The output sentinel detects replacement; it is not represented as an executable
from a prior successful compilation. This control establishes connection-failure
handling, not an accepted request with a lost reply.

## Actual Darwin socket qualification

The [Darwin record](evidence/public-transport-20261119/darwin-host-verification.json)
binds all four compiled host source files to published revision
`724e1d03e0c732b270e5734846e86899321a7a11`. All 30 host tests passed on actual
Darwin arm64, including complete requests followed by missing, old-format,
wrong-build and truncated responses. The real socket in a long directory,
native C-header ABI checks, and process cleanup controls also passed.

This host test executable used Rust 1.95.0. It does not qualify the complete
compiler against its declared Rust 1.102 SDK or replace the native sanitizer,
mixed-package or performance gates. The raw test output and source digests remain
in the record.

The complete-package suite separately exercises ten cases through actual `lust`
and `lustr` executables. Its fake peer validates the installed core's true wire
identity and consumes the entire request before returning the controlled reply.
The original fixture incorrectly used the test executable's identity; its failed
packet comparison and corrected production-encoder control are preserved. Those
ten package cases remain subject to the hosted exact-package result.

## Remaining compiler integration

This repair removes transport replay. The older standalone status-1 and
diagnostic-driven whole-program retries remain separate implementation gaps in
the [completion checklist](completion-checklist.md). They cannot be removed by
renaming this transport change or by treating a native rejection as proof that
Rust is invalid. Shared semantic continuation and normal public routing still
have to supply the missing integrated behavior.
