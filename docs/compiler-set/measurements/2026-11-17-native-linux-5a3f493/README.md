# Full native Linux gate at `5a3f493`

The [original Linux job](https://github.com/glopyglerky/lustr/actions/runs/44111327211/job/134137399589)
failed after **5,527 ordinary differential passes and 5,526 sanitized passes**.
The remaining sanitized case, `unicode_props_hash`, reached its unchanged
ten-second generated-program execution deadline. Fuzz, benchmark smoke and the
final completion step did not run. This is a second recorded failure at that
case, following the [earlier `40d2ac3` gate](../2026-11-17-native-linux-40d2ac3/README.md).

The separate source-built mixed compiler job passed its own correctness,
profiling and timing requirements. That result does not change this gate's
failure. The native macOS job was cancelled without executing any steps.

## Exact original evidence

| Identity | Value |
| --- | --- |
| Candidate head | `5a3f4937d94e8f028dd85385ea65d51fcfe35a3b` |
| Actual checkout | `b3393a4a2907228516546cd34f1a5c60aaf6aaaa` |
| Repository tree | `2fca60e57829890b10a4a23da57bafab2581b092` |
| Native subtree | `cb797ce8ce82192b87b3835267fc223c295dba70` |
| Run / job | `44111327211` / `134137399589` |
| Original artifact | `14348831217` |
| ZIP bytes | `14321` |
| ZIP SHA-256 | `1f037771e512b8ac83d8e22446ad2645757e47703bc0a221fa7af59500360b90` |

[native-gate-original.zip](native-gate-original.zip) is byte-identical to the
downloaded GitHub Actions artifact. Its digest matches GitHub's artifact
metadata and the upload log. [manifest.json](manifest.json) records the exact
three-file inventory, hashes, reached suites and source inputs. The recorded
Cargo manifests, native build script and CI script were checked against their
exact Git blobs. No generated sanitizer program or compiler executable is
included in this original archive; their unavailable bytes are not claimed
as recovered.

## What executed

| Gate | Recorded result |
| --- | --- |
| Diagnostic registry | 112 codes in use, 108 reserved, zero failures |
| Port tool tests | 13 passes |
| Ordinary differential cases | 5,527 passes, zero failures |
| Macro corpus using the Rust reference | 58 passes; 70/70 named macro fixtures |
| Native macro dependency cases | 13 passes |
| Kernel and compatibility cases using Rust/Cargo | Passed |
| Native client cases | 89 passes |
| Native crate fixtures | 85 passes |
| Daemon edit scenarios | 363/363; 28 unsupported versions reported |
| Slot identities | Passed |
| Sanitized differential cases | 5,526 passes, one failure |
| Fuzz / benchmark smoke / completion | Not reached |

The inherited diagnostic says `sanitized build timed out`, but the recorded
script reaches that message after GCC linking succeeds and the generated
program runs through `run_to`. The failure is an execution timeout, not a
GCC compilation timeout. The original diagnostic remains unchanged in the ZIP.

The native subtree equals the earlier failed gate's subtree. This neither
proves the cause nor permits either result to inherit an older passing gate.
The earlier bounded local attempt encountered a LeakSanitizer environment
failure; it did not reproduce or resolve this hosted timeout. No test was
removed, no sanitizer was disabled and no deadline was relaxed to publish
this record.

## Verify the retained archive

From this directory:

```sh
python3 - <<'PY'
import hashlib, json, zipfile
from pathlib import Path
m = json.loads(Path('manifest.json').read_text())
p = Path(m['archive'])
if p.stat().st_size != m['archive_bytes']:
    raise SystemExit('archive size mismatch')
if hashlib.sha256(p.read_bytes()).hexdigest() != m['archive_sha256']:
    raise SystemExit('archive digest mismatch')
with zipfile.ZipFile(p) as z:
    if z.namelist() != [f['path'] for f in m['files']]:
        raise SystemExit('archive inventory mismatch')
    for f in m['files']:
        raw = z.read(f['path'])
        if len(raw) != f['bytes'] or hashlib.sha256(raw).hexdigest() != f['sha256']:
            raise SystemExit('member mismatch: ' + f['path'])
print('Original failed gate archive verified; no compiler execution implied.')
PY
```
