# Native Unix socket bounds and differential portability

## Observed failures

At PR9 head `afb17a8af2cce9facdf03ba6cf12b8533cb3a171`, the
[macOS ARM64 native job](https://github.com/glopyglerky/lustr/actions/runs/44063047989/job/133969743022)
built Lust and passed 5,522 differential cases. Four cases failed:

| Fixture | Observed failure | Correction |
| --- | --- | --- |
| `io_is_terminal_file_ext` | The Rust 1.99.0 oracle cannot import `std::os::linux::net`. | Compile the abstract-socket sections only on Linux. Keep terminal, Unix pathname, and positional file-I/O checks on macOS. |
| `std_bytestr_debug_forms` | The same Linux-only import fails before portable command debugging runs. | Gate only abstract-socket imports and expressions. Keep command, argument, environment, and invalid-byte formatting checks. |
| `std_unix_net` | The same Linux-only import fails before portable Unix socket operations run. | Gate abstract names and their bind/connect path. Keep socket pairs, pathname bind/connect/accept, shutdown, addresses, and errors. |
| `runtime_inline_helpers` | Exact stdout differs for min/max of opposite signed zeros. | Assert each result is one of the two allowed input bit patterns, then use a common printable zero for those ambiguous pairs only. |

The original failed run remains evidence. No case is removed from the native
suite and no failure is converted into success by its runner. The Linux-only
operations still run on Linux. macOS runs every portable operation in the three
socket fixtures.

## Floating-point contract

The exact CI oracle's upstream source documents that `min` and `max` may return
either input when the operands compare equal, including opposite signed zeros:

- [Rust 1.99.0 `f64` source](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/num/f64.rs)
- [Rust 1.99.0 `f32` source](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/num/f32.rs)
- [Rust `f64::min` API](https://doc.rust-lang.org/std/primitive.f64.html#method.min)
- [Rust `f32::min` API](https://doc.rust-lang.org/std/primitive.f32.html#method.min)

The fixture still executes both min and max for every original input pair and
both widths. For an opposite-zero pair, it checks the actual result bits before
normalizing output. Nonzero values, infinities, NaN cases, and equal-sign zero
pairs keep their original exact differential output comparison. This does not
change the compiler's numerical policy, replace its runtime helpers, or mask
signed-zero behavior elsewhere.

The abstract socket API is explicitly platform-specific in the
[Rust Linux extension trait](https://doc.rust-lang.org/std/os/linux/net/trait.SocketAddrExt.html).
The tests now use that same `target_os = "linux"` boundary. The portable
[Unix socket API](https://doc.rust-lang.org/std/os/unix/net/index.html) remains
exercised on both native CI hosts.

## Source identity and validation scope

Checking the previously blocked Unix fixtures also exposed a production defect.
The source library allowed up to 107 pathname bytes on every host. Apple's
[`sockaddr_un` declaration](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/un.h)
has only 104 path bytes, including the terminator. The C adapter trusted the
source-library bound and copied the requested bytes into that smaller buffer.
The exact Rust 1.99.0
[`sockaddr_un` construction and decoding](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/os/unix/net/addr.rs)
uses the target's actual capacity, rejects interior NUL bytes, identifies empty
pathnames as unnamed, and treats padded Darwin addresses as unnamed when their
first path byte is zero.

The corrected source library uses 104 bytes on macOS and preserves 108 on Linux.
Its byte validation also rejects interior NULs with Rust's original error text.
Empty pathname addresses now have the original unnamed classification. The C
adapter independently checks `sizeof(sun_path)` before copying or allocating a
socket, so a mismatched or malformed caller cannot overflow that buffer. Address
decoding bounds reads by the actual buffer and handles padded Darwin unnamed
addresses. Linux empty abstract names remain named even though their payload
has zero bytes. Both syscall output structures are zero-initialized.

The new `std_unix_path_bounds` differential fixture exercises actual maximum
pathname bind/connect/accept round trips for ASCII and multibyte UTF-8, every
length from 103 through 109 bytes, oversize constructor/bind/connect rejection,
NUL rejection, empty pathname classification, and a Linux empty-abstract-name
round trip. These are platform-specific assertions against the original Rust
oracle, not unconditional expected-output substitutions.

`tools/test-native-unix-address.py` reads the actual C helper bodies. It runs
AddressSanitizer and UndefinedBehaviorSanitizer checks using the host layout and
a separate Apple 104-byte layout model. The model tests memory boundaries and
record decoding; it does not simulate Darwin syscalls or establish native macOS
execution. Both native CI jobs run this check before the complete inherited gate.

Local checks at this correction:

- The original helper from `048d824ea0289af513b5dc306eb1d4dc6521c539` reproduces
  `AddressSanitizer: stack-buffer-overflow` at `memcpy` under the Darwin model,
  copying 107 bytes into its 104-byte path. The original source remains in Git.
- Both corrected C models pass ASan/UBSan, including capacities, no writes on
  rejection, zero-length handling, padded addresses, and bounded output reads.
  The hosted workspace prevents LeakSanitizer operation under tracing; local
  runs use `ASAN_OPTIONS=detect_leaks=0`. These stack-only checks do not claim
  heap-leak qualification. CI uses its normal sanitizer settings.
- All five affected Rust fixtures compile with the actual Rust 1.99.0 oracle.
  The terminal/FileExt, ByteStr, and float fixtures execute with exit zero.
- The environment rejects a Unix socket bind with `EPERM`. The socket fixtures'
  complete execution is therefore reserved for native CI, not counted locally.

The macOS job also installs Homebrew's
[`flock` formula](https://github.com/Homebrew/homebrew-core/blob/main/Formula/f/flock.rb).
The original client and sanitizer suites already require it. The job verifies
file-descriptor contention and release before running those suites; no lock
checks are disabled.

The full pristine-to-integrated patch and its declared Git object identities are
regenerated for the runtime, source-library and fixture corrections. The
pristine reference remains `240e08d91d4aed0299639f6f11fd522912b248f9`.
Qualification remains `candidate`.

The complete native Linux and macOS CI gates must execute the corrected fixtures
and all later suites before native platform qualification is complete. A source
review or successful Rust-oracle compile alone cannot establish Lust execution.
The earlier paired measurements remain evidence for their recorded revisions;
this correction supplies no new timing result or release admission. The added
constant bound check is not presumed free; the ordinary paired campaign remains
required for the new compiler identity.
