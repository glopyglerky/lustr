//! Object generation at the mixed compiler's first, scalar C boundary.
//! Checking, borrow checking, lowering, generation and retained body state are
//! the ordinary Lust pipeline. Rust's compiler supplies the caller and metadata.
use crate::{ast::{BinOp, UnOp}, cgen, check::{Builtin, collect::Program}, ir, protocol::{Request, Response}, ty::{IntTy, Ty, TyIntern, TyKind}};
use std::{fmt::Write as _, fs, path::Path, sync::Arc};

fn operand(o: &ir::Operand) -> bool {
    match o {
        ir::Operand::Copy(p) | ir::Operand::Move(p) => p.proj.is_empty(),
        ir::Operand::Const(c) => matches!(c, ir::Const::Int(..) | ir::Const::Bool(_) | ir::Const::Unit),
    }
}

/// Establish a boundary without pretending that the two panic runtimes or
/// aggregate ABIs agree. These operations are total on their integer domains.
pub(super) fn total(b: &ir::Body<'_>) -> bool {
    let mut live = vec![false; b.blocks.len()];
    let mut pending = vec![0usize];
    while let Some(n) = pending.pop() {
        let Some(block) = b.blocks.get(n) else { return false };
        if std::mem::replace(&mut live[n], true) { continue; }
        let mut next = |bb: ir::Bb| pending.push(bb.0 as usize);
        match &block.term {
            ir::Terminator::Goto(bb) => next(*bb),
            ir::Terminator::If(_, a, b) => { next(*a); next(*b); },
            ir::Terminator::Switch(_, arms, other) => { for (_, bb) in arms { next(*bb); } next(*other); },
            // Only a proven total builtin is accepted below. Its unwind edge
            // is therefore unreachable, just like lowering's detached cleanup
            // blocks after a return. Do not reject the entire lowered body
            // merely because those dead blocks contain Unreachable.
            ir::Terminator::Call { target: ir::CallTarget::Builtin(_), next: bb, .. } => next(*bb),
            ir::Terminator::Return => {},
            _ => return false,
        }
    }
    let accepted = b.locals.iter().all(|l| matches!(b.tys.kind(l.ty), TyKind::Int(_) | TyKind::Bool | TyKind::Unit))
        && b.blocks.iter().enumerate().filter(|(n, _)| live[*n]).all(|(_, block)| {
            block.stmts.iter().all(|s| match &s.kind {
                ir::StmtKind::Assign(p, r) if p.proj.is_empty() => match r {
                    ir::Rvalue::Use(o) | ir::Rvalue::Cast(o, _) | ir::Rvalue::Un(UnOp::Not, o) => operand(o),
                    ir::Rvalue::Bin(op, a, b) => matches!(op,
                        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Eq | BinOp::Ne |
                        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::And | BinOp::Or)
                        && operand(a) && operand(b),
                    _ => false,
                },
                ir::StmtKind::Drop(_) | ir::StmtKind::StorageDead(_) | ir::StmtKind::FakeRead(_) => true,
                _ => false,
            }) && match &block.term {
                ir::Terminator::Goto(_) | ir::Terminator::Return => true,
                ir::Terminator::If(o, ..) | ir::Terminator::Switch(o, ..) => operand(o),
                ir::Terminator::Call { target: ir::CallTarget::Builtin(op), args, dest, .. } =>
                    matches!(op, Builtin::WrappingAdd | Builtin::WrappingSub | Builtin::WrappingMul |
                        Builtin::WrappingNeg | Builtin::WrappingShl | Builtin::WrappingShr |
                        Builtin::RotateLeft | Builtin::RotateRight)
                    && args.iter().all(operand) && dest.proj.is_empty(),
                _ => false,
            }
        });
    if !accepted && std::env::var_os("LUST_MIXED_TRACE").is_some() {
        eprintln!("mixed body {:?}: locals {:?}; blocks {:?}", b.fn_id,
            b.locals.iter().map(|l| format!("{:?}", b.tys.kind(l.ty))).collect::<Vec<_>>(), b.blocks);
    }
    accepted
}

fn scalar(p: &Program, t: Ty) -> Option<&'static str> {
    Some(match p.tys.kind(t) {
        TyKind::Int(IntTy::I8) => "int8_t", TyKind::Int(IntTy::I16) => "int16_t",
        TyKind::Int(IntTy::I32) => "int32_t", TyKind::Int(IntTy::I64 | IntTy::Isize) => "int64_t",
        TyKind::Int(IntTy::U8) => "uint8_t", TyKind::Int(IntTy::U16) => "uint16_t",
        TyKind::Int(IntTy::U32) => "uint32_t", TyKind::Int(IntTy::U64 | IntTy::Usize) => "uint64_t",
        _ => return None,
    })
}

pub(super) fn emit(req: &Request, p: &Program, out: &[cgen::FnOut], _glue: &cgen::GlueSet,
    slots: &[u32], names: &Arc<cgen::names::Names>, generated: usize) -> Response {
    match emit_inner(req, p, out, slots, names) {
        Ok(symbol) => Response { code: 0, stdout: symbol, stderr: format!("lustr mixed native: {generated} function(s) generated in retained Lust session\n"), ..Default::default() },
        Err(e) => Response { code: 1, stderr: format!("lustr mixed boundary unavailable: {e}\n"), ..Default::default() },
    }
}

fn emit_inner(req: &Request, p: &Program, out: &[cgen::FnOut],
    slots: &[u32], names: &Arc<cgen::names::Names>) -> Result<String, String> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) || req.debug {
        return Err("the scalar object boundary needs its qualified Linux target and debug format".into());
    }
    let symbol = req.session.krate.package_id.strip_prefix("lustr-mixed-").ok_or("missing mixed unit identity")?;
    if symbol.is_empty() || symbol.len() > 128 || !symbol.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid mixed unit identity".into());
    }
    let symbol = format!("lustr_mixed_{symbol}");
    let mut selected = p.fns.iter().enumerate().filter(|(_, f)| p.interner.get(f.name) == req.session.krate.target_name);
    let (id, f) = selected.next().ok_or("native function was not collected")?;
    if selected.next().is_some() { return Err("ambiguous native function identity".into()); }
    let slot = *slots.get(id).ok_or("missing native function slot")? as usize;
    if !out.get(slot).is_some_and(|o| o.mixed_total) { return Err("function needs an unrepresented runtime operation".into()); }
    let ret = scalar(p, f.ret).ok_or("return needs a wider ABI mapping")?;
    let params: Vec<_> = f.params.iter().map(|t| scalar(p, *t).ok_or("argument needs a wider ABI mapping")).collect::<Result<_, _>>()?;
    let tag = names.fns.get(slot).and_then(|s| s.as_ref()).ok_or("missing stable native symbol")?;
    if !cgen::native_direct_abi(p, &f.params, f.ret) { return Err("native function does not have a direct C ABI".into()); }
    let mut c = cgen::mixed_scalar_unit(&out[slot]).ok_or("function needs nonscalar generation helpers")?;
    write!(c, "\n{ret} {symbol}(").unwrap();
    for (n, t) in params.iter().enumerate() { if n != 0 { c.push(','); } write!(c, "{t} a{n}").unwrap(); }
    if params.is_empty() { c.push_str("void"); }
    write!(c, "){{return f_{tag}(").unwrap();
    for n in 0..params.len() { if n != 0 { c.push(','); } write!(c, "a{n}").unwrap(); }
    c.push_str(");}\n");
    let parent = req.out.parent().ok_or("mixed output needs a parent")?;
    if req.out.exists() { return Err("mixed output already exists; use a fresh generation".into()); }
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let object = req.out.with_extension("unit.o");
    if object.exists() { return Err("mixed object already exists".into()); }
    fs::write(req.out.with_extension("c"), &c).map_err(|e| e.to_string())?;
    crate::tcc::compile_units(&[(object.clone(), c)], false)?;
    // Inspect the real relocatable output before linking it into a Rust process.
    let symbols = std::process::Command::new("nm").args(["--undefined-only", "--format=posix"]).arg(&object)
        .output().map_err(|e| e.to_string())?;
    if !symbols.status.success() || !symbols.stdout.is_empty() {
        return Err(format!("native object requires unrepresented symbols: {}", String::from_utf8_lossy(&symbols.stdout)));
    }
    fs::rename(&object, &req.out).map_err(|e| e.to_string())?;
    fs::File::open(&req.out).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
    fs::File::open(Path::new(parent)).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
    Ok(symbol)
}
