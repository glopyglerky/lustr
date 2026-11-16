//! Checked shared array-to-slice unsizing inside the original assertion macro.
//! Both standard sequence forms use the same element equality and Debug.
//! Rvalues keep the original macro. Places first borrow both original operands
//! in an owned match; original lazy messages retain those loans, and the native
//! assertion macro still performs equality, formatting and panic.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum Site {
    Rvalue(ast::ExprId),
    Place { expr: ast::ExprId, array_left: bool, pats: u32 },
}

impl Site {
    pub(super) fn expr(self) -> ast::ExprId {
        match self { Self::Rvalue(e) | Self::Place { expr: e, .. } => e }
    }
}

pub(super) fn site(p: &check::collect::Program, ck: &check::Checked<'_>, e: ast::ExprId, f: &ast::FnDef) -> Option<Site> {
    let ast::Expr::Macro { kind: ast::MacroKind::AssertEq | ast::MacroKind::AssertNe, args, .. } = p.ast.expr(e) else {
        return None;
    };
    let [left, right, ..] = args.as_slice() else { return None };
    let ty::TyKind::Ref(_, ty::Mut::Shared, a) = ck.tys.kind(ck.ty(*left)) else { return None };
    let ty::TyKind::Ref(_, ty::Mut::Shared, b) = ck.tys.kind(ck.ty(*right)) else { return None };
    let (array, element, other) = match (ck.tys.kind(a), ck.tys.kind(b)) {
        (ty::TyKind::Slice(a), ty::TyKind::Array(b, _)) => (*right, b, a),
        (ty::TyKind::Array(a, _), ty::TyKind::Slice(b)) => (*left, a, b),
        _ => return None,
    };
    let flags = ck.tys.flags(element);
    if element != other || element == ty::ERROR || flags.contains_infer() || flags.contains_param() || flags.contains_projection()
        || !check::collect::has_eq(p, &ck.tys, element) || !check::collect::has_debug(p, &ck.tys, element)
    {
        return None;
    }
    // The original macro borrows both operand places before comparing them.
    // A place needs a whole-macro match; an rvalue keeps its existing fast cast.
    Some(if matches!(p.ast.expr(array), ast::Expr::Path(_) | ast::Expr::SelfValue | ast::Expr::Deref(_)
        | ast::Expr::Field { .. } | ast::Expr::TupleField { .. } | ast::Expr::Index { .. })
    {
        Site::Place { expr: e, array_left: array == *left, pats: f.pats.0 }
    } else { Site::Rvalue(array) })
}

fn fresh(symbols: &crate::intern::Interner, expr: ast::ExprId, side: &str) -> Sym {
    // Check the complete interner, including resolved/macro-generated names.
    // Every binding is absent before insertion, so lazy user messages cannot
    // resolve to it. The '$' namespace alone is not the hygiene proof.
    for suffix in 0u64.. {
        let name = format!("$lustr$assert${}${side}${suffix}", expr.0);
        if symbols.find(&name).is_none() { return symbols.intern(&name); }
    }
    unreachable!()
}

pub(super) fn elaborate(ast: &mut ast::Ast, symbols: &crate::intern::Interner, sites: &[Site], edits: &mut Vec<(ast::ExprId, arena::Addition)>) {
    for &site in sites {
        let expr = site.expr();
        let span = ast.expr_spans[expr.0 as usize];
        // Infer the element again from the unchanged operand. This avoids
        // reifying a checked local type with a spelling that could resolve to
        // another definition. Only the already-proved unsizing is introduced.
        let element = type_node(ast, ast::TyExpr::Infer, span);
        let slice = type_node(ast, ast::TyExpr::Slice(element), span);
        let target = type_node(ast, ast::TyExpr::Ref { lifetime: None, mutable: false, inner: slice }, span);
        let addition = match site {
            Site::Rvalue(_) => arena::Addition::Cast(target),
            Site::Place { array_left, pats, .. } => {
                let ast::Expr::Macro { args, .. } = ast.expr(expr) else { unreachable!() };
                arena::Addition::Assertion {
                    array_left, pats, target, messages: args.len() - 2,
                    left: fresh(symbols, expr, "left"), right: fresh(symbols, expr, "right"),
                }
            }
        };
        edits.push((expr, addition));
    }
}
