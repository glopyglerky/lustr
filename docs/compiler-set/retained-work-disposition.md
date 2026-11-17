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

Current native code still uses the inherited `.so` base path. The new ARM64
hosted jobs were cancelled before execution because GitHub could not acquire
capacity. Once a native macOS runner is available, reproduce the current warm
loader/lifetime behavior, carry forward a bounded correction if required,
and rerun the complete native gate. Cross-target layout tests, Linux success,
C buffer models and mixed Linux campaigns cannot qualify Darwin loader work.

## Historical performance remains separate

PR4's [native cache records](https://github.com/glopyglerky/lustr/tree/a5531b729a20163d59dd7316bb5560ec0f9d4047/docs/compiler-set/measurements/2026-11-11-native-cache)
and PR6's [matched release comparisons](https://github.com/glopyglerky/lustr/pull/6)
retain their own identities, estimators, regressions and incomplete coverage.
They must not be pooled with the current
[four paired campaigns](../../performance/README.md) or presented as current
source speed. Current measurements and phase attribution determine the next
performance investigation.
