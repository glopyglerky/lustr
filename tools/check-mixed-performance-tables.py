#!/usr/bin/env python3
"""Replay committed timing projections and tables, not compiler qualification.

Full artifact verification remains report-mixed-performance.py. This offline
check keeps documentation reproducible after remote artifact expiry; it does
not recreate missing logs or prove their authenticity from a CSV alone.
"""
import csv
import importlib.util
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("report_mixed", ROOT / "tools/report-mixed-performance.py")
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)


def checked_file(path, digest):
    relative = r.Evidence.relative(path)
    file = (ROOT / str(relative)).resolve()
    r.require(file.is_relative_to(ROOT), "campaign path escapes repository")
    data = file.read_bytes()
    r.require(r.sha256(data) == digest, f"changed committed campaign file: {path}")
    return data


def main():
    manifest = r.json_load((ROOT / "performance/campaigns.json").read_bytes())
    r.require(manifest["schema"] == "lustr-committed-campaigns-v1", "unknown campaign manifest")
    total = values = 0
    for campaign in manifest["campaigns"]:
        n, seed = campaign["pairs"], campaign["seed"]
        lanes_by_family, comparisons = r.campaign_layout(campaign.get("campaign_schema", 1))
        r.require(r.integer(n, 2, 10000) and r.integer(seed, 0, r.MASK), "invalid replay controls")
        data = checked_file(campaign["csv"], campaign["csv_sha256"])
        rows = list(csv.reader(data.decode("utf-8").splitlines()))
        r.require(rows[0] == ["series"] + [f"pair_{i}_seconds" for i in range(n)], "CSV header mismatch")
        samples = {}
        for row in rows[1:]:
            r.require(len(row) == n + 1 and row[0] not in samples, "duplicate or malformed series")
            samples[row[0]] = [float(x) for x in row[1:]]
            r.require(all(r.positive(x) for x in samples[row[0]]), "invalid duration")
        expected = {f + "/" + lane + "/" + phase for f, lanes in lanes_by_family.items()
                    for lane in lanes for phase in r.PHASES}
        r.require(set(samples) == expected, "missing or unexpected series")
        summaries = []
        for family, baseline, candidate in comparisons:
            for phase in r.PHASES:
                pairs = list(zip(samples[f"{family}/{baseline}/{phase}"],
                                 samples[f"{family}/{candidate}/{phase}"]))
                analysis = r.summarize(pairs, campaign["bootstrap_resamples"], seed)
                low, high = analysis["bootstrap_95"]
                observation = "faster-on-this-corpus" if low > 1 else "slower-on-this-corpus" if high < 1 else "inconclusive"
                summaries.append(dict(family=family, baseline=baseline, candidate=candidate,
                                      phase=phase, analysis=analysis, observation=observation))
        expected_text = r.markdown(dict(campaign, summaries=summaries)).encode("utf-8")
        actual = checked_file(campaign["table"], campaign["table_sha256"])
        r.require(actual == expected_text, "committed table differs from timing replay")
        total += len(summaries)
        values += sum(map(len, samples.values()))
        print(f"{campaign['head']}: {len(summaries)} tables replayed; no compiler admission")
    print(f"Verified {total} comparisons from {values} committed timing values")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, OverflowError) as exc:
        print(f"committed performance evidence rejected: {exc}", file=sys.stderr)
        raise SystemExit(1)
