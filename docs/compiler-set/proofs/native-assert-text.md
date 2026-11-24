# Original-AST assertion text extension

This record preserves the original experiments and witnesses. The current
sole-goal cleanup removes the diagnostic-driven inference/retry pipeline while
retaining its bounded assertion printer in
`crates/lustr-native-core/src/native_assertion_text.rs` for narrow native
qualification. Preserving this helper does not restore the retired worker or
qualify a general complementary compiler.

The array-repeat candidate binds `Expr::ArrayRepeat` value/length IDs to exact
original spans and the bracket/semicolon tokens. It emits the repeat indentation
box, value expression, semicolon break and length expression. The count is the
anonymous const's value, without invented `const` syntax. Its oracle is Rust
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`,
`pprust/state/expr.rs:138..146,454..456,385..445`; native grammar and child
identity come from Lust `240e08d91d4aed0299639f6f11fd522912b248f9`,
`parser.rs:5180..5201` and `ast.rs:1210`. These sources were inspected on
2026-11-15. No printer implementation is copied. The complete original witness
is `tests/fixtures/native-inference-array-repeat-assert-original.rs`, SHA256
`786203e024d5b411b8268606fb10fcb638c1cefc8bc2da87bcc98797fa38775e`.
Its copied prior component rejects with L0809 while stock compiles and runs.
Executable value/length evaluation, Copy requirements, borrowing, drops,
unwinding and const legality still require full native rechecking. Derived
text support does not itself qualify those semantics. Native/runtime differential
tests cover the repeat's distinct forms; optimized/review/measurement acceptance
requires separate receipts. Const-block and inferred-length expressions remain
outside this bounded printing rule. The original source and executable AST are
unchanged; full language/crate/artifact/product qualification stays open.

This development candidate addresses Lust's L0809 runtime refusal for long
message-free assertions. It changes only Lustr-owned code. The protected Lust
tree and pinned Rust source stay unchanged. No source printer is copied or
patched, and rustc supplies differential evidence only.

The original failing source is retained unchanged as
`tests/fixtures/native-inference-long-assert-original.rs` (SHA256
`612287967c0d03d408b3704aa6451550e04b0555b74d559258f92dad417eef9b`).
Its first copied-component43 rejection occurs after fifteen assertion
unsizings. Its earlier exact stock compile and runtime witnesses succeeded.
No native timing baseline exists for this rejected source.

The owned extension identifies the actual parsed assertion and its unknown-text
marker, then parses its original source tokens into a scratch arena. Only
synthetic enclosing block tokens are added to use the existing parser interface.
The caller's bytes and executable condition AST are unchanged. The scratch AST
drives a bounded formatter. Only successfully derived panic literals replace
the marker; native collection, checking, borrowing, lowering and linking still
determine acceptance. Parsed acceptance is never executable acceptance.

The formatter has independent owned SCAN/PRINT code: exact offline prefix widths
replace streaming lookahead. End resolution waits for the next break outside
the closed box, or EOF. A Begin and its final Break therefore include following
punctuation up to that resolution boundary, counting nested flat widths.
PRINT uses consistent and inconsistent boxes, a 78-byte margin, relative
indentation and a minimum remaining width of 60. This follows the inspected
algorithm and byte-width rules; equivalence to all upstream printer behavior is
not formally proved. Payload differential tests remain necessary.

Source mechanisms consumed on 2026-11-15:

- Rust revision `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`,
  `compiler/rustc_ast_pretty/src/pp.rs`: the algorithm explanation,
  margin/minimum-space constants, PRINT rules and box/break helpers.
- The same revision's `pprust/state/expr.rs`: expression boxes, calls, method
  calls, closures and block dispatch; `pprust/state.rs`: comma lists, original
  token-tree spacing, macro groups, local statements and block closing rules;
  `pprust/state/item.rs`: closure parameter rendering.
- Lust revision `240e08d91d4aed0299639f6f11fd522912b248f9`,
  `expand/assert_msg.rs`, `expand/builtin.rs`, `parser.rs`, `lower.rs`,
  `driver/session.rs` and `diag.rs`: original-token loss, marker identity,
  native parsing, runtime refusal, session AST and original source access.
- [Rust's pretty-printer documentation](https://doc.rust-lang.org/stable/nightly-rustc/rustc_ast_pretty/pp/index.html)
  describes its Oppen/Karlton lineage. This moving documentation is navigation;
  the local source revision above binds the implementation study.
- Derek C. Oppen, *Pretty Printing*, Stanford STAN-CS-79-770 (1979),
  [report](https://i.stanford.edu/pub/cstr/reports/cs/tr/79/770/CS-TR-79-770.pdf).
  Retrieval timed out; the full paper and its mathematical proof remain unread.
  No paper theorem is claimed verified against this candidate.

The current bounded syntax includes original calls, methods, simple paths and
locals, untyped closures, blocks, original parentheses and nested builtin macro token trees. Expanded
macro contexts, unmodeled precedence-sensitive forms,
items, typed locals/closures, generic paths, attributes and unmodeled forms
retain the original refusal. Token source identity, byte limits, recursion and
output bounds constrain the extension. Original block-token coverage also
refuses empty statements dropped by the native AST. These are explicit coverage frontiers,
not evidence of full Rust support. No all-Rust toolchain, general Cargo/rlib
support, product lifecycle acceptance or speed gain follows from this design.

The first broader corpus run found a four-space indentation mismatch for a
wrapped nested macro statement. Rust's statement macro printer bypasses its
expression box; the native AST stores builtin macro statements as expressions.
The owned printer now handles that distinction explicitly. The first source,
native and stock payloads, and failed test invocation remain retained. Corrected
development payload/negative tests and retained literal/token/position/signature
edits, rejection and recovery pass; optimized qualification and review remain
separate obligations until their actual receipts exist.

Fresh Sol review found two additional payload errors, both reproduced with
actual native and stock programs: call-argument attributes dropped by the
native parser, and a missing space after trailing-dot floats before a method
dot. The corrected formatter refuses attribute tokens before scratch parsing
and emits the required literal separation. Their first source and differing
runtime payloads remain retained; new guards exercise these cases explicitly.

The parenthesis extension uses original source tokens and the scratch AST's
widened spans. Lust's `parser.rs:4703` and `parser.rs:5113` retain only deduplicated
parenthesized expression IDs; original nesting must be recovered separately.
A bounded delimiter stack pairs parentheses, brackets and braces. Only complete,
nonempty enclosing parentheses are peeled from a marked expression's span;
call delimiters and a unit expression's own `()` stay structural. A missing
wrapper or mismatched delimiter refuses the extension. Each recovered pair emits
its own expression box, following the pinned Rust printer's `ExprKind::Paren`
dispatch in `pprust/state/expr.rs:817`. The executable AST stays unchanged.
The recursive depth budget counts source wrappers and AST expressions.
Parenthesized macro statements use expression printing; unparenthesized macro
statements retain their special statement printing behavior.

The exact original parenthesized source is retained as
`tests/fixtures/native-inference-parenthesized-assert-original.rs`, SHA256
`b33a77ba19587159f387a51dcad75d7280ebab84c1204eab538512cf4a76e930`.
Its earlier native L0809 rejection and stock compile/runtime success are retained.
First development native parity passes for that complete source and nested
wrappers, callees, closure arguments, unit expressions, macros, blocks and
receivers. Native const evaluation and borrowing checks remain active.
Struct-literal printing is qualified separately below;
parentheses do not make other unmodeled inner expressions supported. Optimized,
retained, independent-review and measurement evidence are separate gates.

The binary extension uses the scratch AST's `BinOp`, original operand spans and
the source tokens between them. The operator must match exactly; native `>>`
has two adjacent `>` tokens. It emits an independent break before and after
the operator within the existing expression box. The inspected Rust revision's
`pprust/state/expr.rs:287`, `state/fixup.rs:236`,
`rustc_ast/src/ast.rs:1055` and `util/parser.rs:116` establish the binary print,
precedence, fixity and contextual-fixup rules. Lust's `parser.rs:4565` and
`ast.rs:927` bind the actual native operator tree and symbols. All binary
operators are left-associative except the non-associative comparisons.

Original parentheses remain explicit unambiguous expressions. A child that
would require newly invented precedence parentheses refuses the extension.
Unparenthesized direct closure operands also refuse because their contextual
precedence is not yet modeled. Unsupported casts, jumps, let expressions,
attributes and expanded-context forms retain refusal. This does not
copy the protected printer or extend its executable AST: the extension still
changes only a derived panic literal before full native rechecking.

The complete original binary source is retained as
`tests/fixtures/native-inference-binary-assert-original.rs`, SHA256
`73b5f6997aa0b6c3942c34fd84596385a887250b294f647f3f834bbaaddb0193`.
Its copied-component45 native L0809 and separate stock compile/runtime witnesses
are retained. Development compilation succeeds; payload/runtime, optimized,
retained, independent-review and speed qualification require actual receipts.

The struct extension retains the complete original source as
`tests/fixtures/native-inference-struct-assert-original.rs`, SHA256
`7469eba3a0b4a45a15d36150ed2ff69d0d4bd3cbb9c1eaa144da7271c2d4341a`.
Its copied-component46 L0809 rejection and separate successful stock compilation
and runtime are retained. No successful native46 timing baseline exists.

The design follows the pinned Rust `pprust/state/expr.rs:131` struct expression
printer and `pp.rs:353,451,608` offset, break and conditional-comma rules. The
native `ast.rs:1180` and `parser.rs:5371` retain path segments, ordered field-name
spans, value IDs and an optional update. Shorthand has the same name/value span;
an explicit field has its original colon and separate value span. Every original
token must be covered by those identities, separators and braces. Simple paths,
raw identifiers, numeric tuple fields, empty structs, nested fields, shorthand
and update expressions use their original scratch AST. Attributes, generic
paths and other unmodeled contexts still refuse. Only the derived panic literal
changes; source and executable field evaluation stay native.

A struct's consistent field box adds a comma at its last break only when that
box breaks. Flat printing uses a space. An update ends with a space break and
no generated comma; its closing break receives the pinned negative indentation
offset. The first development compiler build caught an owned Rust borrow error
in the formatter itself; its source and failure are retained. Differential
payload/runtime, retained edits, optimized execution, independent review and
measurement receipts determine qualification; no general printer proof follows.

The first struct corpus also exposed a separate tuple-field access refusal in
the complete binary-derived source `(Pair { 0: false, 1: 2u8 }).0`. That original
native failure and its separate stock compile/runtime success remain retained.
Numeric fields are checked here through an ordinary consuming function; the
compiler does not rewrite that source or claim tuple-field panic-text support.
The tuple-field extension now retains that exact complete source as
`tests/fixtures/native-inference-tuple-assert-original.rs`, SHA256
`d3cb87af961e7357cd2d2a34bf9882a93ee3a092c79b37f363e3a854484850f8`.
Its copied47 L0809 refusal and separate stock compile/runtime success remain
retained. No successful native47 timing baseline exists for this source.

The owned formatter follows Rust `5b7b37ae`'s
`pprust/state/expr.rs:676` Field expression boxes, numeric identifiers and
trailing-dot separation, with `state/fixup.rs:166` dot context. Lust
`240e08d9`'s `ast.rs:1246`, `parser.rs:4775,5433` bind the actual index and
source identity. A plain index must match its canonical decimal token exactly.
For `value.0.1`, Lust splits one float token into two tuple-field nodes sharing
the complete source span. The formatter validates that exact two-node chain
against both original decimal components and emits both expression boxes.
Rust's `rustc_parse/src/parser/expr.rs:844` independently documents float-token
splitting and requires exact source identity before dividing token spans.
The owned formatter keeps the original spans; it does not synthesize offsets.
Explicit source parentheses remain distinct. Low-precedence unparenthesized
bases, noncanonical indexes and unmodeled forms refuse. Existing token, depth,
arena and output bounds still apply. Only derived panic text changes before
the complete native recheck. Differential runtime, invalid-program, retained
edit/recovery, optimized measurement and fresh review evidence remain separate
qualification gates; no general printer-equivalence proof follows.

The first optimized tuple corpus exposed a four-index panic-layout mismatch:
the eager End widths left a nested struct flat while stock broke its field box.
Its complete source, native and stock outputs, failed suite and frozen first
candidate remain retained. The bounded reverse scan now inherits the next
outer break across End markers, reflecting the pinned `pp.rs:318..457` delayed
End resolution. This includes closing punctuation and field suffixes without
copying or overlaying the protected printer. The full prior differential corpus
and original-source checks must run again on the corrected candidate.

Tuple literals retain the complete original source as
`tests/fixtures/native-inference-tuple-literal-assert-original.rs`, SHA256
`969972eb87c7a664b6e7c54cd51f9cb92e871715ce04957a145042a055b3f61b`.
Its copied48 L0809 refusal and separate stock compile/runtime success remain
retained. This rejected source supplies no successful native48 speed baseline.

The owned rule follows pinned Rust `5b7b37ae`
`pprust/state/expr.rs:205` and `pprust/state.rs:1230..1253`: structural tuple
parentheses enclose an inconsistent comma-list box, and a singleton keeps its
required comma outside that list box. A multi-element trailing comma is omitted
from the printed expression. Lust `240e08d9` `ast.rs:1205` and
`parser.rs:5120..5180` bind the actual ordered elements and distinguish tuples
from parenthesized expressions. The first original element span identifies
where structural tuple parentheses begin; extra source wrappers each retain
their expression box. Every element span and comma must cover the original
tuple tokens. Attributes and unsupported element grammar retain refusal.
Only derived panic text changes before full native rechecking. The executable
tuple, evaluation order, borrows, drops, const behavior and unwind stay native.
Direct/retained payload and rejection checks, optimized measurement and fresh
Sol review remain qualification obligations until their actual results exist.

Array literals retain the complete original source as
`tests/fixtures/native-inference-array-literal-assert-original.rs`, SHA256
`220116ea2640bb8d49562f37ed7b879313d0bdda53e62ac8c4895d76b10fae33`.
Its copied49 L0809 refusal and separate stock compile/runtime success remain
retained. The rejected source supplies no successful native49 speed baseline.

The owned rule follows pinned Rust `5b7b37ae`
`pprust/state/expr.rs:112..118,449..451` and
`pprust/state.rs:1230..1253`: an additional indentation box encloses brackets
and an inconsistent comma list. A singleton needs no comma; original trailing
commas are omitted. Lust `240e08d9` `ast.rs:1207` and
`parser.rs:5180..5210` supply ordered element IDs and distinguish list arrays
from repeats. Every original element span, separator and bracket must be
covered; unrepresented attributes retain refusal. Repeats use the separate rule
described above.
Empty arrays use the same list rule. Source wrapper parentheses remain
ordinary expression boxes. Only derived panic text changes before complete
native checking; element evaluation order, once-only execution, borrowing,
Drop, unwind, const behavior and bounds checks stay native.

The Rust Reference's array-expression grammar and ordered-value semantics
were observed on 2026-11-15. The pinned printer's Oppen/Karlton SCAN/PRINT
lineage informs the existing bounded layout implementation; the original
Oppen report remains unread, and no general printer-equivalence proof is
claimed. Differential payload/rejection/runtime, retained edit/recovery,
optimized measurement and fresh Sol review are separate qualification gates.

Const blocks retain the complete original source as
`tests/fixtures/native-inference-const-block-assert-original.rs`, SHA256
`cfceef37c9056029b90b4482434269ecbe477b2352aa832eb76f07b09e2af688`.
Copied51 rejected it with L0809, while separate stock compile/runtime passed.
That rejection supplies no successful native51 performance baseline.

The owned rule follows Rust `5b7b37ae` `pprust/state/expr.rs:121..137,452..454`:
an extra indentation box wraps `const`, a nonbreaking space and the direct
block printer, without adding another expression box around that block.
Lust `240e08d9` `ast.rs:1257` and `parser.rs:3868..3893` supply the actual
block ID. Its original file/context/end and sole `const` prefix token must
agree with the unwrapped expression span. The existing block printer checks
complete token coverage and refuses unrepresented attributes, empty statements
and unsupported statements. Only derived panic text changes; native const
evaluation, types, Copy, borrowing, moves, Drop and unwind remain subject to
the complete native recheck. The current Rust Reference const-block contract
was observed on 2026-11-15. Direct/retained runtime parity, invalid const cases,
edits/recovery, optimized measurements and independent Sol review remain
qualification obligations until their actual results exist. Full Rust, native
crates/metadata, general Cargo and product lifecycle remain open.
