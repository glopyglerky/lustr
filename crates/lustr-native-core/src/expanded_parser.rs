//! Original-token complement to the pinned parser. No tokens or source bytes
//! are replaced. Complete unaffected items and blocks use the native APIs;
//! affected free functions and their enclosing expression grammar are owned.
use crate::pinned::parser as baseline;
pub use baseline::*;
use crate::{ast::*, diag::{CtxtId, Diag, Span}, intern::{Interner, Known, Sym}, lexer::{Lexed, Tok, Token}};
type Result<T> = std::result::Result<T, Diag>;
const LIMIT: usize = 4096;

fn unsupported(span: Span) -> Diag {
    Diag::new("L0404", span, "this unsafe-block context is not implemented by the native extension", "The original program remains rejected.")
}
fn eof(tokens: &mut Vec<Token>) {
    if tokens.last().is_none_or(|t| t.tok != Tok::Eof) {
        let span = tokens.last().map_or(Span::default(), |t| Span { lo: t.span.hi, ..t.span });
        tokens.push(Token { tok: Tok::Eof, span });
    }
}
fn has_unsafe(tokens: &[Token]) -> bool { tokens.iter().any(|t| t.tok == Tok::Unsafe) }

pub fn parse(l: &Lexed, i: &Interner, k: &Known) -> Result<Ast> { parse_seeded(l, i, k, &[]) }
pub fn parse_seeded(l: &Lexed, i: &Interner, k: &Known, seed: &[Attr]) -> Result<Ast> {
    let failure = match baseline::parse_seeded(l, i, k, seed) {
        Ok(ast) => return Ok(ast),
        Err(d) if d.code == "L0404" => d,
        Err(d) => return Err(d),
    };
    // Scope/attribute and macro-generated safety semantics need their own
    // qualification. Refusal is conservative; cfg_syntax is never enabled.
    if l.tokens.len() > LIMIT || !seed.is_empty()
        || l.tokens.iter().any(|t| t.tok == Tok::Pound)
    { return Err(failure); }
    let scan = crate::scan::scan(l);
    let mut parts = Vec::new();
    let mut at = 0;
    for f in scan.fns.iter().filter(|f| !f.nested && has_unsafe(&l.tokens[f.body.clone()])) {
        let Some(start) = l.tokens.iter().position(|t| t.tok == Tok::Fn && t.span.lo == f.fn_offset) else { return Err(failure) };
        if start < at { return Err(failure); }
        // The complement owns this function's grammar. Unrelated expanded
        // items continue through the original parser with their contexts.
        if l.tokens[start..f.body.end].iter().any(|t| t.span.ctxt != CtxtId::ROOT) {
            return Err(failure);
        }
        if at != start { parts.push(baseline::parse_items_seeded(l, at..start, i, k, seed)?); }
        let mut p = Complement::new(&l.tokens[start..f.body.end], &l.strings, Ast::default(), i, k);
        parts.push(p.function()?);
        at = f.body.end;
    }
    if at == 0 { return Err(failure); }
    if at < l.tokens.len() { parts.push(baseline::parse_items_seeded(l, at..l.tokens.len(), i, k, seed)?); }
    let ast = Ast::concat(parts);
    bounded(&ast)?;
    Ok(ast)
}
pub fn parse_items(l: &Lexed, r: std::ops::Range<usize>, i: &Interner, k: &Known) -> Result<Ast> {
    parse_items_seeded(l, r, i, k, &[])
}
pub fn parse_items_seeded(l: &Lexed, r: std::ops::Range<usize>, i: &Interner, k: &Known, seed: &[Attr]) -> Result<Ast> {
    match baseline::parse_items_seeded(l, r.clone(), i, k, seed) {
        Ok(ast) => return Ok(ast),
        Err(d) if d.code == "L0404" => (),
        Err(d) => return Err(d),
    }
    let mut tokens = l.tokens[r].to_vec(); eof(&mut tokens);
    parse_seeded(&Lexed { tokens, strings: l.strings.clone() }, i, k, seed)
}
pub fn parse_body(t: &[Token], s: &[String], a: &mut Ast, i: &Interner, k: &Known) -> Result<ParsedBody> {
    parse_body_seeded(t, s, a, i, k, &[])
}
pub fn parse_body_seeded(t: &[Token], s: &[String], a: &mut Ast, i: &Interner, k: &Known, seed: &[Attr]) -> Result<ParsedBody> {
    if !has_unsafe(t) { return baseline::parse_body_seeded(t, s, a, i, k, seed); }
    if t.len() > LIMIT || !seed.is_empty() || t.iter().any(|t| t.span.ctxt != CtxtId::ROOT || t.tok == Tok::Pound) {
        return Err(unsupported(t.first().map_or(Span::default(), |t| t.span)));
    }
    // Failed parses are transactional. No partial owned arena is published.
    let mut p = Complement::new(t, s, a.clone(), i, k);
    let begin = (p.ast.exprs.len() as u32, p.ast.pats.len() as u32);
    let body = p.block()?;
    if p.at != t.len() { return Err(p.error()); }
    bounded(&p.ast)?;
    p.ast.paren_exprs.sort_unstable(); p.ast.paren_exprs.dedup();
    let ranges = ((begin.0, p.ast.exprs.len() as u32), (begin.1, p.ast.pats.len() as u32));
    *a = p.ast;
    Ok((body, ranges.0, ranges.1))
}
fn bounded(a: &Ast) -> Result<()> {
    // This count bounds the depth of every acyclic parsed arena, including
    // delegated blocks. The complement never introduces a cyclic reference.
    if a.exprs.len() + a.blocks.len() + a.pats.len() + a.tys.len() > LIMIT {
        Err(unsupported(a.expr_spans.first().copied().unwrap_or_default()))
    } else { Ok(()) }
}

struct Complement<'a> {
    t: &'a [Token], strings: &'a [String], at: usize, gt: bool,
    ast: Ast, i: &'a Interner, k: &'a Known, depth: usize, str_base: u32,
}
impl<'a> Complement<'a> {
    fn new(t: &'a [Token], strings: &'a [String], mut ast: Ast, i: &'a Interner, k: &'a Known) -> Self {
        // Tokens keep their original string indexes; when appending a block,
        // the caller's table is extended and tokens are handled by the native
        // block API instead. Owned parses start with a complete original table.
        let str_base = ast.strings.len() as u32;
        ast.strings.extend_from_slice(strings);
        Self { t, strings, at: 0, gt: false, ast, i, k, depth: 0, str_base }
    }
    fn tok(&self) -> Tok { if self.gt { Tok::Gt } else { self.t.get(self.at).map_or(Tok::Eof, |t| t.tok) } }
    fn span(&self) -> Span { self.t.get(self.at).or_else(|| self.t.last()).map_or(Span::default(), |t| t.span) }
    fn error(&self) -> Diag { unsupported(self.span()) }
    fn bump(&mut self) -> Token {
        if self.gt { self.gt = false; return Token { tok: Tok::Gt, span: self.t[self.at - 1].span }; }
        let Some(t) = self.t.get(self.at).copied() else { return Token { tok: Tok::Eof, span: self.span() }; };
        self.at += 1; t
    }
    fn eat(&mut self, tok: Tok) -> bool { if self.tok() == tok { self.bump(); true } else { false } }
    fn need(&mut self, tok: Tok) -> Result<Token> { if self.tok() == tok { Ok(self.bump()) } else { Err(self.error()) } }
    fn name(&mut self) -> Result<(Sym, Span)> { if let Tok::Ident(n) = self.tok() { Ok((n, self.bump().span)) } else { Err(self.error()) } }
    fn expr_node(&mut self, e: Expr, span: Span) -> Result<ExprId> {
        if self.ast.exprs.len() >= LIMIT { return Err(self.error()); }
        let id = ExprId(self.ast.exprs.len() as u32); self.ast.exprs.push(e); self.ast.expr_spans.push(span); Ok(id)
    }
    fn ty_node(&mut self, t: TyExpr, span: Span) -> TyId {
        let id = TyId(self.ast.tys.len() as u32); self.ast.tys.push(t); self.ast.ty_spans.push(span); id
    }
    fn function(&mut self) -> Result<Ast> {
        let start = self.need(Tok::Fn)?.span;
        let (name, name_span) = self.name()?;
        self.need(Tok::LParen)?;
        let mut params = Vec::new();
        while self.tok() != Tok::RParen {
            let span = self.span(); let mutable = self.eat(Tok::Mut); let (name, _) = self.name()?;
            self.need(Tok::Colon)?; let ty = self.ty()?;
            params.push(Param { name, mutable, pat: None, ty, attrs: vec![], span: span.to(self.ast.ty_spans[ty.0 as usize]) });
            if !self.eat(Tok::Comma) { break; }
        }
        self.need(Tok::RParen)?;
        let ret = if self.eat(Tok::Arrow) { Some(self.ty()?) } else { None };
        let body = self.block()?;
        if self.at != self.t.len() { return Err(self.error()); }
        let span = start.to(self.ast.blocks[body.0 as usize].span);
        let f = FnDef { name, name_span, is_test: false, vis: Vis::Private, attrs: vec![], qual: FnQual::default(), generics: Generics::default(), self_param: None, params, variadic: false, ret, where_clause: WhereClause::default(), body: Some(body), exprs: (0, self.ast.exprs.len() as u32), pats: (0, self.ast.pats.len() as u32) };
        self.ast.paren_exprs.sort_unstable(); self.ast.paren_exprs.dedup();
        self.ast.items.push(Item { vis: Vis::Private, attrs: vec![], kind: ItemKind::Fn(f), span });
        bounded(&self.ast)?;
        Ok(std::mem::take(&mut self.ast))
    }
    fn ty(&mut self) -> Result<TyId> {
        self.depth += 1;
        if self.depth > 128 { return Err(self.error()); }
        let start = self.span();
        let node = if self.eat(Tok::Amp) {
            let mutable = self.eat(Tok::Mut); TyExpr::Ref { lifetime: None, mutable, inner: self.ty()? }
        } else if self.eat(Tok::LParen) {
            self.need(Tok::RParen)?; TyExpr::Unit
        } else { TyExpr::Path(self.path(true)?) };
        let end = self.t[self.at - 1].span;
        self.depth -= 1;
        Ok(self.ty_node(node, start.to(end)))
    }
    fn path(&mut self, types: bool) -> Result<Vec<PathSeg>> {
        let mut path = Vec::new();
        loop {
            let (name, _) = self.name()?;
            let args = if types && self.eat(Tok::Lt) {
                let mut args = Vec::new();
                loop { args.push(GenericArg::Type(self.ty()?)); if !self.eat(Tok::Comma) { break; } }
                if self.tok() == Tok::Shr { self.bump(); self.gt = true; } else { self.need(Tok::Gt)?; }
                GenericArgs::Angle(args)
            } else { GenericArgs::None };
            path.push(PathSeg { name, args });
            if !self.eat(Tok::PathSep) { break; }
        }
        Ok(path)
    }
    fn block_end(&self) -> Result<usize> {
        let mut stack = Vec::new();
        for n in self.at..self.t.len() {
            let tok = self.t[n].tok;
            match tok {
                Tok::LBrace => stack.push(Tok::RBrace), Tok::LParen => stack.push(Tok::RParen), Tok::LBracket => stack.push(Tok::RBracket),
                Tok::RBrace | Tok::RParen | Tok::RBracket => { if stack.pop() != Some(tok) { return Err(unsupported(self.t[n].span)); } if stack.is_empty() { return Ok(n + 1); } }
                _ => (),
            }
        }
        Err(self.error())
    }
    fn block(&mut self) -> Result<BlockId> {
        if self.tok() != Tok::LBrace { return Err(self.error()); }
        let end = self.block_end()?;
        // A complete original block is the native parser's public boundary.
        // Its entire contents, strings, scopes and checking stay native.
        if !has_unsafe(&self.t[self.at..end]) {
            let (b, _, _) = baseline::parse_body(&self.t[self.at..end], self.strings, &mut self.ast, self.i, self.k)?;
            self.at = end; return Ok(b);
        }
        self.depth += 1; if self.depth > 128 { return Err(self.error()); }
        let start = self.need(Tok::LBrace)?.span;
        let mut stmts = Vec::new(); let mut tail = None;
        while self.tok() != Tok::RBrace {
            if self.eat(Tok::Semi) { continue; }
            if let Some(keyword) = self.function_item_keyword() {
                self.function_item(keyword, &mut stmts)?;
                continue;
            }
            if self.eat(Tok::Let) {
                let span = self.t[self.at - 1].span;
                let pat_start = self.span();
                let mutable = self.eat(Tok::Mut); let (name, name_span) = self.name()?;
                let pat = PatId(self.ast.pats.len() as u32);
                self.ast.pats.push(if self.i.get(name) == "_" { Pat::Wild } else { Pat::Binding { name, mutable, by_ref: false } }); self.ast.pat_spans.push(pat_start.to(name_span));
                let ty = if self.eat(Tok::Colon) { Some(self.ty()?) } else { None };
                let init = if self.eat(Tok::Eq) { Some(self.expr(0)?) } else { None };
                let close = self.need(Tok::Semi)?.span;
                stmts.push(Stmt::Let { pat, ty, init, els: None, attrs: vec![], span: span.to(close) });
            } else {
                // Statement-position blocks end at their closing brace.
                // Only a qualified dot continuation changes that boundary;
                // parentheses enter the ordinary expression grammar instead.
                let mut block_like = matches!(self.tok(), Tok::Unsafe | Tok::LBrace);
                let expr = if block_like {
                    let e = self.primary()?;
                    if self.tok() == Tok::Dot {
                        block_like = false;
                        let e = self.postfix(e)?;
                        self.binary_rest(e, 0)?
                    } else { e }
                } else { self.expr(0)? };
                let semi = self.eat(Tok::Semi);
                if !semi && self.tok() == Tok::RBrace { tail = Some(expr); break; }
                if !semi && !block_like { return Err(self.error()); }
                stmts.push(Stmt::Expr { expr, semi, attrs: vec![] });
            }
        }
        let close = self.need(Tok::RBrace)?.span; self.depth -= 1;
        let id = BlockId(self.ast.blocks.len() as u32);
        self.ast.blocks.push(Block { inner_attrs: vec![], stmts, tail, tail_attrs: vec![], span: start.to(close) });
        Ok(id)
    }
    fn function_item_keyword(&self) -> Option<Span> {
        // Discriminate the supported item headers from expressions such as
        // `const { ... }`. The native item parser still owns the complete
        // header, FnQual and body grammar; no qualifier is consumed or reset.
        let keyword = match self.tok() {
            Tok::Fn => self.at,
            Tok::Const if self.t.get(self.at + 1).is_some_and(|t| t.tok == Tok::Fn) => self.at + 1,
            _ => return None,
        };
        Some(self.t[keyword].span)
    }
    fn function_item(&mut self, keyword: Span, stmts: &mut Vec<Stmt>) -> Result<()> {
        // Discover a complete original item, then let the pinned item parser
        // own its grammar and diagnostics. The scan supplies a boundary only;
        // it cannot establish validity, a target binding or call safety.
        let lexed = Lexed { tokens: self.t.to_vec(), strings: self.strings.to_vec() };
        let scan = crate::scan::scan(&lexed);
        let Some(item) = scan.fns.iter().find(|f| f.fn_offset == keyword.lo) else { return Err(self.error()); };
        let end = item.body.end;
        if end <= self.at || end > self.t.len() { return Err(self.error()); }
        let chunk = baseline::parse_items(&lexed, self.at..end, self.i, self.k)?;
        if chunk.items.len() != 1 || !matches!(chunk.items[0].kind, ItemKind::Fn(_))
            || !chunk.inner_attrs.is_empty()
        { return Err(self.error()); }
        // append_chunk shifts every arena reference and nested statement-item
        // index. Keep the item's original FnDef ranges and source spans, and
        // place it in this block rather than the module-level item namespace.
        let base = self.ast.stmt_items.len().try_into().map_err(|_| self.error())?;
        let (items, nested, _) = self.ast.append_chunk(chunk, base);
        self.ast.stmt_items.extend(nested);
        let index = self.ast.stmt_items.len().try_into().map_err(|_| self.error())?;
        self.ast.stmt_items.extend(items);
        stmts.push(Stmt::Item(index));
        self.at = end;
        bounded(&self.ast)
    }
    fn operator(tok: Tok) -> Option<(u8, BinOp)> {
        use BinOp::*;
        Some(match tok { Tok::OrOr => (1,Or), Tok::AndAnd => (2,And), Tok::EqEq => (3,Eq), Tok::Ne => (3,Ne), Tok::Lt => (3,Lt), Tok::Le => (3,Le), Tok::Gt => (3,Gt), Tok::Ge => (3,Ge), Tok::Pipe => (4,BitOr), Tok::Caret => (5,BitXor), Tok::Amp => (6,BitAnd), Tok::Shl => (7,Shl), Tok::Shr => (7,Shr), Tok::Plus => (8,Add), Tok::Minus => (8,Sub), Tok::Star => (9,Mul), Tok::Slash => (9,Div), Tok::Percent => (9,Rem), _ => return None })
    }
    fn expr(&mut self, min: u8) -> Result<ExprId> {
        self.depth += 1; if self.depth > 128 { return Err(self.error()); }
        let first = self.primary()?;
        let lhs = self.postfix(first)?;
        let result = self.binary_rest(lhs, min);
        self.depth -= 1; result
    }
    fn binary_rest(&mut self, mut lhs: ExprId, min: u8) -> Result<ExprId> {
        loop {
            let Some((prec, op)) = Self::operator(self.tok()) else { break };
            if prec < min { break; }
            if prec == 3 && matches!(self.ast.expr(lhs), Expr::Binary { op: BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge, .. }) && !self.ast.paren_exprs.contains(&lhs.0) { return Err(self.error()); }
            self.bump(); let rhs = self.expr(prec + 1)?;
            let span = self.ast.expr_span(lhs).to(self.ast.expr_span(rhs));
            lhs = self.expr_node(Expr::Binary { op, lhs, rhs }, span)?;
        }
        Ok(lhs)
    }
    fn args(&mut self, close: Tok) -> Result<Vec<ExprId>> {
        let mut args = Vec::new();
        while self.tok() != close { args.push(self.expr(0)?); if !self.eat(Tok::Comma) { break; } }
        self.need(close)?; Ok(args)
    }
    fn primary(&mut self) -> Result<ExprId> {
        let start = self.span();
        let e = match self.tok() {
            Tok::Unsafe => { self.bump(); let b = self.block()?; self.expr_node(Expr::UnsafeBlock(b), start.to(self.ast.blocks[b.0 as usize].span))? }
            Tok::LBrace => { let b = self.block()?; self.expr_node(Expr::Block(b), self.ast.blocks[b.0 as usize].span)? }
            Tok::True | Tok::False => { let v = self.bump().tok == Tok::True; self.expr_node(Expr::Bool(v), start)? }
            Tok::Int(v,s,_) => { self.bump(); self.expr_node(Expr::Int(v,s), start)? }
            Tok::Str(s,_) => { self.bump(); self.expr_node(Expr::Str(s + self.str_base), start)? }
            Tok::Char(c,_) => { self.bump(); self.expr_node(Expr::Char(c), start)? }
            Tok::Ident(_) => {
                let path = self.path(false)?;
                if self.eat(Tok::Bang) {
                    if path.len() != 1 || self.i.get(path[0].name) != "assert" { return Err(self.error()); }
                    let open = self.bump(); let close = match open.tok { Tok::LParen => Tok::RParen, Tok::LBracket => Tok::RBracket, Tok::LBrace => Tok::RBrace, _ => return Err(self.error()) };
                    let arg = self.expr(0)?;
                    let comma = if self.tok() == Tok::Comma { Some(self.bump()) } else { None };
                    let mut args = vec![arg];
                    let mut fmt = vec![FmtPiece::Lit(crate::expand::builtin::ASSERT_TEXT_UNKNOWN.into())];
                    if self.tok() != close {
                        // The unchanged native expander's assert_message API
                        // appends `, "{}", UNKNOWN` at the WHOLE call span.
                        // Preserve its string argument and formatting plan;
                        // original-AST seeding later supplies only panic text.
                        // Source-written messages do not have this provenance.
                        let format = self.bump();
                        let separator = self.need(Tok::Comma)?;
                        let payload = self.bump();
                        let end = self.need(close)?.span;
                        let whole = start.to(end);
                        let (Tok::Str(f,None),Tok::Str(s,None)) = (format.tok,payload.tok) else { return Err(self.error()); };
                        if self.strings.get(f as usize).map(String::as_str) != Some("{}")
                            || self.strings.get(s as usize).map(String::as_str) != Some(crate::expand::builtin::ASSERT_TEXT_UNKNOWN)
                            || ![comma.ok_or_else(||self.error())?,format,separator,payload].iter().all(|t|t.span == whole)
                        { return Err(unsupported(whole)); }
                        args.push(self.expr_node(Expr::Str(s + self.str_base),payload.span)?);
                        fmt = vec![FmtPiece::Arg { index: 1, debug: false, spec: FmtSpec::default() }];
                        self.expr_node(Expr::Macro { kind: MacroKind::Assert, args, fmt, repeat: false },whole)?
                    } else {
                        let end = self.need(close)?.span;
                        self.expr_node(Expr::Macro { kind: MacroKind::Assert, args, fmt, repeat: false },start.to(end))?
                    }
                } else { let end = self.t[self.at - 1].span; self.expr_node(Expr::Path(path), start.to(end))? }
            }
            Tok::LParen => {
                self.bump();
                if self.eat(Tok::RParen) { self.expr_node(Expr::Unit, start.to(self.t[self.at-1].span))? } else {
                    let first = self.expr(0)?;
                    if self.eat(Tok::Comma) {
                        let mut es = vec![first]; while self.tok() != Tok::RParen { es.push(self.expr(0)?); if !self.eat(Tok::Comma) { break; } }
                        let end = self.need(Tok::RParen)?.span; self.expr_node(Expr::Tuple(es), start.to(end))?
                    } else { let end = self.need(Tok::RParen)?.span; self.ast.expr_spans[first.0 as usize] = start.to(end); self.ast.paren_exprs.push(first.0); first }
                }
            }
            Tok::LBracket => {
                self.bump();
                if self.eat(Tok::RBracket) { self.expr_node(Expr::Array(vec![]), start.to(self.t[self.at-1].span))? } else {
                    let value = self.expr(0)?;
                    if self.eat(Tok::Semi) { let len = self.expr(0)?; let end = self.need(Tok::RBracket)?.span; self.expr_node(Expr::ArrayRepeat { value, len }, start.to(end))? }
                    else { let mut es = vec![value]; while self.eat(Tok::Comma) && self.tok() != Tok::RBracket { es.push(self.expr(0)?); } let end = self.need(Tok::RBracket)?.span; self.expr_node(Expr::Array(es), start.to(end))? }
                }
            }
            Tok::OrOr | Tok::Move => {
                let is_move = self.eat(Tok::Move); self.need(Tok::OrOr)?;
                let body = self.expr(0)?;
                self.expr_node(Expr::Closure { is_move, is_async: false, params: vec![], ret: None, body }, start.to(self.ast.expr_span(body)))?
            }
            Tok::Bang | Tok::Minus | Tok::Star | Tok::Amp => {
                let tok = self.bump().tok; let mutable = tok == Tok::Amp && self.eat(Tok::Mut); let expr = self.expr(10)?; let span = start.to(self.ast.expr_span(expr));
                let node = match tok { Tok::Star => Expr::Deref(expr), Tok::Amp => Expr::Ref { mutable, raw: false, expr }, Tok::Bang => Expr::Unary { op: UnOp::Not, expr }, _ => Expr::Unary { op: UnOp::Neg, expr } }; self.expr_node(node, span)?
            }
            _ => return Err(self.error()),
        };
        Ok(e)
    }
    fn postfix(&mut self, mut e: ExprId) -> Result<ExprId> {
        loop {
            let left = self.ast.expr_span(e);
            if self.eat(Tok::LParen) { let args = self.args(Tok::RParen)?; e = self.expr_node(Expr::Call { callee: e, args }, left.to(self.t[self.at-1].span))?; }
            else if self.eat(Tok::LBracket) { let index = self.expr(0)?; let end = self.need(Tok::RBracket)?.span; e = self.expr_node(Expr::Index { base: e, index }, left.to(end))?; }
            else if self.eat(Tok::Dot) {
                let (name,name_span) = self.name()?;
                if self.eat(Tok::LParen) { let args = self.args(Tok::RParen)?; e = self.expr_node(Expr::MethodCall { recv: e, name, name_span, generic_args: vec![], args }, left.to(self.t[self.at-1].span))?; }
                else { e = self.expr_node(Expr::Field { base:e,name }, left.to(name_span))?; }
            } else { break; }
        }
        Ok(e)
    }
}
