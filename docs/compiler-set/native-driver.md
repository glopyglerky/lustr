# Native compiler entry candidate

> Historical implementation description for default snapshot
> [`7cea76c9`](https://github.com/glopyglerky/lustr/tree/7cea76c9d3adc587b54ccff168fe07efafd40309).
> The retries and whole-program `rustc-full` routes described below are
> transitional behavior that must be removed from the completed product.
> They are not the required architecture. Follow the [architecture plan](architecture-plan.md),
> [current design checkpoint](design.md) and [completion checklist](completion-checklist.md).
> Later candidate qualification is recorded separately by exact source.

Status: source candidate, not a released or fully qualified replacement.

The primary Cargo project and command entry point are at the repository root.
`src/main.rs` accepts rustc requests and native whole-file `build`/`check`
commands. `src/lib.rs` recognizes the request and artifact contracts representable
by unchanged Lust. The compiler does not select languages by source keywords.

## Execution

1. Representable native requests use the unchanged Lust protocol and daemon.
   Lustr starts the original daemon synchronously for its first request, avoiding
   a separate cold compile followed by duplicate prewarming.
2. A single-file cold request can enter unchanged Lust modules through
   `crates/lustr-native-core`. This adds a C ABI outside the protected source.
   Arguments remain borrowed for the call; diagnostic bytes are copied through
   a C callback. Rust-owned allocations never cross that ABI. The entry permits
   one call per process. It is not a general in-process compiler service.
   Nondefault transparent-huge-page policies and front-image overrides retain
   the unchanged executable path. The library caller restores its original
   stack limit before any full-core fallback.
3. Other rustc requests use the unchanged full Rust executable, `rustc-full`,
   through `exec`. This preserves its startup TLS requirements. A late-loaded
   compiler-library experiment failed that requirement and is excluded.

The thin entry does not link Rust's compiler library or LLVM. The native path
does not load them. Full Rust compatibility comes from the separately identified
upstream Rust core; it is not newly implemented native Lust coverage.

The C ABI library uses PIC-built TinyCC from unchanged pinned TinyCC source.
Its cache namespace combines the installed library SHA256 with the pristine
native runtime-family manifest SHA256. Lustr adds that namespace to its native
request identity, without altering source bytes or the caller's environment.
Upstream cache keys also hash the actual TinyCC runtime. Packaging must generate
`lib/lustr-native.id` from those exact frozen inputs and verify it on extraction.

Cargo-backed Lust requests use the pinned Cargo's compatibility layout setting
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1`. It is carried in the request
because build-script execution restores the client's environment. An explicit
conflicting setting uses the full Rust path. No upstream Cargo or Lust patch is
applied. Build scripts and procedural macros remain real Cargo work.

Manifest requests first resolve their package and target through Cargo metadata.
Explicit package specifications use Cargo's own package-ID resolver, including
workspace-inherited versions. Name, kind and canonical source must agree on one
target. The validated member manifest and target are passed to native Lust;
fallback reuses the same selection and checks the artifact's complete identity.
This preflight can add manifest-request work. Its cost belongs in measurements;
it is not a speed improvement by itself. Single-file requests need no preflight.

Foreground Cargo and captured native subprocesses have owned process groups.
On SIGINT, SIGTERM, SIGHUP or SIGQUIT, Lustr kills its active group and terminates
with the original signal; explicitly ignored signals stay ignored. Errors drop
the child guard, which kills the group and waits for its leader. Normal completion
also closes remaining work in that group before reaping the leader.
An exit observer runs alongside output reading, so a descendant retaining a pipe
cannot prevent cleanup after its leader exits. These
guarantees exclude SIGKILL, deliberately detached descendants and the separately
managed upstream daemon. They do not imply an arbitrary process-tree sandbox.
Successful copied Cargo artifacts use a temporary sibling and an atomic rename;
an interrupted copy cannot publish a partial destination. Forced termination
can leave that temporary file. No crash-durability guarantee is made.

## Contracts and limits

### Native source-library additions

After an ordinary native rejection, diagnostic bytes may suggest a library
retry. They never establish acceptance: unchanged Lust checks the entire
original program with `library/lustr`. The owned library shares original
platform modules by path and supplies Linux executable/shared-library naming
constants, `From<&OsStr>`, `From<&OsString>` and `From<&PathBuf>` for `PathBuf`,
`OsString::reserve`/`reserve_exact` and `PathBuf::reserve`/`reserve_exact`.
The capacity methods restore the
original owned buffer during unwinding. No application source is rewritten.

A stable installed component can retain an ordinary pristine Lust daemon.
Its core is a real hard link to the original core, preserving upstream's actual
device/inode/mtime wire identity. Its executable path selects a separate default
support prefix. Both support links must have their expected relative targets.
The manifest covers the complete owned and shared library source inventory and
the native runtime-family identity file. A successful content check and unchanged
manifest precede reuse; the full manifest digest joins the request's package
identity and a distinct socket key. Added, changed or unsealed sources and an
unsuitable component inode take the explicit cold source-library path. User
source/front overrides do not enter automatic retention.

This assumes a trusted, stable installation throughout a request. It is not a
source snapshot or protection against concurrent mutation. The identity file
does not authenticate the whole runtime by itself; extraction and package
verification remain separate obligations. Archive extraction must preserve the
core hard link. A copying tool that splits it deliberately loses retention.

Successful additions trace `lustr-native`; an expanded rejection and subsequent
compatibility result remain distinct. Runtime/rejection tests compare original
programs with the full Rust oracle, including non-Unicode values, capacity
overflow/unwind, source reseal/restore, added inputs, wrong core/support links
and caller macro environment. Canonical `type_name` involving the nested env
implementation still needs the full core. This is bounded library coverage,
not a full native language or crate interface.

The candidate's nine-sample single-file campaign retains both improvements in
unchanged daemon requests and regressions in cold compilation, edits and
rejection. Those measurements do not establish Cargo no-op or full-project
speed. The current native host is Linux x86_64 GNU on AerynOS 2026.08 with an
observed GLIBC 2.39 floor. The complete ELF audit also records libgcc, zlib and
both `/usr/lib/ld-linux-x86-64.so.2` and `/lib64/ld-linux-x86-64.so.2` interpreter
paths. Those host facilities remain dependencies. Wasm qualification belongs
to full-core linking, not native execution.

Native Lust has no Rust rlib/rmeta emitter. Requests requiring those artifacts,
Rust-compatible structured diagnostics, unrepresented flags or other targets
stay with the full core. Do not forge metadata or dep-info. Native whole-target
compilation and Cargo's per-crate artifact graph remain different interfaces.

The automatic native command interface currently covers `build FILE.rs` and
`check FILE.rs`. A native compiler rejection can fall back before any application
or test executes. Compiler crashes and other failures remain failures. No
executed application is retried. Other Cargo commands retain the separate
compatibility integration; this is not full native coverage of Cargo workspaces.

The full Rust component is an unmodified upstream distribution at source
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`, not a newly bootstrapped blueFe
compiler. [`upstream/bluefe-core.pin`](../../upstream/bluefe-core.pin) binds
compiler, Cargo, driver, LLVM and native binary inputs. Compiler-private metadata
used by withdrawn experiments is not a runtime requirement.

## Qualification

`tools/native-relink.rs` provides an offline recipient rebuild recipe for a
source kit, a matching reference runtime and a new work directory. Its separate
`prepare` and `build` steps preserve recipient TinyCC edits and regenerate all
native consumers, the native pin, runtime identity, source seal and hardlink.
The fixed host build clears caller target settings and rejects discovered
directory configuration and release-profile environment overrides. Its build
completion digest is invalidated before a new attempt; runtime qualification
is still separate. It does not install globally or provide atomic update,
rollback, arbitrary TinyCC correctness, relocation or privacy guarantees.

`tests/compiler_contract.rs` exercises the actual assembled runtime. It requires
explicit `LUSTR_TEST_PACKAGE` and a new `LUSTR_TEST_OUTPUT` directory, including
paths with spaces. Its opt-in tests cover native cold/daemon/edit requests,
full-language fallback, genuine metadata, structured rejection, build-script
reuse, proc macros, doctests, panic/Drop and Wasm linking. The packaged SDK tools
also exercise rustfmt reject/format/pass, Cargo fmt, direct Clippy and Cargo
Clippy rejection with an explicitly enabled boolean lint, and a valid Clippy
check. Ignored tests have not run merely because ordinary `cargo test` succeeds.

Selection regressions cover a nonexistent bin, multiple bins sharing a source,
the wrong package or source, inherited workspace versions and Cargo package
specifications. Cancellation fixtures stop the wrapper by PID and by process
group, and observe Cargo, a build script and its child. See the
[conditional selection and lifecycle proofs](proofs/native-entry-contracts.md).

Every artifact still needs frozen-byte independent review, relocation, notices,
privacy checks and equivalent-workload speed evidence. The preliminary process
dispatch experiment retained a regression; the revised entry's pilot is not
campaign acceptance. These checks do not prove whole-stack memory safety,
universal compiler correctness or a never-slower bound for all Rust programs.
The [full native compiler contract](native-contract.md) remains unfinished.

Work stays in the user's Lustr/blueFe forks. Both upstream compiler source trees
remain protected. See [Lust preservation](upstream-lust.md) and
[blueFe's Rust preservation policy](https://github.com/glopyglerky/blueFe/blob/main/docs/bluefe/upstream-preservation.md).
