# Retained work and remaining qualification

Reviewed 17 November 2026 against the imported-Rust/mixed compiler contract.
The older branches remain intact; none was closed, merged or replaced during
this follow-through. Their full-native reconstruction requirements are
historical. The [current contract](native-contract.md) permits the separately
pinned full Rust implementation and requires preservation of the supported
Lust path and actual combined Cargo behavior.

| Retained work | Exact head | Current disposition |
| --- | --- | --- |
| [PR4: native cache and qualification](https://github.com/glopyglerky/lustr/pull/4) | `a5531b729a20163d59dd7316bb5560ec0f9d4047` | Preserve its cache corrections, fixture matrix, failures and measurements. Its independent-native architecture is superseded; its timings do not qualify the current compiler. |
| [PR5: runner routes and entry corrections](https://github.com/glopyglerky/lustr/pull/5) | `e031e95e6e3e6d0549662408bca0c78a23f7eab1` | Current code covers the Windows-root and external-input macro issues. Optional `CI_RUNNER_ROUTES` was not carried forward and is not required for the hosted workflow. No alternate runner pool is assumed admitted. |
| [PR6: upstream-based native extensions](https://github.com/glopyglerky/lustr/pull/6) | `358ead785dafc85f7bc72538a49bff0ab8228e23` | Preserve its source, reviews, failures and comparisons. The applicable library selector is adapted and qualified separately. Darwin loader/lifetime work remains an explicit investigation below. |

## Library selection carried forward

The [native library-selection correction](native-library-selection.md) adapts
the retained upstream selector to the current process boundary. It fixes the
hyphenated/renamed-library failures, preserves all six Cargo library kinds, and
rejects `--lib` for a bin-only package. The original failures and two review
regressions remain with the corrected 22-state matched qualification and actual
product-entry regression. This does not import unrelated language extensions.

## Darwin base libraries remain an execution gate

The retained [PR6 head](https://github.com/glopyglerky/lustr/commit/358ead785dafc85f7bc72538a49bff0ab8228e23)
contains a specific warm native-loader/lifetime investigation. Its historical
paths are rooted at `src/`, before the current `compiler/native/` layout:

- [`src/tcc.rs`](https://github.com/glopyglerky/lustr/blob/358ead785dafc85f7bc72538a49bff0ab8228e23/src/tcc.rs)
  selects Darwin's shared-library suffix and final `@rpath` install name.
- [`src/driver/compile.rs`](https://github.com/glopyglerky/lustr/blob/358ead785dafc85f7bc72538a49bff0ab8228e23/src/driver/compile.rs)
  and `src/driver/session.rs` retain immutable base identity, live ownership,
  publication and missing-base recovery behavior.
- [`scripts/test-base-lifetime.py`](https://github.com/glopyglerky/lustr/blob/358ead785dafc85f7bc72538a49bff0ab8228e23/scripts/test-base-lifetime.py)
  compares warm, cold and Rust outputs; checks Mach-O install/load/rpath
  commands; exercises replacement, missing-base recovery and an executable
  retained after daemon exit.

The [historical run 43363880870](https://github.com/glopyglerky/lustr/actions/runs/43363880870)
records successful Linux base-lifetime checks. Its macOS first warm builds
failed in pinned TinyCC signing of paths containing spaces/quotes. Its macOS
unit harness also failed to link Linux-specific symbols, so those assertions
did not execute. The retained patch is useful evidence, not a proven repair
for the current candidate. Its
[matched package artifact](https://github.com/glopyglerky/lustr/actions/runs/43363880870/artifacts/14010850498)
and failure logs belong to that exact old source.

Current native code still uses the inherited `.so` base path. Earlier ARM64
hosted jobs were cancelled before execution because GitHub could not acquire
capacity. The `5a3f493` Compiler set macOS job was also cancelled before any
step executed. Its eight successful non-macOS jobs supply no new Darwin
execution. Once a native macOS runner is available, reproduce the current warm
loader/lifetime behavior, carry forward a bounded correction if required,
and rerun the complete native gate. Cross-target layout tests, Linux success,
C buffer models and mixed Linux campaigns cannot qualify Darwin loader work.

## Native Linux failure remains open

The [40d2ac3 original native gate](measurements/2026-11-17-native-linux-40d2ac3/README.md)
passed 5,527 ordinary cases, then reported 5,526 sanitized passes and one failure:
the generated `unicode_props_hash` program exceeded its ten-second execution
deadline after successful GCC linking. The original log and unreached later
suites remain retained. The [bounded local investigation](measurements/2026-11-17-unicode-sanitizer-diagnosis/README.md)
encountered a LeakSanitizer fatal error with separately identified debug
binaries, not the hosted timeout. It changed no fixture, deadline or sanitizer
setting and supplies no passing result. The fresh [`5a3f493` full native gate](measurements/2026-11-17-native-linux-5a3f493/README.md)
also failed at the same sanitized case after 5,527 ordinary and 5,526 sanitized
passes. Its original artifact remains retained separately.

## Historical performance remains separate

PR4's [native cache records](https://github.com/glopyglerky/lustr/tree/a5531b729a20163d59dd7316bb5560ec0f9d4047/docs/compiler-set/measurements/2026-11-11-native-cache)
and PR6's [matched release comparisons](https://github.com/glopyglerky/lustr/pull/6)
retain their own identities, estimators, regressions and incomplete coverage.
They must not be pooled with the current
[seven fixed paired campaigns](../../performance/README.md) or presented as
current source speed. The first schema-2 campaign at `5a3f493` now measures
matched standalone rustc, preserving cold/rejection losses alongside warm
supported-path gains. Two separate unchanged-coordinator campaigns retain
240 timings and ten comparisons under normal dev/test defaults; they do not
establish larger native acceleration.

The [fresh original v2 profile](../../performance/2026-11-17-5a3f493-profile.md)
passed validation and replays 26 profiles, 15 qualification commands and 35
successful native responses. The older `40d2ac3` failed profile and its separate
diagnostic reconstruction remain historical evidence. Current measurements and
phase attribution determine the next performance investigation without turning
instrumented observations or differences between campaigns into causal gains.
