# Unchanged coordinator performance

Project revision: `df8f55aa0430ef81f27e7f403d102256b7fc298b`. Coordinator Git tree: `74f91068723cf2aa4a1dee168713281db7705a1f`.

Full Rust source: `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`; host `x86_64-unknown-linux-gnu`. Compiler components use the constructor's release profile.

The actual `crates/lustr` project retains its normal dev/test defaults, including debug=2. The reference uses the matching SDK's Cargo/rustc; Lustr uses the assembled primary entry and its matching Cargo selectors. This profile is outside current mixed-object eligibility. These results do not establish native acceleration or release admission.

Recorded before the campaign: 2026-11-18T06:10:04.797247+00:00. CPU: Intel(R) Xeon(R) 6975P-C. See host.json for observed affinity, limits and unknown host facts.

Each case contains 12 measured pairs and two excluded warmup pairs. All raw timing values and the actual balanced execution order are retained in samples.json. No process CPU, memory or energy consumption is measured.

| Scenario | Matched Rust median (ms) | Lustr median (ms) | Matched Rust p95 (ms) | Lustr p95 (ms) | Rust / Lustr paired ratio | Bootstrap 95% interval |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `clean-dependency-build` | 518.860 | 522.133 | 526.140 | 530.836 | 0.9918 | [0.9843, 0.9987] |
| `primed-root-build` | 147.217 | 146.981 | 149.175 | 150.413 | 1.0008 | [0.9906, 1.0127] |
| `primed-root-check` | 56.018 | 57.071 | 58.469 | 59.227 | 0.9861 | [0.9666, 1.0057] |
| `no-op` | 9.987 | 9.989 | 10.063 | 10.051 | 0.9617 | [0.9304, 0.9924] |
| `compile-plus-test` | 4775.826 | 4801.302 | 4877.912 | 4907.157 | 0.9941 | [0.9884, 1.0003] |

Ratios above one favor Lustr; below one favor matched Rust. Medians and p95 are empirical sample summaries. The paired bootstrap interval and exact sign-flip probabilities are conditional on this campaign's host and repeated workload; the individual exploratory comparisons are not adjusted for multiplicity. replay.json records verification with the repository's existing Rust statistics implementation.

`primed-root-build`, `primed-root-check` and `compile-plus-test` force `src/lib.rs` with a byte-preserving timestamp change. They are root rebuilds, not body or signature edits. `no-op` changes no timestamps. `clean-dependency-build` clears owned Cargo/native/mixed/temp state after observed successful daemon shutdown; operating-system and dependency source caches remain warm.

The ordinary project test gate reported 41 passed and 9 existing explicitly ignored tests. Both lanes passed matching initial/final executing tests and the structured E0502 rejection gate in every scenario. Observed compiler commands, source/tool guards, Cargo fingerprints, owned state receipts and final daemon shutdown were required. Test-count details and all command/gate exits are in samples.json.

This directory is a derived, path-free public projection. It includes timings, order, command contracts, gate counts, component/source identities and original evidence digests. It is not a byte-identical archive of private reports, exact environments or raw command output. Original report/spec/output hashes identify private originals; SHA256SUMS independently identifies these public files. A temporary CI directory alone does not establish durable private retention.
