# Retained original project artifacts: recovery pending

The actual unchanged `crates/lustr` project benchmark completed five scenarios
with twelve measured pairs per scenario in [source-built run 44187580063,
job 134408105819](https://github.com/glopyglerky/lustr/actions/runs/44187580063/job/134408105819).
These two original GitHub ZIP artifacts are preserved byte for byte, separately
from the three earlier *fully recovered* project campaigns:

| Original artifact | GitHub artifact | SHA-256 |
| --- | --- | --- |
| [Public projection](public-original.zip) | [14378396776](https://github.com/glopyglerky/lustr/actions/runs/44187580063/artifacts/14378396776) | `b334d34241552b4cd583345c49eb83cb6e62ac9ed1322d4a5dfa8b85479827ae` |
| [Encrypted originals](encrypted-original.zip) | [14376978784](https://github.com/glopyglerky/lustr/actions/runs/44187580063/artifacts/14376978784) | `aa71fedf06ac9d6fcc1ae4271dbc1a9d39d96c10c746f4d47bbe395dd757fddc` |

This measures PR head `3e24029`, actual merge checkout `5711bb9` and source
tree `f083723`. It uses normal dev/test debug=2, where mixed native objects are
not currently eligible. The public projection contains the original five-case
summary, 120 timed observations, host facts, execution order, paired statistics
and original stream digests. All five measured ratios favor matching Rust:

| Unchanged project scenario | Matched Rust median (ms) | Lustr median (ms) | Paired ratio [95% interval] |
| --- | ---: | ---: | --- |
| Clean dependency build | 571.655 | 578.141 | 0.9913 [0.9875, 0.9950] |
| Primed root build | 187.757 | 193.076 | 0.8521 [0.6948, 0.9847] |
| Primed root check | 74.138 | 76.333 | 0.9573 [0.9467, 0.9672] |
| No-op build | 10.933 | 12.022 | 0.7849 [0.5716, 0.9304] |
| Compile plus test | 5817.348 | 5884.797 | 0.9901 [0.9857, 0.9952] |

The paired ratio is the geometric mean of reference/candidate times, **not**
the quotient of the displayed medians. The primed build had a 531.983 ms
Lustr p95 outlier and no-op had a 76.868 ms Lustr p95 outlier. They are not
excluded. A different shared runner and source build separate this campaign
from previous project measurements; between-run changes are not a causal
speedup/regression estimate. These results do not measure source edits or prove
general Cargo acceleration.

**Recovery boundary:** The original encrypted ZIP contains an authenticated
AES-256-GCM CMS ciphertext and retention receipts for 600 private original
files; its RSA-OAEP recipient key is intentionally not stored here. Public
archive and ciphertext checks pass without that key, but authenticated
decryption, exact original-file recovery and source-bound reconciliation are
**not done**. Until completed, this campaign must not be counted among the
three authenticated recovered project campaigns. Do not publish plaintext
originals or an authorized private recipient key.

From the repository root, check retained bytes, identities and source-bound
public consistency without executing a recorded command:

```sh
python3 tools/check-pending-project-archives.py
```
