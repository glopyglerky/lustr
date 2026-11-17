# Mixed compilation overhead: source diagnosis and first bounded correction

Status: candidate correction, not compiler speed admission. This note does not
claim that a component experiment closes the full-work performance requirement.

## Measured predecessor

PR9 head `048d824ea0289af513b5dc306eb1d4dc6521c539`, actual merge checkout
`fe4704ec6951b8c22c003476886602368810f637`, tree
`5aec709fcd0df39004973a33320492ba82eb4fc1`.
Source-built job in [run 44079784399](https://github.com/glopyglerky/lustr/actions/runs/44079784399)
completed its correctness chain and paired campaign. The whole workflow is not
successful: the macOS Cargo job was cancelled. Do not promote that to a pass.

[Artifact 14328762034](https://github.com/glopyglerky/lustr/actions/runs/44079784399/artifacts/14328762034)
ZIP SHA-256: `43d2125f28fbcae923f66a60457d0db91a6cd9a2685c0c8066b896246654e1a4`.
Original raw SHA-256: `6a4bd2c321a15ef3a7530b9d13ae68733958361590baef137690aecb4f6d20f9`.
Twelve matched pairs per mode, matching original Lust and complete Rust pins.
All samples are retained. Historical campaign records remain unchanged.

| Route / workload | Reference median ms | Candidate median ms | Paired baseline/candidate ratio [95% interval] |
|---|---:|---:|---:|
| Pristine Lust -> integrated, cold | 39.5343 | 39.1960 | 0.9948 [0.9495, 1.0364] |
| Pristine Lust -> integrated, body edit | 5.2886 | 5.0586 | 1.0186 [0.9546, 1.0867] |
| Pristine Lust -> integrated, signature edit | 4.1541 | 4.0534 | 1.0196 [0.9785, 1.0630] |
| Pristine Lust -> packaged entry, body edit | 5.2886 | 5.0410 | 1.0509 [0.9907, 1.1112] |
| Pristine Lust -> packaged entry, rejection | 1.6194 | 17.7204 | 0.0954 [0.0920, 0.0995] |
| Cargo/rustc -> mixed, cold | 143.8765 | 194.9263 | 0.7320 [0.7119, 0.7474] |
| Cargo/rustc -> mixed, body edit | 45.9784 | 76.5471 | 0.6015 [0.5964, 0.6061] |
| Cargo/rustc -> mixed, signature edit | 48.9447 | 88.6385 | 0.5536 [0.5503, 0.5571] |
| Cargo/rustc -> mixed, check body edit | 22.4915 | 24.4247 | 0.9224 [0.9134, 0.9317] |

The native comparisons are small-fixture, generally inconclusive observations,
not an equivalence proof. They nevertheless separate the functioning Lust fast
path from the demonstrated mixed Cargo regressions. The packaged rejection path
may perform full Rust validation after native rejection and is a separate cost.
Native code generation inside a mixed crate still needs phase-level measurement;
standalone native timing is not that measurement.

## Work accounting

For equivalent complete work, write:

    T_mixed = T_reference - S_saved_Rust + C_native + C_bridge

A win requires `C_native + C_bridge <= S_saved_Rust`. Imported implementation
provides semantic completeness; it does not make additional work free. The
latest body-edit median gap is about 30.57 ms; the signature-edit gap is about
39.69 ms. These differences prioritize profiling, not attribution to one call.

For a simplified partition with accelerated fraction `f`, native speed factor
`r`, and normalized integration cost `h`, the ratio is
`T_mixed / T_reference = (1 - f) + f/r + h`. No measured `f`, `r` or `h` is
invented here. The equation explains why speeding up a tiny scalar function can
lose overall when process launches and filesystem work dominate.

Inspection at the predecessor establishes:

- Cold/signature-miss requests run complete Rust analysis, then send selected
  scalar source units through the native checker/generator. Each generated object
  also has an `nm` validation and `objcopy` symbol-rewrite process.
- Certified body edits retain checked Rust objects and the Lust session, but
  still refresh Rust expansion, compare semantic dependencies, run changed-body
  Rust queries, generate the native object and relink. The observed trace says
  `reused-rust-objects=1 full-analysis=0`; that is not zero Rust work.
- `mixed_objects::relink` copies retained object inputs into a disposable link
  directory. It previously reused `retain_file`, which calls `sync_all` on
  every copied object even though those copies are never published as cache
  generations. This is a concrete extra operation, not a measured attribution
  of the entire 30.57 ms gap.

## First correction: separate durable retention from disposable staging

Only temporary linker-input staging switches to `mixed_link_input::stage`.
All persistent Rust-object copies, checked link recipes, native edited objects,
directory synchronization and final-success certificate publication retain the
existing durability operations. The stale-object/link-failure gates are unchanged.
No native Lust source, Rust pin, ABI decision, metadata encoder, panic policy,
compiler selection or performance threshold changes.

If a relink has `N` real input paths, it no longer issues `N` synchronous data
flushes for the disposable copies. The same bytes still reach the linker. The
removed serial work is the sum of those flush costs, not a promised universal
wall-time improvement. Cold analysis/cache-publication costs are not removed by
this patch, and cold/signature parity is not claimed.

The recovery invariant is: a persistent generation `G` is immutable and remains
the only retained source; scratch copies `S` are used only by the current link.
A successful link consumes `S` through ordinary filesystem visibility. Failure,
interruption or machine loss discards `S`; the next request reconstructs it from
`G`. No future certificate points into `S`. Therefore durable publication of `G`
is required, while durable publication of `S` is not. This is a scoped reasoning
argument, not a formal proof of the entire compiler or a physical power-cut test.

Do not replace these copies with plain hard links: the user can configure a
linker that changes or removes an input. In-place writes through a shared inode
would corrupt the retained generation. Preserve the independent-byte boundary.
A future copy-on-write/reflink optimization needs actual platform qualification.

Eight focused tests exercise the actual helper: absent input, duplicate
basenames, independent bytes/inodes, linker writes, cleanup, missing-input and
missing-directory failures, empty files, and an abandoned-link/retry scenario.
The whole compiler still requires the existing source-built correctness and
complete-work benchmark campaign. Authored tests are not passed tests.

## Evidence-producing validation

The existing focused workflow retains its eighteen benchmark-evidence tests
and adds the actual staging-helper tests. It then runs
`tools/measure-mixed-link-input.rs` with optimized code, 24 balanced randomized
matched pairs, sixteen 64-KiB inputs per lane, independent scratch directories,
and source/compiler/binary identity. Both lanes must preserve every source byte.
All individual times are emitted; neither winning-mode selection nor a speed
admission decision is made.

The predecessor lane uses the exact former `retain_file` copy-and-sync policy;
the candidate calls the production helper. This isolates the copy policy only.
It is not a representative object-size census, a compiler benchmark, a cold-page-
cache experiment or evidence of end-to-end rustc parity. The existing full
Compiler set workflow remains authoritative for actual compiler behavior and
reports complete-work regressions without turning them into speed admission.
Native CI now queues a newer head rather than cancelling an already running
qualification. Its commands, test selection and failure handling are unchanged.

## Upstream mechanisms and next steps

The bounded source review uses Lust's original retained-session/body-dependency
mechanism and the pinned Rust implementation, rather than rebuilding semantics.
Upstream Lust [README at 240e08d9](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md)
separates warm edit work, cold work, memory and generated-program runtime. Its
headline ratios belong to its own adapted workload and cannot qualify Lustr.
The [Rust self-profile documentation](https://doc.rust-lang.org/unstable-book/compiler-flags/self-profile.html)
and [rustc-perf profiling guide](https://rustc-dev-guide.rust-lang.org/profiling/with-rustc-perf.html)
provide phase and matched-comparison methods. Rust's
[`link_or_copy`](https://doc.rust-lang.org/stable/nightly-rustc/rustc_fs_util/fn.link_or_copy.html)
and [`hard_link` contract](https://doc.rust-lang.org/std/fs/fn.hard_link.html)
were considered; their aliasing behavior is not appropriate for this boundary
without additional guarantees about downstream writes.

Next isolate native compilation, process launches, expansion/context hashing,
copy/publication and final linking on the real mixed graph. Profiling must be
outside clean timing. In particular, blindly adding `-Zself-profile` currently
makes `represented_request` decline mixed compilation, so that would profile the
wrong route. Use external observation or explicitly qualify the profiler option
without removing the actual mixed-path evidence check.

After profiling, remove measured redundant work: emit the checked final symbol
without a separate object rewrite where that can be proved equivalent; batch
object validation where appropriate; reuse semantic dependency results rather
than rescanning unaffected work. Retain diagnostic/macro invalidation and durable
publication. Avoid speculative cache reuse, disabling checks, or routing the
whole mixed corpus through rustc merely to improve a headline.

Preserve the whole-target Lust fast path. Prove its preservation against plain
Lust separately from full-Rust integration costs, with representative long edit
sessions as well as this small fixture. Broad coverage, macOS qualification,
independent review and release admission remain open.
