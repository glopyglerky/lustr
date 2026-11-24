# First independent hosted retained-history campaign

This is the original failed execution of the qualifier at PR head
`62da158b33740279947c85f3fc425df6d1623c9f`. GitHub checked out
`a3c878f6123b4f5dc5e2eff598b52c83ab5b4ec3`, whose tree exactly matches the
published and reviewed tree `390e25946a5ebcb8ccef417467a7b29d8034987d`.

The [original job](https://github.com/glopyglerky/lustr/actions/runs/44525981237/job/135566552526)
constructed the native package before running these histories. The campaign
attempted all 349 required histories: **328 passed and 21 failed**. It completed
1,643 version verdicts before the failures stopped their individual histories.
This is not a passing retained campaign or a complete compiler qualification.

## What the first failures showed

Fifteen histories were refused by the public Cargo graph guards before native
compilation. These remain required Lust integration work; no history was moved
to the unresolved set. The affected histories are `cargo10_bs_env_rerun`,
`cargo12_dep_bs_env`, the eleven `h9ms_*` histories, and `mb01`/`mb02`.

The missing trace in `cargo01_bin_discovery/v7` has a different cause: the pinned
overlay deliberately removes the selected binary source. Pinned Cargo build
and check also reject that target. Qualifying that request requires a separate,
fixed selection-rejection expectation and the unchanged v8 recovery; it is not
permission to accept an arbitrary public refusal.

Five histories stopped at the independent Rust diagnostic check. Four errors
are demanded during code generation and do not appear with metadata-only
emission. The fifth uses standard-library macros whose primary spans lead back
to fixture source through expansion call sites. These explain the first blocked
versions; later versions still require execution with the corrected qualifier.

## Retained evidence

- `campaign.json` preserves the complete campaign summary and source/tool hashes.
- `failed-histories.tar.gz` contains all reports and streams for the 21 failures.
  `archive-files.json` binds their original and published bytes.
- `all-histories.tar.gz` also preserves every available passing-history report
  and stream. `all-histories-files.json.gz` binds all 31,907 original files;
  31,903 are included. Only four unused metadata diagnostic artifacts are
  omitted, with their original hashes retained.
- `verification.json` records source, job, original complete hosted archive,
  publication scope, and file hashes.

Known hosted paths are replaced with named placeholders. Original hashes inside
the reports still refer to the original bytes; the companion indices separately
identify normalized copies. Extract either history archive before following a
scenario report's relative stream paths. No outcome or diagnostic is rewritten
apart from those recorded path substitutions.

The same job passed the 90 ordinary root tests, all five explicitly invoked
installed-package contracts, and the 18 exact capability-result cases. Those
successes remain separate from the failed retained-history gate. None establishes
full Rust coverage, current compiler speed, or release admission.
