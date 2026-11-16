//! Insert elaborated nodes while preserving contiguous function arenas.
//! This walks the public AST shapes; semantic checking remains in the native phases.
use crate::ast::*;

struct Positions<'a> {
    exprs: &'a [u32],
    // One entry per inserted pattern, ordered by anchor then owning macro.
    pats: &'a [(u32, u32)],
}

impl Positions<'_> {
    fn boundary(&self, old: u32) -> u32 {
        old.checked_add(self.exprs.partition_point(|&at| at < old).try_into().unwrap()).unwrap()
    }
    fn pat(&self, id: &mut PatId) {
        id.0 = id.0.checked_add(self.pats.partition_point(|&(at, _)| at <= id.0).try_into().unwrap()).unwrap();
    }
    fn pat_boundary(&self, old: u32, expr_boundary: u32) -> u32 {
        // Empty adjacent/nested arenas can share the same pattern boundary.
        // The original expression boundary establishes which inserts belong
        // before it, including a function with no pre-existing patterns.
        old.checked_add(self.pats.partition_point(|&(at, owner)| at < old || (at == old && owner < expr_boundary)).try_into().unwrap()).unwrap()
    }
    fn id(&self, id: &mut ExprId) {
        id.0 = self.boundary(id.0);
    }
    fn opt(&self, id: &mut Option<ExprId>) {
        if let Some(id) = id {
            self.id(id);
        }
    }
    fn ids(&self, ids: &mut [ExprId]) {
        ids.iter_mut().for_each(|id| self.id(id));
    }
    fn path(&self, path: &mut [PathSeg]) {
        for s in path {
            if let GenericArgs::Angle(args) = &mut s.args {
                self.args(args);
            }
        }
    }
    fn args(&self, args: &mut [GenericArg]) {
        for arg in args {
            match arg {
                GenericArg::Const(e) => self.id(e),
                GenericArg::Binding { kind: BindingKind::EqConst(e), .. } => self.id(e),
                GenericArg::Binding { kind: BindingKind::Constraint(bounds), .. } => self.bounds(bounds),
                GenericArg::Type(_) | GenericArg::Lifetime(_) | GenericArg::Binding { kind: BindingKind::EqTy(_), .. } => (),
            }
        }
    }
    fn bounds(&self, bounds: &mut [TraitBound]) {
        for b in bounds {
            if let TraitBound::Trait { trait_ref, .. } = b {
                self.path(&mut trait_ref.path);
            }
        }
    }
    fn attrs(&self, attrs: &mut [Attr]) {
        for a in attrs {
            self.path(&mut a.path);
            if let AttrInput::Eq(e) = &mut a.input {
                self.id(e);
            }
        }
    }
    fn vis(&self, vis: &mut Vis) {
        if let Vis::Restricted { path, .. } = vis {
            self.path(path);
        }
    }
    fn generics(&self, g: &mut Generics) {
        for p in &mut g.params {
            match p {
                GenericParam::Lifetime { attrs, .. } => self.attrs(attrs),
                GenericParam::Type { attrs, bounds, .. } => {
                    self.attrs(attrs);
                    self.bounds(bounds);
                }
                GenericParam::Const { attrs, default, .. } => {
                    self.attrs(attrs);
                    self.opt(default);
                }
            }
        }
    }
    fn predicates(&self, w: &mut WhereClause) {
        for p in &mut w.preds {
            if let WherePred::Type { bounds, .. } = p {
                self.bounds(bounds);
            }
        }
    }
    fn function(&self, f: &mut FnDef) {
        self.attrs(&mut f.attrs);
        self.vis(&mut f.vis);
        self.generics(&mut f.generics);
        self.predicates(&mut f.where_clause);
        for p in &mut f.params {
            self.attrs(&mut p.attrs);
            if let Some(pat) = &mut p.pat { self.pat(pat); }
        }
        if let Some(s) = &mut f.self_param {
            self.attrs(&mut s.attrs);
        }
        f.pats = (self.pat_boundary(f.pats.0, f.exprs.0), self.pat_boundary(f.pats.1, f.exprs.1));
        f.exprs = (self.boundary(f.exprs.0), self.boundary(f.exprs.1));
    }
    fn fields(&self, fs: &mut VariantFields) {
        if let VariantFields::Struct(fs) | VariantFields::Tuple(fs) = fs {
            for f in fs {
                self.attrs(&mut f.attrs);
                self.vis(&mut f.vis);
            }
        }
    }
    fn associated_const(&self, c: &mut AssocConstDef) {
        self.vis(&mut c.vis);
        self.attrs(&mut c.attrs);
        self.opt(&mut c.default);
        if let Some(f) = &mut c.init_fn {
            self.function(f);
        }
    }
    fn associated_type(&self, t: &mut AssocTypeDef) {
        self.vis(&mut t.vis);
        self.attrs(&mut t.attrs);
        self.generics(&mut t.generics);
        self.bounds(&mut t.bounds);
        self.predicates(&mut t.where_clause);
    }
    fn static_item(&self, s: &mut StaticDef) {
        self.opt(&mut s.value);
        if let Some(f) = &mut s.init_fn {
            self.function(f);
        }
    }
    fn uses(&self, u: &mut UseTree) {
        self.path(&mut u.prefix);
        if let UseKind::Group(children) = &mut u.kind {
            for c in children {
                self.uses(c);
            }
        }
    }
    fn item(&self, i: &mut Item) {
        self.attrs(&mut i.attrs);
        self.vis(&mut i.vis);
        match &mut i.kind {
            ItemKind::Fn(f) => self.function(f),
            ItemKind::Struct(s) => {
                self.generics(&mut s.generics);
                self.predicates(&mut s.where_clause);
                self.fields(&mut s.fields);
            }
            ItemKind::Enum(e) => {
                self.generics(&mut e.generics);
                self.predicates(&mut e.where_clause);
                for v in &mut e.variants {
                    self.attrs(&mut v.attrs);
                    self.fields(&mut v.fields);
                    self.opt(&mut v.discriminant);
                }
            }
            ItemKind::Impl(i) => {
                self.attrs(&mut i.inner_attrs);
                self.generics(&mut i.generics);
                self.predicates(&mut i.where_clause);
                if let Some(t) = &mut i.trait_ref {
                    self.path(&mut t.path);
                }
                for f in &mut i.fns {
                    self.function(f);
                }
                for c in &mut i.consts {
                    self.associated_const(c);
                }
                for t in &mut i.types {
                    self.associated_type(t);
                }
            }
            ItemKind::Trait(t) => {
                self.attrs(&mut t.inner_attrs);
                self.generics(&mut t.generics);
                self.predicates(&mut t.where_clause);
                self.bounds(&mut t.bounds);
                for f in &mut t.fns {
                    self.function(f);
                }
                for c in &mut t.consts {
                    self.associated_const(c);
                }
                for t in &mut t.types {
                    self.associated_type(t);
                }
            }
            ItemKind::Type(t) => {
                self.generics(&mut t.generics);
                self.predicates(&mut t.where_clause);
            }
            ItemKind::Const(c) => {
                self.opt(&mut c.value);
                if let Some(f) = &mut c.init_fn {
                    self.function(f);
                }
            }
            ItemKind::Static(s) => self.static_item(s),
            ItemKind::Union(u) => {
                self.generics(&mut u.generics);
                self.predicates(&mut u.where_clause);
                for f in &mut u.fields {
                    self.attrs(&mut f.attrs);
                    self.vis(&mut f.vis);
                }
            }
            ItemKind::Mod(m) => {
                self.attrs(&mut m.inner_attrs);
                if let Some(items) = &mut m.body {
                    for i in items {
                        self.item(i);
                    }
                }
            }
            ItemKind::Use(u) => self.uses(&mut u.tree),
            ItemKind::MacroCall(m) => self.path(&mut m.path),
            ItemKind::ExternBlock(b) => {
                for i in &mut b.items {
                    match i {
                        ForeignItem::Fn(f) => self.function(f),
                        ForeignItem::Static(s) => self.static_item(s),
                        ForeignItem::Type { vis, attrs, generics, where_clause, .. } => {
                            self.vis(vis);
                            self.attrs(attrs);
                            self.generics(generics);
                            self.predicates(where_clause);
                        }
                        ForeignItem::MacroCall(m) => self.path(&mut m.path),
                    }
                }
            }
            ItemKind::MacroRules(_) | ItemKind::ExternCrate(_) => (),
        }
    }
    fn expression(&self, e: &mut Expr) {
        match e {
            Expr::Path(p) => self.path(p),
            Expr::QPath { trait_path, segs, .. } => {
                self.path(trait_path);
                self.path(segs);
            }
            Expr::If { cond, els, .. } => {
                self.id(cond);
                self.opt(els);
            }
            Expr::While { cond, .. } => self.id(cond),
            Expr::For { iter, pat, .. } => { self.id(iter); self.pat(pat); }
            Expr::Match { scrut, arms } => {
                self.id(scrut);
                for a in arms {
                    self.attrs(&mut a.attrs);
                    self.pat(&mut a.pat);
                    self.opt(&mut a.guard);
                    self.id(&mut a.body);
                }
            }
            Expr::Call { callee, args } => {
                self.id(callee);
                self.ids(args);
            }
            Expr::MethodCall { recv, generic_args, args, .. } => {
                self.id(recv);
                self.args(generic_args);
                self.ids(args);
            }
            Expr::Field { base, .. } | Expr::TupleField { base, .. } => self.id(base),
            Expr::Index { base, index } => {
                self.id(base);
                self.id(index);
            }
            Expr::Binary { lhs, rhs, .. } | Expr::Assign { lhs, rhs } | Expr::AssignOp { lhs, rhs, .. } => {
                self.id(lhs);
                self.id(rhs);
            }
            Expr::Unary { expr, .. }
            | Expr::Cast { expr, .. }
            | Expr::Ref { expr, .. }
            | Expr::Deref(expr)
            | Expr::Try(expr)
            | Expr::Await(expr) => self.id(expr),
            Expr::StructLit { path, fields, update } => {
                self.path(path);
                for (_, _, e) in fields {
                    self.id(e);
                }
                self.opt(update);
            }
            Expr::Range { lo, hi, .. } => {
                self.opt(lo);
                self.opt(hi);
            }
            Expr::Break { value, .. } | Expr::Return(value) => self.opt(value),
            Expr::Macro { args, .. } | Expr::Tuple(args) | Expr::Array(args) => self.ids(args),
            Expr::ArrayRepeat { value, len } => {
                self.id(value);
                self.id(len);
            }
            Expr::Closure { body, params, .. } => {
                self.id(body);
                for p in params {
                    self.attrs(&mut p.attrs);
                    self.pat(&mut p.pat);
                }
            }
            Expr::Let { scrut, pat } => { self.id(scrut); self.pat(pat); }
            Expr::MacroCall(m) => self.path(&mut m.path),
            Expr::Int(..)
            | Expr::Bool(_)
            | Expr::Str(_)
            | Expr::Char(_)
            | Expr::Unit
            | Expr::SelfValue
            | Expr::Block(_)
            | Expr::Loop { .. }
            | Expr::Continue(_)
            | Expr::Float(_)
            | Expr::AsyncBlock { .. }
            | Expr::UnsafeBlock(_)
            | Expr::LabeledBlock { .. }
            | Expr::ConstBlock(_)
            | Expr::Underscore
            | Expr::OffsetOf { .. }
            | Expr::ByteStr(_) => (),
        }
    }
    fn all(&self, a: &mut Ast) {
        self.attrs(&mut a.inner_attrs);
        for i in a.items.iter_mut().chain(&mut a.stmt_items) {
            self.item(i);
        }
        for e in &mut a.exprs {
            self.expression(e);
        }
        for p in &mut a.pats {
            match p {
                Pat::Lit(e) => self.id(e),
                Pat::Path(p) => self.path(p),
                Pat::Struct { path, fields, .. } => { self.path(path); for (_, p) in fields { self.pat(p); } }
                Pat::TupleStruct { path, elems } => { self.path(path); for p in elems { self.pat(p); } }
                Pat::QPath { trait_path, segs, .. } => {
                    self.path(trait_path);
                    self.path(segs);
                }
                Pat::Range { lo, hi, .. } => {
                    self.opt(lo);
                    self.opt(hi);
                }
                Pat::Or(ps) | Pat::Tuple(ps) => { for p in ps { self.pat(p); } }
                Pat::Slice { prefix, middle, suffix } => {
                    for p in prefix.iter_mut().chain(suffix) { self.pat(p); }
                    if let Some(p) = middle { self.pat(p); }
                }
                Pat::Ref { inner, .. } | Pat::At { inner, .. } | Pat::Paren(inner) => self.pat(inner),
                Pat::Wild | Pat::Binding { .. } | Pat::Rest => (),
                Pat::MacroCall(m) => self.path(&mut m.path),
            }
        }
        for t in &mut a.tys {
            match t {
                TyExpr::Path(p) => self.path(p),
                TyExpr::Array { len, .. } => self.id(len),
                TyExpr::Dyn { bounds, .. } | TyExpr::ImplTrait { bounds, .. } => self.bounds(bounds),
                TyExpr::QPath { trait_path, segs, .. } => {
                    self.path(trait_path);
                    self.path(segs);
                }
                TyExpr::FnPtr { inputs, .. } => {
                    for i in inputs {
                        self.attrs(&mut i.attrs);
                    }
                }
                TyExpr::Ref { .. }
                | TyExpr::Unit
                | TyExpr::Tuple(_)
                | TyExpr::Slice(_)
                | TyExpr::Ptr { .. }
                | TyExpr::Never
                | TyExpr::Paren(_)
                | TyExpr::Infer => (),
                TyExpr::MacroCall(m) => self.path(&mut m.path),
            }
        }
        for b in &mut a.blocks {
            self.attrs(&mut b.inner_attrs);
            self.attrs(&mut b.tail_attrs);
            self.opt(&mut b.tail);
            for s in &mut b.stmts {
                match s {
                    Stmt::Let { init, attrs, pat, .. } => {
                        self.opt(init);
                        self.attrs(attrs);
                        self.pat(pat);
                    }
                    Stmt::Expr { expr, attrs, .. } => {
                        self.id(expr);
                        self.attrs(attrs);
                    }
                    Stmt::Macro { call, .. } => self.path(&mut call.path),
                    Stmt::Item(_) => (),
                }
            }
        }
        for e in &mut a.paren_exprs {
            *e = self.boundary(*e);
        }
        for c in &mut a.anon_consts {
            self.id(&mut c.value);
            self.function(&mut c.def);
        }
    }
}

pub(super) enum Addition {
    Call(Expr),
    Cast(TyId),
    Assertion { array_left: bool, pats: u32, target: TyId, messages: usize, left: crate::intern::Sym, right: crate::intern::Sym },
}

/// Keep the original id as the new expression's root, inserting its callee or
/// original cast operand immediately after it in the same function arena.
pub(super) fn elaborate(a: &mut Ast, additions: Vec<(ExprId, Addition)>) {
    assert!(additions.windows(2).all(|w| w[0].0.0 < w[1].0.0));
    let indexes: Vec<_> = additions.iter().flat_map(|(id, add)| std::iter::repeat_n(id.0, match add {
        Addition::Assertion { messages, .. } => 9usize.checked_add(messages.checked_mul(6).unwrap()).unwrap(),
        _ => 1,
    })).collect();
    let mut patterns: Vec<_> = additions.iter().filter_map(|(expr, add)| {
        if let Addition::Assertion { pats, left, right, .. } = add { Some((*pats, expr.0, *left, *right)) } else { None }
    }).collect();
    patterns.sort_by_key(|&(at, expr, ..)| (at, expr));
    let pat_indexes: Vec<_> = patterns.iter().flat_map(|&(at, expr, ..)| [(at, expr); 3]).collect();
    let map = Positions { exprs: &indexes, pats: &pat_indexes };
    map.all(a);
    let old_pats = std::mem::take(&mut a.pats);
    let old_spans = std::mem::take(&mut a.pat_spans);
    let mut insertions = patterns.iter().peekable();
    for at in 0..=old_pats.len() {
        while insertions.peek().is_some_and(|&&(anchor, ..)| anchor as usize == at) {
            let &(_, expr, left, right) = insertions.next().unwrap();
            let base = u32::try_from(a.pats.len()).unwrap();
            a.pats.extend([Pat::Binding { name: left, mutable: false, by_ref: false }, Pat::Binding { name: right, mutable: false, by_ref: false }, Pat::Tuple(vec![PatId(base), PatId(base + 1)])]);
            a.pat_spans.extend([a.expr_spans[expr as usize]; 3]);
        }
        if at < old_pats.len() { a.pats.push(old_pats[at].clone()); a.pat_spans.push(old_spans[at]); }
    }
    assert!(insertions.next().is_none());
    let expressions = std::mem::take(&mut a.exprs);
    let spans = std::mem::take(&mut a.expr_spans);
    let mut additions = additions.into_iter().peekable();
    for (old, (expression, span)) in expressions.into_iter().zip(spans).enumerate() {
        if additions.peek().is_some_and(|(id, _)| id.0 as usize == old) {
            let (_, addition) = additions.next().unwrap();
            let child = ExprId(u32::try_from(a.exprs.len()).unwrap().checked_add(1).unwrap());
            match addition {
                Addition::Call(callee) => {
                    let Expr::MethodCall { recv, .. } = expression else {
                        panic!("elaboration target is not a method");
                    };
                    a.exprs.push(Expr::Call { callee: child, args: vec![recv] });
                    a.exprs.push(callee);
                }
                Addition::Cast(ty) => {
                    a.exprs.push(Expr::Cast { expr: child, ty });
                    a.exprs.push(expression);
                }
                Addition::Assertion { array_left, pats, target, messages, left, right } => {
                    let Expr::Macro { kind, mut args, fmt, repeat } = expression else { panic!("assertion target is not a macro") };
                    let base = child.0 - 1;
                    let id = |n| ExprId(base + n);
                    let pat = pats + u32::try_from(pat_indexes.partition_point(|&(at, owner)| (at, owner) < (pats, old as u32))).unwrap() + 2;
                    let (original_left, original_right) = (args[0], args[1]);
                    args[0] = if array_left { id(8) } else { id(5) };
                    args[1] = if array_left { id(7) } else { id(8) };
                    assert_eq!(args.len() - 2, messages);
                    let mut message_nodes = Vec::new();
                    for arg in &mut args[2..] {
                        let start = 10 + u32::try_from(message_nodes.len()).unwrap();
                        // Formatting borrows the original argument place. Keep
                        // that place (including non-Copy messages) through an
                        // explicit &/deref, then use both original operand
                        // borrows AFTER its evaluation. Without these uses,
                        // copying the array ref into the slice cast can end
                        // the original place loan before a lazy message runs.
                        message_nodes.extend([
                            Expr::Deref(id(start + 1)),
                            Expr::TupleField { base: id(start + 2), index: 0 },
                            Expr::Tuple(vec![id(start + 3), id(start + 4), id(start + 5)]),
                            Expr::Ref { mutable: false, raw: false, expr: *arg },
                            Expr::Path(vec![PathSeg::plain(left)]),
                            Expr::Path(vec![PathSeg::plain(right)]),
                        ]);
                        *arg = id(start);
                    }
                    a.exprs.extend([
                        Expr::Match { scrut: id(1), arms: vec![Arm { attrs: vec![], pat: PatId(pat), guard: None, body: id(9) }] },
                        Expr::Tuple(vec![id(2), id(3)]),
                        Expr::Ref { mutable: false, raw: false, expr: original_left },
                        Expr::Ref { mutable: false, raw: false, expr: original_right },
                        Expr::Path(vec![PathSeg::plain(left)]),
                        Expr::Deref(id(4)),
                        Expr::Path(vec![PathSeg::plain(right)]),
                        Expr::Deref(id(6)),
                        Expr::Cast { expr: if array_left { id(5) } else { id(7) }, ty: target },
                        Expr::Macro { kind, args, fmt, repeat },
                    ]);
                    a.expr_spans.extend([span; 10]);
                    a.expr_spans.extend(std::iter::repeat_n(span, message_nodes.len()));
                    a.exprs.extend(message_nodes);
                    continue;
                }
            }
            a.expr_spans.extend([span, span]);
        } else {
            a.exprs.push(expression);
            a.expr_spans.push(span);
        }
    }
    assert!(additions.next().is_none());
}
