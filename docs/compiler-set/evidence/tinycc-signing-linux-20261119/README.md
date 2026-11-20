# TinyCC signing repair: bounded Linux evidence

The [original hosted macOS failure](../macos-odd-path-62da158b/) remains a failure.
Cold compilation completed, but the retained odd-path request lost its daemon
reply when the pinned Mach-O signer interpreted the output directory at its
first space. This record establishes the local repair controls. It does not
establish a successful Darwin compiler campaign.

## Source and integration

The unchanged TinyCC pin is
[`43c7708b85681a2fd4451c8a541af4494a8919b2`](https://github.com/TinyCC/tinycc/blob/43c7708b85681a2fd4451c8a541af4494a8919b2/tccmacho.c).
Its Mach-O signing block formats an unquoted filename into a 1024-byte shell
command and calls the fatal diagnostic on signing failure. The
[declaration](../../../../upstream/tinycc-macho-signing.json) binds the original
blob, original file hash, separate patch hash and complete patched file hash.
The original license header and COPYING remain in the source and installation.

The patch passes the unchanged filename as one argv element, after `--`, to one
`posix_spawnp` invocation. It observes that same child with `waitpid`, repeating
only an interrupted wait. Spawn, wait, nonzero-exit and signal failures return
through the existing state-bound nonfatal TinyCC diagnostic. The helper receives
the actual caller's `TCCState`; there is no compiler retry or second signing
attempt. Darwin's inherited environment comes from `_NSGetEnviron()`.

Both construction paths apply the same patch and retain the same receipt:
`tools/build-tinycc-pic.rs` snapshots the applicator and declaration when built,
then applies them to a fresh archive of the exact pin;
`compiler/native/scripts/build-tcc.sh` applies them to the pinned gate checkout.
The applicator verifies all three hashes and fences Git discovery at the source
parent after clearing inherited Git repository overrides. An enclosing unrelated
worktree cannot redirect or silently skip application. Both hosted TinyCC cache
keys include the patch, declaration and applicator.

## Final local result

The fresh constructor completed all commands, including its real whole-archive
shared-link check, while its output directory was beneath an unrelated Git
worktree. The pristine source checkout remained clean at the same revision and
blob. The final qualifier passed all **16 controls**:

| Scope | Controls | Result |
| --- | ---: | --- |
| Source application | 5 | Standalone source, checkout root, nested unrelated worktree, repeated application and altered-input refusal |
| Actual C integration | 1 | Complete patched `tccmacho.c` compiled against the actual pinned `tcc.h` and generated configuration, with signing enabled |
| Exact helper execution | 9 | Byte-exact metacharacters, 1091-byte argument, leading dash and non-UTF-8 argument; nonzero, signal, spawn and wait errors; interrupted wait without another signer |
| Native publication module | 1 | Actual `compiler/native/src/tcc.rs` linked and ran an executable in a directory containing spaces, quotes, Unicode and shell metacharacters; exact output matched |

The long argument is an argv-boundary control, not a claim that a particular
filesystem admits a path of that length. The C integration control compiles the
Mach-O translation unit on the Linux host; it neither links nor executes a
Darwin artifact. The publication driver imports the actual native module and
uses the freshly built libtcc archive. It supplies one C unit directly and does
not qualify a Rust frontend or a retained compiler session.

The first local run's 15 controls and exact helper are preserved separately in
the archive. That helper-only diagnostic stand-in did not expose TinyCC's
state-bound macro dependency. A subsequent full translation-unit check failed
with the missing `s1` diagnostic. The final patch passes the real state and adds
the full-source integration control. The first run cannot qualify either that
final source or a Darwin build.

## Hosted requirements and limits

Linux and macOS CI invoke the same qualifier and retain its complete command
streams. On Darwin it additionally verifies actual executable and shared-library
signatures with `/usr/bin/codesign`, executes/loads those artifacts, and injects
one signer failure through the actual native publication module. That failure
must return normally and leave the previously published executable byte-for-byte
unchanged. The existing upstream retained odd-path compiler scenario remains in
the normal gate without a cold retry or relaxed assertion.

Those Darwin outcomes were pending when this record was written. This change
adds no compilation preflight or source ownership decision. One signer process
replaces a shell plus signer; no numerical speedup, full compiler coverage,
complete hosted matrix or release admission is claimed.

## Recorded evidence

- [Verification and final source identities](verification.json)
- [Final local report](final-report.json)
- [Original and published stream hashes](source-stream-hashes.json)
- [Constructor, initial failure and both control records](controls-and-constructor.tar.gz)

Only local path prefixes and the inherited PATH were normalized to generic
labels in the published streams. Original hashes remain recorded alongside the
published hashes. Outcomes, diagnostics and tested source rules are preserved.
