# Completed Linux jobs and recovered qualification records

Reviewed on 20 November 2026 UTC. The current constructed-package job and both
previously unfinished broad Linux jobs completed successfully. The current
supplemental capture is complete, and its original available bytes have now
been independently verified. Neither current macOS job executed a step.

This review downloads and checks existing evidence. It runs no compiler,
generated program or archived executable, changes no workflow, and does not
attribute these results to later source changes.

## Exact source and jobs

| Source | Published head | Actual merge checkout | Common tree |
| --- | --- | --- | --- |
| Current | `c8218b7fc0f657314c9e9c59775091b53508c6b0` | `ec8f89f98e2701f7189d4310e54a1501cdd81f34` | `4c796fc9e34ddc09be79f6697125258da9c6c188` |
| Prior | `7c3d095fa8971a4c38aa0edf62e8fcfd7a51589a` | `66963315e9736ce9f151ce8ccac248d35a01d048` | `2a6a51f099bdcdb74871fdc2e65a4df87aa0c867` |

Both source trees contain native tree
`bf933c382fd99e52dd2586ebc46f0d0aceaa71a8`. GitHub's original commit objects,
the checkout lines and the artifact source fields agree. The two public fixture
sources at the current head exactly match their previously reviewed and tested
SHA-256 identities; [verification.json](verification.json) records them.

| Job | Result |
| --- | --- |
| [Current package, 135694089996](https://github.com/glopyglerky/lustr/actions/runs/44562165627/job/135694089996) | Every recorded step passed, including the final supplemental capture and both uploads. |
| [Current broad Linux, 135694090003](https://github.com/glopyglerky/lustr/actions/runs/44562165596/job/135694090003) | Every recorded step passed. |
| [Prior broad Linux, 135661771306](https://github.com/glopyglerky/lustr/actions/runs/44553094710/job/135661771306) | Every recorded step passed; the cleanup warning below remains visible. |
| [Prior package, 135661771824](https://github.com/glopyglerky/lustr/actions/runs/44553094694/job/135661771824) | Its earlier capture failure remains a failure. The passing compiler steps were already recorded [separately](../native-cargo-hosted-7c3d095f-20261119/README.md). |

[source-and-jobs.json](source-and-jobs.json) preserves the selected original API
fields and step outcomes. Actor details are not needed for the source binding.

## Current package execution

The complete decoded current package log is included, byte for byte, in
[original-logs.tar.gz](original-logs.tar.gz). It is 564,891 bytes with SHA-256
`42ed89302a5538faec8d5bb16a69f16e86fa7feb13e541a9d54654d578bcb5e9`.

| Executed check | Current result |
| --- | --- |
| Guarded construction and ownership | Passed; the package remains a partial native development build. |
| Ordinary root targets | 93 passed; two deliberate child-process fixtures remained ignored. |
| Installed-package contracts | All five ran explicitly and passed, including the real socket/ambiguous-status control. |
| Exclusive public entry | 71 passed, none unavailable. |
| Exact capability results | All 18 passed: 17 programs and one complete named-test comparison. |
| Retained qualifier controls | All 55 passed without skips, including the five actual pinned-oracle controls. |
| Required retained inventory | All 349 exact required names and 1,744 versions passed. |
| Selector cache / seed / compact controls | 297 assertions in 156 invocations; 87 in 38; 394 in 83. |
| Supplemental packaging controls | All 13 passed. |
| Supplemental capture | `capture_complete=true`, 36,647 inventory entries, `errors=[]`. |

The retained report still records 13 unresolved histories and 105 versions.
`full_retained_coverage`, `full_rust_coverage` and `release_admission` remain
false. The exact required-name comparison is against the inventory at the
published head, independently of the candidate's own result rows.

The actual process control again reports seven matching native/oracle program
results, 38 permitted information queries and refusal of a compilation request
disguised with a print option. It binds the original daemon to PID 5598 and start
ticks `28970`. Unchanged, body-edit and feature-recovery requests start zero
Cargo or external hashing processes. Fresh explicit package selection starts
five Cargo and four hashing processes; feature selection starts three Cargo
processes; an existing explicit package starts two Cargo processes. The initial
ordinary request follows explicit-package setup and is not a cold-start result.
These are process observations, not a latency comparison.

The current public entry SHA-256 is
`013c580386e3dd4eafd536d3f057902e73f60344d7696f348169f37f17620800` and the
reported native core SHA-256 is
`4c4773be960f60325b993f82cdbc60c5be30ba5a25ceab534ad227268cd20303`.
The core's omitted executable bytes were not downloaded from the full package;
its identity here comes from the original constructor and qualification
receipts. It is not substituted with the prior run's different core hash.

## Broad Linux execution

The current and prior broad Linux logs contain the following observed results:

| Gate | Result in each job |
| --- | --- |
| Native unit groups | 32, 6, 23 and 419 passed; the one ignored pipe fixture is a deliberate child-process fixture. |
| Ordinary and sanitizer differential campaigns | 5,547 passed in each campaign, zero failed. |
| Macros / dependency macros / clients / crates | 58 / 13 / 89 / 85 passed, zero failed. |
| Original daemon scenario gate | 364 of 364 reported `ok`, **with 29 unsupported versions tolerated by that donor gate**. |
| Edit fuzzing | Seed 7, small crate, 40 modules, 1,200 edits, zero failures and zero fired guards. |
| Native and emitted-C unwind differential campaigns | 65 passed each, zero failed or unsupported. |
| Unicode comparison | 1,112,064 scalar values checked. |
| Dedicated retained and ABI controls | Slot, base-lifetime, typed-main transition and erased-callback steps passed. |

The donor gate's 29 tolerated versions are not silently promoted to coverage.
In particular, its typed-main allowances do not replace the dedicated
four-version transition gate. The strict public 349-history qualifier and the
original donor gate have different admission and outcome contracts.

The Linux odd-path fixture passed. Linux helper and publication controls for
the TinyCC signing patch also passed, with `darwin_execution=false` in their
original reports. None of these results establishes Darwin signing or the
Darwin warm odd-path behavior.

[result-lines.json](result-lines.json) retains the exact log lines supporting
the counts, including their original line numbers. The full logs are preserved
under [original-logs.tar.gz](original-logs.tar.gz). Their decoded identities are:

| Log | Bytes | SHA-256 |
| --- | ---: | --- |
| Current Linux | 295,778 | `228ba09ffbfeda6fd7ca3b96d0afd5c0a6ac4ce7ea077a9e90f77991f6c4abda` |
| Prior Linux | 295,898 | `4ca86ee2b9bb7e81753c5c9c734b992bd5efdbb6b19303cf9ae4ae55c4e492e9` |

### Open fixture cleanup issue

The prior Linux log reports `test-crates: 85 passed, 0 failed`, then records:

```text
rm: cannot remove '/tmp/tmp.yxL8JDyUmT/odd-path/sp ace#é/src/.lust-cache/bases': Directory not empty
```

The inspected `compiler/native/scripts/test-crates.sh` installs an EXIT trap
that removes its work directory. Its warm fixture path lets the client start a
private daemon with a 20-second idle limit and does not explicitly stop and
wait for that daemon before removing its directory. The log is consistent with
cleanup racing a still-running private daemon. It does not show a generated
program mismatch, and the job's recorded conclusion remains success.

A bounded repair should give this test ownership of the daemon's lifetime,
observe its stop before deleting the private directory, and preserve both test
and cleanup failures. That would change a protected native test script and
require its normal source declaration. No such edit or new execution is made
in this evidence update; compiler metadata implementation remains separate.

## Independent archive recovery

All four original ZIPs were downloaded through the supported artifact endpoint
within its size limit. Their complete byte hashes match the server-declared
digests. Every included Linux ZIP member was read with ZIP CRC validation;
every supplemental tar member was checked against its original inventory.
The verifier checks safe and unique member names, file kinds, sizes and hashes,
link text without following symlinks, source/helper/workflow identities, exact
required-history names and every recorded stdout/stderr reference.

| Supplemental | Current | Prior |
| --- | ---: | ---: |
| ZIP bytes | 13,585,105 | 13,603,715 |
| Included members verified | 36,619 | 36,619 |
| Regular payloads verified | 35,965 | 35,963 |
| Regular bytes verified | 63,032,243 | 63,031,706 |
| Original retained stdout/stderr hashes verified | 26,126 | 26,126 |
| Capture complete | **true** | **false** |

The current original ZIP is retained once here as two exact binary parts:
[part 1](c821-supplemental-original.zip.part-01) and
[part 2](c821-supplemental-original.zip.part-02). Ordered concatenation produces
the unchanged 13,585,105-byte ZIP with SHA-256
`93c83be3ef1ccef020f3330868aad7f7ab4d3415d550763ff18d24ac3a9cce36`.
Its `inventory.json` SHA-256 is
`29770ebaa0e6a5fd4c1c3c407f8958ad8a7d38aeeffd94adb27f453f3674b63c`;
its inner `records.tar.gz` is 13,572,090 bytes with SHA-256
`9b5221df85d8eb7fb5e85d5181e84c016c58046001ed6ff5044545f7bcf0c0fd`.
All three current receipt members are regular files with their expected exact
bytes, including the seed fixture's intentionally mutated receipt.

The [part manifest](archive-parts.json) records their order, exact lengths and
individual hashes. This storage adaptation was necessary because the connector's
16 MiB request limit cannot carry the complete ZIP's base64 form. The parts are
8,000,000 and 5,585,105 bytes; neither reaches 8 MiB. No archive was recompressed,
record rewritten or qualification result changed. The whole original ZIP was
preserved outside the checkout before replacing its tracked storage.

The updated verifier concatenates the parts in memory, requires the original
size and digest, then performs the same complete ZIP CRC, gzip/tar, inventory,
source, member and retained-stream checks. Its current-only invocation is:

```sh
python3 verify-recovered.py --root . --source-root REPOSITORY \
  --output NEW_VERIFICATION_DIRECTORY --current-supplemental-only
```

`REPOSITORY` must contain the exact published commit object. This mode makes no
new historical or Linux ZIP verification claim. The
[transport verification](transport-verification.json) records the repeated
current-archive integrity proof and five refused damaged-part controls. It
executes no compiler or archived program.

This supplemental is not binary-free: it includes small generated artifacts
and fixture executables. The content review found 304 ELF/archive payloads
totalling 22,242,530 uncompressed bytes. The 28 omitted large compiled files
remain inventoried, and their payloads are unverified here. The content scan
found no credential markers or private-workspace paths. Its six email-pattern
matches are all Rust's diagnostic spelling `closure@main.rs`.
[sanitation-review.json](sanitation-review.json) records the actual reviewed
scope, file classes and matches; it is not an assertion about unobserved files.

The prior original ZIP was independently recovered and verified again. Its
[exact inventory](7c-original-inventory.json.gz) and selected original reports
are preserved here. Its two errors remain unchanged: the cache package receipt
and relocated seed package receipt are symlinks whose referent bytes were not
read. Available-member integrity is true; complete-capture verification remains
false. This recovery does not change the earlier failed job or reconstruct the
unpublished local receipt-repair archive lost with the old workspace.

The two complete Linux ZIPs were verified locally but are not copied into the
repository. Each has 646 members. The much larger complete package artifacts
remain remote and were not downloaded; their server digests are not presented
as local verification. [artifact-metadata.json](artifact-metadata.json) retains
the downloaded artifact identities and expiration metadata.

[verification.json](verification.json) is the compact result;
[verified-members.json.gz](verified-members.json.gz) contains every verified
member identity and per-history count. [selected-original-records.tar.gz](selected-original-records.tar.gz)
preserves selected report, package and tool receipts without rewriting their
payloads, and [selected-record-identities.json](selected-record-identities.json)
indexes those bytes. [verify-recovered-before-split.py](verify-recovered-before-split.py)
preserves the actual initial recovery verifier, and
[verify-recovered.py](verify-recovered.py) adds the exact-byte part reader.
The initial aggregate-report classification error and later
missing historical Git object are recorded in
[verification-attempts.json](verification-attempts.json); neither was a compiler
failure or an archive mismatch. Historical verification resumed after obtaining
the exact original commit object. No compiler campaign was repeated.

## macOS and overall qualification remain open

Current CI job `135694089936` and planning-helper job `135694089751` are both
`completed/cancelled`, with no recorded steps and no assigned runner. Both use
the `macos-26` label. The current API evidence supplies no cancellation cause;
earlier capacity annotations belong to their own earlier attempts.

Consequently the broader CI and compiler-set workflows are not a fully passing
platform matrix, despite their completed Linux jobs. Actual Darwin signature,
failed-publication and warm odd-path results remain unavailable. The separate
current runner-routing canary was also cancelled before executing a step.

These results qualify bounded existing functionality at the two named trees.
They establish neither full upstream-Lust/Rust coverage, complete consumer
metadata, complementary Rust compilation, `no_std`/socket removal nor an
overall or subset speed result. Release admission remains false.
