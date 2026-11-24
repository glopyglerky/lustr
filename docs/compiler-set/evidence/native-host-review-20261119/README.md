# Independent native host-script review

This record preserves actual failures found while reviewing the native
build-script integration and the bounded checks of their corrections. It covers
Linux public one-shot compilation through two source-built partial packages.
It does not qualify a retained daemon campaign, guarded package construction,
full Rust compilation, overall speed or release admission.

The final source review used native host module SHA-256
`bd6109f50094e4f51045f08bfb0973f08678e5f602b596b669d17134dbd931ca`
and cfg module SHA-256
`365c30bf756fdc3971ac0cc16451e569a278af75d8b69d8950464b40b7aff701`.
The [verification record](verification.json) binds both package identities and
the final source scope. The
[source owner's evidence](../native-host-script-20261119/) records implementation
controls and the separate timestamp witnesses.

## Four demonstrated stale results

Every row used an unchanged valid Rust script and consumer. The edit changed a
filesystem observation. The independent oracle was the exact declared Cargo and
Rust SDK; its compiler remained outside the product path.

| Observation changed | Original package v2 result | Independent Rust result | Corrected package v3 result |
| --- | --- | --- | --- |
| Add an empty directory beneath a declared directory input; the script counts its entries | `0` then `0` | `0` then `1` | Both produce `0` then `1` |
| Create a formerly absent declared file with the literal bytes `<missing>`; the script tests existence | `false` then `false` | `false` then `true` | Both produce `false` then `true` |
| Change only a declared input file's modification time; the script reads that time | Original timestamp repeated | Updated timestamp | Both report the updated timestamp |
| Change only `build.rs` modification time; the script reads its own time and emits only a rerun-on-environment directive | Original timestamp repeated | Updated timestamp | Both report the updated timestamp |

The first bug came from hashing descendant files while omitting empty directory
entries. The second was a deterministic encoding alias: the old absence marker
had exactly the same hash input as an ordinary file containing those marker
bytes. These observations are distinct from a probabilistic hash collision.

The corrections include directory names and entry types, preserve original OS
filename bytes, and give missing inputs and file contents separate encodings.
Modification times join explicit input snapshots. Actual host source timestamps
are recorded separately as implicit script-execution dependencies, so an
unchanged compiled binary can still be reused when its script must run again.
Timestamp comparison uses the observed value, including future values; it does
not infer freshness from whether an update happened before a wall-clock cutoff.

The final two directory/missing-file checks made **16 actual commands**: each
case compiled and executed both native and independent Rust artifacts before
and after its edit. Every program's complete stdout, stderr and exit status
matched. Each public native request recorded one Lust ownership event. Source
hashes were unchanged, and artifact hashes were checked around execution. Both
native script executables retained their hash, inode, modification time and
size across the edit: the runtime correction did not republish the compiled
script. The package's binary hashes also remained unchanged.

The final
[cache report](final-cache-report.json) contains those results. Both original
failures, original native records and final native records are in
[the witness archive](witnesses.tar.gz). The two original timestamp failures and
their **16 final commands** are preserved separately in the source owner's
[timestamp evidence](../native-host-script-20261119/timestamps/). The final
timestamp aggregate's original hash is recorded in this verification file.
That separate record includes the original implicit-source witness; it is not
replaced by the later passing result.

## Other bounded observations, kept at their actual revision

The following checks used **package v2**, before the final directory and
timestamp corrections. They were not relabelled as v3 results:

- An unchanged standalone Rust program using `cfg!(target_feature="x87")`
  produced `true` through both public native compilation and pinned Rust. The
  earlier partial package's `false` result is retained in the archive.
- Three selected compile/runtime environment witnesses covered empty features,
  explicit feature names with inherited sentinel variables, and
  `CARGO_BUILD_JOBS=2`. The latter observed `NUM_JOBS=None` at script compilation
  and `NUM_JOBS="2"` at script execution, matching pinned Cargo.
- The observed environment differences remain visible: four reliable-f16/f128
  cfg capabilities were absent from native output, and nine fields identify
  the different fixture, tool or output locations. This is a comparison of the
  explicitly collected fields, not complete Cargo environment equivalence.
- A script that reads `before`, writes `after`, then emits its old observation
  produced `before` on the first native request and `after` on the next. This
  checks the native record contract: bytes observed after execution cannot
  certify an earlier result as reusable.

That last self-write example is **not a claimed native-versus-Cargo differential
win**. The original immediate pinned-Cargo experiment also returned `before`
twice. Both original native and oracle runs are preserved. The later native
result establishes the corrected native reuse contract, with its own package
identity; it does not rewrite the original Cargo observation.

The x87 and environment observations do not establish missing backend
capabilities. The reliable-float flags remain unproved and absent. Requested
default UB configuration is not proof of UB instrumentation, and a pinned
target-default relocation cfg is not proof that a native artifact is PIE/PIC.

## Source review scope

The reviewed product host path compiles the original planned host unit through
the existing native preparation/emission pipeline. Daemon routing does not
execute it. Failed compile, execution, directive, source or publication work
has one terminal outcome; none transfers the unit to another compiler.

The native directive boundary checks the actual lexer tokens and written text
before passing custom bare cfg names or plain string values to the existing cfg
service. Malformed or unrepresented raw/escaped values, builtins, separately
owned feature/test settings and richer check-cfg forms fail explicitly. Their
refusal is an unfinished integration boundary, not complementary ownership.

Compile reuse binds actual checked source and expansion bytes, environment reads,
the script executable and negative module lookups. Runtime reuse separately
binds declared input observations, implicit source times, client values and
outputs. A successful run whose runtime inputs cannot yet be proved returns its
actual output without a reuse certificate; a later logical request may execute
the same compiled script once. The output lease remains held by the checked
consumer through emission. These checks retain the compiled-code reuse boundary
while repairing the demonstrated stale results.

No additional blocking finding remained in these reviewed changes after the
four corrections. This statement is limited to the source and actual witnesses
above. Broader native, retained, platform and full-compiler qualification remain
separate requirements.

## Files and reproduction scope

- [Verification and package identities](verification.json)
- [Final directory/missing-file paired runs](final-cache-report.json)
- [Original/published stream hashes](stream-hashes.json)
- [Source fixtures, drivers, original failures and result streams](witnesses.tar.gz)
- [Implementation controls and timestamp records](../native-host-script-20261119/)

The archive contains **239 files** and no compiled binaries or build caches.
Local path prefixes were replaced with generic labels, including path literals
in reproduction drivers. Actual Rust fixture source and tested semantic values
are unchanged. Original and published hashes remain paired for every archived
file. The retained fixtures and drivers make each edit and command reviewable;
these manual partial-package results do not attest source correspondence for a
later published constructor output.
