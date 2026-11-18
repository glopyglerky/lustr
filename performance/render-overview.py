#!/usr/bin/env python3
"""Render the dated overview chart from retained, source-bound dashboard data."""
import json
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

ROOT = Path(__file__).resolve().parent
SNAPSHOT = ROOT / "2026-11-18-43a7e07"
report = json.loads((SNAPSHOT / "dashboard/report.json").read_text())
rows = {(family["name"], row["operation"]): row
        for family in report["families"] for row in family["rows"]}
plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 11,
                     "svg.fonttype": "none", "svg.hashsalt": "lustr-43a7e07"})
fig, axes = plt.subplots(2, 2, figsize=(11.8, 6.3))
fig.patch.set_facecolor("#f8fafc")
fig.subplots_adjust(top=.81, bottom=.13, left=.14, right=.94, hspace=.85, wspace=.48)
fig.text(.05, .94, "lustr | compiler latency", fontsize=22, weight="bold", color="#142136")
fig.text(.05, .89, "Warm native work stays close to Lust. Mixed Cargo gains depend on the operation.",
         fontsize=11, color="#526276")
panels = [("public-slice", "no-op-check", "Warm subset check", 10),
          ("public-slice", "no-op-build", "Warm subset build", 10),
          ("fixed-cargo", "body-edit", "Mixed Cargo body edit", 50),
          ("fixed-cargo", "cold", "Mixed Cargo cold build", 170)]
colors = {"lustr": "#7253dc", "lust": "#138b9b", "rustc": "#68798e"}
names = {"lustr": "Lustr", "lust": "Pristine Lust", "rustc": "Cargo/rustc"}
for ax, (family, operation, title, limit) in zip(axes.flat, panels):
    row = rows[family, operation]
    lanes = [lane for lane in ("lustr", "lust", "rustc") if row[lane]["status"] == "MEASURED"]
    values = [row[lane]["median_ms"] for lane in lanes]
    ax.set_facecolor("#f8fafc")
    ax.barh(range(len(lanes)), values, height=.53, color=[colors[lane] for lane in lanes])
    ax.set_yticks(range(len(lanes)), [names[lane] for lane in lanes])
    ax.invert_yaxis()
    ax.set_xlim(0, limit)
    ax.set_title(title, loc="left", pad=14, weight="bold", color="#142136", fontsize=12)
    for y, value in enumerate(values):
        ax.text(value + limit * .02, y, f"{value:.3f} ms", va="center", fontsize=10, color="#142136")
    ax.xaxis.set_major_locator(MaxNLocator(4))
    ax.grid(axis="x", color="#e2e8f0", linewidth=.7)
    ax.set_axisbelow(True)
    ax.tick_params(axis="both", length=0, labelcolor="#526276")
    for spine in ax.spines.values():
        spine.set_visible(False)
fig.text(.05, .055, "Median wall time, lower is faster. Each panel has its own scale. PR #26 · 43a7e07d · Linux x86-64",
         fontsize=9, color="#526276")
fig.text(.05, .025, "Adapted supported slice and small mixed Cargo fixture, dev opt0/debug0. Full rows, p95 and paired intervals accompany the chart.",
         fontsize=8.5, color="#526276")
out = ROOT / "overview-2026-11-18"
out.mkdir(exist_ok=True)
fig.savefig(out / "latency.svg", metadata={"Date": None})
fig.savefig(out / "latency.png", dpi=150, metadata={"Software": "Matplotlib"})
print(out / "latency.svg")
