//! Owned original-AST assertion text extension. The application and protected
//! compilers are unchanged. See docs/compiler-set/proofs/native-assert-text.md.
use crate::{
    ast::{self, Expr, FmtPiece, MacroKind, Stmt},
    diag::{CtxtId, FileId, Span},
    driver,
    intern::Interner,
    lexer::{self, Tok, Token},
};
use std::sync::Arc;

const LIMIT: usize = 65536;

#[derive(Clone)]
enum Event {
    Text(String),
    Begin(isize, bool),
    End,
    Break(isize, isize, bool),
}

// A bounded, offline SCAN/PRINT formulation. Widths are measured in bytes,
// matching the pinned Rust printer, with margin 78 and minimum remaining 60.
// End markers are resolved by the next break outside their box (or EOF),
// not eagerly at End. Their following punctuation therefore contributes to
// the box and final break widths. Nested boxes contribute their flat widths.
#[derive(Default)]
struct Layout {
    events: Vec<Event>,
}
impl Layout {
    fn word(&mut self, s: impl Into<String>) {
        self.events.push(Event::Text(s.into()));
    }
    fn begin(&mut self, offset: isize, consistent: bool) {
        self.events.push(Event::Begin(offset, consistent));
    }
    fn end(&mut self) {
        self.events.push(Event::End);
    }
    fn space(&mut self) {
        self.events.push(Event::Break(1, 0, false));
    }
    fn close_space(&mut self) {
        self.events.push(Event::Break(1, -4, false));
    }
    fn struct_close(&mut self, comma: bool) {
        self.events.push(Event::Break(1, -4, comma));
    }
    fn render(self) -> Option<String> {
        let e = self.events;
        if e.len() > LIMIT {
            return None;
        }
        let mut prefix = vec![0isize];
        for event in &e {
            prefix.push(prefix.last()?.checked_add(match event {
                Event::Text(s) => s.len().try_into().ok()?,
                Event::Break(n, _, _) => *n,
                _ => 0,
            })?);
        }
        let mut widths = vec![0; e.len()];
        let mut boundaries = vec![(e.len(), e.len())];
        for i in (0..e.len()).rev() {
            match e[i] {
                Event::End => {
                    let next = boundaries.last()?.1;
                    boundaries.push((next, next));
                }
                Event::Begin(..) => {
                    let (end, _) = boundaries.pop()?;
                    widths[i] = prefix[end] - prefix[i];
                }
                Event::Break(..) => {
                    widths[i] = prefix[boundaries.last()?.1] - prefix[i];
                    boundaries.last_mut()?.1 = i;
                }
                _ => (),
            }
        }
        if boundaries.len() != 1 {
            return None;
        }
        let (mut out, mut space, mut indent, mut pending) = (String::new(), 78isize, 0isize, 0isize);
        let mut frames: Vec<Option<(isize, bool)>> = Vec::new();
        for (i, event) in e.into_iter().enumerate() {
            match event {
                Event::Text(s) => {
                    if pending < 0 || out.len().checked_add(pending as usize)?.checked_add(s.len())? > LIMIT {
                        return None;
                    }
                    out.extend(std::iter::repeat_n(' ', pending as usize));
                    pending = 0;
                    space -= s.len() as isize;
                    out.push_str(&s);
                }
                Event::Begin(offset, consistent) => {
                    if widths[i] > space {
                        frames.push(Some((indent, consistent)));
                        indent = indent.checked_add(offset)?;
                    } else {
                        frames.push(None);
                    }
                }
                Event::End => {
                    if let Some((old, _)) = frames.pop()? {
                        indent = old;
                    }
                }
                Event::Break(n, offset, comma) => {
                    let fits = match frames.last() {
                        Some(None) => true,
                        Some(Some((_, true))) => false,
                        _ => widths[i] <= space,
                    };
                    if fits {
                        pending += n;
                        space -= n;
                    } else {
                        if comma {
                            out.push(',');
                        }
                        out.push('\n');
                        pending = indent + offset;
                        space = (78 - pending).max(60);
                    }
                }
            }
        }
        frames.is_empty().then_some(out)
    }
}

struct Printer<'a> {
    ast: &'a ast::Ast,
    source: &'a str,
    symbols: &'a Interner,
    out: Layout,
    depth: usize,
}
impl Printer<'_> {
    // Ordered exactly as Rust 5b7b37ae's BinOpKind::precedence. Comparisons
    // are non-associative; every other binary operator is left-associative.
    fn binary_precedence(op: ast::BinOp) -> u8 {
        use ast::BinOp::*;
        match op {
            Or => 1, And => 2,
            Eq | Ne | Lt | Le | Gt | Ge => 3,
            BitOr => 4, BitXor => 5, BitAnd => 6, Shl | Shr => 7,
            Add | Sub => 8, Mul | Div | Rem => 9,
        }
    }
    fn binary_operand(&mut self, id: ast::ExprId, op: ast::BinOp, right: bool) -> Option<()> {
        let (_, parens) = self.expression_source(id)?;
        if parens == 0 {
            match self.ast.exprs.get(id.0 as usize)? {
                Expr::Binary { op: child, .. } => {
                    let (parent, child) = (Self::binary_precedence(op), Self::binary_precedence(*child));
                    // An original Paren is already unambiguous in Rust. If a
                    // native parse would need a new pair, refuse rather than
                    // guess a different source AST or change its panic text.
                    if child < parent || child == parent && (right || op.is_cmp()) {
                        return None;
                    }
                }
                // Rust's closure precedence depends on FixupContext's next
                // operator. This direct-operand case is not modeled yet.
                // Closures inside calls keep their established printing rule.
                Expr::Closure { .. } => return None,
                _ => (),
            }
        }
        self.expr(id)
    }
    // The native parser widens a parenthesized expression's span but retains
    // only a deduplicated ID. Recover each source wrapper, never one pair per
    // ID. A call's argument delimiters and a unit's own `()` are not wrappers.
    fn expression_source(&self, id: ast::ExprId) -> Option<(Span, usize)> {
        let mut span = *self.ast.expr_spans.get(id.0 as usize)?;
        if self.ast.paren_exprs.binary_search(&id.0).is_err() {
            return Some((span, 0));
        }
        let ts = self.tokens(span)?;
        let mut pairs = vec![usize::MAX; ts.len()];
        let mut stack = Vec::new();
        for (i, t) in ts.iter().enumerate() {
            match t.tok {
                Tok::LParen | Tok::LBracket | Tok::LBrace => stack.push(i),
                Tok::RParen | Tok::RBracket | Tok::RBrace => {
                    let open = stack.pop()?;
                    if !matches!((ts[open].tok, t.tok),
                        (Tok::LParen, Tok::RParen) | (Tok::LBracket, Tok::RBracket) | (Tok::LBrace, Tok::RBrace)) {
                        return None;
                    }
                    pairs[open] = i;
                }
                _ => (),
            }
        }
        if !stack.is_empty() { return None; }
        let (mut lo, mut hi, mut count) = (0, ts.len(), 0);
        // A tuple's own delimiters are structural, just like Unit's `()`.
        // The first original element starts immediately inside that pair;
        // any earlier pair is a source Paren around the complete tuple.
        let tuple_start = match self.ast.exprs.get(id.0 as usize)? {
            Expr::Tuple(es) => Some(self.ast.expr_spans.get(es.first()?.0 as usize)?.lo),
            _ => None,
        };
        while hi.checked_sub(lo)? > 2 && ts[lo].tok == Tok::LParen && pairs[lo] == hi - 1 {
            if tuple_start == Some(ts.get(lo + 1)?.span.lo) { break; }
            lo += 1;
            hi -= 1;
            count += 1;
        }
        if count == 0 { return None; }
        span.lo = ts.get(lo)?.span.lo;
        span.hi = ts.get(hi.checked_sub(1)?)?.span.hi;
        Some((span, count))
    }
    fn raw(&self, span: Span) -> Option<&str> {
        self.source.get(span.lo as usize..span.hi as usize)
    }
    fn tokens(&self, span: Span) -> Option<Vec<Token>> {
        let l = lexer::lex(self.raw(span)?, span.file, span.lo, self.symbols).ok()?;
        Some(l.tokens.into_iter().filter(|t| t.tok != Tok::Eof).collect())
    }
    fn atom(&mut self, span: Span) -> Option<()> {
        self.atom_with_mode(span, false)
    }
    fn atom_with_mode(&mut self, span: Span, token_tree: bool) -> Option<()> {
        let raw = self.raw(span)?;
        let tokens = self.tokens(span)?;
        let text = match tokens.as_slice() {
            [token] if token.span == span && matches!(token.tok, Tok::Ident(..) | Tok::Lifetime(..)) => {
                let text = crate::expand::tok_text(&token.tok, &[], self.symbols);
                if token_tree { text } else { lexer::ast_name_text(token.tok, &text).into_owned() }
            }
            _ => raw.to_owned(),
        };
        self.out.word(text);
        Some(())
    }
    fn identifier_text(&self, span: Span, name: crate::intern::Sym) -> Option<String> {
        let tokens = self.tokens(span)?;
        let [token] = tokens.as_slice() else { return None };
        if token.span != span || crate::expand::seg_sym(token.tok, self.symbols)? != name {
            return None;
        }
        let text = crate::expand::tok_text(&token.tok, &[], self.symbols);
        Some(lexer::ast_name_text(token.tok, &text).into_owned())
    }
    fn path(&mut self, span: Span) -> Option<()> {
        let tokens = self.tokens(span)?;
        for t in tokens {
            if !matches!(t.tok, Tok::Ident(_, _) | Tok::SelfLower | Tok::SelfUpper | Tok::Crate | Tok::Super | Tok::PathSep) {
                return None;
            }
            self.atom(t.span)?;
        }
        Some(())
    }
    fn struct_literal(&mut self, span: Span, path: &[ast::PathSeg], fields: &[(crate::intern::Sym, Span, ast::ExprId)], update: Option<ast::ExprId>) -> Option<()> {
        if path.is_empty() || path.iter().any(|p| p.has_args()) {
            return None;
        }
        let tokens = self.tokens(span)?;
        let open = tokens.iter().position(|t| t.tok == Tok::LBrace)?;
        if open == 0 || tokens.last()?.tok != Tok::RBrace {
            return None;
        }
        // The scratch parser owns the grammar. Independently bind its path,
        // field names, shorthand distinction and every value to source tokens;
        // no dropped field attribute or unrepresented token may disappear.
        let names: Vec<_> = tokens[..open].iter().filter(|t| t.tok != Tok::PathSep).collect();
        if names.len() != path.len() {
            return None;
        }
        for (token, segment) in names.iter().zip(path) {
            self.identifier_text(token.span, segment.name)?;
        }
        self.path(Span { hi: tokens[open].span.lo, ..span })?;
        self.out.word(" {");
        if fields.is_empty() && update.is_none() {
            if open + 2 != tokens.len() { return None; }
            self.out.word("}");
            return Some(());
        }
        self.out.begin(0, true);
        let mut cursor = open + 1;
        let cover = |cursor: &mut usize, value: Span| -> Option<()> {
            if value.file != span.file || value.ctxt != span.ctxt || tokens.get(*cursor)?.span.lo != value.lo {
                return None;
            }
            while tokens.get(*cursor)?.span.hi <= value.hi {
                *cursor += 1;
            }
            (tokens.get(cursor.checked_sub(1)?)?.span.hi == value.hi).then_some(())
        };
        for (i, &(name, name_span, value)) in fields.iter().enumerate() {
            if tokens.get(cursor)?.span != name_span {
                return None;
            }
            let raw = self.identifier_text(name_span, name)?;
            if i == 0 { self.out.space(); }
            let value_span = *self.ast.expr_spans.get(value.0 as usize)?;
            if value_span == name_span {
                if !matches!(self.ast.exprs.get(value.0 as usize)?, Expr::Path(p) if p.len() == 1 && p[0].name == name && !p[0].has_args()) {
                    return None;
                }
            } else {
                cursor += 1;
                if tokens.get(cursor)?.tok != Tok::Colon { return None; }
                cursor += 1;
                self.out.word(raw.to_owned());
                self.out.word(": ");
            }
            cover(&mut cursor, value_span)?;
            self.expr(value)?;
            if tokens.get(cursor)?.tok == Tok::Comma { cursor += 1; }
            else if i + 1 < fields.len() || update.is_some() { return None; }
            if i + 1 < fields.len() || update.is_some() {
                self.out.word(",");
                self.out.space();
            } else {
                self.out.struct_close(true);
            }
        }
        if let Some(value) = update {
            if fields.is_empty() { self.out.space(); }
            let dots = tokens.get(cursor)?;
            if dots.tok != Tok::DotDot { return None; }
            self.out.word("..");
            let value_span = *self.ast.expr_spans.get(value.0 as usize)?;
            if value_span == dots.span {
                // Native bare-rest representation; value legality is still
                // decided by the unchanged native checker after text seeding.
                if !matches!(self.ast.exprs.get(value.0 as usize)?, Expr::Range { lo: None, hi: None, inclusive: false }) { return None; }
                cursor += 1;
            } else {
                cursor += 1;
                cover(&mut cursor, value_span)?;
                self.expr(value)?;
            }
            self.out.struct_close(false);
        }
        if cursor != tokens.len() - 1 { return None; }
        self.out.end();
        self.out.word("}");
        Some(())
    }
    fn tuple_literal(&mut self, span: Span, elements: &[ast::ExprId]) -> Option<()> {
        let tokens = self.tokens(span)?;
        if elements.is_empty() || tokens.first()?.tok != Tok::LParen || tokens.last()?.tok != Tok::RParen {
            return None;
        }
        // Bind every original element and separator. In particular `(e)` is
        // never a singleton tuple, and ignored attributes cannot disappear.
        let mut cursor = 1;
        for (i, &element) in elements.iter().enumerate() {
            let value = *self.ast.expr_spans.get(element.0 as usize)?;
            if value.file != span.file || value.ctxt != span.ctxt
                || tokens.get(cursor)?.span.lo != value.lo
            { return None; }
            while tokens.get(cursor)?.span.hi <= value.hi { cursor += 1; }
            if tokens.get(cursor.checked_sub(1)?)?.span.hi != value.hi { return None; }
            if tokens.get(cursor)?.tok == Tok::Comma { cursor += 1; }
            else if i + 1 < elements.len() || elements.len() == 1 { return None; }
        }
        if cursor != tokens.len() - 1 { return None; }
        self.out.word("(");
        self.list(elements)?;
        if elements.len() == 1 { self.out.word(","); }
        self.out.word(")");
        Some(())
    }
    fn array_literal(&mut self, span: Span, elements: &[ast::ExprId]) -> Option<()> {
        let tokens = self.tokens(span)?;
        if tokens.first()?.tok != Tok::LBracket || tokens.last()?.tok != Tok::RBracket {
            return None;
        }
        // Bind all elements and commas to the original source, including an
        // empty list. Unrepresented attributes stay refused.
        let mut cursor = 1;
        for (i, &element) in elements.iter().enumerate() {
            let value = *self.ast.expr_spans.get(element.0 as usize)?;
            if value.file != span.file || value.ctxt != span.ctxt
                || tokens.get(cursor)?.span.lo != value.lo
            { return None; }
            while tokens.get(cursor)?.span.hi <= value.hi { cursor += 1; }
            if tokens.get(cursor.checked_sub(1)?)?.span.hi != value.hi { return None; }
            if tokens.get(cursor)?.tok == Tok::Comma { cursor += 1; }
            else if i + 1 < elements.len() { return None; }
        }
        if cursor != tokens.len() - 1 { return None; }
        // Rust's print_expr_vec has a second indentation box around the
        // brackets, in addition to the ordinary expression and list boxes.
        self.out.begin(4, false);
        self.out.word("[");
        self.list(elements)?;
        self.out.word("]");
        self.out.end();
        Some(())
    }
    fn array_repeat(&mut self, span: Span, value: ast::ExprId, len: ast::ExprId) -> Option<()> {
        let tokens = self.tokens(span)?;
        if tokens.first()?.tok != Tok::LBracket || tokens.last()?.tok != Tok::RBracket {
            return None;
        }
        // Bind both native children and the semicolon to their exact source
        // spans. The length is Rust's anonymous const value, printed as an
        // expression; it is not a new `const` block or a comma-separated list.
        let mut cursor = 1;
        for (child, separator) in [(value, Tok::Semi), (len, Tok::RBracket)] {
            let child_span = *self.ast.expr_spans.get(child.0 as usize)?;
            if child_span.file != span.file || child_span.ctxt != span.ctxt
                || tokens.get(cursor)?.span.lo != child_span.lo
            { return None; }
            while tokens.get(cursor)?.span.hi <= child_span.hi { cursor += 1; }
            if tokens.get(cursor.checked_sub(1)?)?.span.hi != child_span.hi
                || tokens.get(cursor)?.tok != separator
            { return None; }
            if separator == Tok::Semi { cursor += 1; }
        }
        if cursor != tokens.len() - 1 { return None; }
        self.out.begin(4, false);
        self.out.word("[");
        self.expr(value)?;
        self.out.word(";");
        self.out.space();
        self.expr(len)?;
        self.out.word("]");
        self.out.end();
        Some(())
    }
    fn tuple_field(&mut self, span: Span, mut base: ast::ExprId, index: u32) -> Option<()> {
        let mut indexes = vec![index];
        // Lust splits `value.0.1` into two TupleFields but gives both the
        // complete float-token span. Recover that exact chain, not guessed
        // byte offsets or an invented source expression.
        while *self.ast.expr_spans.get(base.0 as usize)? == span {
            if self.ast.paren_exprs.binary_search(&base.0).is_ok() { return None; }
            let Expr::TupleField { base: inner, index } = self.ast.exprs.get(base.0 as usize)? else { return None; };
            indexes.push(*index);
            if indexes.len() > 128 { return None; }
            base = *inner;
        }
        indexes.reverse();
        let base_span = *self.ast.expr_spans.get(base.0 as usize)?;
        if base_span.file != span.file || base_span.ctxt != span.ctxt
            || base_span.lo != span.lo || base_span.hi >= span.hi
        { return None; }
        let tokens = self.tokens(Span { lo: base_span.hi, ..span })?;
        let [dot, number] = tokens.as_slice() else { return None; };
        if dot.tok != Tok::Dot || number.span.hi != span.hi { return None; }
        match number.tok {
            Tok::Int(value, None, None) if indexes.len() == 1 && value == indexes[0] as u128 => (),
            Tok::Float(_) if indexes.len() == 2 => (),
            _ => return None,
        }
        let raw = self.raw(number.span)?;
        let parts: Vec<_> = raw.split('.').collect();
        if parts.len() != indexes.len() || parts.iter().zip(&indexes).any(|(part, index)| *part != index.to_string()) {
            return None;
        }
        let (_, parens) = self.expression_source(base)?;
        if parens == 0 && matches!(self.ast.exprs.get(base.0 as usize)?,
            Expr::Binary { .. } | Expr::Closure { .. } | Expr::Ref { .. }
            | Expr::Deref(_) | Expr::Unary { .. })
        { return None; }
        let extra = indexes.len() - 1;
        self.depth = self.depth.checked_add(extra)?;
        if self.depth > 128 { return None; }
        for _ in 0..extra { self.out.begin(4, false); }
        self.expr(base)?;
        // An unsuffixed float ending in a dot needs the same separator as
        // Rust's Field printer, so the two dots cannot become a range token.
        if parens == 0 && matches!(self.ast.exprs.get(base.0 as usize)?, Expr::Float(_))
            && self.raw(base_span)?.ends_with('.')
        { self.out.word(" "); }
        for (i, index) in indexes.iter().enumerate() {
            self.out.word(".");
            self.out.word(index.to_string());
            if i < extra { self.out.end(); }
        }
        self.depth -= extra;
        Some(())
    }
    fn list(&mut self, es: &[ast::ExprId]) -> Option<()> {
        self.out.begin(0, false);
        for (i, &e) in es.iter().enumerate() {
            if i != 0 {
                self.out.word(",");
                self.out.space();
            }
            self.expr(e)?;
        }
        self.out.end();
        Some(())
    }
    fn pat(&mut self, p: ast::PatId) -> Option<()> {
        match self.ast.pats.get(p.0 as usize)? {
            ast::Pat::Wild | ast::Pat::Binding { .. } => {
                let span = self.ast.pat_spans[p.0 as usize];
                for (i, t) in self.tokens(span)?.iter().enumerate() {
                    if i != 0 {
                        self.out.word(" ");
                    }
                    self.atom(t.span)?;
                }
            }
            _ => return None,
        }
        Some(())
    }
    fn block(&mut self, b: ast::BlockId) -> Option<()> {
        self.block_head(b, false)
    }
    fn bool_annotation(&mut self, pat: ast::PatId, ty: ast::TyId) -> Option<()> {
        let ast::TyExpr::Path(path) = self.ast.tys.get(ty.0 as usize)? else { return None };
        if path.len() != 1 || path[0].has_args() || self.symbols.get(path[0].name) != "bool" { return None; }
        let span = *self.ast.ty_spans.get(ty.0 as usize)?;
        let pattern = *self.ast.pat_spans.get(pat.0 as usize)?;
        if span.ctxt != CtxtId::ROOT || pattern.ctxt != CtxtId::ROOT || pattern.file != span.file { return None; }
        let tokens = self.tokens(span)?;
        if !matches!(tokens.as_slice(), [t] if matches!(t.tok, Tok::Ident(s, _) if s == path[0].name) && t.span == span)
            || self.raw(span)? != "bool"
        { return None; }
        let gap = self.tokens(Span { lo: pattern.hi, hi: span.lo, ..span })?;
        if !matches!(gap.as_slice(), [t] if t.tok == Tok::Colon) { return None; }
        // Rust5b7b print_local_decl: word_space(":") then print_type,
        // within the declaration's existing pattern/type indentation box.
        self.out.word(":");
        self.out.space();
        self.out.word("bool");
        Some(())
    }
    fn block_head(&mut self, b: ast::BlockId, unsafe_head: bool) -> Option<()> {
        let b = self.ast.blocks.get(b.0 as usize)?;
        if !b.inner_attrs.is_empty() || !b.tail_attrs.is_empty() {
            return None;
        }
        // The native block AST omits empty statements. Check that its spans
        // cover every original token, so an omitted `;` (or attribute) cannot
        // silently disappear from the panic text.
        let tokens = self.tokens(b.span)?;
        if tokens.first()?.tok != Tok::LBrace || tokens.last()?.tok != Tok::RBrace {
            return None;
        }
        let mut cursor = 1;
        let cover = |cursor: &mut usize, span: Span| -> Option<()> {
            if tokens.get(*cursor)?.span.lo != span.lo {
                return None;
            }
            while tokens.get(*cursor)?.span.hi <= span.hi {
                *cursor += 1;
            }
            (tokens.get(cursor.checked_sub(1)?)?.span.hi == span.hi).then_some(())
        };
        for s in &b.stmts {
            let (span, semi) = match s {
                Stmt::Let { span, .. } => (*span, false),
                Stmt::Expr { expr, semi, .. } => (self.ast.expr_spans[expr.0 as usize], *semi),
                Stmt::Macro { call, semi } => (call.span, *semi),
                _ => return None,
            };
            cover(&mut cursor, span)?;
            if semi {
                if tokens.get(cursor)?.tok != Tok::Semi {
                    return None;
                }
                cursor += 1;
            }
        }
        if let Some(e) = b.tail {
            cover(&mut cursor, self.ast.expr_spans[e.0 as usize])?;
        }
        if cursor != tokens.len() - 1 {
            return None;
        }
        self.out.begin(0, true);
        self.out.begin(0, false);
        if unsafe_head {
            // Pinned Rust prints `unsafe` inside the block's head box with
            // word_space, before bopen closes that box. This is a breakable
            // head, unlike the anonymous-const keyword/extra indentation box.
            self.out.word("unsafe");
            self.out.space();
        }
        self.out.word("{");
        self.out.end();
        for s in &b.stmts {
            self.out.space();
            match s {
                Stmt::Let { pat, ty, init, els: None, attrs, .. } if attrs.is_empty() => {
                    self.out.begin(4, false);
                    self.out.word("let ");
                    self.out.begin(4, false);
                    self.pat(*pat)?;
                    if let Some(ty) = ty { self.bool_annotation(*pat, *ty)?; }
                    self.out.end();
                    if let Some(e) = init {
                        self.out.word(" =");
                        self.out.space();
                        self.expr(*e)?;
                    }
                    self.out.word(";");
                    self.out.end();
                }
                Stmt::Expr { expr, semi, attrs } if attrs.is_empty() => {
                    // Rust's statement MacCall bypasses the expression ibox.
                    // Lust stores builtin macro statements as Expr::Macro.
                    if matches!(self.ast.exprs[expr.0 as usize], Expr::Macro { .. } | Expr::MacroCall(_))
                        && self.ast.paren_exprs.binary_search(&expr.0).is_err()
                    {
                        self.mac(self.ast.expr_spans[expr.0 as usize])?;
                    } else {
                        self.expr(*expr)?;
                    }
                    if *semi {
                        self.out.word(";");
                    }
                }
                Stmt::Macro { call, semi } => {
                    self.mac(call.span)?;
                    if *semi {
                        self.out.word(";");
                    }
                }
                _ => return None,
            }
        }
        if let Some(e) = b.tail {
            self.out.space();
            self.expr(e)?;
        }
        if !b.stmts.is_empty() || b.tail.is_some() {
            self.out.close_space();
        }
        self.out.word("}");
        self.out.end();
        Some(())
    }
    fn expr(&mut self, id: ast::ExprId) -> Option<()> {
        let (span, parens) = self.expression_source(id)?;
        let frames = parens.checked_add(1)?;
        self.depth = self.depth.checked_add(frames)?;
        if self.depth > 128 || self.out.events.len() > LIMIT {
            return None;
        }
        let e = self.ast.exprs.get(id.0 as usize)?;
        self.out.begin(4, false);
        for _ in 0..parens {
            self.out.word("(");
            self.out.begin(4, false);
        }
        match e {
            Expr::Int(..) | Expr::Float(_) | Expr::Bool(_) | Expr::Str(_) | Expr::Char(_) => self.atom(span)?,
            Expr::Unit => self.out.word("()"),
            Expr::Tuple(elements) => self.tuple_literal(span, elements)?,
            Expr::Array(elements) => self.array_literal(span, elements)?,
            Expr::ArrayRepeat { value, len } => self.array_repeat(span, *value, *len)?,
            Expr::Path(path) if path.iter().all(|p| !p.has_args()) => self.path(span)?,
            Expr::SelfValue => self.out.word("self"),
            Expr::StructLit { path, fields, update } => self.struct_literal(span, path, fields, *update)?,
            Expr::TupleField { base, index } => self.tuple_field(span, *base, *index)?,
            Expr::Binary { op, lhs, rhs } => {
                let left = *self.ast.expr_spans.get(lhs.0 as usize)?;
                let right = *self.ast.expr_spans.get(rhs.0 as usize)?;
                if left.file != span.file || right.file != span.file
                    || left.ctxt != span.ctxt || right.ctxt != span.ctxt
                    || left.lo != span.lo || right.hi != span.hi || left.hi > right.lo
                {
                    return None;
                }
                let gap = self.tokens(Span { lo: left.hi, hi: right.lo, ..span })?;
                // The native lexer retains `>>` as two adjacent `>` tokens.
                let operator_matches = match gap.as_slice() {
                    [one] => self.raw(one.span)? == op.symbol(),
                    [a, b] => *op == ast::BinOp::Shr && a.tok == Tok::Gt && b.tok == Tok::Gt
                        && a.span.hi == b.span.lo,
                    _ => false,
                };
                if !operator_matches {
                    return None;
                }
                self.binary_operand(*lhs, *op, false)?;
                self.out.space();
                self.out.word(op.symbol());
                self.out.space();
                self.binary_operand(*rhs, *op, true)?;
            }
            Expr::Call { callee, args } => {
                self.expr(*callee)?;
                self.out.word("(");
                self.list(args)?;
                self.out.word(")");
            }
            Expr::MethodCall { recv, name, name_span, generic_args, args } if generic_args.is_empty() => {
                self.expr(*recv)?;
                // An unsuffixed trailing-dot literal needs separation from
                // the method dot: `0. .eq(..)`, never the range token `0..`.
                if matches!(self.ast.exprs[recv.0 as usize], Expr::Float(_))
                    && self.raw(self.ast.expr_spans[recv.0 as usize])?.ends_with('.')
                {
                    self.out.word(" ");
                }
                self.out.word(".");
                let name = self.identifier_text(*name_span, *name)?;
                self.out.word(name);
                self.out.word("(");
                self.list(args)?;
                self.out.word(")");
            }
            Expr::Field { base, name } => {
                self.expr(*base)?;
                self.out.word(".");
                let tokens = self.tokens(span)?;
                let name = self.identifier_text(tokens.last()?.span, *name)?;
                self.out.word(name);
            }
            Expr::Index { base, index } => {
                self.expr(*base)?;
                self.out.word("[");
                self.expr(*index)?;
                self.out.word("]");
            }
            Expr::Closure { is_move, is_async: false, params, ret: None, body } => {
                if *is_move {
                    self.out.word("move ");
                }
                self.out.word("|");
                self.out.begin(0, false);
                for (i, p) in params.iter().enumerate() {
                    if p.ty.is_some() || !p.attrs.is_empty() {
                        return None;
                    }
                    if i != 0 {
                        self.out.word(",");
                        self.out.space();
                    }
                    self.pat(p.pat)?;
                }
                self.out.end();
                self.out.word("|");
                self.out.space();
                self.expr(*body)?;
            }
            Expr::Block(b) => self.block(*b)?,
            Expr::UnsafeBlock(b) => {
                let block = self.ast.blocks.get(b.0 as usize)?.span;
                if block.file != span.file || block.ctxt != span.ctxt
                    || block.lo <= span.lo || block.hi != span.hi
                { return None; }
                let prefix = self.tokens(Span { hi: block.lo, ..span })?;
                if !matches!(prefix.as_slice(), [t] if t.tok == Tok::Unsafe
                    && t.span.lo == span.lo)
                { return None; }
                self.block_head(*b, true)?;
            }
            Expr::ConstBlock(b) => {
                let block = self.ast.blocks.get(b.0 as usize)?.span;
                if block.file != span.file || block.ctxt != span.ctxt
                    || block.lo <= span.lo || block.hi != span.hi
                { return None; }
                let prefix = self.tokens(Span { hi: block.lo, ..span })?;
                if !matches!(prefix.as_slice(), [t] if t.tok == Tok::Const
                    && t.span.lo == span.lo)
                { return None; }
                // Rust's anonymous const adds an ibox around `const` and its
                // block, without an extra expression box for the block itself.
                self.out.begin(4, false);
                self.out.word("const ");
                self.block(*b)?;
                self.out.end();
            }
            Expr::Ref { mutable, raw: false, expr } => {
                self.out.word(if *mutable { "&mut " } else { "&" });
                self.expr(*expr)?;
            }
            Expr::Deref(e) => {
                self.out.word("*");
                self.expr(*e)?;
            }
            Expr::Unary { op, expr } => {
                self.out.word(match op {
                    ast::UnOp::Neg => "-",
                    ast::UnOp::Not => "!",
                });
                self.expr(*expr)?;
            }
            Expr::Macro { .. } | Expr::MacroCall(_) => self.mac(span)?,
            // Precedence-sensitive or source-elided forms need their own
            // qualified printing rules before they can enter this extension.
            _ => return None,
        }
        self.out.end();
        for _ in 0..parens {
            self.out.word(")");
            self.out.end();
        }
        self.depth -= frames;
        Some(())
    }
    fn mac(&mut self, span: Span) -> Option<()> {
        let ts = self.tokens(span)?;
        let bang = ts.iter().position(|t| t.tok == Tok::Bang)?;
        if bang == 0 {
            return None;
        }
        for t in &ts[..bang] {
            if !matches!(t.tok, Tok::Ident(_, _) | Tok::PathSep | Tok::Crate | Tok::Super | Tok::SelfLower) {
                return None;
            }
            self.atom(t.span)?;
        }
        self.out.word("!");
        let (end, _) = self.group(&ts, bang + 1, true)?;
        (end == ts.len()).then_some(())
    }
    fn group(&mut self, ts: &[Token], open: usize, top: bool) -> Option<(usize, bool)> {
        if self.depth > 128 {
            return None;
        }
        self.depth += 1;
        let (close, left, right, brace) = match ts.get(open)?.tok {
            Tok::LParen => (Tok::RParen, "(", ")", false),
            Tok::LBracket => (Tok::RBracket, "[", "]", false),
            Tok::LBrace => (Tok::RBrace, "{", "}", true),
            _ => return None,
        };
        if brace {
            self.out.begin(4, true);
            if top {
                self.out.word(" ");
            }
        }
        self.out.word(left);
        let content = ts.get(open + 1)?.tok != close;
        let open_space = brace && content && (top || self.alone(ts[open].span));
        if open_space {
            self.out.space();
        }
        self.out.begin(0, false);
        let mut i = open + 1;
        let mut previous: Option<(usize, usize)> = None;
        while ts.get(i)?.tok != close {
            let start = i;
            if let Some((a, b)) = previous {
                if self.space_between(ts, a, b, start) {
                    self.out.space();
                }
            }
            match ts[i].tok {
                Tok::LParen | Tok::LBracket | Tok::LBrace => i = self.group(ts, i, false)?.0,
                Tok::RParen | Tok::RBracket | Tok::RBrace | Tok::Eof | Tok::DocStr(_) | Tok::Dollar => return None,
                _ => {
                    self.atom_with_mode(ts[i].span, true)?;
                    i += 1;
                }
            }
            previous = Some((start, i - 1));
        }
        self.out.end();
        if open_space {
            self.out.close_space();
        }
        self.out.word(right);
        if brace {
            self.out.end();
        }
        let alone = self.alone(ts[i].span);
        self.depth -= 1;
        Some((i + 1, alone))
    }
    fn alone(&self, span: Span) -> bool {
        let s = self.source.get(span.hi as usize..).unwrap_or("");
        s.is_empty()
            || s.starts_with("//")
            || s.starts_with("/*")
            || s.chars().next().is_some_and(|c| {
                matches!(c, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{85}' | '\u{200e}' | '\u{200f}' | '\u{2028}' | '\u{2029}')
            })
    }
    fn space_between(&self, ts: &[Token], a: usize, b: usize, c: usize) -> bool {
        let ident = |t: &Token| self.raw(t.span).and_then(|s| s.chars().next()).is_some_and(|c| crate::unicode::is_alphabetic(c) || c == '_');
        if !self.alone(ts[b].span) {
            return a == b && ident(&ts[a]) && ident(&ts[c]);
        }
        let punct = |t: &Token| {
            !ident(t)
                && !matches!(
                    t.tok,
                    Tok::Int(..) | Tok::Float(_) | Tok::Str(..) | Tok::ByteStr(_) | Tok::ByteChar(_) | Tok::Char(..) | Tok::Lifetime(_, _)
                )
        };
        let first = (a == b).then_some(ts[a].tok);
        let plain = matches!(first, Some(Tok::Ident(_, _)));
        match (first, ts[c].tok) {
            (Some(Tok::Dot), _) if !punct(&ts[c]) => false,
            (_, Tok::Comma | Tok::Semi | Tok::Dot) if a != b || !punct(&ts[a]) => false,
            (_, Tok::Bang) if plain => false,
            (_, Tok::LParen) if plain || matches!(first, Some(Tok::Fn | Tok::SelfUpper | Tok::Pub)) => false,
            (Some(Tok::Pound), Tok::LBracket) => false,
            _ => true,
        }
    }
}

fn original_text(source: &str, span: Span, symbols: &Interner, known: &crate::intern::Known) -> Option<String> {
    if span.ctxt != CtxtId::ROOT || span.hi.checked_sub(span.lo)? as usize > LIMIT {
        return None;
    }
    let src = source.get(span.lo as usize..span.hi as usize)?;
    let l = lexer::lex(src, span.file, span.lo, symbols).ok()?;
    let mut ts: Vec<_> = l.tokens.into_iter().filter(|t| t.tok != Tok::Eof).collect();
    // list_elem drops argument attributes from the native AST. Their source
    // must remain represented in panic text; refuse until that grammar has
    // an owned preserving interface. A literal containing `#` is one token.
    if ts.iter().any(|t| t.tok == Tok::Pound) {
        return None;
    }
    let bang = ts.iter().position(|t| t.tok == Tok::Bang)?;
    if !matches!(ts.get(bang.checked_sub(1)?)?.tok, Tok::Ident(s, _) if symbols.get(s) == "assert") {
        return None;
    }
    // Parse the original macro into a scratch arena, using only synthetic
    // *wrapper tokens*. No caller bytes, condition or executable AST are changed.
    ts.insert(0, Token { tok: Tok::LBrace, span: Span { hi: span.lo, ..span } });
    ts.push(Token { tok: Tok::RBrace, span: Span { lo: span.hi, ..span } });
    let mut raw = ast::Ast::default();
    let (block, _, _) = crate::parser::parse_body(&ts, &l.strings, &mut raw, symbols, known).ok()?;
    let e = raw.blocks[block.0 as usize].tail?;
    let Expr::Macro { kind: MacroKind::Assert, args, fmt, .. } = &raw.exprs[e.0 as usize] else {
        return None;
    };
    if args.len() != 1 || !fmt.iter().all(|f| matches!(f,FmtPiece::Lit(s) if s == crate::expand::builtin::ASSERT_TEXT_UNKNOWN)) {
        return None;
    }
    let cond = args[0];
    let mut p = Printer { ast: &raw, source, symbols, out: Layout::default(), depth: 0 };
    p.expr(cond)?;
    Some(format!("assertion failed: {}", p.out.render()?))
}

#[cfg(test)]
mod nfc_tests {
    use super::*;

    #[test]
    fn nfc_ast_names_and_macro_tokens_keep_distinct_printing_rules() {
        // Expected conditions are the payloads emitted by the declared Rust
        // compiler for tests/pass/identifier_nfc_assertions.rs.
        for (condition, expected) in [
            ("r#cafe\u{301} == 2", "café == 2"),
            ("r#type == 2", "r#type == 2"),
            ("value.r#cafe\u{301} == 2", "value.café == 2"),
            ("value.r#re\u{301}sume\u{301}()", "value.résumé()"),
            ("(S { r#cafe\u{301}: 1 }).r#cafe\u{301} == 2", "(S { café: 1 }).café == 2"),
            ("vec![r#cafe\u{301}] == vec![2]", "vec![r#café] == vec![2]"),
            ("vec![\"e\u{301}\"] == vec![\"é\"]", "vec![\"e\u{301}\"] == vec![\"é\"]"),
        ] {
            let source = format!("{{ assert!({condition}) }}");
            let symbols = Interner::default();
            let known = crate::intern::Known::new(&symbols);
            let lexed = lexer::lex(&source, FileId(0), 0, &symbols).unwrap();
            let mut ast = ast::Ast::default();
            let (block, _, _) = crate::parser::parse_body(&lexed.tokens, &lexed.strings, &mut ast, &symbols, &known).unwrap();
            let tail = ast.blocks[block.0 as usize].tail.unwrap();
            let Expr::Macro { kind: MacroKind::Assert, args, .. } = &ast.exprs[tail.0 as usize] else { panic!("assert macro missing") };
            let mut printer = Printer { ast: &ast, source: &source, symbols: &symbols, out: Layout::default(), depth: 0 };
            assert!(printer.expr(args[0]).is_some(), "{condition}");
            assert_eq!(printer.out.render().as_deref(), Some(expected), "{condition}");
        }
    }
}

pub(super) fn proposal(input: &ast::Ast, map: &crate::diag::SourceMap, root_text: &str, interner: &Interner, known: &crate::intern::Known) -> Option<ast::Ast> {
    let marker = crate::expand::builtin::ASSERT_TEXT_UNKNOWN;
    let mut changes = Vec::new();
    for (i, e) in input.exprs.iter().enumerate() {
        let Expr::Macro { kind: MacroKind::Assert, args, fmt, .. } = e else {
            continue;
        };
        let unknown = fmt.iter().any(|f| matches!(f,FmtPiece::Lit(s) if s == marker))
            || args
                .iter()
                .skip(1)
                .any(|a| matches!(&input.exprs[a.0 as usize],Expr::Str(s) if input.strings[*s as usize] == marker));
        if !unknown {
            continue;
        }
        let span = input.expr_spans[i];
        let source = map.text(span.file).or_else(|| (span.file == FileId(0)).then_some(root_text));
        if let Some(text) = source.and_then(|src| original_text(src, span, interner, known)) {
            changes.push((i, text));
        }
    }
    if changes.is_empty() {
        return None;
    }
    let mut ast = input.clone();
    for (i, text) in &changes {
        let Expr::Macro { args, fmt, .. } = &mut ast.exprs[*i] else { unreachable!() };
        let args = args.clone();
        for piece in fmt {
            if let FmtPiece::Lit(s) = piece
                && s == marker
            {
                *s = text.clone();
            }
        }
        for a in args.iter().skip(1) {
            if let Expr::Str(s) = &mut ast.exprs[a.0 as usize]
                && ast.strings[*s as usize] == marker
            {
                let next = ast.strings.len().try_into().unwrap();
                ast.strings.push(text.clone());
                *s = next;
            }
        }
    }
    eprintln!("native inference: {} original-AST assertion texts; full native recheck follows", changes.len());
    Some(ast)
}

pub(super) fn seed(session: &mut driver::Session) -> bool {
    let front = &session.front;
    let Some(ast) = proposal(&front.ast, &front.map, &front.src, &front.interner, &front.known) else { return false; };
    session.front.ast = Arc::new(ast);
    session.front.kept = [None, None];
    session.front.last_check = None;
    session.caches.ok.fill(false);
    session.caches.has_out.fill(false);
    session.caches.gen_reuse.clear();
    true
}
