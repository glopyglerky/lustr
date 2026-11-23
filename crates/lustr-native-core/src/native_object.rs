//! Native freestanding object seam, under qualification.
//! The existing native semantic pipeline; no std loader,
//! host runtime unit, rustc acceptance, retry or retained-object replay.
//! This is not yet the full core/alloc library or Cargo metadata interface.
use crate::{ast, borrowck, cgen, check, def, diag, intern, lexer, lower, parser, ty};
use std::{collections::BTreeSet, fs, ops::Range, path::PathBuf, process::Command, sync::Arc};
use ty::TyIntern;

struct Request { source: PathBuf, output: PathBuf, name: String, metadata: String, overflow: bool, debug: bool }
fn request(args: &[String]) -> Result<Request, String> {
    let mut source = None; let mut output = None; let mut name = None;
    let mut edition = false; let mut lib = false; let mut obj = false; let mut abort = false;
    let mut overflow = None; let mut debug = None; let mut metadata = None;
    let mut target = false; let mut optimize = false; let mut debuginfo = false; let mut n = 0;
    while n < args.len() {
        let a = &args[n];
        match a.as_str() {
            "--crate-name" | "-o" => {
                n += 1; let value = args.get(n).ok_or("missing unit argument")?;
                if a == "-o" { if output.replace(PathBuf::from(value)).is_some() { return Err("duplicate output".into()); } }
                else if name.replace(value.clone()).is_some() { return Err("duplicate crate name".into()); }
            }
            "--edition=2024" if !edition => edition = true,
            "--crate-type=lib" if !lib => lib = true,
            "--emit=obj" if !obj => obj = true,
            "-Cpanic=abort" if !abort => abort = true,
            "-Coverflow-checks=on" if overflow.is_none() => overflow = Some(true),
            "-Coverflow-checks=off" if overflow.is_none() => overflow = Some(false),
            "-Cdebug-assertions=on" if debug.is_none() => debug = Some(true),
            "-Cdebug-assertions=off" if debug.is_none() => debug = Some(false),
            "--target=x86_64-unknown-linux-gnu" if !target => target = true,
            "-Copt-level=0" if !optimize => optimize = true,
            "-Cdebuginfo=0" if !debuginfo => debuginfo = true,
            _ if a.starts_with("-Cmetadata=") && metadata.is_none() => metadata = Some(a[11..].to_owned()),
            _ if !a.starts_with('-') && a.ends_with(".rs") && source.is_none() => source = Some(PathBuf::from(a)),
            _ => return Err(format!("native object interface does not yet implement argument {a:?}")),
        }
        n += 1;
    }
    if !(edition && lib && obj && abort && target && optimize && debuginfo) { return Err("native object interface requires explicit edition=2024, lib, obj, panic=abort, x86_64-unknown-linux-gnu, opt-level=0 and debuginfo=0".into()); }
    let name = name.ok_or("explicit native crate name required")?;
    if name.is_empty() || !name.bytes().enumerate().all(|(n,b)| b == b'_' || b.is_ascii_alphabetic() || (n != 0 && b.is_ascii_digit())) {
        return Err("invalid native crate name".into());
    }
    let metadata = metadata.ok_or("explicit native unit metadata disambiguator required")?;
    if metadata.is_empty() || metadata.len()>128 || !metadata.bytes().all(|b|b.is_ascii_alphanumeric() || b==b'_') { return Err("invalid native unit metadata disambiguator".into()); }
    Ok(Request { source: source.ok_or("source required")?, output: output.ok_or("output required")?, name, metadata,
        overflow: overflow.ok_or("explicit overflow policy required")?, debug: debug.ok_or("explicit assertion policy required")? })
}

pub(crate) fn core_type(tys: &impl TyIntern, t: ty::Ty, depth: usize) -> bool {
    if depth > 128 { return false; }
    match tys.kind(t) {
        ty::TyKind::Unit | ty::TyKind::Bool | ty::TyKind::Char | ty::TyKind::Never | ty::TyKind::Int(_) => true,
        ty::TyKind::Ref(_,_,t) | ty::TyKind::Slice(t) | ty::TyKind::Array(t,_) => core_type(tys,t,depth+1),
        ty::TyKind::Tuple(ts) => tys.tys_of(ts).iter().all(|t| core_type(tys,*t,depth+1)),
        // Actual builtin/local definitions decide this boundary; no spelling
        // such as Vec or String grants or removes an allocation capability.
        // ADT/lang-item and allocator images are not qualified at this seam.
        _ => false,
    }
}

enum Completion { Durable(String), PublishedUncertain(String) }

pub(crate) use crate::driver::library::completion::{AcceptedBody, ExpressionResolution};

/// Owns the original source, declaration tables and generated native bodies.
/// There is no public constructor, Clone, or mutable view. A metadata adapter
/// can inspect the accepted declarations without parsing or checking them
/// again. Dropping this value creates no object and publishes no artifact.
pub(crate) struct PreparedUnit {
    request: Request,
    source: String,
    map: diag::SourceMap,
    lexed: lexer::Lexed,
    program: check::collect::Program,
    names: Arc<cgen::names::Names>,
    generated: Vec<cgen::FnOut>,
    accepted: Vec<AcceptedBody>,
    artifact: bool,
    capture: bool,
    working_directory: Option<PathBuf>,
}

impl PreparedUnit {
    pub(crate) fn program(&self) -> &check::collect::Program { &self.program }
    pub(crate) fn source(&self) -> &str { &self.source }
    pub(crate) fn source_path(&self) -> &std::path::Path { &self.request.source }
    pub(crate) fn working_directory(&self) -> Option<&std::path::Path> { self.working_directory.as_deref() }
    pub(crate) fn source_map(&self) -> &diag::SourceMap { &self.map }
    /// Recover the exact original name token without another lexer pass. A
    /// parameter's AST span can include its type; metadata needs its name.
    pub(crate) fn identifier_span(&self, name: intern::Sym, enclosing: diag::Span) -> Option<diag::Span> {
        let index = self.lexed.tokens.binary_search_by_key(&enclosing.lo, |token| token.span.lo).ok()?;
        let token = &self.lexed.tokens[index];
        (token.span.file == enclosing.file && token.span.ctxt == enclosing.ctxt
            && token.span.hi <= enclosing.hi
            && matches!(token.tok, lexer::Tok::Ident(actual, _) if actual == name)).then_some(token.span)
    }
    pub(crate) fn crate_name(&self) -> &str { &self.request.name }
    pub(crate) fn metadata(&self) -> &str { &self.request.metadata }
    pub(crate) fn output(&self) -> &std::path::Path { &self.request.output }
    pub(crate) fn overflow_checks(&self) -> bool { self.request.overflow }
    pub(crate) fn debug_assertions(&self) -> bool { self.request.debug }
    pub(crate) fn accepted_bodies(&self) -> &[AcceptedBody] { &self.accepted }
}

/// A source function whose definition is present in this exact emitted object.
/// Nonlocal backend helper symbols are retained separately; they are not
/// fabricated source definitions or Rust monomorphizations.
pub(crate) struct EmittedFunction {
    definition: def::DiskDefId,
    symbol: Vec<u8>,
}

impl EmittedFunction {
    pub(crate) fn definition(&self) -> def::DiskDefId { self.definition }
    pub(crate) fn symbol(&self) -> &[u8] { &self.symbol }
}

pub(crate) struct EmittedUnit {
    prepared: PreparedUnit,
    object: PathBuf,
    scratch: PathBuf,
    bytes: Vec<u8>,
    functions: Vec<EmittedFunction>,
    helpers: BTreeSet<Vec<u8>>,
    runtime: Option<crate::native_crate::runtime::Requirements>,
}

impl EmittedUnit {
    pub(crate) fn prepared(&self) -> &PreparedUnit { &self.prepared }
    pub(crate) fn object(&self) -> &[u8] { &self.bytes }
    pub(crate) fn functions(&self) -> &[EmittedFunction] { &self.functions }
    pub(crate) fn helpers(&self) -> &BTreeSet<Vec<u8>> { &self.helpers }
    pub(crate) fn publish_object(&self) -> Result<super::unit_publication::Publication, String> {
        super::unit_publication::publish(&self.object, self.prepared.output())
            .map_err(|error| format!("cannot publish native object: {error}"))
    }
}

/// Prepare the existing freestanding object request. This is its only native
/// frontend invocation; it does not query a Rust compiler or emit an object.
pub(crate) fn prepare(args: &[String]) -> Result<PreparedUnit, String> {
    prepare_request(request(args)?, false, true, None)
}

/// The enclosing compiler has already assigned the other items to their
/// owners. Keep the full original source and positions for this native item.
pub(crate) fn prepare_item(args: &[String], source: &str, span: Range<u32>) -> Result<PreparedUnit, String> {
    prepare_request(request(args)?, false, true, Some((source, span)))
}

fn item_tokens(mut lexed: lexer::Lexed, extent: Range<u32>) -> Result<lexer::Lexed, String> {
    let tokens = &lexed.tokens;
    let policy = tokens.get(..5).ok_or("missing original crate policy")?;
    let start = tokens.partition_point(|token| token.span.hi <= extent.start);
    let end = tokens.partition_point(|token| token.span.lo < extent.end);
    let item = tokens.get(start..end).ok_or("native item range is out of order")?;
    if !item.first().is_some_and(|token| token.span.lo == extent.start)
        || !item.last().is_some_and(|token| token.span.hi == extent.end)
        || start < policy.len()
    {
        return Err("native item range does not match whole original tokens".into());
    }
    lexed.tokens = policy.iter().chain(item).cloned().collect();
    Ok(lexed)
}

fn prepare_request(r: Request, artifact: bool, capture: bool, item: Option<(&str, Range<u32>)>) -> Result<PreparedUnit, String> {
    if r.output.exists() { return Err("object output already exists; use a fresh path".into()); }
    let working_directory = capture.then(std::env::current_dir).transpose()
        .map_err(|error| format!("cannot retain native request working directory: {error}"))?;
    let source = fs::read_to_string(&r.source).map_err(|e| format!("cannot read source: {e}"))?;
    if source.len() > 2 * 1024 * 1024 { return Err("native object source exceeds current bound".into()); }
    let i = Arc::new(intern::Interner::default()); let k = Arc::new(intern::Known::new(&i));
    let mut map = diag::SourceMap::default(); let file = map.add_file(&r.source.to_string_lossy(), &source);
    if item.as_ref().is_some_and(|(expected, _)| *expected != source) {
        return Err("the original source changed since the item was selected".into());
    }
    let lexed = lexer::lex(&source,file,0,&i).map_err(|e| diag::render(&mut vec![e], &map, &source, false, 20))?;
    let selected = item.is_some();
    let lexed = match item {
        Some((_, extent)) => item_tokens(lexed, extent)?,
        None => lexed,
    };
    // This boundary consumes a real crate attribute and carries its policy
    // into a freestanding object pipeline. Other root attributes still need
    // their native policy interface; none are silently discarded.
    let t = &lexed.tokens;
    let pattern = [lexer::Tok::Pound, lexer::Tok::Bang, lexer::Tok::LBracket];
    if t.len() < 6 || t[..3].iter().map(|t|t.tok).ne(pattern)
        || !matches!(t[3].tok,lexer::Tok::Ident(s, _) if i.get(s)=="no_std") || t[4].tok != lexer::Tok::RBracket
    { return Err("native freestanding object requires a complete original #![no_std] crate attribute".into()); }
    let attr = ast::Attr { style: ast::AttrStyle::Inner, path: vec![ast::PathSeg::plain(i.intern("no_std"))], input: ast::AttrInput::None, span: t[0].span.to(t[4].span) };
    // The artifact route uses the shared native resolver below. The bare
    // object route retains its qualified-path refusal. Additional attributes
    // need policy/input records at both boundaries; they are never discarded.
    if t[5..].iter().any(|t|matches!(t.tok,lexer::Tok::Pound)||!artifact&&t.tok==lexer::Tok::PathSep) {
        return Err("native unit resolution is required for attributes and qualified paths".into());
    }
    let identity = format!("lustr-native-object-v1/{}/{}/x86_64-unknown-linux-gnu/abort/opt0/debug0/overflow{}/assert{}",r.name,r.metadata,r.overflow,r.debug);
    let ids = def::CrateIds { local: def::LOCAL_CRATE, stable: def::stable_crate_hash(&identity,"lib",&r.name,false) };
    let (a,tables) = if artifact {
        // Macro effects and additional files need explicit input/expansion
        // generations. Refuse them before treating this closed image as deps0.
        if t.windows(2).any(|w|matches!(w[0].tok,lexer::Tok::Ident(_, _))&&w[1].tok==lexer::Tok::Bang){return Err("native producer needs macro semantic/input records".into());}
        if t.iter().any(|t|t.tok==lexer::Tok::Extern){return Err("native producer needs extern dependency/ABI semantic records".into());}
        let input=crate::resolve::worklist::CrateInput{procedural_macros:None,root_path:r.source.clone(),root_text:source.clone(),env:crate::resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,crate::protocol::ProfileFlags{panic_abort:true,overflow_checks:r.overflow,debug_assertions:r.debug}),crate_ids:ids,crate_name:r.name.clone(),cargo_env:Vec::new(),deps:Vec::new(),client:Default::default(),std_splice_lo:None,display_root:None};
        let parse=|l:&lexer::Lexed,i:&intern::Interner,k:&intern::Known| {
            // Consume the original, already admitted crate-policy attribute
            // as syntax, preserving its exact span and AST record. This is
            // not global parser acceptance or a core/std identity grant.
            if l.tokens.len()<5||l.tokens[..5].iter().zip(&t[..5]).any(|(a,b)|a.tok!=b.tok||a.span!=b.span){return Err(diag::Diag::new("L0508",attr.span,"native producer root policy changed","Retain the original crate attribute."));}
            let mut a=parser::parse_items(l,5..l.tokens.len(),i,k)?;a.inner_attrs.insert(0,attr.clone());Ok(a)
        };
        let resolved=crate::resolve::worklist::resolve_crate_with_root_parser(&input,&i,&k,&parse).map_err(|mut e|diag::render(&mut e.diags,&e.map,&e.all_text,false,20))?;
        if resolved.map.file_count()!=1||!resolved.expand_deps.is_empty(){return Err("native producer needs additional source/expansion input records".into());}
        map=resolved.map;(resolved.ast,Some(Arc::new(resolved.tables)))
    } else {
        let mut a=parser::parse_items(&lexed,5..t.len(),&i,&k).map_err(|e|diag::render(&mut vec![e],&map,&source,false,20))?;a.inner_attrs.insert(0,attr);(a,None)
    };
    if selected && !matches!(a.items.as_slice(), [ast::Item { kind: ast::ItemKind::Fn(_), .. }]) {
        return Err("the selected native item must be one function definition".into());
    }
    if a.items.iter().chain(&a.stmt_items).any(|item|(!artifact&&matches!(item.vis,ast::Vis::Restricted{..})) || !matches!(item.kind,ast::ItemKind::Fn(_)|ast::ItemKind::Const(_)) && !(artifact && matches!(item.kind,ast::ItemKind::Struct(_)|ast::ItemKind::Enum(_))))
        || a.exprs.iter().any(|e|matches!(e,ast::Expr::Macro{..}|ast::Expr::MacroCall(_)))
    { return Err("native unit resolution/expansion is required for these items or macros".into()); }
    // The resolver must have bound each qualified path to a real local def.
    // Its std compatibility paths cannot stand in for core/alloc identities.
    let represented=|path:&[ast::PathSeg]|path.len()<=1||path.len()==2&&tables.as_ref().is_some_and(|t|t.flat[0].get(&path[0].name).and_then(|d|t.variants.get(d)).is_some_and(|vs|vs.contains(&path[1].name)));
    if artifact&&(a.exprs.iter().any(|e|matches!(e,ast::Expr::Path(p) if !represented(p)))||a.tys.iter().any(|t|matches!(t,ast::TyExpr::Path(p) if !represented(p)))){return Err("native producer needs external typed path records".into());}
    let report = |mut d: Vec<diag::Diag>| diag::render(&mut d, &map, &source, false, 20);
    let mut order=check::collect::DefOrder::default();
    let mut p = check::collect::collect_with_order_resolved(Arc::new(a),i,k,ids,false,&mut order,tables).map_err(report)?;
    p.unwind = false; p.overflow_checks = r.overflow; p.debug_assertions = r.debug; p.unwind_tables = false;
    if artifact{crate::native_crate::prepare_declared_types(&mut p)?;}
    let mut errors = Vec::new();
    if artifact{crate::native_crate::check_initializers(&mut p,&mut errors);}else{check::const_body::check_all(&mut p,&mut errors);}
    if !errors.is_empty() { return Err(report(errors)); }
    if !p.statics.is_empty() { return Err("native freestanding statics artifact interface remains unqualified".into()); }
    let slots: Vec<u32> = (0..p.fns.len()).map(|n|n as u32).collect();
    let names = crate::native_crate::names(&p);
    let mut out = Vec::new();
    let mut accepted = Vec::new();
    let mut methods:BTreeSet<u32>=p.ast.exprs.iter().enumerate().filter_map(|(n,e)|matches!(e,ast::Expr::MethodCall{..}).then_some(n as u32)).collect();
    for (n,f) in p.fns.iter().enumerate() {
        let id = def::LocalDefId(n as u32);
        if p.gx.fn_class.get(n) != Some(&check::generics::FnClass::Plain) || f.def(&p.ast).body.is_none() {
            return Err("native freestanding generic/import artifact interface remains unqualified".into());
        }
        if !f.params.iter().chain([&f.ret]).all(|&t|if artifact{crate::native_crate::signature_type(&p,t)}else{core_type(&p.tys,t,0)}) {
            return Err("native freestanding core/ADT signature image remains unqualified".into());
        }
        let (checked,errors) = check::check_fn(&p,id);
        if !errors.is_empty() { return Err(report(errors)); }
        if artifact&&checked.expr_res.iter().any(|r|matches!(r,check::Res::Builtin(_)|check::Res::PrimAssoc{..}|check::Res::Intrinsic(_))){return Err("native producer needs builtin/library semantic records".into());}
        for (slot,res) in checked.expr_res.iter().enumerate(){
            let expression=checked.expr_base+slot as u32;
            if !methods.contains(&expression) || *res==check::Res::None {continue;}
            let Some(ast::Expr::MethodCall{recv,..})=p.ast.exprs.get(expression as usize) else{unreachable!()};
            // These actual native operation IDs on canonical integer types
            // denote inherent core arithmetic, requiring no imported trait.
            // In particular, an alloc ToOwned call lowered to Clone is NOT
            // qualified by an allocation-free result or identical machine code.
            // Other method origins await the shared unit resolver; this proof
            // must not grow into a method-name allowlist or trait-scope guess.
            if !matches!(res,check::Res::BuiltinMethod{b:check::Builtin::WrappingMul|check::Builtin::RotateLeft,derefs:0,autoref:check::Autoref::None})
                || !matches!(checked.tys.kind(checked.ty(*recv)),ty::TyKind::Int(_))
            {return Err("native unit method-resolution scope is not qualified".into());}
            methods.remove(&expression);
        }
        let expr_base = checked.expr_base;
        let expressions = if capture { checked.expr_res.iter().map(|resolution| match resolution {
            check::Res::None => ExpressionResolution::None,
            check::Res::Local(local) => checked.param_locals.iter().position(|param| param == local)
                .map_or(ExpressionResolution::Other, ExpressionResolution::Parameter),
            check::Res::Fn(function) if (function.0 as usize) < p.fns.len() => ExpressionResolution::Function(p.disk_id(*function)),
            check::Res::Bin { ld: 0, rd: 0 } => ExpressionResolution::PrimitiveOperands,
            _ => ExpressionResolution::Other,
        }).collect() } else { Vec::new() };
        let body = lower::lower(&p,id,checked);
        let errors = borrowck::borrowck(&p,&body);
        if !errors.is_empty() { return Err(report(errors)); }
        if !body.locals.iter().all(|l|if artifact{crate::native_crate::consumer_type(&p,&body.tys,l.ty,0)}else{core_type(&body.tys,l.ty,0)}) {
            return Err("native freestanding core/ADT body image remains unqualified".into());
        }
        if artifact && !cgen::names::with(Some(&names),||f.params.iter().chain([&f.ret]).all(|&t|cgen::native_type_abi(&p,t).is_some())) {return Err("native producer effective C ABI differs from the image".into());}
        let first_line=map.line_display_col(map.call_site(f.def(&p.ast).name_span)).0;
        let generated = cgen::names::with(Some(&names),||cgen::gen_fn(&p,&body,&map,first_line,&slots));
        if let Some(e)=&generated.error { return Err(e.clone()); }
        if capture {
            if body.fn_id != id { return Err("native accepted body belongs to another definition".into()); }
            let mut lowered_calls = Some(BTreeSet::new());
            for block in &body.blocks {
                if let crate::ir::Terminator::Call { target, .. } = block.term {
                    match target {
                        crate::ir::CallTarget::Fn(function) if (function.0 as usize) < p.fns.len() => {
                            if let Some(calls) = &mut lowered_calls { calls.insert(p.disk_id(function)); }
                        }
                        _ => lowered_calls = None,
                    }
                }
            }
            let generated_uses = generated.used.iter().map(|&slot| {
                ((slot as usize) < p.fns.len()).then(|| p.disk_id(def::LocalDefId(slot)))
            }).collect();
            accepted.push(AcceptedBody { local: body.fn_id, definition: p.disk_id(body.fn_id), body: f.def(&p.ast).body.unwrap(), expr_base,
                expressions: std::sync::Arc::new(expressions), pat_base: 0, bindings: Default::default(), lowered_calls, generated_uses });
        }
        out.push(generated);
    }
    if !methods.is_empty(){return Err("native unit has method obligations without original-context resolution evidence".into());}
    Ok(PreparedUnit { request: r, source, map, lexed, program: p, names, generated: out, accepted, artifact, capture, working_directory })
}

pub(crate) fn emit(prepared: PreparedUnit) -> Result<EmittedUnit, String> {
    emit_with_backend(prepared, None)
}

fn emit_with_backend(prepared: PreparedUnit, artifact: Option<(&crate::native_crate::Generation,&crate::native_crate::Backend)>) -> Result<EmittedUnit, String> {
    if prepared.artifact != artifact.is_some() { return Err("prepared native unit has a different artifact/backend contract".into()); }
    let (r, p, map, names, out) = (&prepared.request, &prepared.program, &prepared.map, &prepared.names, &prepared.generated);
    let slots: Vec<u32> = (0..p.fns.len()).map(|n|n as u32).collect();
    let mut glue = cgen::GlueSet::default();
    glue.add(&p,&slots,out.iter().flat_map(|f|f.glue.iter()));
    let indexes: Vec<usize> = (0..out.len()).collect();
    let register = format!("lustr_unit_register_{:016x}",p.stable_crate.0);
    let text = cgen::names::with(Some(&names),||cgen::fn_unit(&p,&out,&indexes,&glue,&register,None));
    let parent = r.output.parent().ok_or("object output parent required")?;
    static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
    let scratch = parent.join(format!(".lustr-object-{}-{}",std::process::id(),NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed)));
    fs::create_dir(&scratch).map_err(|e|format!("cannot own native object scratch: {e}"))?;
    // Retain actual generated input on success and failure for the operation
    // owner to checkpoint, then remove with its other disposable output.
    let object=scratch.join("unit.o");
    // Compile the original generated unit once using an explicit freestanding
    // backend. Referenced symbols must pass the independent ELF closure gate.
    let input=scratch.join("unit.c");fs::write(&input,text).map_err(|e|e.to_string())?;
    if let Some((generation,backend))=artifact {backend.verify(generation)?;}
    let backend=artifact.map_or_else(||std::path::Path::new("clang"),|(_,backend)|backend.executable());
    let mut command=Command::new(backend);
    if prepared.capture {command.args(["--no-default-config","-fno-stack-protector"]);}
    command.args(["--target=x86_64-unknown-linux-gnu","-x","c","-std=gnu11","-ffreestanding","-fno-builtin","-fno-strict-aliasing","-fwrapv","-fno-asynchronous-unwind-tables","-O0","-c"]).arg(&input).arg("-o").arg(&object);
    if prepared.capture {fs::write(scratch.join("backend.command"),format!("{command:?}\n")).map_err(|e|e.to_string())?;}
    let result=command.output().map_err(|e|format!("cannot start explicit native object C backend: {e}"))?;
    if prepared.capture {
        fs::write(scratch.join("backend.stdout"),&result.stdout).map_err(|e|e.to_string())?;
        fs::write(scratch.join("backend.stderr"),&result.stderr).map_err(|e|e.to_string())?;
        fs::write(scratch.join("backend.status"),format!("{}\n",result.status)).map_err(|e|e.to_string())?;
    }
    if !result.status.success(){return Err(format!("native object C backend {}: {}",result.status,String::from_utf8_lossy(&result.stderr)));}
    if prepared.capture && (!result.stdout.is_empty() || !result.stderr.is_empty()) {return Err("native metadata object backend produced unexpected output; exact streams retained".into());}
    if let Some((generation,backend))=artifact {backend.verify(generation)?;}
    // Only a native semantic artifact consumes the runtime source/location
    // record. A freestanding object must have no undefined symbols below;
    // it neither uses this runtime nor needs to hash its unused source.
    let (object,runtime)=if let Some((generation,backend))=artifact {
        let mut runtime=crate::native_crate::runtime::Requirements::capture(&p,&map)?;
        let object=crate::native_crate::runtime::prepare(&object,&scratch,&mut runtime,generation,backend)?;
        (object,Some(runtime))
    } else {(object,None)};
    let bytes = fs::read(&object).map_err(|e|format!("cannot read native object: {e}"))?;
    let symbols = super::unit_object::symbols(&bytes)?;
    if artifact.is_none()&&!symbols.undefined.is_empty() { return Err(format!("native freestanding object has unprovided runtime symbols: {:?}",symbols.undefined)); }
    if runtime.as_ref().is_some_and(|runtime|symbols.undefined!=runtime.imports){return Err("native artifact runtime edge proof changed".into());}
    let mut helpers = symbols.functions;
    let mut functions = Vec::new();
    for (index, name) in names.fns.iter().enumerate().filter_map(|(index, name)| name.as_ref().map(|name| (index, name))) {
        let symbol = format!("f_{name}").into_bytes();
        if !helpers.contains(&symbol) { return Err("native object is missing a checked function definition".into()); }
        if prepared.capture {
            helpers.remove(&symbol);
            functions.push(EmittedFunction { definition: p.disk_id(def::LocalDefId(index as u32)), symbol });
        }
    }
    if prepared.capture && functions.len() != prepared.accepted.len() { return Err("native emitted inventory does not cover the accepted functions".into()); }
    Ok(EmittedUnit { prepared, object, scratch, bytes, functions, helpers, runtime })
}

fn compile(r: Request, artifact: Option<(&crate::native_crate::Generation,&crate::native_crate::Backend)>) -> Result<Completion, String> {
    let prepared = prepare_request(r, artifact.is_some(), false, None)?;
    let EmittedUnit { prepared, object, scratch, bytes, runtime, .. } = emit_with_backend(prepared, artifact)?;
    let PreparedUnit { request: r, source, program: p, names, .. } = prepared;
    // A real object only. Do not emit an empty rmeta/rlib or report Cargo
    // metadata support until the typed producer/consumer contract exists.
    let mut message=format!("native freestanding object: {} checked functions, {} bytes, no undefined symbols; core/alloc/metadata/Cargo completion remains open\n",p.fns.len(),bytes.len());
    for (n,f) in p.fns.iter().enumerate(){message.push_str(&format!("native object export: {} f_{}\n",p.interner.get(f.name),names.fns[n].as_ref().unwrap()));}
    let published = if let Some((generation,_)) = artifact {
        let runtime=runtime.ok_or("native artifact lacks its runtime requirements")?;
        let image=crate::native_crate::Artifact::produce(&p,&r.name,&r.metadata,&source,bytes,generation.clone(),runtime)?.encode()?;
        let path=scratch.join("unit.lustr-native");fs::write(&path,&image).map_err(|e|e.to_string())?;
        message=format!("native semantic artifact: {} actual checked structural function signatures and explicit proven native runtime dependencies; full-image/Cargo/full-language gates remain open\n",p.fns.len());
        path
    } else { object };
    match super::unit_publication::publish(&published,&r.output).map_err(|e|format!("cannot publish native object: {e}"))? {
        super::unit_publication::Publication::Durable=>Ok(Completion::Durable(message)),
        super::unit_publication::Publication::DurabilityUnknown(error)=>Ok(Completion::PublishedUncertain(format!("{message}object publication occurred; final directory durability is unconfirmed: {error}; retain the output and do not automatically replay\n"))),
    }
}

pub(crate) fn run(args: &[String]) -> (i32, String) {
    result(request(args).and_then(|r|compile(r,None)))
}

// Internal component qualifier only. The C object ABI remains unchanged;
// a native artifact is never represented to Cargo as an object/rlib/rmeta.
pub(crate) fn run_artifact(args:&[String],generation:&crate::native_crate::Generation,backend:&crate::native_crate::Backend)->(i32,String){
    if let Err(e)=crate::native_crate::fixed_abi(){return (1,format!("error: {e}\n"));}
    let Some(at)=args.iter().position(|a|a=="--emit=lustr-native-crate") else{return (1,"explicit native semantic artifact output required\n".into())};
    let mut args=args.to_vec();args[at]="--emit=obj".into();
    result(backend.verify(generation).and_then(|_|request(&args)).and_then(|r|compile(r,Some((generation,backend)))))
}
fn result(value:Result<Completion,String>)->(i32,String){match value { Ok(Completion::Durable(s))=>(0,s),Ok(Completion::PublishedUncertain(s))=>(2,s),Err(e)=>(1,format!("error: {e}\n")) }}
