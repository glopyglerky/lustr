# Native target layout correction

The native compiler previously used Linux synchronization layouts even when
the selected target was Darwin or Windows. A check could therefore reject
valid target-specific constant assertions or accept the wrong constants.
This affects `Mutex`, `RwLock`, `OnceLock`, `Condvar`, `Barrier` and enclosing
types such as `Option`.

The correction reuses the layout and main-stack work first recorded in
[`cfc37793`](https://github.com/glopyglerky/lustr/commit/cfc37793590d3c4bef4069447ce43f9abed366a7)
and the Windows `SmallFutex` correction in
[`358ead78`](https://github.com/glopyglerky/lustr/commit/358ead785dafc85f7bc72538a49bff0ab8228e23).
It is adapted to the current compiler's existing resolved root configuration.
It does not import that earlier branch's routing or weaken its failing gates.

## Target identity and reuse

Collection selects the synchronization model from the root crate's resolved
`target_os` before evaluating initializers or function bodies. The type store
and its body overlays carry that selection. A type pool cannot be reused for
a different model, and the generic-body reuse key includes the selection.
The existing whole-request target identity remains in place.

Darwin uses pointer-width state for these locks and `Once`. The selected
Windows targets use a one-byte `SmallFutex` for `Mutex`; their other affected
futex types retain four-byte state. Linux keeps its existing answers. The
model still targets the represented 64-bit layouts; this change does not
qualify arbitrary architectures or standard-library versions.

The Darwin compiler executable also reserves the existing 256 MiB compiler
stack budget through its Mach-O linker setting. Raising `RLIMIT_STACK` alone
did not prevent the previously observed deep-expression stack failures on
Darwin. Other hosts receive no new linker setting. Native macOS execution
must qualify this change separately.

## Independent frontend qualification

`tools/native-target-layout-qualification.rs` asks the supplied Rust compiler
to emit LLVM constants for thirteen standard-library types on each selected
target. It extracts twenty-six size and alignment answers and checks the same
constant assertions with both that compiler and native Lust. It also requires
both compilers to reject a deliberately false assertion with a semantic
diagnostic. A crash, timeout, missing SDK, or unsupported type does not pass.

The comparison uses the matching `nightly-2026-11-09` SDK, revision
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. It does not assume that earlier
Rust 1.99 layout measurements apply to that compiler. The tool retains Rust's
identity, LLVM output, extracted answers, generated assertions and each
compiler's diagnostic output and exit status.

```sh
rustup target add --toolchain nightly-2026-11-09 \
  x86_64-apple-darwin aarch64-apple-darwin x86_64-pc-windows-gnu
rustc --edition=2024 tools/native-target-layout-qualification.rs \
  -o /absolute/build/native-target-layout-qualification
/absolute/build/native-target-layout-qualification \
  /absolute/candidate/bin/lust \
  /absolute/matching-sdk/bin/rustc \
  /absolute/new-evidence-directory \
  x86_64-unknown-linux-gnu x86_64-apple-darwin \
  aarch64-apple-darwin x86_64-pc-windows-gnu
```

The unchanged PR7 compiler at `2add988` passes the selected Linux constants
and fails both selected Darwin targets and Windows GNU. With this correction,
the diagnostic native build passes all twenty-six assertions and rejects the
deliberately wrong assertion on each of the four targets. The selected native
unit test also passes all nine transitions among the three layout models:
same-model pools and body overlays preserve their selection, and different
models discard the pool. These frontend
comparisons do not link or execute a foreign program. Native macOS CI remains
required, including the original differential, daemon and runtime gates.

The native CI workflow now retains source, platform and compiler identities
and the complete original gate log on both success and failure. Its tests,
runner labels and failing exit statuses remain unchanged. The recent empty
macOS jobs failed to acquire a hosted runner under ARM capacity constraints;
they did not execute compiler tests.
