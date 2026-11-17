# Unchanged coordinator performance

Project revision: `b3393a4a2907228516546cd34f1a5c60aaf6aaaa`. Coordinator Git tree: `3f72ccd2f1c5f83f952e2a43564a046a32d75305`.

Full Rust source: `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`; host `x86_64-unknown-linux-gnu`. Compiler components use the constructor's release profile.

The actual `crates/lustr` project retains its normal dev/test defaults, including debug=2. The reference uses the matching SDK's Cargo/rustc; Lustr uses the assembled primary entry and its matching Cargo selectors. This profile is outside current mixed-object eligibility. These results do not establish native acceleration or release admission.

Recorded before the campaign: 2026-11-17T15:42:31.294745+00:00. CPU: Intel(R) Xeon(R) 6975P-C. See host.json for observed affinity, limits and unknown host facts.

Each case contains 12 measured pairs and two excluded warmup pairs. All raw timing values and the actual balanced execution order are retained in samples.json. No process CPU, memory or energy consumption is measured.

| Scenario | Matched Rust median (ms) | Lustr median (ms) | Matched Rust p95 (ms) | Lustr p95 (ms) | Rust / Lustr paired ratio | Bootstrap 95% interval |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `clean-dependency-build` | 536.577 | 537.211 | 543.325 | 546.565 | 0.9974 | [0.9911, 1.0048] |
| `primed-root-build` | 159.417 | 163.345 | 162.272 | 167.196 | 0.9775 | [0.9653, 0.9900] |
| `primed-root-check` | 61.293 | 63.611 | 62.650 | 65.577 | 0.9533 | [0.9415, 0.9667] |
| `no-op` | 10.062 | 11.098 | 10.476 | 11.182 | 0.9319 | [0.9109, 0.9560] |
| `compile-plus-test` | 5099.683 | 5191.029 | 5192.885 | 5266.061 | 0.9814 | [0.9768, 0.9861] |

Ratios above one favor Lustr; below one favor matched Rust. Medians and p95 are empirical sample summaries. The paired bootstrap interval and exact sign-flip probabilities are conditional on this campaign's host and repeated workload; the individual exploratory comparisons are not adjusted for multiplicity. replay.json records verification with the repository's existing Rust statistics implementation.

`primed-root-build`, `primed-root-check` and `compile-plus-test` force `src/lib.rs` with a byte-preserving timestamp change. They are root rebuilds, not body or signature edits. `no-op` changes no timestamps. `clean-dependency-build` clears owned Cargo/native/mixed/temp state after observed successful daemon shutdown; operating-system and dependency source caches remain warm.

The ordinary project test gate reported 40 passed and 9 existing explicitly ignored tests. Both lanes passed matching initial/final executing tests and the structured E0502 rejection gate in every scenario. Observed compiler commands, source/tool guards, Cargo fingerprints, owned state receipts and final daemon shutdown were required. Test-count details and all command/gate exits are in samples.json.

This directory is a derived, path-free public projection. It includes timings, order, command contracts, gate counts, component/source identities and original evidence digests. It is not a byte-identical archive of private reports, exact environments or raw command output. Original report/spec/output hashes identify private originals; SHA256SUMS independently identifies these public files. A temporary CI directory alone does not establish durable private retention.
