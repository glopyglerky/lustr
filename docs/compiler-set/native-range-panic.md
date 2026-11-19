# Stop range checks after the first panic

The Linux native gate on PR #33 at `ec99578d51c93674af3a1bcf9c182f4ad4b17d89`
reported a 37-byte leak in `std_declared_source_locations`. Its sanitizer stage
finished with 5,546 passing cases and one failure. The original
[job](https://github.com/glopyglerky/lustr/actions/runs/44493776529/job/135455703707)
remains failed.

## Cause and correction

The native runtime's `lust_range_bounds_check` called `lust_panic` and then
continued checking later bounds. In flag-mode unwinding, a panic returns with
an exception pending. For `values.drain(3..)` on a two-element vector, the first
check raised the correct range-start panic. The later ordering check then
allocated a second String payload, `slice index starts at 3 but ends at 2`.
That payload contains 37 bytes. The pending-exception check discarded the
second raise after allocation.

Every failing bound now returns immediately after raising its panic. The
helper does not raise another panic or publish output bounds after failure.
Its callers already check pending unwinding before reading those outputs.
The exception-table and abort paths already leave the helper at the panic.
The correction adds no work to successful range checks.

Lust retains ownership of these operations. No compiler selection, fallback,
import, upstream pin, or sanitizer policy changes are involved.

## Observed checks

The [verification record](evidence/native-range-panic-20261119/verification.json)
binds the source trees, helper binaries, fixture, oracle and observations.

| Check | Observed result |
| --- | --- |
| Actual checked-in C helper, independently derived half-open bounds | 2,205 cases, including 1,505 invalid ranges. All pass after correction. The predecessor fails all 1,505 invalid cases because it continues into additional panics or output writes. |
| Fresh native helper build | Both product helpers build with `exclusive-native`; native tree `64975e04ec220dc16642ecd63f7b4429149cc9ce`, build identity `3d56112fccf3a9df`. |
| Existing source-location fixture | Corrected native executable exits zero and preserves the full stdout of the declared Rust oracle at `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. |
| Independent heap-lifetime observation on actual generated code | The predecessor leaves six observed allocations totaling 1,303 bytes. The corrected executable leaves five totaling 1,266 bytes. Exactly the extra 37-byte allocation disappears; the other observed allocations are unchanged. |
| Corrected generated code with UBSan | Exits zero without an undefined-behavior diagnostic. |

The small C-helper check is part of both Linux and macOS CI and saves its
source-bound result. The full existing sanitizer gate remains unchanged.

The heap observer wraps allocation calls at link time and records lifetimes
in static storage. It does not perform reachability analysis and is not a
LeakSanitizer replacement. Local LeakSanitizer cannot inspect a process under
the executor's ptrace environment; its failure is preserved. Hosted sanitizer
execution on the corrected combined source is still required. An initial
stable-Rust oracle had different standard-library source locations, so it was
excluded and its output retained separately before using the declared oracle.

There is no new performance measurement or full compiler qualification.
The build above qualifies this narrow native tree, before subsequent combined
integration changes. The aggregate integration patch and source guard must bind
the final combined native tree before publication.

## Separate macOS failure

The same predecessor's
[macOS job](https://github.com/glopyglerky/lustr/actions/runs/44493776529/job/135455703043)
reported a truncated daemon response for the warm `odd-path` crate fixture.
Its crate stage finished with 84 passes and one failure. That log does not
establish the daemon's cause of exit. Local Unix-domain socket creation fails
with `EPERM`, so this change makes no macOS or retained-daemon repair claim.
