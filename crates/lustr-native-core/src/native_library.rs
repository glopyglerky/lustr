//! Retained scalar library handoff from the native driver. This owns neither a
//! second frontend nor Rust metadata. The caller receives completed native facts
//! and actual per-function TinyCC objects from the same Session.
#[path = "native_completion.rs"]
pub(crate) mod completion;
#[path = "library_history.rs"]
pub(crate) mod history;
use crate::{ast, check, cgen, def, diag, intern, ir, lexer};
use crate::driver::{Request, Response, Session, SessionKey};
use super::session::{Caches, Front};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use completion::{AcceptedBody, ExpressionResolution as Resolution, Function};

struct StoredBody { body_hash: u64, head_hash: u64, origin: Option<diag::Span>, facts: AcceptedBody }
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
    history_policy:history::Policy,
    history_dirty:bool,
    current_definitions:BTreeSet<def::DiskDefId>,
    object_used:BTreeMap<u64,u64>,
    identity_used:BTreeMap<def::DiskDefId,u64>,
    published:Option<Snapshot>,
    pub(crate) history:History,

    pub(crate) generation: u64,
    pub(crate) next_object: u64,
    pub(crate) compiled_objects: usize,
    pub(crate) reused_objects: usize,
    pub(crate) generated: Vec<def::DiskDefId>,
    pub source: Arc<str>,
    pub checked: Vec<def::DiskDefId>,
    pub(crate) checked_names: BTreeMap<def::DiskDefId,String>,
    pub snapshot: Option<Snapshot>,
    pub(crate) dependency_lint_ready: bool,
}

#[derive(Default)]
pub(crate) struct History {
    pub active_objects:usize, pub inactive_objects:usize, pub inactive_object_bytes:usize,
    pub active_definitions:usize, pub inactive_definitions:usize, pub inactive_definition_bytes:usize,
    pub evicted_object_serials:Vec<u64>, pub retired_definition_ids:Vec<def::DiskDefId>,
}
impl StoredBody {
    fn bytes(&self)->usize {
        std::mem::size_of::<Self>()+self.facts.expressions.capacity()*std::mem::size_of::<Resolution>()
            +(self.facts.lowered_calls.as_ref().map_or(0,BTreeSet::len)+self.facts.generated_uses.as_ref().map_or(0,BTreeSet::len))*std::mem::size_of::<def::DiskDefId>()
    }
}
impl Session {
    /// The mode is fixed before the first native request. No existing checked
    /// cache or image can be promoted to library proof state after the fact.
    pub(crate) fn scalar_library(obj_dir: PathBuf, key: SessionKey) -> Result<Self, String> {
        if key.krate.target_kind != "lib" || key.krate.manifest_path.is_some() && key.library.is_none()
            || !key.profile.panic_abort || key.target_triple != "x86_64-unknown-linux-gnu"
            || crate::protocol::host_triple() != "x86_64-unknown-linux-gnu"
            || crate::image::state_dir().is_some() || std::env::var_os("LUST_FRONT_ARENA").is_some() {
            return Err("retained scalar library requires its fixed single-file Linux x64 abort context".into());
        }
        let mut session = Self::new(obj_dir, true, key);
        session.caches.library = Some(State{history_policy:history::policy()?,..Default::default()});
        session.caches.output = super::OutputKind::Standalone;
        // This first join retains immutable object bytes inside the Session.
        // Disk image/object restoration needs a separately qualified proof key.
        session.cache_tried = true;
        Ok(session)
    }
    pub(crate) fn library_state(&self) -> Option<&State> { self.caches.library.as_ref() }
    pub(crate) fn configure_library_history(&mut self,policy:history::Policy) {
        if let Some(state)=&mut self.caches.library {
            state.history_policy=policy;state.history.evicted_object_serials.clear();state.history.retired_definition_ids.clear();
            state.dependency_lint_ready=false;
        }
    }
    pub(crate) fn prune_library_history(&mut self) {prune(&self.front,&mut self.caches);}
    /// Called only after durable publication (or exact existing output reuse).
    /// Maintenance is infallible and cannot turn a committed output into a
    /// claimed pre-publication failure.
    pub(crate) fn library_published(&mut self) {
        if let Some(state)=&mut self.caches.library {
            if state.published.as_ref().map(|s|s.generation)!=state.snapshot.as_ref().map(|s|s.generation) {
                state.published=state.snapshot.clone();state.history_dirty=true;
            }
        }
        self.prune_library_history();
    }
}
/// Public contexts are validated before reading or preparing source. The
/// constructor derives the mode from the same key, including after every reset.
pub(crate) fn request_supported(req: &Request) -> bool {
    let Some(key) = &req.session.library else { return req.library_options == crate::protocol::LibraryOptions::default(); };
    key.valid() && key.source_path == req.file
        && key.valid_output(&req.out, req.cmd == crate::protocol::Cmd::Check, req.library_options)
        && req.session.krate.target_kind == "lib"
        && req.session.krate.target_name == key.crate_name && req.session.profile.panic_abort
        && req.session.target_triple == crate::protocol::library_contract::TARGET
        && crate::protocol::host_triple() == crate::protocol::library_contract::TARGET
        && !req.debug && !req.cc && !req.harness && req.emit_c.is_none() && req.run_args.is_none()
        && crate::image::state_dir().is_none() && std::env::var_os("LUST_FRONT_ARENA").is_none()
}
impl State {
    pub(crate) fn completed_functions(&self) -> Option<&[Function]> {
        let (generation, functions) = self.completed.as_ref()?;
        (*generation == self.generation).then_some(functions.as_slice())
    }

    pub(crate) fn function_name(&self, id:def::DiskDefId)->Option<&str> {
        self.checked_names.get(&id).map(String::as_str).or_else(||self.completed.as_ref()?.1.iter().find(|f|f.definition()==id).map(|f|f.name()))
    }
    pub(super) fn begin(&mut self, source: &str) {
        self.source = Arc::from(source);
        self.checked.clear(); self.checked_names.clear(); self.generated.clear(); self.compiled_objects=0; self.reused_objects=0;
        self.snapshot = None;
        self.dependency_lint_ready = false;
        self.history.evicted_object_serials.clear();self.history.retired_definition_ids.clear();
    }
    pub(super) fn changed(&mut self) { self.generation += 1; self.history_dirty=true; }
    pub(super) fn current(&mut self,ids:impl Iterator<Item=def::DiskDefId>) {
        let current:BTreeSet<_>=ids.collect();
        if current!=self.current_definitions {self.current_definitions=current;self.history_dirty=true;}
    }
    fn prune(&mut self)->Vec<def::DiskDefId>{
        if !self.history_dirty{return Vec::new();}
        self.history_dirty=false;
        let mut protected_ids=self.current_definitions.clone();
        let mut protected_objects:BTreeSet<_>=self.by_definition.iter().filter(|(id,_)|self.current_definitions.contains(*id))
            .map(|(_,(_,object))|object.serial).collect();
        if let Some(published)=&self.published {
            protected_ids.extend(published.functions.iter().map(Function::definition));
            protected_objects.extend(published.objects.iter().map(|(object,_)|object.serial));
        }
        if let Some(snapshot)=&self.snapshot {
            protected_objects.extend(snapshot.objects.iter().map(|(object,_)|object.serial));
        }
        for id in &self.current_definitions {self.identity_used.insert(*id,self.generation);}
        for serial in &protected_objects {self.object_used.insert(*serial,self.generation);}
        let object_bytes=|key:&String,object:&Object|key.capacity()+object.c_source.len()+object.bytes.capacity()+object.symbol.capacity()+std::mem::size_of::<Object>();
        let inactive:Vec<_>=self.objects.iter().filter(|(_,o)|!protected_objects.contains(&o.serial))
            .map(|(key,o)|(key.clone(),*self.object_used.get(&o.serial).unwrap_or(&0),object_bytes(key,o))).collect();
        for key in history::victims(inactive,self.history_policy) {
            if let Some(object)=self.objects.remove(&key) {
                self.history.evicted_object_serials.push(object.serial);self.object_used.remove(&object.serial);
                self.by_definition.retain(|_,(_,held)|held.serial!=object.serial);
            }
        }
        let identities:BTreeSet<_>=self.bodies.keys().chain(self.by_definition.keys()).chain(self.code_generation.keys()).copied().collect();
        let identity_bytes=|id:&def::DiskDefId|self.bodies.get(id).map_or(0,StoredBody::bytes)
            +usize::from(self.by_definition.contains_key(id))*std::mem::size_of::<(def::DiskDefId,u64,Arc<Object>)>()
            +usize::from(self.code_generation.contains_key(id))*std::mem::size_of::<(def::DiskDefId,u64)>();
        let inactive:Vec<_>=identities.iter().filter(|id|!protected_ids.contains(*id))
            .map(|id|(*id,*self.identity_used.get(id).unwrap_or(&0),identity_bytes(id))).collect();
        let retired=history::victims(inactive,self.history_policy);
        for id in &retired {
            self.bodies.remove(id);self.by_definition.remove(id);self.code_generation.remove(id);self.identity_used.remove(id);
        }
        self.history.retired_definition_ids.extend(retired.iter().copied());
        self.history.active_objects=self.objects.values().filter(|o|protected_objects.contains(&o.serial)).count();
        self.history.inactive_objects=self.objects.len()-self.history.active_objects;
        self.history.inactive_object_bytes=self.objects.iter().filter(|(_,o)|!protected_objects.contains(&o.serial)).map(|(k,o)|object_bytes(k,o)).sum();
        let identities:BTreeSet<_>=self.bodies.keys().chain(self.by_definition.keys()).chain(self.code_generation.keys()).copied().collect();
        self.history.active_definitions=identities.iter().filter(|id|protected_ids.contains(*id)).count();
        self.history.inactive_definitions=identities.len()-self.history.active_definitions;
        self.identity_used.retain(|id,_|identities.contains(id));
        let object_serials:BTreeSet<_>=self.objects.values().map(|o|o.serial).collect();
        self.object_used.retain(|serial,_|object_serials.contains(serial));
        self.history.inactive_definition_bytes=identities.iter().filter(|id|!protected_ids.contains(*id)).map(|id|
            self.bodies.get(id).map_or(0,StoredBody::bytes)
                +usize::from(self.by_definition.contains_key(id))*std::mem::size_of::<(def::DiskDefId,u64,Arc<Object>)>()
                +usize::from(self.code_generation.contains_key(id))*std::mem::size_of::<(def::DiskDefId,u64)>()).sum();
        retired
    }
}

fn prune(front:&Front,caches:&mut Caches) {
    let Some(state)=&mut caches.library else{return;};
    for id in state.prune() {
        // Retire the joined native verdict with its metadata facts. A later
        // reintroduction enters the existing worklist once; stale accepted
        // slots cannot bypass checking after their facts were evicted.
        if let Some(&slot)=front.slot_of.get(&id) {
            let slot=slot as usize;
            if let Some(ok)=caches.ok.get_mut(slot){*ok=false;}
            if let Some(output)=caches.has_out.get_mut(slot){*output=false;}
            if let Some(ir)=caches.slot_ir.get_mut(slot){*ir=None;}
            if let Some(reads)=caches.slot_reads.get_mut(slot){*reads=None;}
            if let Some(output)=caches.out.get_mut(slot){*output=cgen::FnOut::default();}
        }
    }
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
// FnDef has only a name span. IR rebinding needs the complete function item
// interval, including its body. Other declaration locations stay unrepresented
// here and remain subject to the existing metadata completion refusal.
fn function_span(p: &check::collect::Program, id: def::LocalDefId) -> Option<diag::Span> {
    let check::collect::FnLoc::Item(index) = p.fns.get(id.0 as usize)?.loc else { return None; };
    let item = p.ast.items.get(index as usize)?;
    matches!(&item.kind, ast::ItemKind::Fn(_)).then_some(item.span)
}

pub(super) fn store(p: &check::collect::Program, state: &mut State, facts: AcceptedBody, body_hash: u64, head_hash: u64) {
    let origin = function_span(p, facts.local());
    state.history_dirty=true;
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
    let origin = stored.origin?;
    let current = function_span(p, id)?;
    if current.file!=origin.file || current.ctxt!=origin.ctxt
        || current.hi.checked_sub(current.lo)?!=origin.hi.checked_sub(origin.lo)? {return None;}
    let shift=|span:&mut diag::Span|->Option<()> {
        if *span==diag::Span::default(){return Some(());}
        if span.file!=origin.file || span.ctxt!=origin.ctxt || span.lo<origin.lo || span.hi>origin.hi {return None;}
        span.lo=current.lo.checked_add(span.lo.checked_sub(origin.lo)?)?;
        span.hi=current.lo.checked_add(span.hi.checked_sub(origin.lo)?)?;
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
    state.current(functions.iter().map(Function::definition));
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
    match result { Ok(snapshot)=>{state.snapshot=Some(snapshot);prune(front,caches);Response::default()},
        Err(error)=>Response{code:3,stderr:format!("native library emission failed: {error}\n"),..Default::default()} }
}
