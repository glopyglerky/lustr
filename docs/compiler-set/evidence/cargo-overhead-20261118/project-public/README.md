# Unchanged coordinator performance

Project revision: `480114c2dc973770fbfbbc3c2b32915b516c6716`. Coordinator Git tree: `74f91068723cf2aa4a1dee168713281db7705a1f`.

Full Rust source: `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`; host `x86_64-unknown-linux-gnu`. Compiler components use the constructor's release profile.

The actual `crates/lustr` project retains its normal dev/test defaults, including debug=2. The reference uses the matching SDK's Cargo/rustc; Lustr uses the assembled primary entry and its matching Cargo selectors. This profile is outside current mixed-object eligibility. These results do not establish native acceleration or release admission.

Recorded before the campaign: 2026-11-18T05:52:43.192421+00:00. CPU: AMD EPYC 7763 64-Core Processor. See host.json for observed affinity, limits and unknown host facts.

Each case contains 12 measured pairs and two excluded warmup pairs. All raw timing values and the actual balanced execution order are retained in samples.json. No process CPU, memory or energy consumption is measured.

| Scenario | Matched Rust median (ms) | Lustr median (ms) | Matched Rust p95 (ms) | Lustr p95 (ms) | Rust / Lustr paired ratio | Bootstrap 95% interval |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `clean-dependency-build` | 700.455 | 699.135 | 715.776 | 708.187 | 1.0037 | [0.9967, 1.0108] |
| `primed-root-build` | 220.356 | 220.771 | 227.007 | 228.222 | 1.0023 | [0.9924, 1.0114] |
| `primed-root-check` | 83.356 | 84.774 | 86.420 | 87.973 | 0.9790 | [0.9702, 0.9875] |
| `no-op` | 13.218 | 13.222 | 16.744 | 13.396 | 1.0120 | [0.9794, 1.0607] |
| `compile-plus-test` | 6998.462 | 6975.685 | 7045.992 | 7230.114 | 0.9992 | [0.9929, 1.0045] |

Ratios above one favor Lustr; below one favor matched Rust. Medians and p95 are empirical sample summaries. The paired bootstrap interval and exact sign-flip probabilities are conditional on this campaign's host and repeated workload; the individual exploratory comparisons are not adjusted for multiplicity. replay.json records verification with the repository's existing Rust statistics implementation.

`primed-root-build`, `primed-root-check` and `compile-plus-test` force `src/lib.rs` with a byte-preserving timestamp change. They are root rebuilds, not body or signature edits. `no-op` changes no timestamps. `clean-dependency-build` clears owned Cargo/native/mixed/temp state after observed successful daemon shutdown; operating-system and dependency source caches remain warm.

The ordinary project test gate reported 41 passed and 9 existing explicitly ignored tests. Both lanes passed matching initial/final executing tests and the structured E0502 rejection gate in every scenario. Observed compiler commands, source/tool guards, Cargo fingerprints, owned state receipts and final daemon shutdown were required. Test-count details and all command/gate exits are in samples.json.

This directory is a derived, path-free public projection. It includes timings, order, command contracts, gate counts, component/source identities and original evidence digests. It is not a byte-identical archive of private reports, exact environments or raw command output. Original report/spec/output hashes identify private originals; SHA256SUMS independently identifies these public files. A temporary CI directory alone does not establish durable private retention.
