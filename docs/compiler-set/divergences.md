# Fork ownership and divergences

Lustr and BlueFe are projects with their own entry points, source maps and design
decisions. Required upstream implementation remains where it is needed for a
working compiler and for reliable updates. Inherited branding and historical
research are identified rather than pasted into the primary project README.

## Lustr

The root Cargo workspace builds the compiler entry in `src/`. It uses
`serde_json`, a bounded native C ABI loader and the separately pinned full Rust
executable. The compatibility coordinator in `crates/lustr` remains independent,
with no registry or C build dependencies and Rust source that forbids unsafe
code. `crates/lustr-native-core` wraps unchanged native modules outside the
protected tree. Owned library additions live in `library/lustr`; explicit links
share original source instead of changing the protected baseline.

`crates/lustr-semantic` remains independent because its rustc-private interface
requires a different pinned development toolchain. It is not part of the normal
BlueFe application build. The stock GitHub workflows remain at `.github` because
GitHub discovers workflows there; native steps select their component directory.

The native package is named `lustr-native`. Its existing `lust`/`lust-core`
commands and protocol names remain compatibility interfaces. The optional pinned
ts-rust source is a native qualification fixture, not another product entry point.
Upstream licenses and source history remain intact.

Current owned changes include compiler request routing, Cargo target identity,
bounded foreground cleanup, source-library APIs and separately identified
retained library sessions. Compatibility backend selection and its qualification
remain separate. Earlier Lust kernel patches and experiments are historical;
they are not admitted to the exact active upstream tree. Correctness alone does
not admit a protected kernel divergence: necessity and faster equivalent-workload
evidence are both required by the preservation policy.

## BlueFe

BlueFe retains exact pinned upstream Rust compiler, library, bootstrap and
stage0 source under its preservation guard. Its owned integration pins Lustr
and records dependency boundaries. The compiler candidate currently ships an
official unmodified SDK, not a newly bootstrapped fork-produced Rust compiler.
That dependency supplies full Rust compatibility while native replacement is
unfinished.

Old compiler/bootstrap overrides and heuristic stage0 selection were removed
from the active fork. Optional Cranelift research does not implement native
Lustr coverage and retains its separate unwind/drop and ABI obligations.

## Iteration policy

Make the working path usable before replacing compiler internals. Every inherited
component needs an identified role, preserved license, version boundary and a
reason to remain. Remove or replace it when equivalent functionality and focused
qualification exist. Keep source movement separate from semantic changes in
review and report any changed build or installation contracts.

Full native Rust support, persistent full-Rust analysis, native rlib publishing,
and migration toward memory-safe Rust remain future goals. TinyCC and LLVM are
current external code-generation dependencies. Safe Rust at a command boundary
does not prove their implementation safe. Track migrations and validate behavior
as they land; do not use the future goal to block a working toolchain.

The same ownership and evidence policy applies when working on the user's other
forks. Public repositories can supply frozen performance workloads without
changing their source, default branches, services or deployment state.

## Historical native retirement

An earlier fork revision removed `compiler/native/spikes`. Exact upstream
preservation subsequently restored the complete tree, including those probes.
They remain historical upstream research, not native language coverage or
current package acceptance. The earlier fork state remains in
[the prior source snapshot](https://github.com/glopyglerky/lustr/tree/39349b63b976e76702ca14fbb5f19718eaf0ed2d/compiler/native/spikes).

Required non-Rust code remains explicit: the current native compiler's C runtime
and generation path, the TinyCC interface, C ABI qualification fixtures, and
build/test/research helpers used by current development. Rust source, licenses
and the relevant compiler fixtures are retained. These exceptions preserve the
working toolchain while the native Rust migration remains future work.

## Matched compiler configuration study

The [CGU16/ThinLTO study](measurements/2026-11-09-cgu16-thinlto-v3/README.md)
compares stage2 compiler packages from the same source and Cargo. It changes
compiler CGU256/off-LTO to CGU16/ThinLTO, retaining std CGU256 and project settings.
Preparation and native build effects belong to the joint treatment. Lustr's
default routing and frontend settings remain unchanged. The numerical decision
passed for three metadata-only roots; stock Rust remained faster on those cases.
Full-build performance and individual knob effects require separate experiments.
The [study proofs](proofs/cgu16-thinlto.md) distinguish deterministic arithmetic
properties from statistical assumptions and compiler-correctness evidence.
