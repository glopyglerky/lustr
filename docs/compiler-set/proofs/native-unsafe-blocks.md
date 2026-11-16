# Owned native unsafe-block frontend candidate

This opt-in implementation is under qualification. It does not establish full
unsafe Rust support, full native language coverage, or a performance gain.
The default native entry still includes the pristine Lust executable source.

The composition in `crates/lustr-native-core/src/native_modules.rs` refers to
the unchanged modules at Lust revision
`240e08d91d4aed0299639f6f11fd522912b248f9`. Owned parser and collection facades
provide the new interfaces. No upstream implementation is copied or patched.
The collection facade forwards the original checker's private numeric relation
through an additive access function beside its unchanged include.

The complementary parser consumes original tokens for an affected free function,
its signature, body and enclosing expression grammar. Complete unaffected items
and complete original blocks without an unsafe token use native parser APIs.
It creates `UnsafeBlock(BlockId)` nodes with original spans and arena ranges.
It does not remove an unsafe keyword, substitute a placeholder expression,
enable removed-code grammar, or rewrite application bytes. Unimplemented syntax,
attributes and expansion contexts remain rejected.

An original function item in an affected enclosing block uses the pinned item
parser on its complete unchanged token range. The native scan supplies only the
boundary. `Ast::append_chunk` shifts its expression, pattern, type, block,
string and nested statement-item references; the returned function is attached
as `Stmt::Item` in that original block. Its body and `FnDef` ranges remain native.
Native collection owns block-wide visibility, forward declarations, duplicate
rejection and separate function scopes. Local captures remain errors. Original
`const fn` headers use the same item interface: the `const` token reaches the
native parser, and its `FnQual` survives arena insertion. Native const-function
checking applies even when the function is called only at runtime, including
non-const calls and destructors. Other qualified headers and items inside the
bounded safety derivation remain unsupported.

Before collection, a separate safety derivation checks boolean literals,
initialized boolean locals and lexical blocks. It checks each initializer in
the previous scope before adding its binding, resolves only previously checked
local names with the same syntax context, and removes nested bindings on every
block return. Shadowing therefore preserves the initializer's old binding and
restores outer bindings after nested blocks. Plain `bool` annotations are
permitted; native checking still resolves their types and item-pattern ambiguity.
Items, arbitrary captures, pointer operations, attributes and unmodeled
patterns or type expressions remain outside this derivation.

Direct path calls with boolean-grammar arguments create deferred obligations.
The tentative clone stays inside the collection facade until native body checking
resolves every exact call ID in its original enclosing function, including its
local/item shadowing and syntax context. Each call must resolve to a direct,
non-generic safe Rust function with a body, boolean inputs/output, no method
receiver, no foreign ABI, no async/variadic qualifiers and no skipped body.
Unrelated conversion-constraint diagnostics remain for the production native
Check, which retains the errors and failed program for existing bounded
elaboration. A call still needs positive resolved target and boolean type facts;
unknown obligations refuse. Every recollection repeats this qualification.
Other query diagnostics refuse immediately. Complete successful native checking
is required before acceptance. An unresolved or unrepresented scope refuses
the extension. Closures, function pointers and actual unsafe calls
remain outside this rule. There is no function-name allowlist.

Nested unsafe sites inherit local context only through an actual successful
traversal from an enclosing checked body. Sorting outer sites first is an
optimization, not a source-containment proof. A depth bound and traversal budget
conservatively refuse overly complex bodies. Every admitted value derives from
a boolean literal, an earlier checked boolean local, a nested checked block or
a call whose actual target and boolean signature pass the native checks;
this bounded structural argument is not a proof of general Rust soundness.

Each admitted body contains no operation that needs unsafe permission. The gate clones the AST and
tentatively replaces the bounded node by an ordinary block with the same body, scope, arena
identity and span. Native collection, type checking, borrow checking, lowering
and generation then process that clone. Calls must discharge their native
resolution, safety, ABI and type obligations before collection can return.
The driver's original representation
and assertion-text scratch parse retain the unsafe node. This bounded lowering
does not permit unsafe/foreign/indirect calls, raw-pointer operations, mutable statics, unions or other
unsafe operations. Their implementation and safety-context checks remain open.

Unsafe blocks are ordinary runtime value expressions, not inline consts. Array
repeat Copy checks, evaluation count, moves, temporary scopes, drops and unwind
cleanup remain obligations of the native pipeline. The formatter follows the
pinned Rust block-head rules: the keyword uses a breakable space inside the
block head, before the opening brace. It does not use the anonymous-const box.

The design follows the distinction between operations and safety obligations
in the [Rust Reference](https://doc.rust-lang.org/reference/unsafe-keyword.html).
Initializer and binding scope follow its [let statements](https://doc.rust-lang.org/reference/statements.html#let-statements)
and [pattern binding scopes](https://doc.rust-lang.org/reference/names/scopes.html#pattern-binding-scopes).
Nested function items follow [item scopes](https://doc.rust-lang.org/reference/names/scopes.html#item-scopes),
including visibility before their declaration and exclusion of local captures.
The native checker remains authoritative for annotations, item ambiguity,
statement result types, borrow/move checks and the complete requested program.
[RustBelt](https://plv.mpi-sws.org/rustbelt/popl18/) motivates explicit verified
extension boundaries; its subset proof is not a proof of this compiler.
Runtime and rejection evidence, independent review, cold/retained edit recovery
and successful equivalent-work measurements are required separately.

Native crates, metadata, rlib, general Cargo, macro-generated unsafe grammar,
runtime-loader closure, supervised lifecycle and representative project speed
remain open. The public release gates remain held.
