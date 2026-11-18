# Measured compiler latency overview

The [SVG](latency.svg) and [PNG](latency.png) are generated from the exact
[PR #26 dashboard](../2026-11-18-43a7e07/dashboard/report.json). They show warm
public-slice check/build and both a winning and a regressing mixed Cargo mode.
Each panel starts at zero and states its own scale. Values are medians in ms.

All 29 operation rows, p95 and paired intervals remain in the linked snapshot.
This chart introduces no new measurement or release qualification. Regenerate
from the repository root with `python3 performance/render-overview.py` using
Matplotlib. The inspected rendering uses Matplotlib 3.10.8.
