# Original macOS odd-path failure

The macOS ARM64 job for published compiler PR head `62da158b33740279947c85f3fc425df6d1623c9f`
failed its native crate gate: **84 passed, one failed**. The failed fixture was
`odd-path`. This is original failure evidence, not a repaired-run result.

Source: [CI run 44525981171, job 135566551943](https://github.com/glopyglerky/lustr/actions/runs/44525981171/job/135566551943).
The hosted merge checkout `a3c878f6123b4f5dc5e2eff598b52c83ab5b4ec3` has the same
tree (`390e25946a5ebcb8ccef417467a7b29d8034987d`) as the reviewed published head.

The cold native executable, cold daemon executable, and independent Cargo
executable each exited zero with identical complete output. The preserved
stdout includes generated text, root/dependency build-script environment
values, and successful file checks.

The warm request exited three, with empty stdout and this stderr:

```text
error: daemon returned a truncated or oversized response header
```

The daemon log ends with a missing-path diagnostic at the prefix ending
`/odd-path/sp`. Its actual source and cache directory contains `sp ace#é`.
Pinned TinyCC `43c7708`, `tccmacho.c`, constructs a `codesign` shell command
using the unquoted output filename under `CONFIG_CODESIGN`. The native shared
base is written beneath that source directory. The path splitting explains
the observed prefix; the signing failure terminates the daemon before a
response header. The stricter response-status range check runs after a
complete header and is not the cause of this failure.

`original-streams-and-source.tar.gz` preserves the selected original fixture,
compiler/runtime streams, statuses and daemon log without rewriting bytes.
`archive-files.json` records every regular member of the original case archive,
including hashes for generated binaries and other files not duplicated here.
`verification.json` binds these files to the original artifact and checkout.
`failure.json` preserves the collector's original capture limits.

A later repair must preserve the failing fixture and verify a real macOS warm
request. A local Linux check cannot establish that result, and a cold retry
cannot replace the missing warm outcome. The original CI also skipped its
later base-image-lifetime step after this gate failed; that skipped step is
not counted as passing.
