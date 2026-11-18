//! Object generation at the mixed compiler's first, scalar C boundary.
//! Checking, borrow checking, lowering, generation and retained body state are
//! the ordinary Lust pipeline. Rust's compiler supplies the caller and metadata.
use crate::{ast::{BinOp, UnOp}, cgen, check::{Builtin, collect::Program}, def::{DiskDefId, LocalDefId}, ir, protocol::{MixedOutcome, Request, Response}, ty::{IntTy, Ty, TyIntern, TyKind}};
use std::{fmt::Write as _, fs, path::Path, sync::Arc};

#[path = "mixed_symbols.rs"]
mod symbols;

fn operand(o: &ir::Operand) -> bool {
    match o {
        ir::Operand::Copy(p) | ir::Operand::Move(p) => p.proj.is_empty(),
        ir::Operand::Const(c) => match c {
            ir::Const::Int(_, int) => !matches!(int, IntTy::I128 | IntTy::U128),
            ir::Const::Bool(_) => true,
            _ => false,
        },
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
    // The minimal mixed C unit has integer typedefs, not Lust's two-word
    // 128-bit types/helpers or its unit-value representation. Those are real
    // complementary capabilities, even when their Rust operations are total.
    let accepted = b.locals.iter().all(|l| matches!(b.tys.kind(l.ty), TyKind::Int(i) if !matches!(i, IntTy::I128 | IntTy::U128))
            || matches!(b.tys.kind(l.ty), TyKind::Bool))
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

/// The checked body's boundary proof follows the ordinary native read-set
/// validation. Its source/definition keys also prevent a changed or renumbered
/// slot from using the previous proof during a metadata-only check.
pub(super) struct CheckedTotal {
    definition: DiskDefId,
    body_hash: u64,
    head_hash: u64,
    total: bool,
}

pub(super) enum Admission<'p> {
    Native(PreparedMixed<'p>),
    Complementary(&'static str),
}

/// This non-Clone value is created only after checking/lowering/borrowck and
/// consumed by emission in that same invocation. The Program borrow keeps the
/// exact semantic state alive; this is not a second check/build request.
pub(super) struct PreparedMixed<'p> {
    request: &'p Request,
    program: &'p Program,
    slot: usize,
    symbol: String,
    ret: &'static str,
    params: Vec<&'static str>,
}

pub(super) fn request_complement(req: &Request) -> Option<&'static str> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64"))
        || crate::protocol::host_triple() != "x86_64-unknown-linux-gnu"
        || !matches!(req.session.target_triple.as_str(), "" | "x86_64-unknown-linux-gnu")
        || req.debug
    {
        Some("the scalar object boundary needs its qualified Linux target and debug format")
    } else if req.cc || req.emit_c.is_some() || req.harness || req.run_args.is_some() {
        Some("the scalar object boundary does not represent release-C, harness or execution requests")
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare<'p, 'body, 'ir: 'body>(
    req: &'p Request,
    p: &'p Program,
    slots: &[u32],
    body_for: impl Fn(usize) -> Option<&'body ir::Body<'ir>>,
    checked_now: impl Fn(usize) -> bool,
    body_hashes: &[u64],
    head_hashes: &[u64],
    ok: &[bool],
    retained: &mut Option<CheckedTotal>,
) -> Result<Admission<'p>, String> {
    if let Some(reason) = request_complement(req) {
        return Ok(Admission::Complementary(reason));
    }
    let identity = req.session.krate.package_id.strip_prefix("lustr-mixed-").ok_or("missing mixed unit identity")?;
    if identity.is_empty() || identity.len() > 128 || !identity.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid mixed unit identity".into());
    }
    let mut selected = p.fns.iter().enumerate().filter(|(_, f)| p.interner.get(f.name) == req.session.krate.target_name);
    let (id, f) = selected.next().ok_or("native function was not collected")?;
    if selected.next().is_some() { return Err("ambiguous native function identity".into()); }
    let slot = *slots.get(id).ok_or("missing native function slot")? as usize;
    if ok.get(slot) != Some(&true) { return Err("native function has no passing checked state".into()); }
    let definition = p.disk_id(LocalDefId(id as u32));
    let body_hash = *body_hashes.get(slot).ok_or("missing checked body identity")?;
    let head_hash = *head_hashes.get(slot).ok_or("missing checked signature identity")?;
    let total = if let Some(body) = body_for(id) {
        if body.fn_id != LocalDefId(id as u32) { return Err("native lowered body identity mismatch".into()); }
        let total = total(body);
        *retained = Some(CheckedTotal { definition, body_hash, head_hash, total });
        total
    } else if !checked_now(id) {
        retained.as_ref().filter(|proof| proof.definition == definition && proof.body_hash == body_hash && proof.head_hash == head_hash)
            .ok_or("native function has no retained lowered boundary proof")?.total
    } else {
        return Err("native checked function lost its lowered body before admission".into());
    };
    if !total {
        return Ok(Admission::Complementary("function needs an unrepresented runtime operation or scalar type"));
    }
    let Some(ret) = scalar(p, f.ret) else { return Ok(Admission::Complementary("return needs a wider ABI mapping")); };
    let Some(params) = f.params.iter().map(|t| scalar(p, *t)).collect::<Option<Vec<_>>>() else {
        return Ok(Admission::Complementary("argument needs a wider ABI mapping"));
    };
    if !cgen::native_direct_abi(p, &f.params, f.ret) {
        return Ok(Admission::Complementary("native function does not have a direct C ABI"));
    }
    Ok(Admission::Native(PreparedMixed { request: req, program: p, slot, symbol: format!("lustr_mixed_{identity}"), ret, params }))
}

pub(super) fn complementary(reason: &str) -> Response {
    Response { mixed: Some(MixedOutcome::Complementary), stderr: format!("lustr mixed complementary: {reason}\n"), ..Default::default() }
}

pub(super) fn emit(prepared: PreparedMixed<'_>, out: &[cgen::FnOut], names: &Arc<cgen::names::Names>, generated: usize) -> Response {
    let req = prepared.request;
    // Read the requesting client's opt-in, not a possibly older daemon's
    // environment. This helper stays in the already-hashed owned source file.
    let mut profile = emitter_profile::Profile::new(req.client.env.as_deref());
    let result = emit_inner(prepared, out, names, &mut profile);
    let detail = profile.finish(req.session.krate.package_id.strip_prefix("lustr-mixed-").unwrap_or(""), result.is_ok());
    let mut response = match result {
        Ok(symbol) => Response { mixed: Some(MixedOutcome::Emitted), code: 0, stdout: symbol, stderr: format!("lustr mixed native: {generated} function(s) generated in retained Lust session\n"), ..Default::default() },
        Err(e) => Response { mixed: Some(MixedOutcome::Failed), code: 3, stderr: format!("lustr mixed emission failed: {e}\n"), ..Default::default() },
    };
    if let Some(detail) = detail { response.stderr.push_str(&detail); }
    response
}

fn emit_inner(prepared: PreparedMixed<'_>, out: &[cgen::FnOut],
    names: &Arc<cgen::names::Names>, profile: &mut emitter_profile::Profile) -> Result<String, String> {
    let PreparedMixed { request: req, program: _program, slot, symbol, ret, params } = prepared;
    if !out.get(slot).is_some_and(|o| o.mixed_total) { return Err("admitted function lost its scalar generation proof".into()); }
    let tag = names.fns.get(slot).and_then(|s| s.as_ref()).ok_or("missing stable native symbol")?;
    let mut c = cgen::mixed_scalar_unit(&out[slot]).ok_or("admitted function generated unrepresented helpers")?;
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
