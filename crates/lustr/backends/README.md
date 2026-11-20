# Historical Cranelift qualification fixtures

The Cranelift pin and immutable setup history preserve an independent
Rust-backend experiment. They do not implement Lustr's missing Rust capabilities, and the
retired coordinator no longer executes `--engine cranelift` commands.

The experiment found that the selected preview backend did not preserve the
fixture's panic recovery. Its replacement built the pinned upstream backend
with the `unwinding` feature and checked panic catching and destructor execution.
The complete [source identities, commands, findings and limits](https://github.com/glopyglerky/lustr/blob/ec99578d51c93674af3a1bcf9c182f4ad4b17d89/crates/lustr/backends/README.md)
remain attached to the original implementation.

Use these fixtures only for an explicitly scoped independent oracle or backend
comparison. Compiler/version identity, ABI, runtime and failed-result evidence
remain relevant. None of those checks admits a complete Rust frontend into
Lustr's production path.

Current work follows the
[exclusive coverage contract](../../../docs/compiler-set/exclusive-coverage-contract.md)
and [completion checklist](../../../docs/compiler-set/completion-checklist.md).
Full Rust source stays in blueFe. Any production source reuse needs its own
missing-capability and dependency witness.
