# Compare independently built complete native packages

Status: separately named measurement infrastructure. Local controls exercise
the real orchestration and replay with explicitly synthetic child results. They
are not compiler execution or speed evidence. A hosted complete-package run is
still required. No native speedup, Lust parity or release admission follows from
this infrastructure change.

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

The corrected local harness passes 29 evidence controls and eight startup
controls. Four new controls reject missing or conflicting layout settings in
the unit and differential evidence for both package lanes, even after raw-log
checksums are recomputed. These use synthetic children and establish harness
behavior only. Corrected real native execution and complete-package timing
still require a new hosted run.

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
`lustr-complete-package-gates-v1`:

1. All native client and core unit tests, compiled with the source's `gate`
   profile, one compiler job and one test thread. Cargo's actual test-executable
   artifact records and separate executable digests are preserved. These
   correctness binaries are not the release performance binaries.
2. The unmodified ordinary native differential corpus. An exact Git export of
   that package's native source receives a `target/release` link to its already
   built installed package binaries. The original differential script runs
   every top-level `.rs` case in `tests/pass`, `tests/fail` and `tests/reject`,
   with two workers and no Rust result cache. The complete case count must
   match the recorded source inventory, with zero failures. The replay binds
   the export command, exported root, script/case-pool hashes, fixture inventory
   and actual release client/core identities back to that package.
3. The complete ignored public-entry contract from that source, using its own
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
comparison with pristine Lust. The target remains strict: **both warm modes
must have a paired pristine-Lust/candidate geometric speed ratio whose
conditional bootstrap 95% lower bound is at least 1.0**. An interval crossing
one does not prove equality. A completed target miss exits 2 while preserving
the full result and allowing replay. An incomplete build, gate or campaign
fails without a performance table. No speedup or parity claim may be made
before a real complete run satisfies the rule.

## Execution and replay

The workflow builds and qualifies both packages serially before measurement,
using a 180-minute job allowance for the two full constructors and differential
corpora. It always uploads setup/gate/campaign receipts and captured streams,
including when no timing was admitted. Gate-profile test executables and
constructor binaries are identified by digest; the portable evidence omits
those executable bytes. Source revisions and archived constructor inputs remain
explicit. The complete ephemeral package itself is not a release artifact.

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
