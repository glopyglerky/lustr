#!/usr/bin/env python3
"""Render all verified fixed-fixture comparisons as a standalone SVG.

Requires Python 3.10+ and Matplotlib (verified with 3.10.8). The existing
report-mixed-performance.py validates the original artifact and replays every
statistic; this tool adds no estimator and never executes recorded commands.

From the repository root:
  python3 tools/plot-mixed-performance.py ORIGINAL-QUALIFICATION.zip \
    --output /absolute/new-chart-directory --png

The output directory must not exist. Outputs: performance.svg, plot-data.json,
README.md, SHA256SUMS, and an optional performance.png inspection copy. Original
evidence is never modified. The figure describes one completed campaign, not a
cross-run trend, a causal optimization effect, or compiler/release admission.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import platform
import sys
import zipfile


PANELS = (
    ("native", "pristine", "integrated", "Native integration", "Pristine Lust / integrated Lust"),
    ("native", "pristine", "entry", "Packaged entry", "Pristine Lust / packaged Lustr"),
    ("cargo", "reference", "mixed", "Mixed Cargo", "Matched Cargo/rustc / mixed Lustr"),
)
REFERENCE_PANEL = ("native", "rustc", "entry", "Standalone Rust comparison",
                   "Matched nonincremental rustc / packaged Lustr")
LABELS = {
    "cold": "Cold build",
    "no-op": "No-op build",
    "body-edit": "Body edit",
    "signature-edit": "Signature edit",
    "check": "Check",
    "check-no-op": "No-op check",
    "check-body-edit": "Check after body edit",
    "rejection": "Rejection",
    "compile-plus-test": "Compile + test",
}
COLORS = {
    "slower-on-this-corpus": "#B34735",
    "inconclusive": "#64758A",
    "faster-on-this-corpus": "#007A7A",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def reporter():
    path = Path(__file__).resolve().with_name("report-mixed-performance.py")
    spec = importlib.util.spec_from_file_location("lustr_strict_performance_report", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module, path


def panels_for(result):
    return PANELS + (REFERENCE_PANEL,) if result.get("campaign_schema", 1) == 2 else PANELS


def ordered_rows(result, phases):
    by_key = {tuple(row[key] for key in ("family", "baseline", "candidate", "phase")): row
              for row in result["summaries"]}
    # Exact scope comes from the strict verifier; no winning-mode selection.
    return [by_key[family, baseline, candidate, phase]
            for family, baseline, candidate, _, _ in panels_for(result) for phase in phases]


def render(result, rows, phases):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.lines import Line2D
    from matplotlib.ticker import FixedLocator, FuncFormatter, NullLocator

    plt.rcParams.update({
        "font.family": "DejaVu Sans", "font.size": 10,
        "svg.fonttype": "path", "svg.hashsalt": "lustr-paired-performance-v1",
        "axes.spines.top": False, "axes.spines.right": False,
        "axes.spines.left": False, "axes.spines.bottom": False,
        "text.color": "#172B42", "axes.labelcolor": "#40546B",
        "xtick.color": "#40546B", "ytick.color": "#172B42",
    })
    panels = panels_for(result)
    four = len(panels) == 4
    figure = plt.figure(figsize=(18, 14.5 if four else 8.5), facecolor="white")
    figure.text(.035, .952, "Lustr performance · completed paired campaign", fontsize=23, weight="bold")
    figure.text(.035, .912,
                f"Baseline time / candidate time   ·   Point = paired geometric mean   ·   Bar = conditional 95% interval   ·   n = {result['pairs']} pairs per mode",
                fontsize=11.5)
    figure.text(.035, .875,
                "Below 1: candidate took longer     |     1: equal measured time     |     Above 1: candidate took less time",
                fontsize=11, weight="medium")
    legend = [Line2D([], [], color=COLORS[key], marker="o", linewidth=1.8, markersize=5, label=label)
              for key, label in (("slower-on-this-corpus", "Interval below 1"),
                                 ("inconclusive", "Interval includes 1: inconclusive"),
                                 ("faster-on-this-corpus", "Interval above 1"))]
    figure.legend(handles=legend, loc="upper left", bbox_to_anchor=(.03, .854),
                  ncol=3, frameon=False, handlelength=2.2, columnspacing=2.1, fontsize=10)
    if four:
        figure.text(.035, .818,
                    "Fourth comparison: identical standalone source and dev semantics; rustc is nonincremental. Its no-op is a new compiler invocation, not Cargo Fresh.",
                    fontsize=10)

    lower = min([1.] + [min(row["analysis"]["bootstrap_95"][0], row["analysis"]["geometric_ratio"]) for row in rows]) / 1.2
    upper = max([1.] + [max(row["analysis"]["bootstrap_95"][1], row["analysis"]["geometric_ratio"]) for row in rows]) * 1.2
    tick_candidates = [factor * 10.**power
                       for power in range(math.floor(math.log10(lower)), math.ceil(math.log10(upper)) + 1)
                       for factor in (1, 2, 5)]
    ticks = sorted(set([1.] + [value for value in tick_candidates if lower <= value <= upper]))
    for index, panel in enumerate(panels):
        family, baseline, candidate, title, reference = panel
        column = index % 2 if four else index
        bottom, height, plot_width, value_width = ((.522 - (index // 2) * .320, .215, .190, .140)
                                                   if four else (.345, .397, .145, .118))
        left = .132 + column * (.460 if four else .285)
        axis = figure.add_axes((left, bottom, plot_width, height))
        values = figure.add_axes((left + plot_width + .009, bottom, value_width, height))
        for current in (axis, values):
            current.set_ylim(-.65, len(phases) - .35)
        axis.set_xscale("log")
        axis.set_xlim(lower, upper)
        axis.xaxis.set_major_locator(FixedLocator(ticks))
        axis.xaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:g}"))
        axis.xaxis.set_minor_locator(NullLocator())
        axis.tick_params(axis="both", length=0, pad=8, labelsize=10)
        axis.set_yticks(range(len(phases)), [LABELS[phase] for phase in reversed(phases)] if column == 0 else [])
        axis.set_xlabel("Ratio · shared logarithmic scale", fontsize=9, labelpad=11)
        axis.axvspan(lower, 1, color="#FCF7F5", zorder=-3)
        axis.axvspan(1, upper, color="#F3F9F8", zorder=-3)
        for tick in ticks:
            axis.axvline(tick, color="#DCE3EA", linewidth=.65, zorder=-2)
        axis.axvline(1, color="#263E58", linewidth=1.2, linestyle=(0, (3, 3)), zorder=-1)
        values.set_xlim(0, 1)
        values.axis("off")
        figure.text(left, bottom + height + .055 if four else .797, title, fontsize=14, weight="bold")
        figure.text(left, bottom + height + .033 if four else .769, reference, fontsize=10, color="#40546B")
        figure.text(left + plot_width + .009, bottom + height + .009 if four else .744, "Ratio [95% interval]", fontsize=9, color="#40546B")
        panel_rows = rows[index * len(phases):(index + 1) * len(phases)]
        for row_index, row in enumerate(panel_rows):
            y = len(phases) - 1 - row_index
            analysis, color = row["analysis"], COLORS[row["observation"]]
            low, high = analysis["bootstrap_95"]
            point = analysis["geometric_ratio"]
            if row_index % 2 == 0:
                for current in (axis, values):
                    current.axhspan(y - .44, y + .44, color="#EAF0F4", alpha=.32, zorder=-2)
            # Drawing bounds separately does not assume the point lies inside
            # a percentile interval. Every coordinate is from the verified row.
            axis.hlines(y, low, high, color=color, linewidth=2.1, zorder=2)
            axis.vlines((low, high), y - .085, y + .085, color=color, linewidth=1.1, zorder=2)
            axis.plot(point, y, "o", markersize=5.8, color=color, markeredgecolor="white", markeredgewidth=.6, zorder=3)
            values.text(0, y, f"{point:.3f} [{low:.3f}, {high:.3f}]", va="center",
                        fontsize=10.3, color=color, fontfamily="DejaVu Sans Mono")

    figure.text(.035, .151 if four else .286,
                "Interval classifications use unrounded bounds; printed numbers are rounded.", fontsize=10)
    figure.text(.035, .129 if four else .258,
                "Scope: small native scalar corpus and fixed mixed Cargo graph. Command wall time; all modes and samples retained.",
                fontsize=10.5)
    figure.text(.035, .110 if four else .230,
                "Conditional, exploratory shared-host intervals; no equivalence test or multiple-comparison correction. An interval including 1 does not prove parity.",
                fontsize=10.3)
    figure.text(.035, .091 if four else .202,
                "One campaign only: no cross-run trend or causal optimization claim. Native and packaged rejection routes can perform different semantic validation.",
                fontsize=10.3)
    figure.text(.035, .072 if four else .174,
                "This replay grants no compiler, broad-workload speed, platform, or release admission. Exact profiles, identities and values accompany the SVG.",
                fontsize=10.3)
    rust = result["rust_reference_version"].splitlines()[0]
    provenance = (
        f"Measured checkout  {result['checkout']}     Tree  {result['tree']}",
        f"Pristine Lust      {result['pristine_revision']}     Reference  {rust}",
        f"Original ZIP SHA256  {result['archive_sha256']}",
        f"Raw JSONL SHA256     {result['raw_sha256']}",
    )
    for index, line in enumerate(provenance):
        figure.text(.035, (.048 - index * .012) if four else (.119 - index * .025), line,
                    fontsize=8.2 if four else 8.8, color="#52657A", fontfamily="DejaVu Sans Mono")
    return figure, matplotlib.__version__


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("artifact", type=Path, help="original completed fixed-fixture qualification ZIP")
    parser.add_argument("--output", type=Path, required=True, help="new output directory; earlier charts are never overwritten")
    parser.add_argument("--png", action="store_true", help="also render a PNG inspection copy")
    args = parser.parse_args()
    evidence = None
    try:
        if args.output.exists():
            raise ValueError("output already exists; preserve earlier evidence")
        if not args.artifact.is_file():
            raise ValueError("an original qualification ZIP is required")
        strict, strict_path = reporter()
        evidence = strict.Evidence(args.artifact)
        result = strict.verify(evidence)
        rows = ordered_rows(result, strict.PHASES)
        figure, matplotlib_version = render(result, rows, strict.PHASES)
        source_keys = ("checkout", "tree", "archive_sha256", "raw_sha256", "pairs", "seed",
                       "pristine_revision", "pristine_tree", "rust_reference_version", "profiles", "scope",
                       "cold_scope", "warm_scope", "timing_scope", "inference_limits", "compiler_sha256")
        data = {
            "schema": "lustr-paired-chart-v1", "kind": "derived visualization of one strictly verified campaign",
            "ratio_direction": strict.DIRECTION, "interval": "conditional paired bootstrap 95% interval",
            "source": {key: result[key] for key in source_keys}, "comparisons": rows,
            "renderer": {"python": platform.python_version(), "matplotlib": matplotlib_version,
                         "generator_sha256": digest(Path(__file__).resolve()), "strict_reporter_sha256": digest(strict_path)},
            "verified": result["verified"], "performance_admission": result["performance_admission"],
        }
        if result.get("campaign_schema", 1) == 2:
            data["source"].update(campaign_schema=2, native_rustc_reference=result["native_rustc_reference"],
                                  native_lane_executables=result["native_lane_executables"])
        # Validation and all plotting setup must succeed before publishing files.
        args.output.mkdir(parents=True, exist_ok=False)
        (args.output / "plot-data.json").write_text(json.dumps(data, indent=2, sort_keys=True, allow_nan=False) + "\n", encoding="utf-8")
        metadata = {"Title": "Lustr performance: all paired comparisons",
                    "Description": f"{result['pairs']} matched pairs per mode; baseline/candidate time; conditional 95% paired intervals. Measured checkout {result['checkout']}, tree {result['tree']}. Small fixed corpus; no release or general speed admission.",
                    "Creator": "Lustr plot-mixed-performance.py", "Date": None}
        figure.savefig(args.output / "performance.svg", format="svg", metadata=metadata, facecolor="white")
        if args.png:
            figure.savefig(args.output / "performance.png", format="png", dpi=150, metadata={"Software": "Lustr plot-mixed-performance.py"}, facecolor="white")
        readme = (
            "# Paired performance chart\n\n"
            f"`performance.svg` shows all nine modes for all {len(panels_for(result))} comparisons on the same logarithmic ratio scale. "
            "A point is the paired geometric mean of baseline/candidate time; a bar is its conditional paired-bootstrap 95% interval. "
            "Below 1 means the candidate took longer. Colors use the existing report's interval classification, based on unrounded bounds; printed numbers are rounded.\n\n"
            f"Measured checkout: `{result['checkout']}`; tree: `{result['tree']}`. "
            f"There are {result['pairs']} pairs per mode and {len(rows)} comparisons. "
            "The original ZIP and raw-data SHA256 values appear on the figure and in `plot-data.json`. "
            "The JSON retains every plotted analysis without recomputation, exact component identities, profiles, workload limits and rendering versions.\n\n"
            "The generator imports `report-mixed-performance.py` and requires its complete evidence verification and statistical replay. "
            "It never executes a command from the artifact. No new estimator, filtered modes, cross-run trends or causal speedup claims are added. "
            "The small fixed corpus and conditional shared-host intervals do not establish equivalence, general speed preservation, platform coverage or release admission.\n\n"
            "Reproduce with Python 3.10+ and Matplotlib (this rendering used " + matplotlib_version + "):\n\n"
            "```sh\npython3 tools/plot-mixed-performance.py ORIGINAL-QUALIFICATION.zip --output /absolute/new-chart-directory --png\n```\n\n"
            "Existing output directories are refused. SVG fonts are embedded as paths; rendering versions are recorded because plotting-library changes can alter SVG bytes. "
            "`SHA256SUMS` identifies the generated files; it does not replace the original artifact's identities.\n"
        )
        if result.get("campaign_schema", 1) == 2:
            readme += ("\nThe fourth comparison uses matched standalone rustc on exactly the same native source. "
                       "It is nonincremental: a no-op still invokes rustc and is not Cargo Fresh. The Rust test sample includes "
                       "compiling and executing the test. Exact flags, identities and this contract are retained in `plot-data.json`.\n")
        (args.output / "README.md").write_text(readme, encoding="utf-8")
        with (args.output / "SHA256SUMS").open("x", encoding="utf-8") as hashes:
            for path in sorted(args.output.iterdir()):
                if path.name != "SHA256SUMS":
                    hashes.write(f"{digest(path)}  {path.name}\n")
        print(json.dumps({"chart": str(args.output / "performance.svg"), "pairs": result["pairs"], "comparisons": len(rows), "checkout": result["checkout"]}, sort_keys=True))
    except (ValueError, KeyError, TypeError, OSError, OverflowError, zipfile.BadZipFile, ImportError) as error:
        print(f"performance chart rejected: {error}", file=sys.stderr)
        return 1
    finally:
        if evidence is not None:
            evidence.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
