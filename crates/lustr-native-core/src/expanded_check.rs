//! Collection interface for the opt-in native frontend. The original parsed
//! AST remains the driver's source representation. Only a safety-qualified
//! clone reaches unchanged native collection, checking and generation.
use crate::pinned::check as baseline;
pub use baseline::*;
pub(crate) use baseline::collect::Program;

// The pinned generic checker calls this private-root numeric relation through
// crate::check. Forward to the original private function through the additive
// interface beside its include; no numeric semantics are reimplemented here.
pub(crate) struct Ctx;
impl Ctx {
    pub(crate) fn numeric_from_ok(from: crate::ty::TyKind, to: crate::ty::TyKind) -> bool {
        baseline::lustr_numeric_from_ok(from, to)
    }
}

pub mod collect {
    pub use super::baseline::collect::*;
    use crate::{ast::{Ast, BlockId, Expr, ExprId, GenericArgs, Pat, Stmt, TyExpr}, def::CrateIds, diag::{CtxtId, Diag}, intern::{Interner, Known, Sym}};
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use super::baseline::collect as original;
    #[path = "safe_calls.rs"]
    mod safe_calls;
    #[path = "provenance.rs"]
    pub(crate) mod provenance;

    // A safety derivation for boolean locals, not permission for arbitrary
    // paths or let initializers. Every local value comes from this grammar.
    // The original native checker still resolves patterns/items, annotations,
    // types and lifetimes on the unmodified body after this safety check.
    struct SafeBody<'a> {
        ast: &'a Ast,
        interner: &'a Interner,
        locals: Vec<(Sym, CtxtId)>,
        unsafe_sites: BTreeSet<usize>,
        calls: BTreeSet<u32>,
        remaining: usize,
    }
    impl SafeBody<'_> {
        fn expr(&mut self, id: ExprId, depth: usize) -> bool {
            // The leaf is one expression visit beyond its enclosing block.
            // Keep the previous 0..=128 block-nesting bound unchanged.
            if depth > 129 || self.remaining == 0 { return false; }
            self.remaining -= 1;
            let Some(span) = self.ast.expr_spans.get(id.0 as usize) else { return false };
            if span.ctxt != CtxtId::ROOT { return false; }
            match self.ast.exprs.get(id.0 as usize) {
                Some(Expr::Bool(_)) => true,
                Some(Expr::Path(path)) if path.len() == 1 && matches!(path[0].args, GenericArgs::None) => {
                    // Search from the newest binding. The key includes the
                    // name's syntax context; items and outer captures cannot
                    // acquire safety permission through a name match.
                    self.locals.iter().rev().any(|key| *key == (path[0].name, span.ctxt))
                }
                Some(Expr::Block(b)) => self.block(*b, depth),
                Some(Expr::UnsafeBlock(b)) => {
                    if !self.block(*b, depth) { return false; }
                    self.unsafe_sites.insert(id.0 as usize);
                    true
                }
                Some(Expr::Call { callee, args }) => {
                    // This is a deferred obligation, never a name-based
                    // safety grant. Native body checking must resolve this
                    // exact call in its actual scope before collection returns.
                    if self.ast.expr_spans.get(callee.0 as usize).is_none_or(|s| s.ctxt != CtxtId::ROOT)
                        || !matches!(self.ast.exprs.get(callee.0 as usize), Some(Expr::Path(p))
                            if !p.is_empty() && p.iter().all(|s| matches!(s.args, GenericArgs::None)))
                    { return false; }
                    for arg in args { if !self.expr(*arg, depth + 1) { return false; } }
                    self.calls.insert(id.0);
                    true
                }
                _ => false,
            }
        }
        fn block(&mut self, id: BlockId, depth: usize) -> bool {
            if depth > 128 || self.remaining == 0 { return false; }
            self.remaining -= 1;
            let Some(b) = self.ast.blocks.get(id.0 as usize) else { return false };
            if b.span.ctxt != CtxtId::ROOT || !b.inner_attrs.is_empty() || !b.tail_attrs.is_empty() { return false; }
            let base = self.locals.len();
            let result = (|| {
                for stmt in &b.stmts {
                    match stmt {
                        Stmt::Let { pat, ty, init: Some(init), els: None, attrs, .. } if attrs.is_empty() => {
                            // A type annotation is a plain boolean type only;
                            // no const arguments or hidden type expressions.
                            if let Some(ty) = ty {
                                if self.ast.ty_spans.get(ty.0 as usize).is_none_or(|s| s.ctxt != CtxtId::ROOT)
                                    || !matches!(self.ast.tys.get(ty.0 as usize), Some(TyExpr::Path(p))
                                        if p.len() == 1 && matches!(p[0].args, GenericArgs::None)
                                        && self.interner.get(p[0].name) == "bool")
                                { return false; }
                            }
                            // The initializer sees the previous scope. Only
                            // after it checks does a new binding shadow it.
                            if !self.expr(*init, depth + 1) { return false; }
                            let Some(span) = self.ast.pat_spans.get(pat.0 as usize) else { return false };
                            if span.ctxt != CtxtId::ROOT { return false; }
                            match self.ast.pats.get(pat.0 as usize) {
                                Some(Pat::Binding { name, by_ref: false, .. }) => self.locals.push((*name, span.ctxt)),
                                Some(Pat::Wild) => (),
                                _ => return false,
                            }
                        }
                        Stmt::Expr { expr, attrs, .. } if attrs.is_empty() => {
                            if !self.expr(*expr, depth + 1) { return false; }
                        }
                        _ => return false,
                    }
                }
                b.tail.is_some_and(|tail| self.expr(tail, depth + 1))
            })();
            // Nested bindings never escape, including on failed derivations.
            self.locals.truncate(base);
            result
        }
    }
    fn lower(ast: Arc<Ast>, interner: &Interner) -> Result<(Arc<Ast>, BTreeSet<u32>), Vec<Diag>> {
        let mut sites = Vec::new();
        for (n,e) in ast.exprs.iter().enumerate() {
            if let Expr::UnsafeBlock(b) = e {
                let span = ast.expr_spans[n];
                let valid_span = ast.blocks.get(b.0 as usize).is_some_and(|block|
                    span.file == block.span.file && span.ctxt == block.span.ctxt
                    && span.lo < block.span.lo && span.hi >= block.span.hi);
                if !valid_span {
                    return Err(vec![Diag::new("L0404", span, "native unsafe-block span is outside the qualified safety grammar", "The original unsafe context must be preserved.")]);
                }
                sites.push((n,*b));
            }
        }
        if sites.is_empty() { return Ok((ast, BTreeSet::new())); }
        // Outer blocks establish the lexical context for nested unsafe sites.
        // A nested site is covered only by an actual successful traversal;
        // source containment alone never establishes its safety.
        sites.sort_by_key(|(n,_)| std::cmp::Reverse(ast.expr_spans[*n].hi - ast.expr_spans[*n].lo));
        let mut qualified = BTreeSet::new();
        let mut calls = BTreeSet::new();
        for (n,_) in &sites {
            if qualified.contains(n) { continue; }
            let mut check = SafeBody { ast: &ast, interner, locals: Vec::new(), unsafe_sites: BTreeSet::new(), calls: BTreeSet::new(), remaining: 4096 };
            if !check.expr(ExprId(*n as u32), 0) {
                return Err(vec![Diag::new("L0404", ast.expr_spans[*n], "native unsafe-block body is outside the qualified safety grammar", "Boolean values, initialized boolean locals, lexical blocks and checked safe Rust function calls are implemented here.")]);
            }
            qualified.extend(check.unsafe_sites);
            calls.extend(check.calls);
        }
        let mut lowered = (*ast).clone();
        for (n,b) in &sites { lowered.exprs[*n] = Expr::Block(*b); }
        eprintln!("native frontend: {} bounded unsafe blocks, {} pending call obligations; original AST retained; full native recheck follows", sites.len(), calls.len());
        Ok((Arc::new(lowered), calls))
    }
    pub(crate) fn restore_original_unsafe(ast: &mut Ast, original: &Ast) -> bool {
        for (n,node) in original.exprs.iter().enumerate() {
            if let Expr::UnsafeBlock(b) = node {
                if ast.expr_spans.get(n) != original.expr_spans.get(n)
                    || !matches!(ast.exprs.get(n),Some(Expr::Block(actual)) if actual == b)
                { return false; }
                ast.exprs[n] = node.clone();
            }
        }
        true
    }
    pub fn collect(a: Arc<Ast>, i: Arc<Interner>, k: Arc<Known>, c: CrateIds) -> Result<Program,Vec<Diag>> {
        let (a,calls) = lower(a,&i)?;
        safe_calls::finish(original::collect(a,i,k,c)?,calls)
    }
    pub fn collect_mode(a: Arc<Ast>, i: Arc<Interner>, k: Arc<Known>, c: CrateIds, walk: bool) -> Result<Program,Vec<Diag>> {
        let (a,calls) = lower(a,&i)?;
        safe_calls::finish(original::collect_mode(a,i,k,c,walk)?,calls)
    }
    pub fn collect_with_order(a: Arc<Ast>, i: Arc<Interner>, k: Arc<Known>, c: CrateIds, walk: bool, order: &mut DefOrder) -> Result<Program,Vec<Diag>> {
        let (a,calls) = lower(a,&i)?;
        safe_calls::finish(original::collect_with_order(a,i,k,c,walk,order)?,calls)
    }
    pub fn collect_with_order_resolved(a: Arc<Ast>, i: Arc<Interner>, k: Arc<Known>, c: CrateIds, walk: bool, order: &mut DefOrder, r: Option<Arc<crate::resolve::imports::ResolvedTables>>) -> Result<Program,Vec<Diag>> {
        let (a,calls) = lower(a,&i)?;
        safe_calls::finish(original::collect_with_order_resolved(a,i,k,c,walk,order,r)?,calls)
    }
    pub fn collect_reuse(a: Arc<Ast>, i: Arc<Interner>, k: Arc<Known>, c: CrateIds, walk: bool, order: &mut DefOrder, r: Option<Arc<crate::resolve::imports::ResolvedTables>>, reuse: &mut crate::driver::gen_reuse::GenReuse, routes: &[&'static crate::ty::lang::StdFnDef]) -> Result<Program,Vec<Diag>> {
        let evidence = provenance::input(&a);
        let extension = a.exprs.iter().any(|e| matches!(e,Expr::UnsafeBlock(_)));
        let (lowered,calls) = lower(a,&i)?;
        if extension { reuse.clear(); }
        let result = original::collect_reuse(lowered,i,k,c,walk,order,r,reuse,routes);
        if extension { reuse.clear(); }
        let program = safe_calls::finish(result?,calls)?;
        provenance::collected(evidence,&program);
        Ok(program)
    }
    pub(crate) fn collect_with_definitions(a: Arc<Ast>, i: Arc<Interner>, k: Arc<Known>, c: CrateIds, walk: bool, order: &mut DefOrder, r: Option<Arc<crate::resolve::imports::ResolvedTables>>, definitions: &mut dyn FnMut(&mut Program)->Result<(),Vec<Diag>>) -> Result<Program,Vec<Diag>> {
        let evidence=provenance::input(&a);let (a,calls)=lower(a,&i)?;
        let program=safe_calls::finish(original::collect_with_definitions(a,i,k,c,walk,order,r,definitions)?,calls)?;
        provenance::collected(evidence,&program);Ok(program)
    }
}
