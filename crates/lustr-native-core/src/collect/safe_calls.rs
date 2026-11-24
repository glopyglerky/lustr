//! Discharge call obligations on the exact native program and body scopes.
//! The tentative AST clone cannot leave collection until every call qualifies.
use super::Program;
use crate::{ast::{Expr, ExprId}, def::LocalDefId, diag::Diag, ty::BOOL};
use crate::pinned::check::{self, generics::FnClass, Res};
use std::collections::BTreeSet;

fn refusal(p: &Program, id: u32) -> Vec<Diag> {
    vec![Diag::new("L0404", p.ast.expr_spans[id as usize],
        "native unsafe-body call has no qualified safe Rust function target",
        "A checked direct, non-generic safe Rust function with boolean inputs and output is required.")]
}

pub(super) fn finish(p: Program, mut calls: BTreeSet<u32>) -> Result<Program, Vec<Diag>> {
    if calls.is_empty() { return Ok(p); }
    let total = calls.len();
    for (index, info) in p.fns.iter().enumerate() {
        if calls.is_empty() { break; }
        let owner = LocalDefId(index.try_into().unwrap());
        let def = info.def(&p.ast);
        if p.gx.fn_class.get(index) != Some(&FnClass::Plain)
            || p.check_skips(owner) || def.body.is_none()
        { continue; }
        let sites: Vec<_> = calls.range(def.exprs.0..def.exprs.1).copied().collect();
        if sites.is_empty() { continue; }
        // The native checker establishes the actual lexical binding, hygiene,
        // callable resolution and types. No spellings or span containment are
        // used as a substitute. This is a call-safety query, not the production
        // whole-function acceptance boundary. The driver's subsequent native
        // Check retains conversion errors and the failed Program for bounded
        // elaboration (compile.rs check+borrowck/patch::keep_err). Recollection
        // repeats every call obligation after each elaboration. Other query
        // errors still refuse here; no unresolved call can leave this facade.
        let (checked, errors) = check::check_fn(&p, owner);
        if errors.iter().any(|d| !matches!(d.code, "L0610" | "L0640" | "L0110")) { return Err(errors); }
        if !errors.is_empty() {
            eprintln!("native frontend: {} constraint diagnostics remain for the complete native Check and retained bounded inference", errors.len());
        }
        for id in sites {
            let Some(Expr::Call { callee, args }) = p.ast.exprs.get(id as usize) else { return Err(refusal(&p,id)); };
            let slot = |e: ExprId| e.0.checked_sub(checked.expr_base).map(|n| n as usize);
            let Some(i) = slot(ExprId(id)) else { return Err(refusal(&p,id)); };
            let Some(Res::Fn(target)) = checked.expr_res.get(i) else { return Err(refusal(&p,id)); };
            let Some(info) = p.fns.get(target.0 as usize) else { return Err(refusal(&p,id)); };
            let def = info.def(&p.ast);
            // Reject fn pointers, closures, methods, foreign/unsafe/async fns,
            // generic instances and skipped bodies. Their semantics are not
            // inferred from a safe-looking name or the enclosing unsafe block.
            if p.gx.fn_class.get(target.0 as usize) != Some(&FnClass::Plain)
                || p.check_skips(*target) || info.owner.is_some() || info.self_kind.is_some()
                || def.body.is_none() || def.qual.is_unsafe || def.qual.is_async
                || def.qual.abi.is_some() || def.variadic || !def.generics.params.is_empty()
                || !def.where_clause.preds.is_empty() || info.ret != BOOL
                || info.params.len() != args.len() || info.params.iter().any(|t| *t != BOOL)
                || checked.expr_ty.get(i) != Some(&BOOL)
                || !slot(*callee).and_then(|j| checked.expr_res.get(j)).is_some_and(|r| *r == Res::None || *r == Res::Fn(*target))
                || args.iter().any(|arg| slot(*arg).and_then(|j| checked.expr_ty.get(j)) != Some(&BOOL))
            { return Err(refusal(&p,id)); }
            calls.remove(&id);
        }
    }
    if let Some(&id) = calls.first() { return Err(refusal(&p,id)); }
    eprintln!("native frontend: {total} direct safe Rust call obligations checked in original lexical scopes");
    Ok(p)
}
