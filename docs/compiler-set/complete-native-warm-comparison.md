# Compare independently built complete native packages

Status: separately named measurement infrastructure. Local controls exercise
the real orchestration and replay with explicitly synthetic child results. They
are not compiler execution or speed evidence. Both hosted attempts stopped
before measurement. The latest built both packages and passed the baseline's
native unit tests, then failed its differential gate with the wrong Rust oracle.
A completed, correctly qualified hosted comparison is still required. No native
speedup, Lust parity or release admission follows from this infrastructure change.

The retained-check materialization candidate changes native source. PR25's
existing entry-only comparison requires identical native runtime source and
cannot qualify that change. Its runner and workflow remain unchanged. The new
runner is `tools/run-warm-native-comparison.py`; the new workflow is
`.github/workflows/warm-native-performance.yml`.

## First hosted failure and harness correction

[Run 44337186487](https://github.com/glopyglerky/lustr/actions/runs/44337186487)
at candidate `43a7e07d32929309fc6e402d7bca6b5673e186a2` built both complete
packages through their own constructors. The baseline's native unit gate then
failed before any measurement. Its client suite passed 15 tests. Its core suite
passed 386 and failed `driver::buildscript::tests::script_reruns_when_what_it_read_changes`:
the first build-script observation returned empty cfg/environment lists instead
of `bs_a` and `BS_DATA=v1`. Candidate gates and all timing steps were skipped.

The direct native gate omitted
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1`. The public lustr entry already
supplies this pinned-Cargo compatibility setting for Lust's build-script
handling. The corrected harness supplies and records the same setting for
both packages' correctness commands. Portable replay requires its exact value.
The packages' constructors, source pins, native compilers, measured commands,
corpus, test assertions and strict timing decision are unchanged.

The original failure remains in
[artifact 14443389636](https://github.com/glopyglerky/lustr/actions/runs/44337186487/artifacts/14443389636),
`complete-native-warm-1`, ZIP SHA256
`ff8f1c40cb90dec85df8703a08855ebe5987745a13a54f72dac44fd4c6c01aa1`.
Its baseline unit stdout has SHA256
`b4216170dca888f5f70aed28fd74a9c65d7b67a8b180daac72097ee3c92171a2`;
stderr has SHA256
`22cea66855c97e83732ff632204cc7075de7abc4d9f0e707ac2012952ce3affe`.

The first corrected local harness passed 29 evidence controls and eight startup
controls. Four new controls reject missing or conflicting layout settings in
the unit and differential evidence for both package lanes, even after raw-log
checksums are recomputed. These use synthetic children and establish harness
behavior only. The subsequent hosted attempt below verifies the unit-gate
correction but remains incomplete.

## Second hosted failure: native differential oracle mismatch

[Run 44351483409](https://github.com/glopyglerky/lustr/actions/runs/44351483409),
job `134968011454`, built both complete packages. Its candidate was GitHub's PR
merge `1e2f30b1d9330d7d0e8d717f793f44409672c330`, tree
`83dad205038ab41271c94429c12970282d0dbea4`, for head
`3853cbceb4c93258ba6e919ae44af4e4c7366ce7`. The baseline remains the exact frozen
revision in the lane table below. This run predates later product fixes and does
not qualify them.

The baseline passed all **15 client and 387 core unit tests**, including
`driver::buildscript::tests::script_reruns_when_what_it_read_changes`. That
confirms the earlier layout correction worked on the hosted runner. The next
command, the original native differential corpus, exited 1 with **5,428 passed
and 99 failed out of 5,527 cases**. It neither timed out nor exceeded the output
limit. The preserved failure lines divide as follows:

| Recorded failure | Cases |
| --- | ---: |
| Program stderr differed | 62 |
| Program stdout differed | 16 |
| Rust rejected `macro_fragment_kinds` because `box_patterns` had been removed | 1 |
| Rust accepted a fixture that the corpus requires it to reject | 20 |

The harness ran the corpus with **Rust 1.102.0-nightly**, source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, from the package build SDK. The frozen
native [README](https://github.com/glopyglerky/lustr/blob/3c378a11d60149a3ec99fa49ea1b336e9406fca8/compiler/native/README.md)
declares Rust 1.99 as its oracle, and its
[CI](https://github.com/glopyglerky/lustr/blob/3c378a11d60149a3ec99fa49ea1b336e9406fca8/compiler/native/.github/workflows/ci.yml)
installs and selects 1.99.0. The unchanged `case-pool.sh` calls literal `rustc`
through `PATH`; the receipt records the newer nightly's directory first. Its
script hashes match the frozen source. This establishes the oracle-selection
error. It does **not** establish that all 99 failures will disappear under 1.99;
that corrected corpus has not been executed in this continuation.

The original evidence remains in
[artifact 14453918048](https://github.com/glopyglerky/lustr/actions/runs/44351483409/artifacts/14453918048),
`complete-native-warm-1`: ZIP SHA256
`572ee0aa2ab681d72516334b6b7243ae548153106e3dac90c6cf1d804a0afbcc`,
1,090,171 bytes and 361 files. All nine recorded build/gate command rows and
their captured stdout/stderr hashes were checked against the receipts. The
upstream differential script prints bounded failure diffs and deletes its
per-case temporary directories; the archive preserves those emitted streams,
not unavailable full case outputs. Candidate qualification, pristine build,
measurement, practical assessment and startup diagnostics were skipped. There
is no complete-package timing result from this run.

The second correction installs a **separate Rust 1.99.0 oracle** and requires
`qualify-package --native-oracle-sdk`. The gate records its actual `rustc -vV`,
compiler digest and SDK file/link inventory, including compiler-driver, LLVM and
standard-library files. It requires source commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, release 1.99.0 and the Linux x86-64
host. Only the differential command and its identity preflight receive this
SDK's `bin` directory first in `PATH` and its compiler in `RUSTC`. Unit builds,
the public-entry contract, both constructors and the measured Cargo/Rust lane
retain the existing nightly SDK. No default toolchain is changed.

Both package gates must bind the same oracle identity. Live checks compare its
inventory before and after qualification and before measurement; portable
replay checks the recorded identity, command, environment and paired bindings.
The new gate receipt schema is `lustr-complete-package-gates-v2`; the original
failed v1 receipts remain unchanged. The full corpus, expected diagnostics and
output comparisons, frozen baseline, package sources, build and measurement
schemas, and strict timing result are unchanged. No new campaign or hosted
rerun was started to validate this second correction.

The corrected harness passes **43 complete-package and workflow controls**,
**eight startup controls** and **14 practical-assessment controls**, all with
synthetic compiler children. New negative controls reject missing or wrong
oracle identities, changed live runtime files, missing runtime inventories,
different oracles between packages, and a differential `PATH` or `RUSTC` that
falls back to the build SDK. Recomputed evidence checksums do not bypass these
checks. The positive fixture still replays all 384 synthetic measurements with
the original SDK and complete gate sequence. These are harness results only.

## Independently built packages

| Lane | Source and construction | Executed entry and daemon |
| --- | --- | --- |
| `baseline` | Complete frozen revision `3c378a11d60149a3ec99fa49ea1b336e9406fca8`, tree `0256606d5793bfb890130f05fae013d0c5c5f9e1`; that source's own normal release constructor. | Its own `toolchain/bin/lustr` and `toolchain/bin/lust`, with its own core, shared library, source library and installed selection seed. |
| `entry` | Complete candidate checkout at the recorded Git revision/tree; that source's own normal release constructor. | Its own `toolchain/bin/lustr` and `toolchain/bin/lust`, with its own core, shared library, source library and installed selection seed. |
| `pristine` | Unmodified Lust `240e08d91d4aed0299639f6f11fd522912b248f9`, tree `0ab1b09815802894e6078de0af59273cbdf46cb0`, built with the matched release profile. | The separately built pristine client and its own daemon/core. |
| `cargo` | Matching imported Rust SDK `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. | Cargo plus `rustc-reference`; the application retains its declared dev profile. |

Both Lustr packages are built from scratch into different output/work roots.
Neither public launcher is copied into the other package. Each constructor is
compiled from its own exact tracked source, including its local source-module
closure. The command, compiler identity, constructor source/blob hashes,
constructor executable digest and original stdout/stderr/status are retained.
Both constructors run normally with `--release`, including the native source
guard, component builds, source-library seal, selection-toolchain receipt and
installed `--lustr-prime-selection-tools` initialization.

The two packages share the selected SDK and pinned TinyCC dependency, not their
compiled native components or package output. Their host SDK receipts and Rust
tool digests must agree. TinyCC is source-built from
`43c7708b85681a2fd4451c8a541af4494a8919b2` using the existing PIC builder; its
installation inventory is matched between packages. Native build hashes and
binary digests are recorded independently and may differ.

The portable build receipt has schema `lustr-complete-source-package-v1`. It
contains source identities, constructor commands, constructor source snapshots,
normal constructor logs, profile/component/native/SDK receipts, package file
and link bindings, the actual installed selection certificate, and the native
differential input inventory. The replay checks that each seed names that
package's own installed launcher and selection-toolchain receipt. It rejects
the older entry-only receipt schema.

The public baseline lane uses its own package `PATH`, `CARGO`, `RUSTC` and
`RUSTDOC`. The candidate uses its own equivalent paths. Pristine and Cargo use
the matched SDK tools through the candidate package's reference-tool paths;
the pristine native client/daemon/core remain independent. Every lane owns
its Cargo home, target/intermediate directories, native cache, temporary
directory and socket. Each warm native command must execute while that lane's
recorded daemon is alive, with the matching package environment. The replay
rejects a baseline launcher or daemon substituted with the candidate's,
cross-package tool paths, shared active sockets, and mismatched environments.

## Correctness gates before timing

Each complete package must pass the following commands, with source/package
identities and original streams retained under schema
`lustr-complete-package-gates-v2`:

1. The separate native oracle's recorded identity preflight, requiring Rust
   1.99.0 at the exact source commit and host above, with an unchanged compiler
   and runtime inventory. Both packages must use the same oracle.
2. All native client and core unit tests, compiled with the source's `gate`
   profile, one compiler job and one test thread. Cargo's actual test-executable
   artifact records and separate executable digests are preserved. These
   correctness binaries are not the release performance binaries.
3. The unmodified ordinary native differential corpus. An exact Git export of
   that package's native source receives a `target/release` link to its already
   built installed package binaries. The original differential script runs
   every top-level `.rs` case in `tests/pass`, `tests/fail` and `tests/reject`,
   against the separate 1.99.0 oracle with two workers and no Rust result cache.
   The complete case count must
   match the recorded source inventory, with zero failures. The replay binds
   the export command, exported root, script/case-pool hashes, fixture inventory
   and actual release client/core identities back to that package.
4. The complete ignored public-entry contract from that source, using its own
   package and the release entry build. It must execute successful tests; a
   filtered, skipped or failed gate does not permit measurement.

This first candidate campaign additionally requires the two
`no_change_check_` tests in a fresh process with `LUST_CHECK_RESOLVE=1`, using
the same gate-profile core test executable identified by Cargo. That gate is
deliberately specific to the retained-check materialization candidate. A
different candidate needs an explicit reviewed gate change, not an empty test
filter accepted as success.

The original differential harness has its own bounded compile-timeout retry
and normalization of source paths/thread IDs. Its individual cases are not
being described as unnormalized byte-for-byte stderr comparisons or as a
retry-free campaign. No new retry or relaxation is added here. The separately
recorded public-slice runtime gates below compare exact exit status, stdout and
stderr and reject signals/timeouts.

Full sanitizer, fuzz and platform qualification remains in the separate
automatic native CI. Passing this comparison's Linux package gates does not
substitute for those jobs or permit release admission. The native source guard
continues to require exact integration declarations and candidate status.

## Fixed comparison and decision rule

The measurement receipt uses schema `lustr-warm-native-packages-v1`. It contains
24 balanced, seeded four-lane pairs for each of cold check, warm check, cold
build and warm build: **384 complete-command timings**. The existing paired
statistics, retained-work counters and process/runtime helpers are reused.
The complete-package source/receipt and lane validation is separate; the old
identical-native guard is never weakened or bypassed.

The fixture is the existing public `goport-util-small` slice and unchanged
upstream driver, with the same explicit standalone manifest. It is a public
pruned/adapted workload, not unchanged full ts-rust. Its application profile is
dev opt0/debug0 with unwind panic, overflow checks and debug assertions, and
Cargo incremental compilation enabled. Native builds use TinyCC's ordinary
development path. Both baseline and candidate source fixture inputs must match
the preserved pristine reference.

Cold requests receive new lane directories with native daemon/disk cache
disabled; operating-system caches are not flushed. Every warm state receives
three complete priming commands before the measured no-op. Each prime remains
in the raw evidence and its cost table. Timed commands carry no native stats,
entry trace or selection diagnostics.

Initial and final public-slice gates require successful cold compilation,
workload-specific rejection, actual retained check/build counters and exact
runtime agreement with Rust. Both public package lanes must show actual Cargo
selection hits and their own retained daemon route through the sequence
original source, valid body edit, invalid type edit, restoration. The invalid
build cannot leave an executable. Every measured successful build is executed
and bound by digest to that build command. These edit transitions qualify
behavior; they do not measure edit speed.

All observations, regressions, priming costs and failed commands are retained.
The report includes baseline/candidate comparisons and each package's
comparison with pristine Lust. The original recorder retains its strict target: both warm modes must have a
paired pristine-Lust/candidate geometric speed ratio whose conditional bootstrap
95% lower bound is at least 1.0. An interval crossing one does not prove equality.
A completed strict-target miss still exits 2 and preserves the full result.
An incomplete build, gate or campaign fails without a performance table.

The continuation workflow applies a separate
[practical warm acceptance policy](practical-performance.md) after this complete
record has passed original replay. It records the measurement's actual exit
status and accepts only completed status 0 or 2 for assessment. Missing, failed,
tampered or unexpectedly terminated evidence remains a failure. Each warm mode
must fit the 10% or 0.5 ms budget and retain a demonstrated win over matching
Cargo/rustc. The assessor writes separate JSON and Markdown; original raw
records, statistics and strict-target fields remain unchanged. Passing the warm
budget does not waive runtime/platform qualification or establish a universal
Rust speed ratio.

## Execution and replay

The workflow builds and qualifies both packages serially before measurement,
using a 180-minute job allowance for the two full constructors and differential
corpora. It always uploads setup/gate/campaign receipts and captured streams,
including when no timing was admitted. Gate-profile test executables and
constructor binaries are identified by digest; the portable evidence omits
those executable bytes. The separate native-oracle SDK is recorded by version
and inventory, without embedding its executable bytes. Source revisions and
archived constructor inputs remain explicit. The complete ephemeral package
itself is not a release artifact.

After downloading the campaign evidence, replay it with:

```sh
python3 tools/run-warm-native-comparison.py verify /absolute/path/to/native-warm-comparison
```

Portable replay checks both construction and qualification receipts, raw stream
digests, source/package identity bindings, owned daemon/environment records,
both edit sequences, complete initial/final gates, all build/runtime bindings,
the exact 384-command order, all priming records, statistics and rendered table.
It does not re-execute a compiler or establish missing platform qualification.

The local harness controls run with:

```sh
python3 tools/test-warm-native-comparison.py
```

They run actual orchestration with labeled synthetic child outputs, then test
rejection of missing or corrupted evidence and real shell handling of failed
and completed-target-miss statuses. Their generated commands and measurements
are temporary test data, not performance or compiler qualification evidence.
