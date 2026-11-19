# Remove the unused whole-rustc request policy

The current source still contained `crates/lustr-semantic/src/mixed_entry.rs`,
although the compiler that used it had already been removed. The file's
`full_rust_only` function assigned selected request flags to a complete Rust
compiler. Its tests explicitly expected that ownership policy.

At published head
[`7c3d095f`](https://github.com/glopyglerky/lustr/commit/7c3d095fa8971a4c38aa0edf62e8fcfd7a51589a),
tracked source and tooling contain no reference to that module or function
outside the file itself. The semantic inspection executable imports only its
`collect` and `contract` modules, and its manifest has no mixed compiler target.
The bounded mechanism qualifiers do not include this file.

The original caller remains in
[`47371e914`](https://github.com/glopyglerky/lustr/blob/47371e9143392d9de52eb32c82e8437ca4d37f77/crates/lustr-semantic/src/mixed_main.rs).
That retired launcher used the policy to select `rustc_driver::compiler_entrypoint`
with `CompleteRust` callbacks. The removed policy's exact Git blob is
`1149799bbd7880715ed6aa4753c3d9c36fecdaad`, with SHA-256
`94acc8580cb963295c032485f69f4d8a86d0757c11fff5c68119f6857c63511c`.
Its source and original history remain available through the published commit.

This change deletes only the unused policy source and adds its path to the
existing structural retirement guard. One filesystem control restores an
unimported policy file, requires rejection, removes it and requires recovery.
The existing provenance and dependency controls still run. The retained backend,
cache, diagnostics, object and metadata mechanisms remain available under the
[retained-work disposition](retained-work-disposition.md).

[Verification](evidence/dead-entry-policy-20261119/verification.json) records the
audit and focused checks. No new live retry route was demonstrated. This dead
source cleanup changes neither the production/native compiler tree nor its
integration declaration, adds no user compilation work and makes no new
coverage or performance claim.
