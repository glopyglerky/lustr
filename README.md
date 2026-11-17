# lustr

lustr combines [Lust](https://github.com/t3dotgg/lust-lang)'s fast native
compilation of its supported subset with complete Rust compilation through a
pinned Rust implementation. blueFe delivers the combined toolchain.

## Numbers and current qualification

**Not yet release-ready. Cargo acceleration is the main performance priority.**
The latest completed mixed Cargo campaign is slower in eight of nine modes,
including builds, edits, checks and tests. The supported native path has warm
speed gains against standalone rustc on a small corpus; those results do not
establish faster Cargo work. Native macOS and the complete native Linux gate
remain separate qualification requirements.

[PERFORMANCE.md](PERFORMANCE.md) records all workloads, source/compiler pins,
profiles, caches, uncertainty, failures and reproduction commands. The
[performance index](performance/README.md) retains **seven fixed campaigns:
3,888 timed command values, 198 comparisons and 12,768 original output files**.
Two separate actual-project campaigns retain another 240 timings and ten
comparisons, with successful authenticated recovery of both sets of originals.

Latest retained measurements: Linux x86-64, PR12 candidate `5a3f493`, actual
checkout `b3393a4`, twelve matched pairs per mode. Times are median milliseconds.
Ratios are paired geometric means of reference/candidate time; below one means
Lustr took longer. Ratios are not calculated from the displayed medians.

### Mixed Cargo compilation

These are complete commands on a small unchanged Cargo graph with a real
procedural macro, async code and a `no_std`/`alloc` dependency. Cargo and the
matching Rust SDK retain the graph and metadata contracts; eligible scalar
functions use Lust. This fixture explicitly uses dev debug=0. Normal debug=2
project builds have a separate compatibility comparison below.

| Cargo workload | Cargo/rustc median | Mixed Lustr median | Ratio [95% interval] |
| --- | ---: | ---: | --- |
| Cold build | 151.860 | 206.257 | 0.773 [0.739, 0.823] |
| No-op build | 8.869 | 9.666 | 0.910 [0.886, 0.933] |
| Body edit | 48.793 | 78.386 | 0.624 [0.619, 0.629] |
| Signature edit | 51.841 | 94.646 | 0.548 [0.545, 0.552] |
| Check | 40.938 | 46.584 | 0.888 [0.879, 0.898] |
| No-op check | 8.779 | 9.723 | 0.916 [0.896, 0.937] |
| Check after body edit | 24.750 | 26.424 | 0.939 [0.933, 0.946] |
| Rejection | 34.027 | 33.405 | 1.020 [1.014, 1.026] |
| Compile plus test | 74.531 | 75.719 | 0.978 [0.972, 0.985] |

By the displayed medians, cold builds take approximately **36% longer**, body
edits **61% longer** and signature edits **83% longer**. Eight mixed modes are
slower in this campaign. Every mode, p95 and conditional interval appears in
the [36-comparison table](performance/2026-11-17-5a3f493.md),
[648-value CSV](performance/2026-11-17-5a3f493.csv) and
[four-panel vector chart](performance/plots/2026-11-17-5a3f493/README.md).

### Actual project under normal defaults

The [unchanged coordinator project](performance/projects/2026-11-17-5a3f493/README.md)
passed executing tests and rejection gates under normal dev/test defaults.
At debug=2, outside native-object eligibility, median overhead was 2.5% for a
primed root build, 3.8% for a primed check, 10.3% for no-op and 1.8% for compile
plus test. The forced build, check and test scenarios only change a source
file's timestamp. They measure rebuilds without source edits and do not
demonstrate native acceleration. Four intervals favor matching Rust; clean
build is inconclusive.
The newest campaign's 600 original files and 510 output streams were recovered
and reconciled with its public report. The earlier campaign is also retained.

### Historical full-project Cargo result

The separate [November 8 Cranelift experiment](docs/compiler-set/verification.md#qualified-full-project-result)
built the unchanged Lust compiler through Cargo with a custom unwind-enabled
backend. Cold-build medians were 59.536 s for LLVM and 39.400 s for Cranelift,
a 1.511 ratio of medians across only two measured pairs. No-op builds regressed
from 19.532 to 21.149 ms. This was an explicit compatibility-coordinator route
with an older matching SDK, not the current mixed compiler. Its
[raw report and identities](docs/compiler-set/evidence/lust-project-42ab5dc-summary.json)
remain separate; a current-package claim requires a fresh matched campaign.

### Current correction and qualification

The current candidate removes three costs from the Cargo path: external `nm`
validation, external `objcopy` symbol renaming, and the intermediate
`cargo-lustr` executable startup. The [symbol-renaming checks](docs/compiler-set/mixed-symbol-rename.md)
exercise real objects and linking; the [direct entry regression](docs/compiler-set/cargo-dispatch.md)
exercises real Cargo freshness, errors and program exit status. These are
correctness checks. The tables above retain the earlier completed campaign
until the changed package has fresh end-to-end measurements.

The [in-process symbol checker](docs/compiler-set/mixed-symbol-validation.md)
replaces the costly symbol-tool process in the mixed emitter. It has focused
correctness and component measurements, but still needs a fresh complete
source-built qualification and Cargo campaign. None of the tables above
measure that change.

The [larger upstream slice campaign](docs/compiler-set/upstream-slice-benchmark.md)
compares supported native compilation with incremental Cargo on pruned upstream
source. Its cold/no-op and runtime scenarios broaden the corpus; they do not
measure edited-source mixed Cargo acceleration. Adding the runner supplies no
new speed result.

The [fresh v2 profile](performance/2026-11-17-5a3f493-profile.md) passed original
validation: 26 compiler profiles, 15 qualification commands and 35 bound native
responses. One instrumented body request took 37.681 ms, including 34.545 ms
in the complete symbol-inspection process call. This identifies a dominant
observed span; the child binary and launch/wait breakdown were not recorded,
so its cause remains unresolved. The earlier concurrent trace-framing failure,
its separate reconstruction and the correction stay in the evidence history.

[Source-built CI](https://github.com/glopyglerky/lustr/actions/runs/44111326777/job/134137398335)
passed all correctness prerequisites, profiling and both measurement campaigns.
Eight non-macOS jobs passed; macOS compatibility was cancelled before execution.
The separate [combined native Linux gate at `40d2ac3`](docs/compiler-set/measurements/2026-11-17-native-linux-40d2ac3/README.md)
passed 5,527 ordinary differential cases, then recorded 5,526 sanitized passes
and a ten-second generated-program execution timeout. Fuzz and benchmark smoke
were not reached. The [fresh `5a3f493` full native gate](docs/compiler-set/measurements/2026-11-17-native-linux-5a3f493/README.md)
also failed at the same sanitized case after 5,527 ordinary and 5,526 sanitized
passes. Both original failures remain recorded; their cause is not established.

The [upstream audit](docs/compiler-set/upstream-benchmark-audit.md) compares this
coverage with Lust's README and repository studies. Larger native projects,
broader edit families, isolated application runtime, CPU/memory measurements
and performance acceptance remain open. Upstream headline ratios remain
attributed to their own compiler, corpus and methods.

### Native Lustr, pristine Lust and matching rustc

These native results use the same completed `5a3f493` campaign and twelve
matched pairs per mode. Times are median milliseconds; ratios retain the
paired geometric estimator.

| Native workload | Pristine Lust median | Standalone rustc median | Lustr median | rustc / Lustr ratio [95% interval] |
| --- | ---: | ---: | ---: | --- |
| Cold build | 41.808 | 38.147 | 40.285 | 0.914 [0.876, 0.950] |
| No-op build | 3.542 | 37.982 | 3.106 | 12.208 [12.037, 12.383] |
| Body edit | 5.494 | 38.224 | 5.590 | 6.894 [6.797, 7.004] |
| Signature edit | 4.841 | 38.424 | 4.856 | 7.935 [7.759, 8.131] |
| Check | 2.217 | 11.619 | 2.069 | 5.621 [5.446, 5.812] |
| No-op check | 1.761 | 11.356 | 1.778 | 6.426 [6.267, 6.605] |
| Check after body edit | 2.051 | 11.285 | 1.882 | 5.970 [5.797, 6.179] |
| Rejection | 2.255 | 17.038 | 18.016 | 0.945 [0.930, 0.959] |
| Compile plus test | 30.535 | 53.267 | 30.960 | 1.740 [1.705, 1.776] |

The standalone Rust reference uses the identical source, opt-level 0, debug 0,
unwinding, overflow checks and debug assertions. It is **nonincremental**:
a no-op launches rustc again. Cargo's cached no-op appears in the separate
Cargo comparison above. Compile plus test includes compiling and actually
executing the test. The [reference contract](docs/compiler-set/native-rustc-comparison.md)
records exact commands, child receipts and cache behavior.

Against pristine Lust, native cold, body/signature edits and compile-plus-test
remain inconclusive. Native no-op favors Lustr at 1.153 [1.123, 1.186]. Packaged
rejection is slower at 0.128 [0.121, 0.136]; the entry may perform full-Rust
validation after native rejection. The full language and diagnostic contract
remains part of that measured cost.

## Compiler integration

Using or importing upstream rustc code is allowed. The project does not require
rebuilding every Rust feature independently. Speed preservation applies to the
Lust-supported path; upstream speed ratios are not measurements of this
integration. See the [current contract](docs/compiler-set/native-contract.md).

The existing candidate packages `lustr`, a Cargo-facing `rustc` entry, Lust,
Cargo and the full Rust compiler/runtime. Start with its
[commands and qualification](docs/compiler-set/native-preview-2026.11.15-1.md).
Full-language requests retain real Rust metadata and Cargo behavior. Requests
that Lust supports can compile through its native path. Further work must
qualify this combined journey and preserve the fast path.

Whole-target selection is initial progress. The finished integration must also
support mixed work while retaining Lust's frontend and body-edit acceleration.

The [mixed compiler experiment](docs/compiler-set/mixed-compiler.md) now builds
and runs one unchanged Cargo executable containing Lust-generated scalar code,
Rust-generated async code, a real procedural macro and a `no_std`/`alloc`
dependency. Its first qualified body edit reuses the retained Lust session and
checked Rust objects. Signature, macro and dependency changes invalidate that
reuse; invalid Rust is rejected. Ordinary Cargo checks can also reuse checked
scalar bodies in application and library targets while Rust writes fresh,
valid metadata for the edited source. General mixed-function coverage and equivalent
complete-work performance remain open.

The [source toolchain builder](tools/build-mixed-toolchain.rs) assembles the
complete local development toolchain from this checkout, the matching upstream
Rust SDK and TinyCC. It builds the lustr entry, Lust, native C ABI library,
Cargo integration and mixed driver without an older prebuilt lustr package.
See the [build instructions](docs/compiler-set/mixed-compiler.md#build-and-qualify).

The following native expansion experiments are retained research. Their
incomplete language and metadata coverage is not a requirement to replace the
imported full compiler before delivering lustr.

An experimental [native semantic artifact](docs/compiler-set/proofs/native-crates.md)
now binds checked structural signatures to their complete object and imports real
definitions before native resolver rewriting. Consumer-local IDs are remapped
from independent stable crate identities. The development qualifier executes
typed native calls and function values across independent producer and
consumer units, including separate-process 128-bit pointer-call ABI checks.
Callee evaluation preserves effects, borrow uses and the pointer read before
arguments. General metadata, Cargo and
full-language coverage remain open; the integration is unadmitted.
The experimental producer now uses the existing resolver and records inline
modules, visibility and re-export bindings. Native path/re-export/glob cases
execute with stock-oracle parity; general core/alloc and input records remain open.
Format sixteen retains bounded tuple, array, float and safe function-pointer
graphs with native layout, Copy, drop and effective C ABI checks. Large and
zero-sized returns use the existing backend rules. Compiler-generated memory
calls use Lust's upstream C memory unit and an exact licensed musl fill
dependency, localized within each object. This
retains an explicit C dependency and does not qualify a general no_std runtime.
It also records original native region signatures for shared and mutable
references to represented sized types. Independent borrowed calls and local
pointer coercions use the existing native region and loan rules. Region bounds,
erased-type agreement, escapes, aliasing and static restrictions are checked;
Borrowed callback exports retain the original pointer output-slot mask and
width, including static distinctions, and use the original pointer proof.
Unbounded named late-bound function lifetimes retain their exact original
region records. Written declarations must match the native names, count and
binding classification. Lifetime-only function bounds and early-bound parameters
retain independent declaration names and canonical original constraints. Imported
calls and pointer coercions use the original native region engine. Free item
regions and general nominal/type-generic region images remain open.

Sized references to slices and `str` retain their wide native layout and C ABI,
mutability, lifetime and represented slice element type. The image constructs
the original unsized referent type without assigning it a fabricated layout.
Original native checking, borrow rules and code generation handle these values.
This remains an experimental crate interface; real core, general unsized
aggregates, native Cargo and no_std runtime qualification remain open.

The experimental image also carries closed nongeneric struct and enum identities,
variant shapes, discriminants, fields and privacy. Pending shapes validate across all images before any
imported semantic definitions or signatures become visible. The unchanged
`Packet` case has native direct, pointer and independent-identity runtime
evidence. The unchanged array-backed nominal gate now executes large/nested
and borrowed consumers through explicit native runtime dependencies. Actual
original runtime and location providers validate before imported semantic
definitions. Independent producer and consumer panic locations match the
stock oracle. Tuple and unit constructors, enum imports and re-exports, tagged
and niche layouts, and cross-crate matching have native runtime evidence.
Empty enums, malformed records and consumer table remapping are also checked.
General nominal semantics, core/alloc and a complete no_std
runtime remain open.

The candidate also records lifetime-only nominal declarations and their original
kept field types, with separate applied lifetime arguments. It validates nested
applications and applied enum layouts before imported semantic publication.
Existing native variance, outlives and borrow checks consume fresh tables.
Reference cycles now retain stable nominal identities and complete lifetime
applications. Independent recursive and mutually recursive consumers pass runtime,
borrow rejection and malformed-image/remapping checks through the original native engine.
Separate symbolic nominal templates now preserve type and const parameter kinds,
combined parameter indexes, implicit Sized obligations and scalar, tuple and array
fields. Templates receive no invented layout or ABI. The original native
instantiation engine builds consumer instances, including types and const values
never instantiated by the producer. Independent struct/enum, array-length and
bool/char const cases have native runtime and rejection evidence. A focused
resolver interface reads collected parameter declarations when an imported item
has no local AST header, retaining the original const type and range checks.
General producer generic applications, lifetime-bearing template fields,
defaults, derives, Drop, general predicates and generic bodies remain open,
alongside borrowed nominal function-pointer images and complete crate support.

Full native Rust coverage is incomplete. Lustr now supplies bounded native
source-library additions and an opt-in native constraint-inference experiment.
Full frontend and crate/metadata expansion remain open.
The opt-in ABI also has an owned original-token frontend for bounded boolean
`unsafe` blocks, including initialized boolean locals with lexical scope and
shadowing checks. Direct safe Rust calls with boolean inputs and output have
separate native target, ABI and lexical-context checks before collection returns;
local function items, including `const fn`, in affected enclosing blocks use native item parsing,
arena integration and lexical item scopes;
see the [interface and limits](docs/compiler-set/proofs/native-unsafe-blocks.md).
`compiler/native` preserves the pristine reference pinned by
[`upstream/lust.pin`](upstream/lust.pin) plus an explicit integration diff.
Focused shared phase/state interfaces may use that diff;
imported Rust components retain separate identities. The guard binds reference,
patch, qualification record and integrated result under the
[source policy](docs/compiler-set/upstream-lust.md). Correctness, independent
review and appropriate performance evidence are required. No hidden overlay or
unexplained drift is admitted.
The root Cargo project now builds the native-first compiler entry candidate in
`src/`. Its [execution and artifact contracts](docs/compiler-set/native-driver.md)
identify the declared native integration, the C ABI cold entry and the inherited full
Rust core. Preview packaging does not establish a complete native compiler.
The separate compatibility coordinator remains in `crates/lustr`.
The experimental `native-inference` features in the root and native ABI crate
compile that unchanged coordinator through native Lust phases. They elaborate
bounded checked `OsString` and `PathBuf` conversion constraints in an owned AST clone, then
run full native checking and generation. The optional product worker uses a
separate inference library after ordinary native rejection. This is not enabled
in the published preview and does not supply general Rust metadata or Cargo
library artifacts; see the [entry contract](docs/compiler-set/native-driver.md).
An additional bounded ABI experiment reuses a validated successful executable
within one process. It discards inference state after each compile, recompiles
after semantic edits and retires on native runtime-source changes. Its host
file-input proof now covers the pinned Linux TinyCC header/linker path through
actual opens and identical-object replay. Its [fresh optimized measurements](docs/compiler-set/measurements/2026-11-15-native-host-inputs-v36/README.md)
retain substantial cold/edit regressions and a coordinator no-op slower than
stock Cargo. Runtime-loader closure and product integration remain open; this
is not a general build cache or native Cargo freshness implementation.
The owned original-token assertion formatter preserves nested source parentheses,
binary operators, bounded struct expressions, tuple fields, tuple/list-array
literals, array repeats and bounded const blocks,
and exact long panic payloads through full native rechecking.
Its [component51 qualification and measurements](docs/compiler-set/measurements/2026-11-15-native-array-repeat-v51/README.md)
cover direct and retained execution, edits, rejection, Drop and unwind; full
language coverage and full-project speed remain open. The const-block candidate
has separate [design and qualification obligations](docs/compiler-set/proofs/native-assert-text.md).
The [entry contract proofs](docs/compiler-set/proofs/native-entry-contracts.md)
cover target identity, bounded cancellation and copied-artifact publication;
their assumptions and performance limits are explicit.

Build the native compiler with an existing Rust bootstrap toolchain and the
pinned TinyCC assets. Keep generated output outside the source checkout:

```sh
LUST_TCC_HOME=/absolute/tcc CARGO_TARGET_DIR=/absolute/lustr-native-build \
  cargo build --manifest-path compiler/native/Cargo.toml --release --locked
/absolute/lustr-native-build/release/lust check unchanged.rs
/absolute/lustr-native-build/release/lust build unchanged.rs -o application
```

## Compiler entry candidate

The [native preview 2026.11.15-1 record](docs/compiler-set/native-preview-2026.11.15-1.md)
describes the packaged runtime, corresponding source, exact host dependencies,
executed checks and remaining coverage/performance requirements.

The assembled candidate provides `bin/lustr`, the rustc-compatible `bin/rustc`,
the declared integrated `bin/lust`/`bin/lust-core`, and the unchanged upstream full Rust core as
`bin/rustc-full`. Native examples use:

```sh
/absolute/candidate/bin/lustr build program.rs -o application
/absolute/candidate/bin/lustr check program.rs
/absolute/candidate/bin/lustr build src/main.rs --manifest-path Cargo.toml -o application
```

Full Cargo artifact requests can use the matched tools explicitly:

```sh
CARGO=/absolute/candidate/bin/cargo RUSTC=/absolute/candidate/bin/rustc \
  RUSTDOC=/absolute/candidate/bin/rustdoc /absolute/candidate/bin/cargo test --locked
```

These commands describe the candidate interface. Versioned previews record
their exact source, runtime, notices, host requirements and qualification.
Full Rust still uses the inherited Rust implementation. Native Rust library
metadata, general mixed-engine crate interoperability and full native expansion
remain unfinished. Working routing is not proof that those tasks are complete.

The Linux x86_64 GNU candidate adds executable/shared-library naming constants,
borrowed OS-string/path conversions to `PathBuf`, and `reserve`/`reserve_exact`
for `OsString` and `PathBuf`. These compile the original program through Lust with
an explicitly composed source library. Retained sessions verify the installed
source inventory and hashes and use a separate component identity. Explicit
source overrides stay cold. See the [entry contract](docs/compiler-set/native-driver.md)
for qualification and limits. The current host binary requires GLIBC 2.39,
libgcc and zlib, and the recorded `/usr/lib/ld-linux-x86-64.so.2` and
`/lib64/ld-linux-x86-64.so.2` loader paths. The payload was exercised on AerynOS
2026.08; its ELF report records each object's actual interpreter and dependencies.
Other native hosts are unqualified. Cold compilation and edits still regress
against stock Rust on the measured expansion fixtures.

## Historical compatibility toolchain

For versioned binary bundles, see [BlueFe releases](https://github.com/glopyglerky/blueFe/releases)
and [Lustr releases](https://github.com/glopyglerky/lustr/releases). Use a release's
recorded platform, component manifest and checksums. Keep the extracted toolchain
at a versioned absolute path; ordinary project builds do not rebuild the compiler.

[Preview 2026.11.09-2](https://github.com/glopyglerky/lustr/releases/tag/preview-2026.11.09-2)
is available for Linux x86_64 GNU with GLIBC 2.39 and GLIBCXX_3.4.32. Download all
six assets and verify `SHA256SUMS`. It includes the matched rustc-based BlueFe
compiler, host/Wasm libraries, Clippy/rustfmt and experimental unwind-enabled Cranelift.
Other hosts and Wasm execution are unqualified. The release records coordinator
source `07470b2`; the later default-branch benchmark/docs commit leaves that
coordinator source unchanged. See the
[exact release record](https://github.com/glopyglerky/blueFe/blob/main/docs/bluefe/releases/preview-2026.11.09-2.md).
It does not include native Lust. Its measurements do not measure native Lust
acceleration or establish full native Lustr coverage.

```sh
lustr --bluefe-root /absolute/toolchain build --workspace
lustr --bluefe-root /absolute/toolchain test --workspace
lustr --bluefe-root /absolute/toolchain build --target wasm32-unknown-unknown
lustr --bluefe-root /absolute/toolchain doctor
```

The older assembled BlueFe directory supplies Cargo, rustc, rustdoc and the produced
sysroot. In that historical coordinator, `lustr build` without a root
preserves the project's existing Cargo toolchain. Arguments, profiles, features,
compiler wrappers and failure status remain effective. Builds install nothing
and failed programs are never retried through another compiler.

An explicit Cranelift engine provides development code generation when a matching,
unwind-enabled backend has passed qualification. WebAssembly builds use
LLVM. A separately pinned backend cannot be loaded into a different BlueFe compiler.
The default forwarding path removes no compiler work by itself.

An experimental, explicit frontend job option applies to one selected Cargo
`rustc` target without replacing project flags or compiler wrappers:

```sh
lustr --bluefe-root /absolute/toolchain --frontend-jobs 4 \
  rustc --profile check -p your-package --lib
```

Select an actual package/target and a bounded CPU budget. More threads can regress
small targets or increase memory. The [argument-preservation proof and executable
checks](docs/compiler-set/proofs/frontend-scheduling.md) state the guarantees and
limits; measured speed claims require fixed-workload evidence.

Building the coordinator itself is optional contributor work:

```sh
cargo build --manifest-path crates/lustr/Cargo.toml --release --locked
cargo install --path crates/lustr --locked
```
## Source map

| Path | Responsibility |
| --- | --- |
| `src/`, `Cargo.toml`, `build.rs` | Primary compiler entry candidate, request contracts, native loading and full-core invocation. |
| `crates/lustr-native-core` | Owned C ABI entry around the declared native Lust integration; optional checked constraint elaboration and arena reindexing; compiled separately with PIC TinyCC. |
| `crates/lustr-native-core/src/{native_modules,expanded_parser,expanded_check}.rs` | Explicit opt-in module composition, original-token unsafe-block parsing and safety-qualified AST lowering; default pristine entry retained. |
| `crates/lustr-native-core/src/collect/safe_calls.rs` | Original-context native call resolution and bounded target, safety, ABI and signature qualification. |
| `crates/lustr-native-core/src/native_crate.rs`, `native_namespace.rs`, `native_types.rs`, `native_nominals.rs`, `native_generic_nominals.rs`, `native_regions.rs`, `native_runtime.rs`, `native_memory.rs`, `native_fill.rs`, `tools/native-crate-qualification.rs` | Experimental typed native artifact, concrete layout/ABI, symbolic nominal templates and original region records, explicit runtime/location dependencies, licensed memory providers, producer resolver projection and stable crate/definition/module remapping. Early imports and calls use no fabricated function ASTs. General metadata/Cargo integration remains open. |
| `tools/native-crate-adts-qualification.rs` | Native tuple/unit struct and enum producers, constructor/privacy/move/matching runtime and rejection cases, checksum-valid malformed variant/layout records and consumer remapping. |
| `tools/native-crate-borrowed-nominals-qualification.rs` | Lifetime-bearing nominal producer/consumer, kept-field/application completeness, applied-enum layouts, borrow rejection and malformed-image/remapping qualification. |
| `crates/lustr-native-core/src/native_consts.rs`, `tools/native-crate-consts-qualification.rs` | Experimental closed constant identities, typed values and promotion qualifiers through original native checking and fresh consumer constant tables; generic bodies and reference/allocation/function identities remain open. |
| `src/native_worker.rs`, `tests/native_inference.rs` | Opt-in inference child, direct native/oracle and product routing/failure qualification. |
| `crates/lustr-native-core/src/inference/{reuse,host_inputs}.rs`, `tools/native-{retained-probe,tcc-input-observer}.rs`, `tests/native_reuse.rs` | Bounded artifact proof, native host-input observation and edit/cold-oracle qualification; not selected by the product. |
| `library/lustr`, `library/lust` | Owned source-library additions and an explicit link to the guarded native component. Protected changes are recorded in the complete declared integration diff. |
| `compiler/native` | Pristine pinned Lust reference plus the declared reviewed integration diff; compiler, libraries, generation and differential fixtures. Focused shared interfaces follow the source policy. |
| `crates/lustr` | Transitional Cargo/rustc coordinator, BlueFe tool selection and compatibility qualification. |
| `upstream/lust.pin`, `upstream/bluefe-core.pin` and `tools/lust-upstream-check.rs` | Upstream source identity, exact compiler-component bindings and the Lust source-preservation guard. |
| `crates/lustr-semantic` | Optional version-pinned rustc inspection. It exports inspection data, not executable MIR. |
| `crates/lustr-bench` | Standalone paired benchmark runner and correctness gates. It is outside the coordinator workspace. |
| `examples/native_*` | Rust delivery, checksum-pinned notice collection and native measurement helpers; generated output belongs in an explicit build filesystem. |
| `docs/compiler-set` | Current design, decisions, divergence records, measured evidence and future work. |

The coordinator does not depend on TinyCC or registry packages. `cargo-lustr` and
`cargo-lust` remain command aliases. The native component has a separate Cargo
workspace so its C dependencies do not become prerequisites for installing the compatibility coordinator.

## Working contract and research

Native execution, workspace tests, doctests, invalid-borrow rejection and wasm
linking have dedicated BlueFe gates. Backend qualification also checks both
Rust ABI directions, ownership transfer and panic cleanup. Exact compiler heads,
source fingerprints, backend hashes and unsupported targets belong with results.

See the [integration and performance model](docs/compiler-set/bluefe-performance.md),
[verification record](docs/compiler-set/verification.md),
[design](docs/compiler-set/design.md), and [roadmap](docs/compiler-set/roadmap.md).
Performance claims are workload-specific. No-op, edit, rejection and executed-test
costs count alongside cold compilation. Slower candidates remain in the evidence.

The [measurement protocol](docs/compiler-set/benchmark-method.md) records exact
tools, dependency selection, compiler commands, correctness gates and every
paired sample. Its [mathematical analysis](docs/compiler-set/proofs/benchmark-analysis.md)
proves estimator invariants and conditional statistical properties. The
[metadata comparison proof](docs/compiler-set/proofs/metadata-compatibility.md)
states the one supported absent/null schema equivalence and its limits. These proofs
do not establish compiler equivalence or an unmeasured speedup. The runner's tiny
fixture qualifies the measurement machinery; public-repository campaigns remain
separate evidence.

The runner also supports [optional Linux CPU and peak-RSS observations](docs/compiler-set/proofs/benchmark-resources.md).
Fresh collectors preserve payload outcomes and avoid cumulative-peak leakage.
RSS means the largest waited process, so it cannot establish aggregate build
memory. The historical shared-v2 dataset remains wall-time-only. The matched pilot below
records CPU and scoped RSS; the collector itself removes no compiler work.

The [matched CGU16/ThinLTO pilot](docs/compiler-set/measurements/2026-11-09-cgu16-thinlto-v3/README.md)
completed 108 timed root checks plus nine excluded warmups. The optimized stage2
package used 17.9% less wall time than the matched no-PGO stage2 control across
the three fixed cases, with conditional candidate/control interval [0.7995, 0.8401].
It still used more time than stock Rust 1.99.1 on every workload. That secondary
comparison has unequal compiler/Cargo versions. The report includes every command
record, CPU and scoped RSS, regressions, independent review, a Rust replay and
[mathematical arguments](docs/compiler-set/proofs/cgu16-thinlto.md). These are
metadata-only checks, not full-build or application-speed results.

The [2026-11-09 public-repository dataset](docs/compiler-set/measurements/2026-11-09-shared-v2/README.md)
contains 72 timed root checks and nine excluded warmups. Explicit four-thread
BlueFe beat stock on t3code-ls, with a paired ratio of 1.268, but regressed by
point estimate on gitr and bluefin. One-thread BlueFe regressed on all three.
Same-compiler N1/N4 ratios were 2.345, 1.342 and 1.802. Unequal toolchain versions,
shared-host effects and fixed middle position for N1 limit interpretation;
runtime gates did not apply the timed frontend options. These are metadata-only
root checks, not default whole-build results. Full samples, conditional intervals,
regressions, scoped proofs and a Rust statistics replay command accompany the data.

For future campaigns, the runner now balances lane execution positions in cyclic
blocks. Its [position theorem and checks](docs/compiler-set/proofs/order-balance.md)
prove the count bound, without claiming to remove cache effects or establish a
speedup. Published historical measurements keep their original order and limits.

Backend campaigns can use the [routed test gate](docs/compiler-set/proofs/routed-test-gates.md)
with an explicit Lustr toolchain and engine. It checks the core tool selectors;
the native fixture observes LLVM and matching Cranelift compiler commands and
tests panic cleanup. Project-specific gates and matching backend artifacts are
still required before reporting performance.

## Fork ownership

Lustr owns this layout, its entry points and its documentation. Required inherited
compiler code is maintained as part of the native component. Upstream research
results describe their original revisions and workloads; they are not claims
about Lustr's current toolchain.

[Changes from upstream and retained dependencies](docs/compiler-set/divergences.md)
records why inherited components remain and how updates are qualified. The source
base is [Lust](https://github.com/t3dotgg/lust-lang) at
`240e08d91d4aed0299639f6f11fd522912b248f9`. Its original README and history remain
available there and in this repository's Git history. Licensing is preserved.

This is pre-alpha software. The coordinator forbids unsafe Rust in its own code.
The current native compiler, TinyCC, rustc, LLVM and platform dependencies still
require review and eventual migration before a whole-toolchain memory-safety
claim is justified.
