set -euo pipefail
{
  git rev-parse HEAD HEAD^{tree}
  rustc +1.99.1 -vV
  cargo +1.99.1 --version
  sha256sum .github/workflows/native-probe-tests.yml crates/lustr-bench/Cargo.toml crates/lustr-bench/Cargo.lock crates/lustr-bench/src/bin/lustr-bench-gate.rs crates/lustr-bench/src/main.rs crates/lustr-bench/tests/routed.rs
  cargo +1.99.1 test --manifest-path crates/lustr-bench/Cargo.toml --locked --bin lustr-bench-gate --test routed
} 2>&1 | tee "$RUNNER_TEMP/cargo-gate-controls.log"
