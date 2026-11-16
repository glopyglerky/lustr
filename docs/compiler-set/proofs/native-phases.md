# Borrowed native semantic phases

Status: explicit, unadmitted integration candidate. The canonical admitted Lust
diff remains empty. Normal source-guard refusal is required for this worktree
proposal; exact source checks do not admit it or establish performance.

`driver::compile_observed` adds optional call-borrowed access to the current
Program, original input AST, resolver tables, request, source map and root-text
fallback. An opaque process-local phase ID distinguishes observed invocations;
it is neither a semantic fingerprint nor an independent crate identity.
The worklist names only the functions actually checked in that invocation.

Prepared access occurs after native collection or qualified native kept-program
reuse, before body checks. Native workers expose each actual Checked result
before lowering consumes it, and actual IR after borrow checking. Those worker
callbacks run outside derive-read logging. Aggregate diagnostics arrive after
all workers finish and before rendering, dropping results or `keep_err`.
Parse/header/template failures and cached Check answers can return before this
phase. Reused functions get no invented Checked result. Callbacks neither
replace native checking nor change its worklist or acceptance decisions.

The owned shared-session consumer gathers existing elaboration sites directly
from those Checked results. It builds an owned AST proposal while the current
Program is still alive, so it never obtains authority from an older preferred
Program retained by `keep_err`. No borrowed compiler pointer survives a callback.
Weak input identity binds the proposal to the same current AST. The V59 proof
of actual fresh collection, collected AST, resolver and crate identity remains
required; warm-only elaboration remains conservatively rejected. Final native
collection, type/borrow checks and generation still decide acceptance. The
bounded retries and retirement of transformed sessions remain unchanged.

The legacy retained-artifact and one-shot APIs retain their existing paths and
host-input proofs. Their elaboration can still query bodies again. The shared
consumer removes that particular second query; it does not eliminate separate
native generic discovery or safety-obligation checks.

The native phase qualifier checks live borrows, work-item identity, check and
borrow diagnostics, early failures, recovery, derive-read isolation, cached
Check silence and observer-off parity. The shared-session gate exercises the
unchanged mixed complement and supported workloads. Helpers are explicitly
SDK-bootstrapped; qualification applications compile natively. Runtime results,
independent review and performance must each retain their exact scope.

Full Rust, independently identified typed crate producers/consumers, native
library artifacts, Cargo composition, arbitrary profiles and equivalent
complete-work speed remain open. Native metadata must join the actual resolver
early and carry stable crate/definition, type, trait, generic, layout, drop,
symbol and complete dependency/input identities. This interface supplies no
serialized crate format and no Rust binary-ABI compatibility claim. Rust
bootstrap, TinyCC/C, mimalloc and host-platform boundaries remain explicit.
