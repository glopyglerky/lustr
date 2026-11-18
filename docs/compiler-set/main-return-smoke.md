# Current candidate main-return smoke

Five bounded `main` return cases passed on the original source-built PR #26
package (`43a7e07d32929309fc6e402d7bca6b5673e186a2`) on the current Linux
x86_64 host. Every public build used `lust / native-library`; none fell back
to full Rust. Executable exit status, stdout, and stderr matched the same
package's pinned `rustc-reference` exactly.

This observation uses the original launcher and native components, without
the separate single-file metadata-check repair. The
[sanitized numeric summary](main-return-smoke.json) contains source hashes,
component hashes, exact stream hashes, routes, and result counts.

| `main` return | Reference exit | Public native exit | Output comparison |
| --- | ---: | ---: | --- |
| Unit `()` | 0 | 0 | Exact stdout and stderr match |
| `Result<(), &'static str>` with `Ok(())` | 0 | 0 | Exact match |
| `Result<(), &'static str>` with `Err("intentional main failure")` | 1 | 1 | Exact match, including `Error: "intentional main failure"` on stderr |
| `std::process::ExitCode::SUCCESS` | 0 | 0 | Exact match |
| `std::process::ExitCode::FAILURE` | 1 | 1 | Exact match |

Each program printed a distinct fixed line before returning. The runner
compiled and executed the reference once, then compiled and executed the
public candidate once. It disabled the daemon and native cache with
`LUST_NO_DAEMON=1` and `LUST_NO_CACHE=1`, used a 10-second timeout for every
child, and would stop at the first compiler or runtime mismatch. All
20 commands completed without timeout or retry. The data check verified
20 source snapshots, all 80 recorded stream hashes, and unchanged package
component identities after execution. All child processes were reaped.

These results do not reproduce the older PR #24 Result/ExitCode failure on
this current cold native package. They do not establish daemon-state
behavior, custom `Termination` implementations, every `Result` payload, or
other target ABIs. Successful execution on this host also does not resolve
a separate source-level C function-type concern on every platform. No
compiler source or regression test was changed for this diagnostic.
