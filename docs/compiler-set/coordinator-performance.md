# Unchanged coordinator project campaign

[`tools/measure-coordinator-project.py`](../../tools/measure-coordinator-project.py)
prepares and runs the real `crates/lustr` package through the existing
[`lustr-bench`](benchmark-method.md) paired runner. It adds no measured fixture
or second timing/statistics implementation. The package has its library, three
executables, integration tests and existing fixture projects; the measured source
comes directly from its Git subtree. This campaign needs a completed source-built
package on Linux x86-64. Preparing specifications or passing runner unit tests
does not establish a compiler result.

The two lanes are `matched-rust`, using the original SDK's Cargo and rustc, and
`lustr-mixed`, using the assembled package's primary `lustr` entry with its own
Cargo/rustc/rustdoc selectors. The SDK comes from the constructor's
`rustc-reference` link. Preflight requires identical Cargo, matching verbose
rustc identity and host, the protected full-Rust source pin, optimized component
receipts, and the primary entry's full-core identity. Compiler binaries and
runtime/sysroot inputs are hash guarded. These are measurements of this one
project and this compiler composition; they do not supply a plain Lust result.
The candidate's actual Cargo compiler is `bin/lustr-cargo/rustc`, a verified
link to the existing `bin/rustc-full` mixed compiler; its path, binary hash and
version are recorded as the compiler identity. The public `bin/rustc` launcher
remains available, but is not the compiler that Cargo executes in this lane.

## Exact work

The coordinator's source bytes, lockfile, default features, profiles and panic
behavior stay unchanged. No debug/profile override is inserted. Cargo's normal
dev and test profiles emit `-C debuginfo=2` here. That profile is outside the
current mixed-object eligibility boundary, so this campaign measures complete
project compatibility and overhead; it does not assert native object participation
or transfer any of upstream Lust's speed ratios. Native/retained-body speed still
needs the separately qualified supported corpus.

| Scenario | Timed Cargo arguments | State |
| --- | --- | --- |
| `clean-dependency-build` | `build --locked --offline -vv` | Owned Cargo artifacts, native/mixed caches and temporary state cleared before every warmup and measured command, after observed daemon shutdown. |
| `primed-root-build` | `build --locked --offline -vv` | Retained state; force the root with a byte-preserving `src/lib.rs` timestamp change. |
| `primed-root-check` | `check --locked --offline -vv` | Retained state; same root timestamp change; actual metadata-only compiler emission required. |
| `no-op` | `build --locked --offline -vv` | Retained state; source timestamps unchanged. |
| `compile-plus-test` | `test --locked --offline -vv -- --test-threads=1` | Retained state; force the root with the same byte-preserving timestamp change, then execute the normal complete test command. |

The forced scenarios are root rebuilds, not body/signature edits. The clean
scenario leaves operating-system page caches and dependency source caches warm.
The package currently has no registry dependencies; its integration tests still
execute their existing nested project builds. Two excluded warmup pairs precede
12 measured pairs in each scenario. The existing seeded cyclic-block schedule,
paired log ratios, 10,000-resample bootstrap and exact sign-flip calculation are
retained. Ratios in `analysis` are **matched Rust / Lustr**: above one favors
Lustr, below one favors matched Rust. Every timing remains in the export.

The ordinary executing project tests and structured `E0502` invalid-borrow
rejection gate run before and after measurement in both lanes. Existing
explicitly ignored tests remain ignored and their counts are disclosed. The
campaign neither filters tests nor treats a successful `--no-run` as execution.
Every forced sample must contain the actual root compiler command and shared
semantic flags. Source/input/tool guards, resolved Cargo output-directory
isolation and observed Cargo fingerprints must also pass. Failure removes
conclusions and retains the failed private evidence.
If the private runner fails, its driver prints only an allowlisted diagnostic
receipt (category, lane, phase and exit code) in CI. The original error text,
commands, source paths and output remain in the encrypted private report, not
the public log or performance conclusions.

## Owned daemon and cache scope

The generic runner enables this behavior only when its specification includes
`native_state.kind = "owned-lustr-mixed-v1"`. Historical specifications omit it
and retain their existing behavior. The policy names the source-built toolchain,
`COMPONENTS.txt` and `NATIVE-BUILD-RECEIPT.txt`. All five component binaries and
both receipts must be external guarded inputs; the manifest's optimized build
hash must agree with the native build receipt and actual binaries.

The runner owns private socket names and each lane's `TMPDIR`, `XDG_CACHE_HOME`,
`LUSTR_MIXED_CACHE_DIR` and `LUST_CACHE_DIR`. Lane specifications cannot override
these controls or disable native caching/daemons. Each command has a receipt for
the state it used. The shared daemon owner uses the same Linux child-subreaper
and documented socket-removal shutdown as the fixed mixed benchmark. It waits
for successful child exits before any state deletion; a failed exit or missing
shutdown refuses the cold transition. Warm commands retain state. Final
shutdown must succeed before the whole campaign can pass. A zero observed
daemon count records that observation without claiming native work occurred.

State creation/clearing, shutdown, hashing, correctness gates and statistics
are outside measured commands. A daemon lazily started during a compiler command
is included in that command's wall time. Resource collection is explicitly
disabled: this campaign makes no CPU, RSS, aggregate memory or energy claim.
Per-command host observations stay in the private originals; selected host facts
are captured before the campaign for the public export.

## Run and replay

After constructing and qualifying the source-built package, build the existing
runner, gate and replay example. Use a new output directory outside the repository.
The output parent must already exist. The default budget is two Cargo jobs; an
explicit `--jobs` value must fit the process's observed CPU affinity.

```bash
cargo build --manifest-path crates/lustr-bench/Cargo.toml --release --locked \
  --bin lustr-bench --bin lustr-bench-gate --example replay \
  --target-dir "$RUNNER_TEMP/lustr-bench-target"

cargo test --manifest-path crates/lustr-bench/Cargo.toml --locked \
  --test native_state --bin lustr-bench --bin lustr-mixed-bench \
  --target-dir "$RUNNER_TEMP/lustr-bench-tests"
python3 tools/test-coordinator-campaign.py \
  --replay "$RUNNER_TEMP/lustr-bench-target/release/examples/replay"

python3 tools/measure-coordinator-project.py \
  --repository "$GITHUB_WORKSPACE" \
  --toolchain "$RUNNER_TEMP/lustr-source-built/toolchain" \
  --runner "$RUNNER_TEMP/lustr-bench-target/release/lustr-bench" \
  --gate "$RUNNER_TEMP/lustr-bench-target/release/lustr-bench-gate" \
  --replay "$RUNNER_TEMP/lustr-bench-target/release/examples/replay" \
  --output "$RUNNER_TEMP/lustr-coordinator-campaign"

"$RUNNER_TEMP/lustr-bench-target/release/examples/replay" \
  "$RUNNER_TEMP/lustr-coordinator-campaign/public/samples.json"
```

`--prepare-only` performs identity checks and writes the exact frozen source and
five specifications. It reports `prepared-only` and creates no measured result.
The paired campaign requires normal Unix-socket and child-process permissions;
an environment that denies native daemon sockets cannot qualify that behavior.

### CI execution

The [source-built job](../../.github/workflows/compiler-set.yml) now builds and
tests the runner, gate and replay example, then executes the five scenarios
after source construction, target-layout checks, the failed-link negative
control, mixed execution, retained checks, mixed libraries and the complete
entry/runtime contract all pass. The host collector and project-runner checks
must also pass. A failed profiling or separate fixed-fixture measurement step
does not mask that failure or prevent an independent qualified project campaign.

The separate `coordinator-project-performance` artifact explicitly includes
only `public/README.md`, `campaign.json`, `samples.json`, `replay.json`,
`host.json` and `SHA256SUMS`. The public directory is created only after every
scenario passes. The upload never includes `private/`, and the workflow test
checks its exact file allowlist. Keep this artifact separate from the
byte-identical historical fixed-fixture archives.

A separate `always()` step seals the available private originals after either
success or failure. Its upload explicitly includes only the ciphertext,
receipt and allowlisted status described below. The private recipient key is
absent from the workflow. A missing archive or encryption failure remains a
failed step; `always()` preserves evidence without converting the campaign
failure into success. Interrupted runner shutdown can still prevent all final
steps, so recovery is claimed only after the actual artifact is retrieved and
verified.

The focused regression workflow also exercises ownership and public-export
validation when the runner or project-campaign tools change. It also exercises
relocated recovery, original/public tampering and Git replacement-ref handling
with the existing statistical replay executable. These harness
tests are separate from successful real package execution and speed evidence.

## Private originals and public evidence

`private/` contains source, exact specifications, raw output, full reports and
runner logs. The generic runner's [private-report policy](benchmark-method.md)
still applies. Do not commit or publicly upload that directory. The workflow
uses the authenticated-encryption retention below; a temporary CI directory
alone does not establish durable retention. Preserve the retrieved encrypted
artifact and its receipt beyond Actions expiry, and verify decryption before
describing originals as recoverable.

`public/` is created only after all five reports pass consistency checks and the
existing Rust replay verifies every comparison. It contains all raw **timing
values**, command/gate statuses, actual execution order, scenario/command
contracts with symbolic paths, test counts including ignores, cache/daemon
receipts with private paths removed, source/tool identities, selected host
facts, original report/spec/output SHA256 values, and replayed statistics. It
omits raw command output, absolute workspace/socket paths, environment values,
source files and test names. It is a derived projection, not a byte-identical
copy of the full originals. Original hashes identify those private bytes;
`public/SHA256SUMS` separately hashes the rewritten public files. The public
timings are sufficient to replay the published comparisons, while full log
inspection requires the private originals.

Ownership tests run subreaper lifecycles in separate processes. They verify warm
retention, successful-exit-before-clear, refusal of failed exits and symlink
escapes, and optimized component-receipt binding. Export tests remove shutdown,
cache, constructor and Cargo fingerprint evidence, corrupt raw output, change
profiles, and remove paired samples. Synthetic verifier durations are explicitly
test data and are never published as compiler measurements.

## Encrypted retention, including failed campaigns

Run [`encrypt-coordinator-evidence.py`](../../tools/encrypt-coordinator-evidence.py)
after the campaign step even when measurement or setup fails. This closes the
private-original retention gap without uploading plaintext reports. The helper
uses the standard OpenSSL CMS command with AES-256-GCM, RSA-OAEP, explicit SHA-256
for both OAEP and MGF1, binary content and DER output. OpenSSL documents that
GCM selects CMS `AuthEnvelopedData`, and binary mode prevents MIME newline
translation. Its CMS `-recip`/`-keyopt` interface supports the explicit OAEP
parameters. Sources: [CMS encryption](https://docs.openssl.org/3.0/man3/CMS_encrypt/),
[CMS command](https://docs.openssl.org/3.5/man1/openssl-cms/), and
[RSA OAEP options](https://docs.openssl.org/3.5/man1/openssl-pkeyutl/#rsa-algorithm).

Only a public RSA recipient certificate and its independently selected SHA256
fingerprint enter the helper. The recipient's private key stays outside the
repository and hosted runner. Keep that private key in durable private storage
before relying on the encrypted artifact for recovery. The current dedicated
public certificate is `tools/benchmark-evidence-recipient.pem`.

```bash
python3 tools/test-encrypted-evidence.py
python3 tools/encrypt-coordinator-evidence.py \
  --campaign "$RUNNER_TEMP/lustr-coordinator-campaign" \
  --certificate "$GITHUB_WORKSPACE/tools/benchmark-evidence-recipient.pem" \
  --expected-certificate-sha256 69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f \
  --output "$RUNNER_TEMP/lustr-coordinator-campaign/encrypted"
```

Upload only these fixed files from the separate `encrypted/` directory:

- `private-evidence.tar.gz.cms` — authenticated encrypted archive.
- `receipt.json` — original archive and ciphertext hashes/sizes, encrypted
  manifest hash, original file count/bytes, public certificate/SPKI fingerprints
  and explicit algorithm identifiers.
- `status.json` — allowlisted retention/campaign/scenario states. Original errors,
  filenames and paths are not copied into this public status.

The archive contains the original `identity.json`, `host.json`,
`campaign-result.json`, `reject.rs`, the five specifications, the frozen snapshot,
runner stdout/stderr and the five scenarios' direct reports/specifications/raw
streams, including interrupted `.pending` writes. Existing request/usage records
are included if present. File bytes and permission bits are preserved. Targets,
native/mixed caches, home/Cargo-home directories, derived public projections and
compiler binaries are excluded. The encrypted manifest records the exact selected
relative paths, byte counts and per-file SHA256 values. Archive headers normalize
account names and timestamps; the receipt's original archive hash identifies
these exact generated tar/gzip bytes, not an earlier report or public projection.

Missing or incomplete result JSON does not prevent retention of the existing
originals. If no private evidence exists, the helper returns nonzero and writes
only `status.json` with `not-retained`; it never invents an archive receipt.
Encryption failure also leaves only an allowlisted failure status. A ciphertext
and receipt appear only after encryption and archive checks succeed. Existing
outputs are never overwritten. Plaintext staging stays under `private/` and is
removed on ordinary success or failure. The original private files remain there;
only the ciphertext and allowlisted JSON files belong in the artifact upload.

### Recovery

Download the encrypted artifact from the intended CI run. Verify its ciphertext
hash against `receipt.json`, then decrypt into a private temporary directory.
OpenSSL can write output before discovering an authentication failure: neither
an existing output file nor readable tar bytes indicate successful decryption.
Require exit zero and the helper's archive/manifest verification before extraction.
For example, with the three path variables set to the intended locations:

```bash
set -eu
umask 077
python3 - "$ENCRYPTED_DIRECTORY" <<'PY'
import hashlib, json, pathlib, sys
root = pathlib.Path(sys.argv[1])
receipt = json.loads((root / "receipt.json").read_bytes())
with (root / "private-evidence.tar.gz.cms").open("rb") as stream:
    assert hashlib.file_digest(stream, "sha256").hexdigest() == receipt["ciphertext_sha256"]
PY
RECOVERY_STAGE=$(mktemp -d "${TMPDIR:-/tmp}/lustr-recovery.XXXXXXXX")
trap 'rm -rf -- "$RECOVERY_STAGE"' EXIT HUP INT TERM
openssl cms -decrypt -binary -inform DER \
  -in "$ENCRYPTED_DIRECTORY/private-evidence.tar.gz.cms" \
  -recip tools/benchmark-evidence-recipient.pem \
  -inkey "$PRIVATE_RECIPIENT_KEY" -out "$RECOVERY_STAGE/originals.tar.gz"
python3 tools/encrypt-coordinator-evidence.py \
  --verify-archive "$RECOVERY_STAGE/originals.tar.gz" \
  --receipt "$ENCRYPTED_DIRECTORY/receipt.json"
mkdir -m 700 -- "$RESTORE_DIRECTORY"
tar --same-permissions -xzf "$RECOVERY_STAGE/originals.tar.gz" -C "$RESTORE_DIRECTORY"
```

Restore the verified permission bits explicitly. The private parent directory
remains mode 0700, but a restrictive extraction umask or a tar data filter can
otherwise change ordinary archived modes such as 0664 to 0644. Reconciliation
checks the original modes as well as bytes and correctly rejects that change.

### Reconcile the recovered originals with the public report

For a successful five-scenario campaign, run
[`tools/reconcile-coordinator-evidence.py`](../../tools/reconcile-coordinator-evidence.py)
after the authenticated decryption and archive verification above. Keep the
restored file modes intact. `RESTORE_DIRECTORY` must contain `MANIFEST.json`
and `private/`; `PUBLIC_DIRECTORY` contains the six allowlisted public files.
`SOURCE_REPOSITORY` must be a Git repository containing the campaign's exact
recorded revision. Restore that source from the intended run's retained source
evidence if necessary. The helper performs no fetch, checkout, extraction or
compilation and has no private-key argument.

Build the existing `crates/lustr-bench` replay example from the recorded source
revision, or use a separately verified copy. Point `REPLAY_EXECUTABLE` at it:

```bash
python3 tools/reconcile-coordinator-evidence.py \
  --restored "$RESTORE_DIRECTORY" \
  --receipt "$ENCRYPTED_DIRECTORY/receipt.json" \
  --public "$PUBLIC_DIRECTORY" \
  --repository "$SOURCE_REPOSITORY" \
  --replay "$REPLAY_EXECUTABLE"
```

The helper freshly hashes every recovered manifest original, checks sizes and
modes, and verifies the exact file inventory. It verifies every public file
against `SHA256SUMS`. All five report/specification digests and every raw
stdout/stderr digest must agree with the public cases and private campaign
result. It recomputes the frozen source's SHA-256 tree and checks its file
inventory, Git blob identities and executable modes against the recorded
coordinator Git subtree.

The live exporter normally needs its original absolute paths and installed
compiler binaries. Reconciliation preserves its original logical path
comparisons while redirecting private-file reads to the restored tree. It
loads the exporter from the recorded revision's Git blob, so an unrelated or
modified working-tree exporter cannot replace it. Source paths cannot escape
the restored tree through parent links or `..`. The regenerated public cases
must match semantically, `samples.json` and the README must reproduce byte for
byte, and campaign/host data must agree. The existing Rust replay verifies all
five comparisons; its JSON receipt must also reproduce semantically.

`source_identity` identifies the exact source revision/tree, exporter blob and
SHA-256, runner source tree, recorded runner binary hash, and local and recorded
replay binary hashes. Original reports must agree with the campaign's runner
identity. A local replay build may have a different binary hash; the report
states whether it matches the recorded binary. It does not claim that the
helper established a binary-to-source build proof. Campaign version 1 did not
record an exporter runtime hash, so that field remains explicitly null even
though the expected exporter source is verified and used.

Omitted compiler, runtime and constructor files receive recorded-identity
consistency checks. Their bytes are unavailable in this archive and are never
claimed freshly hashed. The result explicitly reports
`compiler_binaries_freshly_rehashed: false` and
`runner_binary_freshly_rehashed: false`. No source, original evidence or public
file is rewritten; failures return fixed check categories without private
paths or raw exceptions.

This step requires a complete public projection. For a failed or partial
campaign, the authenticated archive/manifest verification and allowlisted
retention status establish what was retained; a complete public performance
report is unavailable. Reconciliation does not execute compiler gates or
measure a new campaign.

The focused tests reuse the existing synthetic campaign-verifier fixture,
delete its original host tree before reconciliation, and exercise source
binding, relocation, private/public tampering, replay-receipt tampering and
path rejection. They need only the existing replay executable:

```bash
python3 tools/test-reconciled-evidence.py --replay "$REPLAY_EXECUTABLE"
```

### Encryption and recovery tests

Focused tests use temporary RSA keys and actual OpenSSL calls. They verify exact
round trips of failed/partial evidence, exclusion of build state, path-free
receipts, recipient-pin mismatch, refusal of evidence symlinks, no overwrites,
and rejection plus temporary-output cleanup for a wrong private key or changed
GCM authentication tag. The tests also inspect the generated CMS structure for
AuthEnvelopedData, AES-256-GCM, RSA-OAEP and both SHA-256 parameters. They provide
no benchmark timings or compiler-performance claims.
