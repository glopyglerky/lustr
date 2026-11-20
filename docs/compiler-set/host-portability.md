# Native host runtime and daemon portability

> The source-bound host findings below remain useful. Its historical mixed
> package instructions and installed full-compiler layout are superseded by the
> [exclusive coverage cleanup](exclusive-coverage-cleanup.md). The current
> partial native constructor is `tools/build-native-toolchain.rs`; removed
> constructor commands below apply only to their recorded source snapshots.

The daemon starts directly as `lust-core daemon`. A client claims its socket
lock, starts a detached process and sends requests over local IPC. It has no
systemd dependency. The foreground entry, daemon and installed Cargo/Rust SDK
still need host services: processes, files, sockets, threads and dynamic loading.

The 18 November dependency audit checked 12,417 active source, build, workflow
and lock files, including all 100 distinct package names recorded by their
lockfiles. It found no service-manager invocation, socket-activation API,
service unit or systemd package. The actual Linux native client, native core,
matching Rust compiler and Cargo loader dependency graphs contain no systemd
library either. There was no such dependency to remove.

`python3 tools/check-host-dependencies.py` now enforces this boundary in native
CI. It checks the tracked and unignored active source tree and lockfiles,
rejects service-manager commands/APIs and unit files, and fails if required
compiler files are absent. Separate controls introduced a process invocation,
a dependency declaration and a service unit; all three were rejected and the
temporary controls were removed. Historical evidence and documentation stay
intact. This is a source/dependency guard, not a claim that text scanning proves
the behavior of every dynamically constructed command or every host library.

The subsequent guard review found a coverage defect, not a runtime dependency:
its directory allowlist omitted `library/lustr`, examples and future packaging
directories. Extensionless entry scripts and several unit kinds also escaped
the scan. The guard now examines source, build recipes and script entry points
throughout the checkout, including the runtime library and root install paths.
It recognizes the complete service-unit suffix set and the bus, event, journal
and login APIs supplied by the same library. Historical records and the guard's
own planted controls remain explicit exclusions.

The [source-bound guard record](evidence/host-dependencies-20261119/verification.json)
reports 12,528 inspected files and no dependency violations at candidate
`28ed726e6a55dd55f98222069c650bd923cc43b2` with the recorded guard revision.
The previous guard missed 25 of the planted controls; the corrected guard passes
all 31 observations across eight tests, including missing-checkout rejection and
preservation of historical records. Both native CI jobs run these controls
before inspecting the actual checkout. This closes the demonstrated scan gap;
it does not claim that a text audit proves every possible dynamic dependency.

Rust's `std` library is distinct from systemd. Removing a service-manager
requirement does not remove the host APIs that this compiler currently uses.

## Erased destructor callback types

The completed [PR29 macOS run](https://github.com/glopyglerky/lustr/actions/runs/44420439581/job/135211656703)
passed all 5,528 ordinary differential cases. Its sanitizer pass completed with
5,481 passes and 47 failures. Every reported failure called a concrete drop
function through `void (*)(void *)` even though its definition took a concrete
type pointer. The strict main-return transition gate passed separately. These
results establish a generated-C defect after the sanitizer setup repair.

The repair gives all generated drop glue the exact C signature
`void drop_T(void *raw)`. Each body restores its concrete pointer before reading
the value. Direct drop calls, object vtables, channel cleanup and unwind
callbacks use the same function type. This changes neither the allocation's
layout nor the Rust drop ordering and adds no extra generated callback call.
The runtime-owned String panic vtable and the thread exception catcher use
small adapters with the exact erased signature. Casting a function pointer
alone would not change the function's C type and would leave the undefined
behavior intact.

The new `tools/test-native-erased-callbacks.py` gate compiles real native C
output with `_Static_assert` checks on its callback declarations. The previous
source fails this gate on the String panic-vtable target. A separate check of
the actual generated `dyn_box_drop` vtable also rejects both old typed targets.
This catches the function-type mismatch on GCC, whose undefined-behavior
sanitizer does not provide the Clang function check used by the macOS gate.

On Linux, all 47 previously failing fixtures passed with the candidate. Across
444 emitted C units, the C compiler accepted 2,647 callback declaration checks,
including repeated declarations. Both the native executables and their
separately compiled GCC versions matched Rust 1.99's output and exit status.
Ten representative fixtures also passed using the generated exception-handling
tables and their linked C++ runtime, covering panic payloads, threads, channel
cleanup and object destruction. This second mode tests the linked C/C++ output,
as the existing upstream EH lane does.

The [verification record](evidence/erased-callbacks-20261118/verification.json)
binds these results to exact native source and compiler identities. These are
type checks and runtime comparisons, not a new local sanitizer or performance
claim. The existing full platform sanitizer and differential gates remain
required, including their stderr comparisons. Unicode alignment and the wider
compiler integration remain separate qualification requirements.

The owned adapter in [`src/host_runtime.rs`](../../src/host_runtime.rs) contains
the host ABI and socket addressing rules. The compiler's retained sessions,
frontend, body-edit reuse, allocator policy and native compiler algorithms stay
in the existing implementation. This is a source-built development candidate;
it does not admit a native macOS release or promise a new speed result.

## Current host boundary

| Component | Linux x86_64 GNU | macOS Arm64 / x86_64 | Windows |
| --- | --- | --- | --- |
| `crates/lustr` Cargo coordinator | Existing route | Existing route | Existing route |
| Primary native entry and daemon | Existing source route, focused host corrections | Host adapter and source package route; native execution qualification required | No primary native host adapter |
| Complete Rust | Matching imported Cargo/Rust SDK | Matching imported Cargo/Rust SDK | Coordinator with an installed matching toolchain |
| Mixed native objects in Cargo | Existing ELF candidate | Current package uses whole-target Lust and matching upstream Rust | No mixed-object qualification |
| Persistent heap images | Existing Linux-only optimization | Disabled; warm daemon sessions remain available | Unavailable |

No `no_std` conversion is needed to remove a service manager. Moving pure
compiler data structures to `core`/`alloc` would be a separate architectural
change. It would not supply process creation, file watching, IPC, threads or
the native loader. No `std` versus `no_std` performance comparison has been
measured. The relevant current speed mechanism is retaining the compiler's
session state between requests.

## Host corrections

### Startup, IPC and cleanup

Short socket paths keep their existing address and add no allocation. On
Linux, an overlong path still uses an open directory and `/proc/self/fd` to
reach the requested socket. All clients now apply that rule, including the
thin upstream client. Startup retains the directory descriptor across exec;
the daemon sets close-on-exec again before starting compiler subprocesses.

Darwin has a 104-byte socket pathname field and no `/proc/self/fd` namespace.
An overlong requested socket is mapped to a deterministic short filename in
`/tmp/lustr-daemons-<effective uid>`. The key covers the canonical requested
directory and filename. The directory must be owned by the effective user,
must be a real directory, and must grant no group/other permissions. A shared
directory or symlink is rejected. The daemon logs the actual bound path, which
is the path to remove when stopping a daemon by unlinking its socket. Normal
short explicit socket paths remain unchanged.

The existing single-instance lock spans client startup and the daemon's entire
lifetime. Lock files remain in place; deleting an active lock file could permit
two daemons. Controlled shutdown removes the bound socket and temporary compile
directory. On hosts without an active inotify watch, idle polling checks socket
removal at most one second later. Requests and signals still wake `poll`
immediately. Linux keeps its normal inotify path.

Process liveness uses a checked positive PID and signal zero. Only `ESRCH`
proves the process gone; permission and unknown errors retain its cache files.
This prevents a macOS sweep from treating every absent `/proc/<pid>` pathname
as a dead process and deleting another live daemon's base library.

### Process and loader ABI

Signal-set representation, mask operations, `WNOWAIT`, `nfds_t` and
`RTLD_LOCAL` have explicit Linux and Darwin definitions. The process owner
still waits without reaping the group leader, kills remaining descendants,
drains inherited pipes and then reaps the leader. Native tests compare the
Rust definitions with values compiled from the host C headers.

The native library loader calls Linux `prctl` only on Linux and uses the
host's dynamic-library filename. Darwin executables reserve the native
compiler's existing main-thread stack budget in their Mach-O link arguments;
the cdylib build does not inherit an argument naming the separate `lust-core`
binary. Linux heap-image syscalls remain in their Linux paths; other hosts
cannot enable the Linux fixed-address arena via an inherited environment value.

### Immutable native base libraries

The Darwin base-library correction is recovered from the prior work at
[`358ead785dafc85f7bc72538a49bff0ab8228e23`](https://github.com/glopyglerky/lustr/commit/358ead785dafc85f7bc72538a49bff0ab8228e23).
Darwin uses `.dylib` and a final `@rpath` install name before the temporary
library is renamed into place. Its immutable name covers ordered object bytes,
compiler identity, session, target, debug setting and session ownership. A
body edit cannot overwrite a library still needed by an earlier executable.
The missing-base recovery path reconstructs the same install name. Linux
keeps its existing ELF name derived from linked bytes.

The restored `compiler/native/scripts/test-base-lifetime.py` checks actual
loader metadata, output against cold Lust and Rust, immutable replacement,
missing-base recovery, retained object reuse, semantic oracles, daemon cleanup
and kept binaries after daemon exit. A native Darwin run is required to
qualify these loader contracts.

## Source package identity

`upstream/bluefe-core.pin` remains unchanged. Linux builds retain its exact
compiler, Cargo, driver and LLVM binary checks. A Darwin source build requires
an explicit `HOST-SDK-RECEIPT.txt` generated from the selected local SDK. The
receipt includes the same pinned Rust source commit, the exact native host,
and SHA256 hashes of rustc, Cargo, the rustc driver and supplied LLVM libraries.
The primary entry's build rechecks those bytes and its actual build compiler.
A changed source commit, host or compiler fails the build.

The receipt records a development input selection; it is not a published
binary pin or a release approval. Native Lust build output supplies the fresh
Lust identity. The owned host adapter is included in that identity. On Darwin,
`tools/build-mixed-toolchain.rs` installs the host `.dylib`, uses the available
SHA256 utility, and installs the matching upstream compiler as `rustc-full`.
It records that there is no mixed Mach-O bridge. Linux keeps the existing mixed
driver and its qualification requirements.

That constructor and its coordinator/Windows installation instructions are
retired. The [original source-bound recipe](https://github.com/glopyglerky/lustr/blob/ea26fe16636d390763257a19c19df3db992cdbcb/docs/compiler-set/host-portability.md) remains
available for historical review. Current product construction follows the
[active correction](active-correction.md) and cannot install a full Rust
compiler as its complement. Historical host receipts and test results below
retain their original scope.

## Focused qualification

The host contracts do not require a compiler SDK:

```sh
LUST_BUILD_HASH=0123456789abcdef rustc --edition=2024 --test tools/host-runtime-qualification.rs -o /absolute/work/host-runtime
/absolute/work/host-runtime --test-threads=1
rustc --edition=2024 --test tools/host-sdk.rs -o /absolute/work/host-sdk
/absolute/work/host-sdk --test-threads=1
```

The first command compiles the actual shared protocol, host adapter and owned
child implementation. It exercises a real long-path request/response, native
C-header ABI values, process-group cleanup, private namespace validation and
the retained protocol tests. The SDK tests reject mismatched source and host
identities and verify the hashing utility.

On the Linux executor, Rust 1.99.1 (`58cf8f9c3`, 2026-10-13) built the locked
native compiler against TinyCC
`43c7708b85681a2fd4451c8a541af4494a8919b2`. The host harness passed 22 test
invocations with the real socket test excluded because of the executor
restriction below; its ignored item is a subprocess fixture. The SDK harness
passed six tests. Ten focused native tests passed: base naming and owner
liveness, base/stale-temp sweeping, removal-event decoding, sparse image
copying/packing, TinyCC external compilation/rejection, huge-page policy and
thread detection. The package builder and primary build script compile.

The full native core and its tests, thin client, host harness and package
builder also pass Rust metadata compilation for `aarch64-apple-darwin`. This
uses the actual locked mimalloc Rust sources and target standard library;
it does not compile or substitute a Mac C runtime. The local compiler used for
these source checks is distinct from the source-pinned SDK required by the
primary package builder.

Local results and limitations are recorded in the integration qualification.
The executor permits child-process tests but denies creation of Unix sockets
with `EPERM`, before any bind or pathname lookup. The real IPC test is retained
as a required native-host test; it is not counted as passed. Darwin Rust
metadata compilation checks source and target types only. It does not execute
Darwin code, link a native Mac package, or qualify TinyCC/Mach-O behavior.
The first hosted native campaign at `17571bcc8242573448a701a7e76d6aced71146a3`
ran on actual macOS 26 arm64. Its host suite passed 23 tests, including native
C-header ABI checks and the long-directory request, its SDK suite passed six,
and the thin client passed 15 tests. The full compiler test run then exposed a
Darwin peer-close check, a Mach-O output-name comparison, and a parser test that
used libtest's small stack rather than the compiler's stack. The Linux campaign
also reproduced that parser-test overflow. The original failure is retained in
[the native gate run](https://github.com/glopyglerky/lustr/actions/runs/44196344927).

The follow-up uses Darwin readable-EOF detection without consuming queued data,
holds the output filename fixed for byte-for-byte signed Mach-O comparison, and
runs the deep parser assertion on the existing production compiler stack. It
does not lower the recursion cap, omit the test, strip code signatures or change
compiler algorithms. Apple ld derives its signing identifier from the output's
install path basename; see
[the primary code-signing implementation](https://github.com/apple-oss-distributions/ld64/blob/main/src/ld/LinkEdit.hpp).
The follow-up at `f510fd9db32d7ad5ab4393b95c9cd4b80d3a4b25` passed all 385 native
compiler tests, 15 client tests, 23 host tests and six SDK tests on actual
macOS 26 arm64. Its release build passed, followed by 5,527 differential cases,
58 macro cases, 13 macro dependency cases and 43 kernel cases. The full gate
then stopped at two client assertions that used Linux `ldd` to inspect a Mach-O
executable. The other 87 client assertions passed. That failure is retained in
[the follow-up native run](https://github.com/glopyglerky/lustr/actions/runs/44197714419).

Client and fuzz base checks now inspect actual native loader metadata: Mach-O
load commands, runtime search paths and the base install name on Darwin, or ELF
dependencies and runtime search paths on Linux. They require the exact daemon
owner, immutable content name and existing base file. A missing inspection tool
or malformed executable cannot pass a standalone assertion. The fuzz gate also
uses the native library suffix and actual Darwin resident-memory observation;
the scenario gate respects Darwin's shorter Unix socket path. Two focused
loader tests pass against actual Linux C-linker artifacts, including rejection
of missing bases, wrong owners, misleading embedded names and missing tools.
Those tests also passed on hosted macOS 26 arm64 at
`9fff6e2c0c4aac3ee086b71e8f6414978bb06a9f`. The remainder of the full compiler gate
is still required, along with package and matched performance campaigns. Earlier
Linux measurements keep their original source identities and are not attributed
to these corrections.

The remaining-stage audit also reproduced a sanitizer-toolchain failure in a
bounded Apple silicon diagnostic. Apple Clang 21.0.0 (`clang-2100.3.34.2`) on
Darwin 27 compiled a clean ASan/UBSan C program, but its runtime aborted when
`detect_leaks=1` was requested. The same program ran with leak detection off.
That second result is diagnostic evidence only; the compiler gate keeps leak
detection enabled. [Upstream LLVM documents macOS leak support](https://clang.llvm.org/docs/AddressSanitizer.html#memory-leak-detection),
which does not establish that Apple's bundled runtime provides it.

`SAN_CC` now selects one sanitizer compiler executable for both the differential
and optional crate sanitizer paths; it defaults to `gcc` and does not change
the primary TinyCC build. The macOS workflow selects upstream LLVM 23.1.6
Clang and its symbolizer as a candidate pending hosted qualification.
`tools/test-native-sanitizers.py` first requires a clean
program to run and actual planted address, signed-overflow and leak faults to
produce their corresponding diagnostics. It fails if any capability is absent;
it does not retry with checks disabled. All four cases passed on actual Apple
silicon with the already installed upstream LLVM 22.1.8 runtime on Darwin 27.
That result qualifies the small capability fixture, not the full compiler gate.
The separately selected LLVM 20.1.8 on hosted macOS 26 reached the 15-second
deadline on the clean case, so that runtime remains unqualified. The same run's
Linux GCC 13 correctly rejected the planted heap error through UBSan before
ASan could report it. The address fixture now allocates through a separate C
translation unit so it specifically demonstrates ASan while retaining both
instrumentations. Its address and undefined-behavior diagnostics pass under
local GCC 14, and all four corrected cases pass under native Darwin LLVM 22.1.8.
The Linux executor's restricted tracing environment prevents LeakSanitizer from
running there. The cause of LLVM 20's observed timeout is unproven.

The earlier LLVM 23.1.5 selection remained unqualified. At head
`0b89794c60a90b2a7d4a1db5e8e78abb6ecdf281`, the
[18 November macOS job](https://github.com/glopyglerky/lustr/actions/runs/44418360296/job/135204490513)
installed Homebrew LLVM 23.1.6, then failed the exact 23.1.5 identity check with
exit status 1. Sanitizer capability tests and the native build were skipped.
The always-run main-return gate subsequently returned status 2 during setup
without a built compiler; no main-return case ran. The workflow now requires
exactly 23.1.6 and reports both expected and acquired versions on a mismatch.
Homebrew formula acquisition still follows its current catalog, so a later
version change fails this check until deliberately reviewed. LLVM 23.1.6 must
pass the unchanged capability probes and full native gate before qualification;
the setup correction supplies no new runtime result.

## Process-test directory ownership

The same admission head's
[macOS Cargo job](https://github.com/glopyglerky/lustr/actions/runs/44418360328/job/135204489920)
failed `cargo_alias_can_select_release_without_enabling_cranelift`. Its actual
rustc command used release optimization and no Cranelift backend option. Cargo
reported that it could not create the fingerprint output file after the other
process test had finished. The job does not prove a profile-selection error.

Both tests allocated an owned directory from only the process ID and a wall-clock
sample, using `create_dir_all`. If two samples match, both owners accept the
same directory, and either owner's cleanup can remove the other's work. A
bounded regression supplied the same sample to two instances of that actual
allocator. It failed on the prior allocation logic because the directories
were identical. This establishes the ownership defect; the hosted log does not
record both original samples, so it does not prove that this caused that one
macOS failure.

The test helper now adds a process-local atomic sequence and uses exclusive
`create_dir`, retrying only an existing-name collision. Each successful owner
therefore reserves its own directory. The regression checks separate paths and
that dropping one owner preserves the other's file. The alias failure message
now reports the failed command without attributing every error to routing.
Production compiler and Cargo behavior are unchanged.

All three process tests passed locally with two test threads: the new ownership
regression, the existing argument/process contract, and the actual Cargo release
alias build and program execution. The [verification record](evidence/macos-harness-20261118/verification.json)
preserves the failing predecessor and successful candidate identities. This is
Linux validation of the harness repair. A new macOS run is still required; no
existing assertion, sanitizer capability or platform gate was removed.
