# Native host-script repair evidence

This change connects the existing native compiler to the selected Cargo host
script unit. The two unchanged upstream histories now complete through public
Lustr in the tested cold Linux context. The final follow-up fixes four reproduced
stale-result cases without recompiling unchanged host source bytes. This is a
bounded implementation result under the [exclusive coverage
contract](../../exclusive-coverage-contract.md).

The package is still a partial source-built package. These local runs use fresh
public processes because the local environment refuses Unix socket creation.
They do not qualify daemon retention, the full constructor, all Rust, stock-Lust
speed, overall speedup or release admission.

## Source and package identities

The source base is `62da158b33740279947c85f3fc425df6d1623c9f`. The host-script
implementation depends on the Cargo planning and selected-unit interfaces in
`683bb8ef`. [source-inputs.json](source-inputs.json) records the five owned source
files and their exact hashes. The final native host module is
`bd6109f50094e4f51045f08bfb0973f08678e5f602b596b669d17134dbd931ca`.

The [v2 receipt](controls-and-identities/package-v2-identity.json) binds build
`a5d84827d9e89e28`. The [v3
receipt](controls-and-identities/package-v3-identity.json) binds build
`0390b1143f2d2e93`, its executable hashes and the composed source inputs. The
[514-file source inventory](controls-and-identities/v3-all-repository-inputs-before-build.json)
remained unchanged during the v3 build. Native compilation and public entry
construction have separate command records in `controls-and-identities/`.

Independent oracle commands use SDK
`nightly-2026-11-09-x86_64-unknown-linux-gnu`. Its rustc executable SHA256 is
`ca9d833d09469a534ca1449e71ca9961608409cdb257df96cf8035338fac28de`;
Cargo's is
`da29c2a9ff7f85bda5587e9c974e05c9e2a7196377e54466f6e9d6f9e00b2530`.
The product does not use this independent oracle to compile user inputs.

## Results and their limits

[results.json](results.json) records the counts and exact observed outputs.
Each generation retains its own evidence; the final cache changes do not turn
the earlier v2 runs into v3 qualification.

| Check | Package | Observed result |
| --- | --- | --- |
| Original native host controls | Initial frozen source | 12 passed, 2 failed; original logs retained |
| Native host controls after record/directive correction | v2 | 15 passed |
| Target/profile controls | v2 | 5 passed |
| Cargo planner/source controls | v2 | 8 passed |
| Selected-unit wire control | v2 | 1 passed |
| `cargo10_bs_env_rerun` | v2 | All 7 unchanged versions passed |
| `cargo12_dep_bs_env` | v2 | All 6 unchanged versions passed |
| Same-package and diamond host sharing | v2 | One execution per required run, reuse on unchanged requests |
| Failed script ownership | v2 | One actual execution per logical request, no success record |
| New alternative module and recovery | v2 | Native rejection agrees with independent Rust; removal recovers |
| Directive witnesses | Before and v2 | Malformed forms refused; represented plain value still agrees |
| Final native host controls | v3 | 17 passed |
| Four demonstrated stale-cache cases | v3 | Exact program results agree with independent Cargo/Rust |

For every history version the public entry checks and builds the unchanged
overlaid source, then the generated program and the separately built oracle
program each execute once. Stdout, stderr and exit status must all match.
The scripts use the original donor's overlay ordering, environment operations
and timestamps. `CARGO_BUILD_JOBS=2` matches the hosted job context.

Host executable path, content hash, inode and modification time stay unchanged
through both histories. Changing a trace path or an unread environment variable
does not recompile the host. Declared runtime environment edits rerun it. The
same-package and diamond counters exercise shared host-plan ownership without
reacquiring a lease already held by the request. The deliberately failed script
increments its counter once on each of two distinct requests. These observations
establish the tested reuse and execution behavior, without a timing claim.

The histories' daemon-only poisoned environment remains untested locally. Fresh
hosted retained qualification is still required for that context.

## Cache defects found and corrected

The original v2 cache accepted several different observations as unchanged.
The independent [native host review](../native-host-review-20261119/) preserves
the first two failures and their final controls. This directory preserves both
timestamp failures and their final controls.

| Input change with unchanged source bytes | Original v2 result | Independent Cargo/Rust result | Final v3 result |
| --- | --- | --- | --- |
| Add an empty directory under a watched directory | `0` then `0` | `0` then `1` | `0` then `1` |
| Create a missing watched file containing exactly `<missing>` | `false` then `false` | `false` then `true` | `false` then `true` |
| Change only a watched file's modification time | `1795097626` twice | `1795097626` then `1795097756` | Native and oracle both `1795098487` then `1795098617` |
| Change only the host script source's modification time | `1795097882` twice | `1795097882` then `1795098012` | Native and oracle both `1795098488` then `1795098618` |

The original timestamp commands and streams are in
[timestamps/explicit-v2](timestamps/explicit-v2/) and
[timestamps/implicit-v2](timestamps/implicit-v2/). The
[final timestamp report](timestamps/v3/review.json) binds all 16 final commands,
their exact source hashes and tool identities. Its SHA256 is
`643dcbf767040edbf3a6858e911ff996b7e8b24124e946872fca9d95910497ab`.
Both timestamp controls preserve the host executable's path, hash, inode,
modification time and size. The compiler reuses the checked source bytes and
reruns the script for the changed runtime observation.

Input hashes now distinguish absent paths, regular files and directories.
They include captured empty-directory entries and raw relative path bytes.
File and directory modification times join the already obtained runtime
observations. Equality checks replace the unreliable process-start timestamp
cutoff. Actual host source dependencies have a separate timestamp list, so
runtime invalidation does not force compilation when their bytes are unchanged.
Output checks include directory identities and raw names needed by generated
file consumers; they do not claim to model every filesystem property.

A changed input during execution or a newly declared input without a prior
observation makes that successful output record uncacheable. The current result
can still be consumed under its lease. The implementation never retries the
script within that request to invent a stable record. The unchanged record path
does no host compile or execution. Previous explicit rerun paths are observed
narrowly before execution; a full package snapshot remains a first-use or
default-package-contract cost.

An earlier self-writing-input witness exposed a separate false certificate:
post-run bytes were associated with output derived from old bytes. The final
code preserves pre-run observations. The immediate pinned Cargo witness also
returned the old value twice, so that case is evidence about native record
soundness, not a claimed Cargo/native differential. The independent review
preserves this distinction.

## Environment, directives and unfinished integration

The script compiles once through native `prepare` and emits once from that
prepared state. Its source and expansion observations come from normal native
compilation. Negative module alternatives come from the existing resolver.
No extra Rust frontend or compiler trial determines ownership. Generated output
leases remain alive through consumer checking and emission, including deferred
reads.

The environment witnesses cover empty and active feature sets, caller-supplied
values and positive `CARGO_BUILD_JOBS=2`. `CARGO_CFG_FEATURE` is always present;
active feature variables override caller values, while unrelated or inactive
caller values retain pinned Cargo's behavior. A nonempty inherited jobserver
contract is refused because its file descriptors are not transported. This
patch does not implement jobserver forwarding.

The native lexer validates the custom bare and plain-string cfg forms the
existing config service can represent. Before correction, malformed and builtin
cfg inputs could be accepted, and an escaped value could select a different
branch from Rust. The final witnesses refuse those unrepresented forms
explicitly and preserve `mode="a=b"` correctly. Escaped cfg literals remain
valid Rust with unfinished native integration; their refusal does not make them
complement-owned or establish full directive support. The pinned Rust grammar
is in [`parse_cfg`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_interface/src/interface.rs),
with builtin restrictions in
[`disallow_cfgs`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_session/src/config/cfg.rs).

The justified Linux host configuration additions include x87, ELF, threads,
atomic load/store widths, full Debug formatting, the requested UB-check mode
and pinned target-default relocation metadata. UB instrumentation is still
unfinished. The relocation cfg does not certify PIC or PIE output. Four
reliable-float backend capability flags remain absent and are recorded as
actual environment differences. The comparisons do not establish complete
Cargo environment equivalence or new nightly-language acceptance.

This host path covers the selected Linux development context with no host
build-dependency graph. General host graphs, procedural macros, unrepresented
Cargo configuration, broader directives and other targets remain explicit
integration work. These limits do not transfer Lust-owned work to a stock
compiler. The compiler still uses `std` and the existing retained transport.

## Evidence layout

[raw-evidence.tar.gz](raw-evidence.tar.gz) contains the unchanged donor fixtures,
their loader, capture scripts, history/counter/module reports, directive
before/after streams and the initial temporary trace-assertion failure.
[raw-evidence-index.json](raw-evidence-index.json) lists each archive member,
its hash and original path. Executables, toolchains and rebuildable Cargo/cache
intermediates are excluded. [file-index.json](file-index.json) hashes the direct
evidence files; the index itself and this README are excluded from that list.

The first temporary history/counter scripts expected the trace engine string
`native`. The established public schema uses `lust` or `lust-failed`; the
archived correction changes that assertion only. Original command results and
the initial scripts remain in the archive. Separately, the first native unit
checkpoint exposed two real record-envelope failures. Their unchanged logs
remain in [initial-controls](initial-controls/). Neither kind of failure has
been replaced by its later passing output.
