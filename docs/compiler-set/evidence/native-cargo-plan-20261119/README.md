# Native Cargo planning evidence

This directory preserves the implementation checkpoints behind the native Cargo
planning repair. The first local package was built from published commit
`62da158b33740279947c85f3fc425df6d1623c9f` plus the recorded source overlay. It was
a manually assembled partial package with freshly built public and native
executables. It did not contain the dynamic native library and did not pass
through the guarded package constructor.

The added service asks the declared Cargo for its unit graph without building
Rust units. It admits the represented Linux host, development profile, path
dependencies, features and host-script context. Native source compilation stays
in Lust. Other profiles, external dependencies, procedural macros, build-script
dependencies and injected Cargo compiler settings still have explicit integration
boundaries. This is progress toward compiler completion, not full Rust coverage,
stable-Cargo compatibility, portability or a measured speed result.

## Preserved first checkpoints

`local-checkpoints.tar.gz` contains 887 source, command, report and stream files.
`local-checkpoints-index.json` binds every archived member and the archive itself
by SHA-256. Existing failures are retained alongside later passes.

| Checkpoint | Actual result | Scope |
| --- | --- | --- |
| Initial path-graph package, build `b7c0d8a7c7855f75` | All 67 versions of 13 unchanged histories passed: 61 exact program comparisons and 6 independent source-rejection matches | Fresh processes; no retained-state claim |
| Source and target binding | 9 real controls passed, including wrong source, unknown target, ambiguous aliases, inferred library check/build and recovery | Failed selection did not run a host script or replace the successful artifact |
| Initial public entry controls | 38 synthetic controls passed | Process ownership and selection; synthetic children do not prove Rust semantics |
| Finite-query spy, third local run | One compiler-request refusal and five exact program comparisons passed; 75 allowed information queries | Actual pinned Cargo and public Lustr, with a spy refusing compilation-class requests |
| Frozen host-envelope tests | 12 of 14 passed, 2 failed | Reloaded records lost rerun directives; the host-service repair and subsequent evidence are recorded separately |
| Same frozen native target | Planner 8, configuration 5 and selector-wire 1 passed | Focused controls, not the full native suite |

The 13 path histories are `h9ms_depsig`, `h9ms_genmacro`, `h9ms_glob`,
`h9ms_late_shadow`, `h9ms_late_vec`, `h9ms_late_vec2`, `h9ms_late_vec3`,
`h9ms_rename`, `h9ms_shadow`, `h9ms_subglob`, `h9ms_xcrate_names`,
`mb01_dep_macro_body_edits` and `mb02_main_macro_body_edits`. These are a
diagnostic subset of the unchanged required inventory. They do not replace its
complete 349-history retained campaign.

The local host refuses Unix socket creation and ptrace with `EPERM`. The records
retain that restriction. Local cold successes cannot establish daemon reuse,
unchanged-request process counts or retained speed. The default
`tools/test-native-cargo-plan.py` mode requires the original native daemon and
syscall tracing; it does not silently downgrade to the local `--cold-only` mode.

## Replay and identity

The archived command JSON retains exact argument arrays, environments, source
hashes, artifact hashes and original streams. The source overlay is relative to
the published base above. The pre-configuration source manifest lists actual
Cargo dependency-file inputs, including the newly added planner and the copied
host-service sources. Earlier binaries are not attested by a later working-tree
hash.

For a new guarded package built from the final combined tree, run:

```sh
python3 tools/test-native-cargo-plan.py \
  --source "$SOURCE" --package "$PACKAGE" --sdk "$SDK" --output "$OUTPUT"
```

`SOURCE` is the matching checkout, `PACKAGE` is its constructed toolchain root,
`SDK` is the SDK identified by `upstream/bluefe-core.pin`, and `OUTPUT` is a new
directory on the same Linux runner. Keep the default retained mode in CI. A
local run explicitly using `--cold-only` establishes only its named cold checks.

Native focused tests use the pinned SDK Cargo with
`--manifest-path compiler/native/Cargo.toml --features exclusive-native
--bin lust-core --locked --offline`. Relevant filters are `product_cargo_plan`,
`cargo_target_selector_fact`, `driver::buildscript::native::tests` and
`resolve::cfg::tests`. Build environments set `RUSTC` to the pinned SDK compiler,
`LUST_TCC_HOME` to the declared TinyCC installation, and task-specific
`CARGO_HOME`, `CARGO_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR`. The recorded local
builds used `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`,
`CARGO_PROFILE_DEV_DEBUG=0` and, for tests, `CARGO_PROFILE_TEST_DEBUG=0`;
`RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER` and `LD_LIBRARY_PATH` were unset.

The independent oracle uses the pinned Rust SDK only in qualification commands.
The candidate planning spy admits only the three recorded information-query
argument forms, requires empty stdin for `-`, and explicitly refuses a source
compilation carrying `--print=link-args`. The default retained control binds the
initial core process by PID, start time and executable before and after each
request; it does not use tracer liveness as a substitute. All traced commands
have owned process groups and bounded cleanup.

## Final local checkpoints

`verification.json` indexes `final-checkpoints.tar.gz` and records the exact
source and binary identities. The 514 repository input hashes captured before
the approved native build were unchanged afterward. `selector-component.json`
and its archive separately preserve the earlier selector-component checks,
including the initial unsuccessful direct helper builds against the SDK's split
metadata artifacts and the successful ordinary Cargo build.

The approved native source has build identity `0390b1143f2d2e93`. Its core SHA-256
is `21d8043046e5277a011854cc45ae9df89d8851928fd3a48cf4aa78de41647587`.
The immutable v3 package has public-entry SHA-256
`13a749328496d3576a51eff7580e8aecfa47195ce42e2a03a91fcc4f11988a13`.
The independent host-cache regressions qualify that package and are recorded in
the host-service evidence. They are not relabeled as executions of a later entry.

A fresh explicit-package fixture then exposed a public planning defect: Cargo's
`pkgid` needs a lockfile, while `metadata --no-deps` did not create it. The
original failure remains in the final archive. Explicit package binding now
uses full Cargo metadata resolution before `pkgid`, with the original network
and lock flags. Cargo creates the lockfile without compiling source. Ordinary
existing-source manifest requests still go directly to the native resolver.
The explicit-package path has public planning cost and is excluded from the
ordinary warm zero-Cargo-process assertion.

That entry repair was built into a separate partial package with the exact same
native helper bytes. Its public-entry SHA-256 is
`bda58fc525e6c9a15ca84237359fb0570132483725a3fba2c21cf6d13603df2e`.
The new query receipt binds that entry. The first receipt-generation attempt
refused the old receipt copied during assembly; its original failure and stale
bytes are preserved, followed by successful generation after removing only that
new package's copied receipt.

| Final local control | Result |
| --- | --- |
| Approved native host controls | 17 passed |
| Public entry process controls | 38 passed |
| Public selector cache regression | 294 assertions passed across 156 invocations |
| Finite query spy | Compiler request refused; seven exact native/Rust program results passed; 113 information calls accepted |
| Fresh and existing explicit package specs | Both passed the same query-only and exact-result proof |
| Fresh explicit package with path dependency and native host script | Check and build passed; native and pinned Rust both returned status 0, stdout `host:2:42\n`, and empty stderr |

The last fixture uses `CARGO_BUILD_JOBS=2` and observes the native host script's
`NUM_JOBS` value. Source and manifest bytes remained unchanged across candidate
and oracle compilation. The qualifier's new fresh-package case starts without a
lockfile and requires normal planning to create it. Its existing-package case
also keeps the actual Cargo package-spec grammar (`name@version`).

All local process qualifiers explicitly used cold mode because sockets and
ptrace remain unavailable here. The default retained query control, complete
349-history campaign and guarded construction on the combined published tree
remain hosted requirements. None of these local receipts admits a release or
establishes compiler speed.
