# Unix socket creation, pathname evaluation and ownership

The declared Rust source creates and owns a Unix socket before evaluating a
caller's `AsRef<Path>` or validating pathname bytes. Native `UnixStream::connect`
and `UnixListener::bind` previously evaluated and validated the path first. When
socket creation fails, those operations have different observable effects and
can return different errors. The preserved long-path witness returns a pathname
error in the old native program and `EPERM` in matching Rust.

The source library now immediately owns a successfully created descriptor.
Pathname evaluation, validation, connect, bind and listen happen inside that
owner's scope. Errors and pathname panics drop it; a successful result transfers
that same owner to the caller. The address-taking methods use the same creation
boundary. The C operations on an existing descriptor return status without
closing it, so ownership is explicit and cleanup cannot occur twice.

The change also follows the pinned listener backlog choices. On macOS, creation
checks `ioctl(FIOCLEX)` and then `SO_NOSIGPIPE`; a setup failure closes the
descriptor while preserving the original error, before user pathname code runs.
These changes are confined to this Unix creation path. They do not establish
that every unrelated networking operation is equivalent to upstream Rust.

## Pinned source

The implementation was checked against Rust source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`:

- [UnixStream](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/std/src/os/unix/net/stream.rs)
- [UnixListener](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/std/src/os/unix/net/listener.rs)
- [Socket initialization](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/std/src/sys/net/connection/socket/unix.rs)
- [File descriptor initialization](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/library/std/src/sys/fd/unix.rs)

## Verification

The [source-bound record](evidence/unix-socket-ownership-20261119/verification.json)
identifies the combined compiler and frozen source library. A deterministic
syscall matrix links the actual generated C and matching Rust program to the
same socket/connect/bind/listen/close stand-ins. The stand-ins allocate real
`/dev/null` descriptors, record calls and inject errors; no network service is
involved. The program checks its descriptor count after each operation.

The matrix covers valid, overlong, NUL-containing and panicking paths against
creation failure, success, connect failure, bind failure and listen failure.
The old compiler disagrees in 19 of 20 scenarios. The repaired compiler matches
status, stdout, stderr and descriptor ownership in all 20 scenarios with native
setjmp unwinding, and again in all 20 with GCC exception tables.

A separate C test compiles the exact Apple helper bytes with simulated syscalls.
All 57 cases pass, including descriptor zero, failures at each initialization
step, close occurring once, and close overwriting `errno`. Deliberately removing
cleanup or returning the overwritten error makes the controls fail. This tests
the helper's algorithm on Linux; it is not an Apple SDK or macOS runtime pass.

The permanent [pathname-order fixture](../../compiler/native/tests/pass/unix_socket_path_evaluation.rs)
and [pathname-panic fixture](../../compiler/native/tests/unwind/unix_socket_path_panic.rs)
run through the ordinary and required unwind differential gates. Full native
platform and sanitizer qualification remain separate requirements.
