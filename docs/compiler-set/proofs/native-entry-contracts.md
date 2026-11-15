# Native entry selection and lifecycle contracts

These are conditional program arguments, with executable regression checks.
They do not prove full Rust coverage, compiler equivalence, memory safety or
performance. The protected upstream compiler implementations are unchanged.

## Target selection

Assume trusted Cargo metadata and no concurrent mutation of the compilation
inputs. Let a target identity be `(package ID, target name, target kind, source)`.
For a request, let `F` be the canonical source path, `K` its target kind, `N` its
explicit target name when present, and `P` the result of Cargo's package-spec
resolver when a package is specified. Define

```text
C = { (p,t) in metadata : canonical(t.source) = F
                         and t.kind matches K
                         and (N absent or t.name = N)
                         and (P absent or p.id = P) }.
```

The entry rejects unless `|C| = 1`. Consequently a nonexistent explicit target
or a mismatched explicit package/source cannot be replaced by an available
target. The selected member manifest and target name bind the native request.
Fallback uses the same package ID and target, then accepts a compiler-artifact
message only when all four identity fields match. A successful Cargo exit alone
cannot authorize copying another target's artifact. This is an identity proof,
not a proof of the artifact's semantics or source immutability.

Selectors are read only from option positions. Values consumed by other options
cannot become selectors. Cargo owns package-spec syntax and dependency/lockfile
resolution; synthetic Lust package strings are not parsed as Cargo identities.

## Foreground child ownership

Assume Linux x86_64 process/signal semantics, the entry's one active foreground
command, functioning system calls, and descendants that remain in its process
group. The child starts a new process group whose ID is its PID. While the
leader is alive or held by `waitid(WNOWAIT)`, its PID cannot be reused. Cleanup
therefore addresses the owned group before `wait` reaps the leader, preventing
a later process group from being mistaken for that command. Error-path guard
drop kills the group before waiting for its still-unreaped leader.

The exit observer waits concurrently with stdout/stderr reading and Cargo's
JSON loop. Once the leader exits, it kills the group while retaining that leader
with `WNOWAIT`. A same-group descendant holding either output pipe therefore
cannot delay cleanup until EOF: cleanup no longer depends on draining that
pipe. The owner joins the observer before reaping, including on error paths.
This assumes successful observer creation/system calls and eventual kernel
process termination; creation failure drops the guard and kills the child.

Spawn blocks cancellation signals on the spawning thread. Other native worker
threads can still receive signals. The handler first stores a pending signal,
then reads the group; spawn first stores the new group, then exchanges pending
with zero. All four operations are sequentially consistent. If the handler
reads the transition sentinel, its pending store precedes spawn's exchange;
spawn observes cancellation. Otherwise the handler observes the published group
and cancels it directly. Thus cancellation is not lost in the group-publication
window. The handler uses only lock-free atomics and async-signal-safe C calls.

Caught SIGINT/SIGTERM/SIGHUP/SIGQUIT kill that group before restoring the signal's
default disposition and re-raising it. Explicitly ignored signals are preserved.
The tests exercise the ordinary SIGINT/SIGTERM cases by PID and process group.
They do not qualify SIGKILL, custom caller signal handlers, escaping sessions,
all possible signal interleavings, or termination of the retained Lust daemon.

## Artifact publication

For a copied artifact whose destination differs from Cargo's output, assume
POSIX atomic-rename semantics. Before rename, all copying and permission updates
address a newly created sibling file; the destination is unchanged. Rename is
the single publication point. After it, the destination names the complete
copied artifact. A signal during copying can leave a temporary sibling, but
cannot expose a partially copied destination. This does not assert durability
after a crash, source-file immutability, or ownership of Cargo's output files.

## Checks and performance obligation

`tests/compiler_contract.rs` contains opt-in actual-runtime regressions for
selection, cancellation, native/full routing and artifact contracts. Set
`LUSTR_TEST_PACKAGE` to the candidate and `LUSTR_TEST_OUTPUT` to a new directory,
then explicitly enable the ignored tests. Root unit tests also check selector
value boundaries and retained cancellation during group creation. A normal
test invocation that skips those integration tests does not qualify a package.

If an adapter saves `S` units of compiler work and adds `H` units, then
`T_new = T_original - S + H`. Non-regression requires `S >= H`. Neither source
equality nor correct dispatch proves that inequality. Target preflight, process
supervision and fallback attempts must be included in equivalent-workload
measurements, with regressions retained. No universal speed claim follows from
these proofs or from finite fixture tests.
