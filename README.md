# lustr

lustr combines [Lust](https://github.com/t3dotgg/lust-lang)'s fast native
compilation of its supported subset with complete Rust compilation through a
pinned Rust implementation. blueFe delivers the combined toolchain.

The daemon starts directly and does not require systemd. The
[native host runtime](docs/compiler-set/host-portability.md) documents the Linux
and Darwin adapters, source-built SDK receipts, focused tests and remaining
native macOS qualification. The primary native daemon is not yet a Windows
runtime; the independent Cargo coordinator remains the portable entry there.

## Numbers and current qualification

**Cargo body-edit builds now have a measured win; general Cargo acceleration
and release qualification remain open.** The latest complete paired campaign
favors Lustr for body edits and rejection, while seven other Cargo modes are
slower. The supported native path retains substantial warm gains over
standalone rustc on a small corpus. Default-profile project builds, larger
workloads and platform qualification remain distinct requirements.

[PERFORMANCE.md](PERFORMANCE.md) records every workload, source/compiler pin,
profile, cache condition, interval, failure and reproduction command. The
[performance index](performance/README.md) retains **nine fixed campaigns:
5,184 timed command values, 270 comparisons and 17,088 original output files**.
Three separate actual-project campaigns retain another **360 timings and 15
comparisons**, with authenticated recovery and source-bound reconciliation;
a fourth fresh project campaign has its
[original public and encrypted ZIPs retained](performance/projects-pending/2026-11-17-3e24029/README.md)
but has not undergone authenticated private-original recovery.

Latest retained fixed measurements: Linux x86-64, PR13 head `3e24029`, actual
merge checkout `5711bb9`, twelve matched pairs per mode. Times are median milliseconds.
Ratios are paired geometric means of reference/candidate time; below one means
Lustr took longer. Ratios are not calculated from the displayed medians.

### Mixed Cargo compilation

These are complete commands on a small Cargo graph with a real procedural
macro, async code and a `no_std`/`alloc` dependency. Cargo and the matching Rust
SDK retain graph and metadata contracts; eligible scalar functions use Lust.
This fixture explicitly uses dev debug=0. Normal debug=2 project builds have a
separate compatibility comparison below.

| Cargo workload | Cargo/rustc median | Mixed Lustr median | Ratio [95% interval] |
| --- | ---: | ---: | --- |
| Cold build | 168.135 | 194.613 | 0.847 [0.799, 0.877] |
| No-op build | 10.109 | 10.730 | 0.947 [0.920, 0.975] |
| Body edit | 55.296 | 53.101 | 1.046 [1.036, 1.055] |
| Signature edit | 59.561 | 69.471 | 0.852 [0.845, 0.860] |
| Check | 46.012 | 51.969 | 0.826 [0.712, 0.898] |
| No-op check | 10.272 | 10.800 | 0.954 [0.919, 0.991] |
| Check after body edit | 28.767 | 30.800 | 0.896 [0.823, 0.941] |
| Rejection | 39.364 | 37.976 | 1.055 [1.022, 1.101] |
| Compile plus test | 82.475 | 84.234 | 0.979 [0.974, 0.984] |

The body-edit build takes **4.0% less median time**, with a paired estimate of
**1.046× [1.036, 1.055]** against matching Cargo/rustc. Rejection also favors
Lustr. Seven other modes remain slower: cold builds take about **16% longer**,
signature edits **17% longer**, and checks **13% longer** by the medians.
This is a narrow Cargo improvement on the retained scalar-body fixture, not
general Cargo acceleration. The preceding `6bf6e3e` campaign measured a
1.079× body-edit gain on a different hosted run; cross-run changes do not
isolate a code optimization. No historical sample was removed or replaced.

Every mode, p95 and conditional interval appears in the
[36-comparison table](performance/2026-11-17-3e24029.md),
[648-value CSV](performance/2026-11-17-3e24029.csv) and
[four-panel chart](performance/plots/2026-11-17-3e24029/README.md).

### Actual project under normal defaults

The [unchanged coordinator project](performance/projects/2026-11-17-6bf6e3e/README.md)
passed executing tests and rejection gates under normal dev/test defaults.
At debug=2, outside native-object eligibility, median overhead was 0.9% for
clean builds, 1.0% for primed root builds, 3.4% for primed checks, 11.6% for
no-op builds and 1.5% for compile plus test. Four intervals favor matching Rust;
primed build is inconclusive. The forced build, check and test scenarios change
only a source file's timestamp. They measure rebuilds without source edits.
The newest campaign's 600 original files and 510 output streams were recovered
and reconciled with its public report. Both earlier campaigns remain retained.
The newer [PR13 merge checkout project result](performance/projects-pending/2026-11-17-3e24029/README.md)
also completed five scenarios, but is not yet a fourth *authenticated recovered*
archive. On its different host the no-op paired ratio was 0.785 [0.572, 0.930],
with a 76.868 ms candidate p95 outlier. Retain this slowdown rather than
substituting the older stable samples or claiming a causal regression.

### Historical full-project Cargo result

The separate [November 8 Cranelift experiment](docs/compiler-set/verification.md#qualified-full-project-result)
built the unchanged Lust compiler through Cargo with a custom unwind-enabled
backend. Cold-build medians were 59.536 s for LLVM and 39.400 s for Cranelift,
a 1.511 ratio of medians across only two measured pairs. No-op builds regressed
from 19.532 to 21.149 ms. This was an explicit compatibility-coordinator route
with an older matching SDK. Its
[raw report and identities](docs/compiler-set/evidence/lust-project-42ab5dc-summary.json)
remain separate; a current-package claim requires a fresh matched campaign.

### Current correction and qualification

The measured package removes three costs from the Cargo path: external `nm`
validation, external `objcopy` symbol renaming, and the intermediate
`cargo-lustr` executable startup. The
[in-process symbol validator](docs/compiler-set/mixed-symbol-validation.md),
[validated renaming](docs/compiler-set/mixed-symbol-rename.md) and
[direct Cargo entry](docs/compiler-set/cargo-dispatch.md) retain real object,
metadata, linking, freshness, diagnostics and process-exit contracts. Standard
Cargo commands and aliases also reach Cargo correctly. The current tables
measure the combined package; cross-run differences do not isolate each change.

The [fresh profile](performance/2026-11-17-6bf6e3e-profile.md) records an
instrumented body request of 1.821 ms, including 0.008 ms for in-process symbol
validation. Symbol renaming takes 0.043 ms; the original linker takes 15.320 ms.
These phase spans identify remaining work and are separate from paired timings.
The earlier 34.545 ms symbol-process body span remains attributed to its own
recording; its child process identity and launch/wait breakdown were absent.

[Source-built CI](https://github.com/glopyglerky/lustr/actions/runs/44179667793/job/134380277750)
passed all sixteen entry/runtime tests, the mixed compiler prerequisites,
profiling, fixed paired measurements and the normal-profile project campaign.
That job **failed its separate public-slice recorder before timing** because
statistics were enabled only in the client, not its daemon. The
[original zero-sample failure](performance/slices/2026-11-17-6bf6e3e/README.md)
remains failed. The corrected recorder subsequently **passed on a fresh hosted
compiler build** in [run 44187580063](https://github.com/glopyglerky/lustr/actions/runs/44187580063/job/134408105819).
Its [byte-identical public-slice archive](performance/slices/2026-11-17-3e24029/README.md)
contains 288 timings and 18 independently replayed comparisons. The latest
completed PR13 compiler-set run still includes a macOS job cancelled before
execution; source-built success is not an all-platform gate.

On the adapted larger upstream slice, the standalone Lustr entry has a
**1.516× [1.475, 1.571] cold-check ratio** against matched incremental Cargo,
but a **0.293× [0.288, 0.298] no-op-check ratio**. Its 35.689 ms no-op-check
median also trails pristine Lust's 1.565 ms. This is a standalone native
entry comparison, **not** a mixed Cargo build or source-edit speedup. The
[full slice table and replay instructions](performance/slices/2026-11-17-3e24029/README.md)
retain all the regressions. The next Cargo priority is separating package/target
validation, daemon entry and Rust fallback costs without losing Cargo's graph
or semantic guarantees.

Earlier full native Linux attempts at `40d2ac3` and `5a3f493` each passed 5,527
ordinary differential cases, then passed 5,526 sanitized cases and timed out
on one generated program. Their [original failures](docs/compiler-set/measurements/2026-11-17-native-linux-5a3f493/README.md)
remain recorded. Lower parallelism in the current run preserves the cases,
sanitizers and deadlines; it does not by itself resolve those failures.

The [upstream audit](docs/compiler-set/upstream-benchmark-audit.md) compares this
coverage with Lust's README and repository studies. Larger native projects,
broader Cargo edit families, isolated application runtime, CPU/memory
measurements and performance acceptance remain open. Upstream headline ratios
remain attributed to their own compiler, corpus and methods.

### Native Lustr, pristine Lust and matching rustc

These native results use the same completed `3e24029` paired campaign. Times
are median milliseconds; ratios retain the paired geometric estimator.

| Native workload | Pristine Lust median | Standalone rustc median | Lustr median | rustc / Lustr ratio [95% interval] |
| --- | ---: | ---: | ---: | --- |
| Cold build | 48.204 | 41.666 | 47.016 | 0.859 [0.830, 0.888] |
| No-op build | 4.070 | 42.418 | 3.525 | 12.040 [11.895, 12.192] |
| Body edit | 5.961 | 41.854 | 6.033 | 6.924 [6.805, 7.037] |
| Signature edit | 5.333 | 41.810 | 5.396 | 7.817 [7.620, 8.035] |
| Check | 2.435 | 12.856 | 2.449 | 5.459 [5.254, 5.683] |
| No-op check | 2.067 | 12.719 | 2.120 | 6.075 [5.894, 6.260] |
| Check after body edit | 2.272 | 12.817 | 2.174 | 5.907 [5.700, 6.136] |
| Rejection | 2.650 | 19.314 | 20.810 | 0.925 [0.908, 0.945] |
| Compile plus test | 26.053 | 59.128 | 26.322 | 2.250 [2.214, 2.290] |

The standalone Rust reference uses the identical source, opt-level 0, debug 0,
unwinding, overflow checks and debug assertions. It is **nonincremental**:
a no-op launches rustc again. Cargo's cached no-op appears in the separate
Cargo comparison above. Compile plus test includes compiling and actually
executing the test. The [reference contract](docs/compiler-set/native-rustc-comparison.md)
records exact commands, child receipts and cache behavior.

Against pristine Lust, native cold, body edits and compile-plus-test are
inconclusive. Native no-op favors Lustr at 1.154 [1.138, 1.168] and check
at 1.047 [1.006, 1.088]. Packaged rejection is slower at
0.127 [0.121, 0.133]; the entry may perform full-Rust validation
after native rejection. The full language and diagnostic contract remains
part of that measured cost.

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
