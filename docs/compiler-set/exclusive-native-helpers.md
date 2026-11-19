# Product native helpers without compiler delegation

The `exclusive-native` Cargo feature makes the shipped `lust` and `lust-core`
helpers obey Lustr's exclusive compilation ownership. The feature also exists in
`lustr-native-core`, whose included native implementation and build helper use the
same selection. Product construction must enable it and require
`cargo:rustc-env=LUST_EXCLUSIVE_NATIVE=1` from the source-built native receipt.
The default feature selection retains the separately built research/oracle
behavior. The pristine donor pin is unchanged.

This is a boundary repair. It does not deliver the missing complementary
compiler, prove all Rust coverage, or qualify a replacement package.

## Execution boundary

| Native operation | Product behavior |
| --- | --- |
| Ordinary native `check`, `build`, `run`, `test`, and daemon requests | Use the existing native implementation and semantic environment. |
| Actual `ReleaseCc` native release driver | Retained; no Rust compiler is substituted. |
| Delegated `Release`/`--profile` and `RustcCheck`/`--rustc` | Reject before launching Cargo or rustc; the delegating implementations are excluded from product builds. |
| `LUST_PROBE_FAKE_DIR` | Both executable entries reject it before argument parsing, image setup, probe artifacts, or daemon communication. The fake-probe module is excluded. |
| Nonempty `RUSTC_WRAPPER` or `RUSTC_WORKSPACE_WRAPPER` | Reject the unsupported compiler context before dispatch. |
| Cargo graph requiring the existing compilation probe | Return an integration-incomplete error before consulting probe feature caches. |
| Cargo build script | Refresh and output acquisition return an integration-incomplete error before consuming oracle records, replaying a compiler command, or changing the script. No-script paths remain available. |
| Dependency-free single-package graph without a build script | Retain native graph selection and Cargo metadata facts. No Cargo feature-resolution algorithm is reconstructed. |

The native resolver also rejects Cargo's authoritative `custom-build` target
before selecting unit features. The guarded build-script path handles later
checks independently, so a prior root-entry classification does not authorize
host compilation. A cache cannot convert an incomplete integration into admitted
support.

These explicit failures identify unfinished integration. They do not remove
formerly supported programs from the actual Lust-supported subset or authorize
another compiler to own them. Completing their native/service boundary remains
Lustr work.

## Read-only Cargo queries

Product metadata commands locate `cargo` and `lustr` beside the running helper.
Missing packaged tools fail; they do not fall through to caller `PATH`.
Only the child command receives the packaged `RUSTC` query path and empty compiler
wrappers. The request's semantic environment, including values observable by
`env!` and `option_env!`, is unchanged.

The exact internal invocation `lust-core --lustr-print-cfg` reports the existing
native `CfgEnv` for the host, no features, no test mode, and `DEV_UNWIND`, before
compiler/image setup. It does not invent a second target table. The product
entry must admit only the query modes those native facts actually describe and
reject unsupported target/profile/cfg arguments. This helper does not itself
claim any additional Rust artifact format or target support.

## Identity and evidence scope

The feature bit enters `LUST_BUILD_HASH`, which already participates in daemon
wire identity, disk compiler identity, and image compatibility. A marker alone
would not prevent reuse of an oracle-enabled daemon or artifact. The actual
feature-sensitive hash is therefore required as well as the product receipt.
Both executable helpers also answer the exact feature-only
`--lustr-runtime-identity` query with `lustr-exclusive-native-v1` and their full
build hash. Product binding checks each executable against the receipt, so a
guarded client cannot conceal an old unguarded core.

The native tree for this repair is
`a5223b8045f75660281342bf547769fc1b098725`; its complete donor-to-integration patch
is `cf3585b60d0d3c5ace5824f0f321db40bc3daa0c`. Both retain the pristine donor
revision `240e08d91d4aed0299639f6f11fd522912b248f9`. The
[upstream declaration](../../upstream/lust.integration) binds the complete patch
and the source-scoped qualification record.

An identical-source build-helper control with the same TinyCC installation
produced:

| Selection | Receipt marker | Native build hash |
| --- | --- | --- |
| Research/default | `0` | `2e78fd94c0970b12` |
| Product `exclusive-native` | `1` | `b48307d68f2f2b4e` |

The pinned Rust SDK at `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494` successfully
built both product helpers on x86_64 Linux in the development profile. Both actual
executables answered the exact product-identity query successfully with empty
stderr. The exact candidate source guard passed. The
[verification record](evidence/exclusive-native-helpers-20261119/verification.json)
binds binary hashes, inputs, commands, and scope. The earlier native-core feature
check also passed, but is explicitly attributed to its earlier native tree
`43fe4adf36c1bb0f5a8340229e69325bc6035eea` rather than this final source. The first
guarded draft failed to compile because one build-script caller still referenced
the excluded probe module; that exposed the recorded-compiler replay route and
was corrected before the successful build. The unsuccessful draft is not counted
as a successful qualification.

Runtime guard and integration controls are recorded separately from compilation
and identity checks. Earlier native correctness, platform, sanitizer, and timing
results retain their original source identities. Neither development build
duration nor the absence of an external compiler call establishes near-Lust
latency, complete package compatibility, or full compiler correctness.
