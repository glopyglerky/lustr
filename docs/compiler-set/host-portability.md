# Native host runtime and daemon portability

The daemon starts directly as `lust-core daemon`. A client claims its socket
lock, starts a detached process and sends requests over local IPC. It has no
systemd dependency. The foreground entry, daemon and installed Cargo/Rust SDK
still need host services: processes, files, sockets, threads and dynamic loading.

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

Build on the destination host, using the SDK whose source commit matches
`upstream/bluefe-core.pin` and the pinned native TinyCC build:

```sh
rustc --edition=2024 tools/build-mixed-toolchain.rs -o /absolute/work/build-toolchain
/absolute/work/build-toolchain /absolute/lustr /absolute/rust-sdk /absolute/tcc /absolute/new-output --release
```

Keep `HOST-SDK-RECEIPT.txt`, `NATIVE-BUILD-RECEIPT.txt`, `COMPONENTS.txt`, logs and
actual native test results with that candidate. Windows users can build the
independently installable coordinator under `crates/lustr`; the primary native
daemon still requires a Windows process/IPC/loader adapter and backend
qualification.

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
Their native Darwin execution and the remainder of the full compiler gate are
still required, along with package and matched performance campaigns. Earlier
Linux measurements keep their original source identities and are not attributed
to these corrections.
