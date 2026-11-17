//! Object generation at the mixed compiler's first, scalar C boundary.
//! Checking, borrow checking, lowering, generation and retained body state are
//! the ordinary Lust pipeline. Rust's compiler supplies the caller and metadata.
use crate::{ast::{BinOp, UnOp}, cgen, check::{Builtin, collect::Program}, ir, protocol::{Request, Response}, ty::{IntTy, Ty, TyIntern, TyKind}};
use std::{fmt::Write as _, fs, path::Path, sync::Arc};

#[path = "mixed_symbols.rs"]
mod symbols;

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
    // Read the requesting client's opt-in, not a possibly older daemon's
    // environment. This helper stays in the already-hashed owned source file.
    let mut profile = emitter_profile::Profile::new(req.client.env.as_deref());
    let result = emit_inner(req, p, out, slots, names, &mut profile);
    let detail = profile.finish(req.session.krate.package_id.strip_prefix("lustr-mixed-").unwrap_or(""), result.is_ok());
    let mut response = match result {
        Ok(symbol) => Response { code: 0, stdout: symbol, stderr: format!("lustr mixed native: {generated} function(s) generated in retained Lust session\n"), ..Default::default() },
        Err(e) => Response { code: 1, stderr: format!("lustr mixed boundary unavailable: {e}\n"), ..Default::default() },
    };
    if let Some(detail) = detail { response.stderr.push_str(&detail); }
    response
}

fn emit_inner(req: &Request, p: &Program, out: &[cgen::FnOut],
    slots: &[u32], names: &Arc<cgen::names::Names>, profile: &mut emitter_profile::Profile) -> Result<String, String> {
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
    profile.advance("output-prepare");
    let parent = req.out.parent().ok_or("mixed output needs a parent")?;
    if req.out.exists() { return Err("mixed output already exists; use a fresh generation".into()); }
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let object = req.out.with_extension("unit.o");
    if object.exists() { return Err("mixed object already exists".into()); }
    profile.advance("c-write");
    fs::write(req.out.with_extension("c"), &c).map_err(|e| e.to_string())?;
    profile.advance("libtcc");
    crate::tcc::compile_units(&[(object.clone(), c)], false)?;
    profile.advance("symbol-check");
    // Inspect the real relocatable output before linking it into a Rust process.
    symbols::check(&object, &symbol)?;
    profile.advance("rename");
    fs::rename(&object, &req.out).map_err(|e| e.to_string())?;
    profile.advance("file-sync");
    fs::File::open(&req.out).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
    profile.advance("directory-sync");
    fs::File::open(Path::new(parent)).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
    Ok(symbol)
}

/// This record travels in the existing response diagnostics. It contains
/// durations only; a daemon clock is not aligned to the client's timeline.
/// No profiling clock, allocation or file operation runs without the opt-in.
mod emitter_profile {
    use std::{fmt::Write as _, time::Instant};

    pub(super) struct Profile(Option<Active>);
    struct Active {
        start: Instant,
        last: Instant,
        phase: &'static str,
        phases: Vec<(&'static str, u128, bool)>,
    }

    impl Profile {
        pub(super) fn new(environment: Option<&[(String, String)]>) -> Self {
            Self(environment.is_some_and(|vars| vars.iter().any(|(key, _)| key == "LUSTR_MIXED_PROFILE_DIR")).then(|| {
                let start = Instant::now();
                Active { start, last: start, phase: "c-prepare", phases: Vec::new() }
            }))
        }

        pub(super) fn advance(&mut self, next: &'static str) {
            if let Some(active) = &mut self.0 {
                let now = Instant::now();
                active.phases.push((active.phase, now.duration_since(active.last).as_nanos(), true));
                active.last = now;
                active.phase = next;
            }
        }

        pub(super) fn finish(self, identity: &str, success: bool) -> Option<String> {
            let mut active = self.0?;
            // The daemon compares failed warm/fresh responses byte for byte.
            // Timing text would create a false mismatch and discard retained
            // state. Keep error diagnostics unchanged; the client records the
            // failure code and explicitly unavailable emitter detail instead.
            if !success { return None; }
            let end = Instant::now();
            active.phases.push((active.phase, end.duration_since(active.last).as_nanos(), success));
            // Caller-generated identities are hex. A malformed request must
            // not introduce arbitrary diagnostics into the structured record.
            let identity = if identity.len() <= 128 && identity.bytes().all(|b| b.is_ascii_hexdigit()) { identity } else { "" };
            let mut stderr = String::new();
            let _ = write!(stderr, "lustr mixed native profile: {{\"schema\":\"lustr.mixed-native-emitter\",\"version\":2,\"native\":\"{}\",\"unit_identity\":\"{identity}\",\"success\":{success},\"elapsed_ns\":{},\"phases\":[", env!("LUST_BUILD_HASH"), end.duration_since(active.start).as_nanos());
            for (n, (phase, elapsed, completed)) in active.phases.iter().enumerate() {
                if n != 0 { stderr.push(','); }
                let _ = write!(stderr, "{{\"phase\":\"{phase}\",\"elapsed_ns\":{elapsed},\"completed\":{completed}}}");
            }
            stderr.push_str("]}\n");
            Some(stderr)
        }
    }
}
