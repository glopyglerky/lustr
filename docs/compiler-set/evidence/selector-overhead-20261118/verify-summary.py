#!/usr/bin/env python3
"""Check this public projection; original private process outputs are not replayed."""
import hashlib
import json
import math
import re
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[3]


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text())


def canonical(value):
    return json.dumps(value, separators=(",", ":"), sort_keys=True).encode()


manifest = read("MANIFEST.json")
actual_files = {
    str(path.relative_to(ROOT))
    for path in ROOT.rglob("*") if path.is_file() and path.name != "MANIFEST.json"
}
require(actual_files == set(manifest["files"]), "Public evidence file set differs")
for name, identity in manifest["files"].items():
    data = (ROOT / name).read_bytes()
    require(len(data) == identity["bytes"] and digest(data) == identity["sha256"], f"Public file changed: {name}")

summary = read("summary.json")
samples = read("samples.json")
require(summary["schema"] == "lustr-selector-public-summary-v1", "Unexpected summary schema")
require(samples["schema"] == "lustr-selector-public-samples-v1", "Unexpected sample schema")
require(samples["source"] == summary["source"] == "43a7e07d32929309fc6e402d7bca6b5673e186a2", "Wrong measured source")
require(samples["helper_sha256"] == summary["helper_sha256"], "Helper identities differ")
require(re.fullmatch(r"[0-9a-f]{64}", summary["helper_sha256"]), "Malformed helper identity")
require(summary["schedule"] == {
    "total_commands": 53, "measured_pairs": 24, "measured_commands": 48,
    "excluded_primes": 3, "fresh_boundary_controls": 2,
    "pair_order": "even: cached,fresh; odd: fresh,cached", "dropped_measured_samples": 0,
}, "Schedule differs")

labels = ["fresh-before", "prime-0", "prime-1", "prime-2"]
order = []
for pair in range(24):
    for lane in (("cached", "fresh") if pair % 2 == 0 else ("fresh", "cached")):
        labels.append(f"measured-{pair}-{lane}")
        order.append((pair, lane))
labels.append("fresh-after")
commands = samples["all_commands"]
require([row["label"] for row in commands] == labels, "Command order differs")
require([row["command_index"] for row in commands] == list(range(53)), "Command indices differ")
empty = digest(b"")
selection_sha = summary["local_validation"]["selection_sha256"]
for row in commands:
    require(row["exit"] == 0 and row["stderr_sha256"] == empty, "Projected command failure")
    require(type(row["elapsed_ns"]) is int and row["elapsed_ns"] > 0, "Invalid component duration")
    require(type(row["wall_ns"]) is int and row["wall_ns"] >= row["elapsed_ns"], "Invalid wall duration")
    require(row["selection_sha256"] == selection_sha, "Projected selection hashes differ")
    require(row["measured"] == row["label"].startswith("measured-"), "Wrong distribution membership")
rows = samples["samples"]
require([(row["pair"], row["lane"]) for row in rows] == order, "Pair order or membership differs")
require([row["elapsed_ns"] for row in rows] == [row["elapsed_ns"] for row in commands if row["measured"]], "Samples differ from command projection")
for lane in ("cached", "fresh"):
    values = sorted(row["elapsed_ns"] for row in rows if row["lane"] == lane)
    require(samples["distribution"][lane] == {
        "n": len(values), "minimum_ns": values[0], "median_ns": statistics.median(values),
        "p95_ns": values[math.ceil(0.95 * len(values)) - 1], "maximum_ns": values[-1],
    }, f"Wrong {lane} distribution")

trace = samples["excluded_trace_attempt_commands"]
require([row["label"] for row in trace] == labels[:4] + ["cached-strace"], "Trace failure history differs")
require(all(row["exit"] == 0 for row in trace[:4]) and trace[-1]["exit"] == 1, "Wrong trace statuses")
require(trace[-1]["elapsed_ns"] is None and trace[-1]["stdout_sha256"] == empty, "Failed trace was counted")
require(summary["trace"]["syscall_counts"] is None and summary["trace"]["trace_bytes"] == 0, "Unsupported syscall observations")
require(summary["environment"]["selection_diagnostics"] is False, "Timing instrumentation differs")
require(summary["environment"]["fresh_adds"] == {"LUSTR_NO_SELECTION_CACHE": "1"}, "Wrong fresh control")

# Check recorded source hashes against the exact measured commit, even if the
# current checkout has since acquired independent correctness changes.
source = summary["source"]
tree = subprocess.check_output(["git", "rev-parse", f"{source}^{{tree}}"], cwd=REPO, text=True).strip()
require(tree == summary["source_tree"], "Wrong source tree")
for path, expected in summary["helper_source_sha256"].items():
    data = subprocess.check_output(["git", "show", f"{source}:{path}"], cwd=REPO)
    require(digest(data) == expected, f"Measured source hash differs: {path}")
fixture_root = REPO / summary["fixture"]["repo_path"]
fixture = {
    str(path.relative_to(fixture_root)): digest(path.read_bytes())
    for path in sorted(fixture_root.rglob("*")) if path.is_file()
}
require(len(fixture) == summary["fixture"]["file_count"] and digest(canonical(fixture)) == summary["fixture"]["file_hash_map_sha256"], "Fixture differs")
print("Verified public projection: 53 commands, 24 complete pairs, every numeric sample, distributions, source hashes and fixture identity.")
print("Private original outputs and omitted binaries are not independently replayed by this verifier.")
