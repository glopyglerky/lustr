# Deferred selector spans: the remaining work is spread across validation

This is a local component profile of the published compact-selection and
access-check source at `3c378a11d60149a3ec99fa49ea1b336e9406fca8`. It does not
measure public commands or establish a performance gain. The diagnostic adds
fixed arrays of inclusive nanosecond counters to the actual production
`full::select`; formatting happens after selection and its outer timer finish.
The diagnostic source revision is `1f26c898944e5e911becffc24be031e1d06dadf6`,
tree `a766b464c62be6913482138e8b5128292931b060`. The production PR does not include
this instrumentation.

The predeclared schedule ran once: three primes, eight excluded warm calls and
96 measured retained selections. Two tool initialization commands and two fresh
uninstrumented Cargo selection observations bring the recorded command count to
111. Every measured observation was a cache hit. All complete selected package
and target values agreed with fresh Cargo before and after the schedule. The
fixture bytes remained unchanged. No measured command enabled a trace or stats
output path, and no observation was removed.

The native arguments match the fourth hosted selector diagnostic: explicit
manifest and bin, offline and locked operation, the host target, unwind panic,
overflow checks and debug assertions. There is no package selector. The fixture
is the unchanged public-slice fixture from that run, relocated into this local
diagnostic workspace.

## Observations

| Inclusive span | Median microseconds | Calls per retained selection |
| --- | ---: | ---: |
| Outer `full::select` elapsed | 285.341 | 1 |
| Fresh snapshot capture | 224.921 | 1 |
| Toolchain observation | 120.885 | 1 |
| Executable observations, within toolchain | 49.779 | 5 |
| Cargo configuration validation | 22.519 | 1 |
| Environment collection and digest | 21.407 | 1 |
| Target discovery | 16.139 | 1 |
| Initial manifest/lock/context reads | 15.017 | 1 |
| Retained record load | 52.539 | 1 |
| Snapshot digest, within load | 12.885 | 1 |
| Record JSON parsing, within load | 11.157 | 1 |
| Record payload clone, within load | 4.487 | 1 |
| Record checksum, within load | 4.392 | 1 |
| Unclassified top-level work | 2.904 | 1 |

These are nested inclusive spans. They cannot be added as independent costs.
The last row is computed per observation from the internal full-selection span
minus only its disjoint capture, load and diagnostic spans. The outer median is
285.341 microseconds; the minimum is 221.270, the 95th percentile is 455.329 and
the maximum is 525.514. The original variation is retained. Each timer itself
adds a clock call and a thread-local counter update.

This profile identifies live input validation as the main remaining category.
It does not show a single JSON clone or path operation large enough to remove
the whole public-entry gap. The profile and hosted public benchmark use different
processes, environments and package layouts; subtracting these timings from a
hosted command would not establish attributable overhead.

## Package and build limits

The diagnostic uses real pinned Cargo, reference Rust, rustdoc and SDK shared
libraries from nightly-2026-11-09, plus a genuine `lustr-mixed` executable compiled
from the published source with the fourth run's native identity
`be64fca1236b2d1f`. The Cargo and reference compiler file hashes match the protected
pin. Direct SDK symlinks and a copied full wrapper reproduce the constructor's
layout pattern. The package has no native compiler or native shared library;
the production selector naturally admits this smaller real layout. There are
five executable and three library observations, compared with the complete
package's additional native library. No fake executable or library is supplied.

The exact package inventory, selection receipt, helper hashes, source closures,
Cargo manifests/locks and build streams are retained. The original helper is
compiled from the unchanged published source; the profiled helper includes the
exact diagnostic patch. An initial instrumented build failed because of a module
filename and a missing local guard variable; those diagnostics remain. An initial
full-wrapper build used an earlier native hash and was not executed or timed.
Its hash and the subsequent corrected build streams remain. The final measured
helper and wrapper identities are bound in the predeclared plan.

The before/after fresh Cargo calls qualify complete Selection equality for this
fixture; they are not a timing reference. They do not qualify native runtime,
compiler output, arbitrary credential changes or the complete installed package.
After the fixed schedule, source closure, both helpers, selected package file
hashes and fixture bytes were checked against the plan without rerunning it.

## Replay and original retention

Run `python3 verify-evidence.py` from this directory. Replay checks every public
file hash, source/helper bindings, the fixed command schedule, original status
and projected streams, complete Selection equality, all 96 measured cache hits,
and the exact span distributions. It runs no compiler or workload.

The public projection replaces the task workspace prefix with
`<DIAGNOSTIC_WORKSPACE>`. Source code and numeric measurements are unchanged;
original and projected stream hashes are distinct and retained. A checksummed
archive contains the projected streams. The exact original plan, report,
streams, sources and build inputs are also preserved with the repository's
existing public evidence-recipient certificate using authenticated CMS
encryption. No private key was accessed. Public replay verifies the projected
records; it does not decrypt originals or restamp absent binaries.
