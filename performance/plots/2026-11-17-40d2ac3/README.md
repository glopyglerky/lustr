# Paired performance chart

`performance.svg` shows all nine modes for all 3 comparisons on the same logarithmic ratio scale. A point is the paired geometric mean of baseline/candidate time; a bar is its conditional paired-bootstrap 95% interval. Below 1 means the candidate took longer. Colors use the existing report's interval classification, based on unrounded bounds; printed numbers are rounded.

Measured checkout: `3b4b0bd67401085129d26be1afbd20e96c9b9d02`; tree: `78318c4b01f8eb6f9518ea331efe69f0c457b1ce`. There are 12 pairs per mode and 27 comparisons. The original ZIP and raw-data SHA256 values appear on the figure and in `plot-data.json`. The JSON retains every plotted analysis without recomputation, exact component identities, profiles, workload limits and rendering versions.

The generator imports `report-mixed-performance.py` and requires its complete evidence verification and statistical replay. It never executes a command from the artifact. No new estimator, filtered modes, cross-run trends or causal speedup claims are added. The small fixed corpus and conditional shared-host intervals do not establish equivalence, general speed preservation, platform coverage or release admission.

Reproduce with Python 3.10+ and Matplotlib (this rendering used 3.10.8):

```sh
python3 tools/plot-mixed-performance.py ORIGINAL-QUALIFICATION.zip --output /absolute/new-chart-directory --png
```

Existing output directories are refused. SVG fonts are embedded as paths; rendering versions are recorded because plotting-library changes can alter SVG bytes. `SHA256SUMS` identifies the generated files; it does not replace the original artifact's identities.
