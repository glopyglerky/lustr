//! Retained scalar library handoff from the native driver. This owns neither a
//! second frontend nor Rust metadata. The caller receives completed native facts
//! and actual per-function TinyCC objects from the same Session.
#[path = "native_completion.rs"]
pub(crate) mod completion;
use crate::{ast, check, cgen, def, diag, intern, ir, lexer};
use crate::driver::{Request, Response, Session, SessionKey};
use super::session::{Caches, Front};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use completion::{AcceptedBody, ExpressionResolution as Resolution, Function};

struct StoredBody { body_hash: u64, head_hash: u64, origin: diag::Span, facts: AcceptedBody }
#[derive(Clone)]
pub(crate) struct Object {
    pub definition: def::DiskDefId,
    pub symbol: String,
    pub c_source: Arc<str>,
    pub bytes: Arc<Vec<u8>>,
    pub serial: u64,
}
#[derive(Clone)]
pub(crate) struct Snapshot {
    pub generation: u64,
    pub source: Arc<str>,
    pub functions: Arc<Vec<Function>>,
    pub objects: Vec<(Arc<Object>, bool)>,
    pub checked: Vec<def::DiskDefId>,
    pub generated: Vec<def::DiskDefId>,
}
#[derive(Default)]
pub(crate) struct State {
    bodies: BTreeMap<def::DiskDefId, StoredBody>,
    completed: Option<(u64, Arc<Vec<Function>>)>,
    objects: BTreeMap<String, Arc<Object>>,
    by_definition: BTreeMap<def::DiskDefId, (u64, Arc<Object>)>,
    code_generation: BTreeMap<def::DiskDefId, u64>,
    next_code: u64,
    pub(crate) generation: u64,
    pub(crate) next_object: u64,
    pub(crate) compiled_objects: usize,
    pub(crate) reused_objects: usize,
    pub(crate) generated: Vec<def::DiskDefId>,
    pub source: Arc<str>,
    pub checked: Vec<def::DiskDefId>,
    pub(crate) checked_names: BTreeMap<def::DiskDefId,String>,
    pub snapshot: Option<Snapshot>,
}

impl Session {
    /// The mode is fixed before the first native request. No existing checked
    /// cache or image can be promoted to library proof state after the fact.
    pub(crate) fn scalar_library(obj_dir: PathBuf, key: SessionKey) -> Result<Self, String> {
        if key.krate.target_kind != "lib" || key.krate.manifest_path.is_some()
            || !key.profile.panic_abort || key.target_triple != "x86_64-unknown-linux-gnu"
            || crate::protocol::host_triple() != "x86_64-unknown-linux-gnu"
            || crate::image::state_dir().is_some() || std::env::var_os("LUST_FRONT_ARENA").is_some() {
            return Err("retained scalar library requires its fixed single-file Linux x64 abort context".into());
        }
        let mut session = Self::new(obj_dir, true, key);
        session.caches.library = Some(State::default());
        session.caches.output = super::OutputKind::Standalone;
        // This first join retains immutable object bytes inside the Session.
        // Disk image/object restoration needs a separately qualified proof key.
        session.cache_tried = true;
        Ok(session)
    }
    pub(crate) fn library_state(&self) -> Option<&State> { self.caches.library.as_ref() }
}
impl State {
    pub(crate) fn function_name(&self, id:def::DiskDefId)->Option<&str> {
        self.checked_names.get(&id).map(String::as_str).or_else(||self.completed.as_ref()?.1.iter().find(|f|f.definition()==id).map(|f|f.name()))
    }
    pub(super) fn begin(&mut self, source: &str) {
        self.source = Arc::from(source);
        self.checked.clear(); self.checked_names.clear(); self.generated.clear(); self.compiled_objects=0; self.reused_objects=0;
        self.snapshot = None;
    }
    pub(super) fn changed(&mut self) { self.generation += 1; }
}

/// The native resolver's ordinary original-root parser, with the one admitted
/// root policy consumed as syntax. It is called once at the normal parse point
/// and keeps original tokens/spans; body patches remain native body patches.
pub(super) fn parse_root(l: &lexer::Lexed, i: &intern::Interner, k: &intern::Known) -> Result<ast::Ast, diag::Diag> {
    let t = &l.tokens;
    if t.len() < 6 || t[0].tok != lexer::Tok::Pound || t[1].tok != lexer::Tok::Bang
        || t[2].tok != lexer::Tok::LBracket
        || !matches!(t[3].tok, lexer::Tok::Ident(s, _) if i.get(s)=="no_std")
        || t[4].tok != lexer::Tok::RBracket {
        return Err(diag::Diag::new("L0508", t.first().map_or(diag::Span::default(),|t|t.span), "native library requires its original #![no_std] root policy", "Retain the original crate policy."));
    }
    let mut a = crate::parser::parse_items(l, 5..t.len(), i, k)?;
    a.inner_attrs.insert(0, ast::Attr { style:ast::AttrStyle::Inner,
        path:vec![ast::PathSeg::plain(i.intern("no_std"))], input:ast::AttrInput::None,
        span:t[0].span.to(t[4].span) });
    Ok(a)
}

pub(super) fn capture(p: &check::collect::Program, id: def::LocalDefId, checked: &check::Checked<'_>) -> AcceptedBody {
    let expressions = checked.expr_res.iter().map(|resolution| match resolution {
        check::Res::None => Resolution::None,
        check::Res::Local(local) => checked.param_locals.iter().position(|param|param==local).map_or(Resolution::Other,Resolution::Parameter),
        check::Res::Fn(function) if (function.0 as usize)<p.fns.len() => Resolution::Function(p.disk_id(*function)),
        check::Res::Bin { ld:0,rd:0 } => Resolution::PrimitiveOperands,
        _ => Resolution::Other,
    }).collect();
    AcceptedBody { local:id,definition:p.disk_id(id),body:p.fn_def(id).body.unwrap(),expr_base:checked.expr_base,
        expressions:Arc::new(expressions),lowered_calls:None,generated_uses:None }
}
pub(super) fn lowered(p: &check::collect::Program, facts: &mut AcceptedBody, body: &ir::Body<'_>) {
    let mut calls = Some(BTreeSet::new());
    for block in &body.blocks {
        if let ir::Terminator::Call { target, .. } = block.term {
            match target {
                ir::CallTarget::Fn(id) if (id.0 as usize)<p.fns.len() => { if let Some(c)=&mut calls { c.insert(p.disk_id(id)); } }
                _ => calls = None,
            }
        }
    }
    facts.lowered_calls=calls;
}
pub(super) fn store(state: &mut State, facts: AcceptedBody, body_hash: u64, head_hash: u64, origin:diag::Span) {
    state.bodies.insert(facts.definition(),StoredBody {body_hash,head_hash,origin,facts});
}

/// Rebind the original native structural IR using the same validated body/head
/// and read-set ownership as the native worklist. Only original spans inside
/// the unchanged function move; no parser, checker or lowerer runs again.
pub(super) fn rebuild<'p>(p:&'p check::collect::Program, id:def::LocalDefId, front:&Front, caches:&Caches,
    map:&diag::SourceMap, line:u32)->Option<ir::Body<'p>> {
    let state=caches.library.as_ref()?; let disk=p.disk_id(id);let slot=front.fn_slot[id.0 as usize] as usize;
    let stored=state.bodies.get(&disk)?;
    if caches.ok.get(slot)!=Some(&true) || stored.body_hash!=front.slot_hash[slot] || stored.head_hash!=front.slot_head[slot] {return None;}
    let cached=caches.slot_ir.get(slot)?.as_ref()?;
    let mut body=super::ir_cache::rebuild(p,cached,id)?;
    let current=p.fn_def(id).span;
    if current.file!=stored.origin.file || current.ctxt!=stored.origin.ctxt
        || current.hi.checked_sub(current.lo)?!=stored.origin.hi.checked_sub(stored.origin.lo)? {return None;}
    let shift=|span:&mut diag::Span|->Option<()> {
        if *span==diag::Span::default(){return Some(());}
        if span.file!=stored.origin.file || span.ctxt!=stored.origin.ctxt || span.lo<stored.origin.lo || span.hi>stored.origin.hi {return None;}
        span.lo=current.lo.checked_add(span.lo.checked_sub(stored.origin.lo)?)?;
        span.hi=current.lo.checked_add(span.hi.checked_sub(stored.origin.lo)?)?;
        Some(())
    };
    for block in &mut body.blocks {
        for statement in &mut block.stmts {shift(&mut statement.span)?;}
        shift(&mut block.term_span)?;
    }
    (super::ir_cache::ir_fingerprint(p,&body,map,line)==cached.ir_fp).then_some(body)
}

pub(super) fn complete(p: &check::collect::Program, map: &diag::SourceMap, source: &str, front: &Front, caches: &Caches, verified: u64)
    -> Result<Arc<Vec<Function>>, completion::Error> {
    let state=caches.library.as_ref().ok_or(completion::Error::Failed("missing library session"))?;
    let mut bodies=Vec::with_capacity(p.fns.len());
    for (n,f) in p.fns.iter().enumerate() {
        let id=def::LocalDefId(n as u32); let disk=p.disk_id(id); let slot=front.fn_slot[n] as usize;
        let stored=state.bodies.get(&disk).ok_or(completion::Error::Failed("native retained body has no checked metadata facts"))?;
        if caches.ok.get(slot)!=Some(&true) || caches.slot_reads.get(slot).and_then(Option::as_ref)
            .is_none_or(|r|r.verified[usize::from(p.gx.walk_instances)]!=verified)
            || stored.body_hash!=front.slot_hash[slot] || stored.head_hash!=front.slot_head[slot] {
            return Err(completion::Error::Failed("native retained metadata facts do not match the current validated body reads"));
        }
        let f=f.def(&p.ast);
        if stored.facts.expressions.len()!=f.exprs.1.saturating_sub(f.exprs.0) as usize {
            return Err(completion::Error::Failed("native retained expression range changed without a check"));
        }
        let mut current=stored.facts.clone(); current.local=id;
        current.body=f.body.ok_or(completion::Error::Unintegrated("bodyless function metadata"))?;
        current.expr_base=f.exprs.0;
        bodies.push(current);
    }
    if let Some((generation,functions))=&state.completed && *generation==state.generation { return Ok(functions.clone()); }
    // The original parser starts a plain parameter's span at its name token
    // (parser.rs: plain parameter branch), retaining that token's context.
    // Completion excludes patterns, attributes, mutation and non-ASCII source.
    // This validates that exact AST-bound interval; it does not search/lex.
    let identifier=|name,span:diag::Span| {
        let name=p.interner.get(name);
        let hi=span.lo.checked_add(name.len().try_into().ok()?)?;
        (hi<=span.hi && source.get(span.lo as usize..hi as usize)==Some(name))
            .then_some(diag::Span {hi,..span})
    };
    completion::complete(completion::View {program:p,source,map,bodies:&bodies,identifier:&identifier},false).map(Arc::new)
}

pub(super) fn remember(caches: &mut Caches, functions: Arc<Vec<Function>>) {
    let state=caches.library.as_mut().unwrap();
    state.completed=Some((state.generation,functions));
}

pub(super) fn finish(req: &Request, p: &check::collect::Program, front: &Front, caches: &mut Caches,
    functions:Arc<Vec<Function>>, names:&cgen::names::Names, generated:&[usize]) -> Response {
    let state=caches.library.as_mut().unwrap();
    // Generated C may already be current when a later TinyCC operation fails.
    // Invalidate old definition-to-object joins before any fallible emission;
    // content-addressed immutable objects remain available for exact reuse.
    for &n in generated {
        let id=p.disk_id(def::LocalDefId(n as u32));
        state.generated.push(id);
        state.next_code+=1; state.code_generation.insert(id,state.next_code);
        state.by_definition.remove(&id);
    }
    let mut disk_at=vec![None;caches.out.len()];
    for (n,&slot) in front.fn_slot.iter().enumerate() { disk_at[slot as usize]=Some(p.disk_id(def::LocalDefId(n as u32))); }
    let generated_set:BTreeSet<_>=generated.iter().copied().collect();
    let result=(|| -> Result<Snapshot,String> {
        if req.debug || req.cc || req.emit_c.is_some() || req.harness || req.run_args.is_some() {
            return Err("retained library output context changed".into());
        }
        let mut objects=Vec::new();
        for f in functions.iter() {
            let slot=front.fn_slot[f.local().0 as usize] as usize;
            let output=&caches.out[slot];
            if !caches.has_out[slot] { return Err("native library function has no current generated body".into()); }
            let uses:Option<BTreeSet<_>>=output.used.iter().map(|&slot|disk_at.get(slot as usize).copied().flatten()).collect();
            let mut expected=f.calls().clone();expected.insert(f.definition());
            if uses.as_ref()!=Some(&expected) { return Err("native library generated source graph differs from completed calls".into()); }
            let symbol=format!("f_{}",names.fns[slot].as_ref().ok_or("native library stable symbol missing")?);
            let unchanged=!generated_set.contains(&(f.local().0 as usize));
            let code_generation=*state.code_generation.get(&f.definition()).ok_or("missing current native C generation")?;
            if unchanged && let Some((bound_generation,object))=state.by_definition.get(&f.definition()) {
                if *bound_generation!=code_generation || object.symbol!=symbol { return Err("retained native object symbol context changed".into()); }
                state.reused_objects+=1; objects.push((object.clone(),true));continue;
            }
            let text=cgen::native_scalar_unit(output).ok_or("native library function requires unrepresented runtime/glue")?;
            if let Some(object)=state.objects.get(&text) {
                if object.definition!=f.definition() || object.symbol!=symbol { return Err("native object cache owner differs".into()); }
                state.by_definition.insert(f.definition(),(code_generation,object.clone()));
                state.reused_objects+=1; objects.push((object.clone(),true));continue;
            }
            state.next_object+=1;
            std::fs::create_dir_all(&caches.obj_dir).map_err(|e|format!("native object staging: {e}"))?;
            let path=caches.obj_dir.join(format!("native-library-{}.o",state.next_object));
            // A singleton uses Lust's original in-process libtcc path. Never
            // hand a prior artifact to compile_units, which removes its output.
            crate::tcc::compile_units(&[(path.clone(),text.clone())],false)?;
            let bytes=std::fs::read(&path).map_err(|e|format!("native object bytes: {e}"))?;
            std::fs::remove_file(&path).map_err(|e|format!("native object staging cleanup: {e}"))?;
            let object=Arc::new(Object {definition:f.definition(),symbol,c_source:Arc::from(text.as_str()),bytes:Arc::new(bytes),serial:state.next_object});
            state.objects.insert(text,object.clone()); state.by_definition.insert(f.definition(),(code_generation,object.clone()));
            state.compiled_objects+=1; objects.push((object,false));
        }
        Ok(Snapshot { generation:state.generation,source:state.source.clone(),functions,
            objects,checked:state.checked.clone(),generated:generated.iter().map(|&n|p.disk_id(def::LocalDefId(n as u32))).collect() })
    })();
    match result { Ok(snapshot)=>{state.snapshot=Some(snapshot);Response::default()},
        Err(error)=>Response{code:3,stderr:format!("native library emission failed: {error}\n"),..Default::default()} }
}
